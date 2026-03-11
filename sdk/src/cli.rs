use std::path::PathBuf;

use clap::{Parser, Subcommand};

use canis_sdk::{
    instruction::{InstructionMeta, registry::get_instruction_meta},
    log::Verbosity,
};

#[derive(Parser)]
#[command(arg_required_else_help(true))]
#[clap(version, about)]
pub struct Cli {
    /// Suppress all log output
    #[arg(short, long, global = true, group = "verbosity")]
    quiet: bool,

    /// Uses verbose log output
    #[clap(short, long, global = true, group = "verbosity")]
    verbose: bool,

    /// The command to run
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn get_verbosity(&self) -> Verbosity {
        if self.quiet {
            Verbosity::Quiet
        } else if self.verbose {
            Verbosity::Verbose
        } else {
            Verbosity::Normal
        }
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Assembles the inputted file to a .hex file
    Assemble {
        /// The file containing the assembly to assemble
        file: PathBuf,

        /// Outputs the final hex code to this file, instead of program.hex
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Get information about available instructions
    #[clap(subcommand)]
    Instruction(InstructionCommand),
}

#[derive(Subcommand)]
pub enum InstructionCommand {
    /// List all available instructions, and some simple details
    List,

    /// Explain a given instruction in detail
    Explain {
        #[clap(value_parser = valid_instruction)]
        meta: &'static InstructionMeta,
    },
}

const YELLOW: anstyle::Style =
    anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow)));
const RESET: anstyle::Reset = anstyle::Reset;

fn valid_instruction(input: &str) -> Result<&'static InstructionMeta, String> {
    let metas = get_instruction_meta();
    if let Some(found) = metas.iter().find(|meta| meta.name == input) {
        Ok(found)
    } else {
        match find_similar(input, metas) {
            closest if closest.len() > 1 => {
                let mut result =
                    format!("'{YELLOW}{input}{RESET}' is not a valid instruction, did you mean ");
                let last = closest.len() - 1;
                for (idx, meta) in closest.iter().enumerate() {
                    if idx > 0 {
                        result.push_str(", ")
                    }
                    if idx == last {
                        result.push_str("or ")
                    }
                    result.push_str(format!("'{YELLOW}{}{RESET}'", meta.name).as_str());
                }
                result.push('?');
                Err(result)
            }
            closest if closest.len() == 1 => Err(format!(
                "'{YELLOW}{input}{RESET}' is not a valid instruction, did you mean '{YELLOW}{}{RESET}'?",
                closest[0].name
            )),
            _ => Err(format!(
                "'{YELLOW}{input}{RESET}' is not a valid instruction."
            )),
        }
    }
}

fn find_similar(
    incorrect: &str,
    metas: Vec<&'static InstructionMeta>,
) -> Vec<&'static InstructionMeta> {
    metas
        .iter()
        .map(|meta| (meta, strsim::damerau_levenshtein(incorrect, meta.name)))
        .filter(|(_meta, distance)| *distance < 2)
        .map(|(meta, _distance)| *meta)
        .collect()
}
