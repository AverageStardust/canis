pub mod instruction;
pub mod log;
pub mod processes;

mod assemble;

pub enum CommandError {
    Anyhow(anyhow::Error),
    InternallyPrinted,
}

impl From<anyhow::Error> for CommandError {
    fn from(value: anyhow::Error) -> Self {
        CommandError::Anyhow(value)
    }
}
