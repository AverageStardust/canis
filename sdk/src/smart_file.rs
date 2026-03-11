use std::{fs::File, io::Write, path::PathBuf};

/// A create-on-write and truncate-on-write file handle
pub enum SmartFile {
    Declared(PathBuf),
    Created(File),
}

impl SmartFile {
    pub fn new(path: PathBuf) -> Self {
        SmartFile::Declared(path)
    }

    fn as_file(&mut self) -> std::io::Result<&mut File> {
        match self {
            Self::Created(file) => Ok(file),
            Self::Declared(path) => {
                let file = File::create(path)?;
                *self = Self::Created(file);
                let Self::Created(file) = self else {
                    unreachable!()
                };
                Ok(file)
            }
        }
    }
}

impl Write for SmartFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.as_file()?.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.as_file()?.flush()
    }
}
