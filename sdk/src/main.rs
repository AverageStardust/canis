use std::{fmt::Display, fs::File, io::Read, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

use crate::{
    cli::{Cli, Command},
    smart_file::SmartFile,
};

use canis_sdk::{CommandError, instruction::registry::get_instruction_meta, log::Log, processes};

mod cli;
mod smart_file;

fn main() {
    let cli = Cli::parse();
    let mut log = Log::new(cli.get_verbosity());

    if let Err(err) = run_command(&mut log, cli.command) {
        if let CommandError::Anyhow(err_msg) = err {
            log.error_with(|| {
                format!(
                    "failed to run command due to error:\n{}",
                    render_and_indent(&err_msg)
                )
            });
        }

        std::process::exit(1);
    }
}

fn run_command(log: &mut Log, command: Command) -> Result<(), CommandError> {
    match command {
        Command::Assemble { file, output } => {
            let file_canonicalized = file
                .canonicalize()
                .with_context(|| format!("input file path does not exist ({})", file.display()))?;

            let output = match output {
                Some(path) => path,
                None => PathBuf::from("program.hex"),
            };

            let mut input_file = File::open(&file_canonicalized).with_context(|| {
                format!(
                    "failed to open input file for reading ({})",
                    file_canonicalized.display()
                )
            })?;
            let mut output_file = SmartFile::new(output.clone());
            let mut input = String::new();
            input_file.read_to_string(&mut input).with_context(|| {
                format!(
                    "failed to read input file ({})",
                    file_canonicalized.display()
                )
            })?;

            log.status_with("Assembling", || {
                format!("from `{}`", file_canonicalized.display())
            });
            processes::assemble(log, &input, file.display(), &mut output_file).map_err(|err| {
                match err {
                    CommandError::Anyhow(error) => CommandError::Anyhow(error.context(format!(
                        "failed to assemble input ({})",
                        file_canonicalized.display()
                    ))),
                    CommandError::InternallyPrinted => CommandError::InternallyPrinted,
                }
            })?;

            let output_name = output.canonicalize().with_context(|| {
                format!("output file path does not exist ({})", output.display())
            })?;

            log.status_with("Finished", || {
                format!("assembling `{}`", output_name.display())
            });
        }
        Command::Instruction(cli::InstructionCommand::List) => {
            let mut meta = get_instruction_meta();
            meta.sort();

            println!("{}", render_header("Instructions:"));

            let (widest_name, widest_display) =
                meta.iter().fold((0, 0), |(name, display), meta| {
                    (
                        name.max(meta.name.chars().count()),
                        display.max(
                            meta.details
                                .as_ref()
                                .map(|details| details.display_name.chars().count())
                                .unwrap_or(0),
                        ),
                    )
                });

            for meta in meta {
                print!(
                    "  {}{:widest_name$}{}",
                    anstyle::Style::new().bold(),
                    meta.name,
                    anstyle::Reset
                );

                if let Some(details) = &meta.details {
                    print!(
                        "  {:widest_display$}  {}",
                        details.display_name, details.short_description
                    );
                }

                println!()
            }
        }
        Command::Instruction(cli::InstructionCommand::Explain { meta }) => {
            if let Some(explain) = meta.explain() {
                for (idx, variant) in explain.into_iter().enumerate() {
                    if idx > 0 {
                        println!("\n")
                    }
                    print!("{} {}", render_header("Usage:"), render_bold(meta.name));
                    for (name, _types) in variant.args.iter() {
                        print!(" <{}>", name)
                    }
                    println!("\n");
                    if variant.args.len() > 0 {
                        println!("{}", render_header("Arguments:"));
                        let max_width = variant
                            .args
                            .iter()
                            .map(|(name, _types)| name.chars().count())
                            .max()
                            .unwrap_or(0);
                        for (name, types) in variant.args {
                            print!(
                                "  {}{name:max_width$}{}  {}",
                                anstyle::Style::new().bold(),
                                anstyle::Reset,
                                types
                                    .iter_names()
                                    .map(|(name, val)| {
                                        let mut string = String::from(name);
                                        if val.is_immediate() {
                                            let bits = types.expected_bits();
                                            string.push_str(
                                                format!(
                                                    " ({} bit{})",
                                                    bits,
                                                    (bits == 1).then_some("").unwrap_or("s")
                                                )
                                                .as_str(),
                                            );
                                        }
                                        string
                                    })
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            );
                            println!();
                        }
                        println!();
                    }
                    println!(
                        "{}\n  {}",
                        render_header("Description:"),
                        variant.description
                    );
                }
            } else {
                println!(
                    "No explain details available for instruction '{}'",
                    render_bold(meta.name)
                );
            }
        }
    }

    Ok(())
}

fn render_and_indent(err: &anyhow::Error) -> String {
    let err = format!("{err:?}");
    let mut lines = err.lines();
    let mut result = String::new();
    if let Some(line) = lines.next() {
        result.push_str(format!("             {line}").as_str());
        for line in lines {
            result.push_str(format!("\n             {line}").as_str());
        }
    }
    result
}

fn render_bold(display: impl Display) -> String {
    format!(
        "{}{display}{}",
        anstyle::Style::new().bold(),
        anstyle::Reset
    )
}

fn render_header(display: impl Display) -> String {
    format!(
        "{}{display}{}",
        anstyle::Style::new().bold().underline(),
        anstyle::Reset
    )
}
