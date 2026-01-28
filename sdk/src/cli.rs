use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::log::Verbosity;

#[derive(Parser)]
#[command(arg_required_else_help(true))]
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
}
