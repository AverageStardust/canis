use std::{fs::File, io::Read, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

use crate::{
    cli::{Cli, Command},
    log::Log,
};

mod cli;
mod log;
mod processes;

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
                Some(path) => {
                    // TODO: Validate
                    path
                }
                None => PathBuf::from("program.hex"),
            };

            let mut input_file = File::open(&file_canonicalized).with_context(|| {
                format!(
                    "failed to open input file for reading ({})",
                    file_canonicalized.display()
                )
            })?;
            let output_file = File::create(&output).with_context(|| {
                format!(
                    "failed to open output file for writing ({})",
                    output.display()
                )
            })?;

            let output = output.canonicalize().with_context(|| {
                format!("output file path does not exist ({})", output.display())
            })?;

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
            processes::assemble(log, &input, file.display(), &output_file).map_err(
                |err| match err {
                    CommandError::Anyhow(error) => CommandError::Anyhow(error.context(format!(
                        "failed to assemble input ({})",
                        file_canonicalized.display()
                    ))),
                    CommandError::InternallyPrinted => CommandError::InternallyPrinted,
                },
            )?;
            log.status_with("Finished", || format!("assembling `{}`", output.display()));
        }
    }

    Ok(())
}

enum CommandError {
    Anyhow(anyhow::Error),
    InternallyPrinted,
}

impl From<anyhow::Error> for CommandError {
    fn from(value: anyhow::Error) -> Self {
        CommandError::Anyhow(value)
    }
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
