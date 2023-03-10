use crate::error::ErrorDetail;
use std::fs;
use std::path::Path;

/// An abstraction of a CHIP-8 ROM, ready for loading into the Chipolata emulator.
#[derive(Debug, PartialEq)]
pub struct Program {
    /// A byte vector containing the program data as loading from the ROM.
    program_data: Vec<u8>,
}

impl Default for Program {
    /// Constructor that returns an empty [Program] instance.
    fn default() -> Self {
        Program {
            program_data: Vec::new(),
        }
    }
}

impl Program {
    /// Constructor that returns a [Program] instance representing the passed program data.
    pub fn new(data: Vec<u8>) -> Self {
        Program { program_data: data }
    }

    /// Builder method that instantiates [Program] from the specified binary ROM file
    pub fn load_from_file(file_path: &Path) -> Result<Program, ErrorDetail> {
        // attempt to open the file and read as a byte vector
        if let Ok(program_data) = fs::read(file_path) {
            return Ok(Program::new(program_data));
        }
        // if we fall through to here, an error has occurred reading from the file
        return Err(ErrorDetail::FileError {
            file_path: file_path.to_str().unwrap_or_default().to_owned(),
        });
    }

    /// Method that serialises the passed [Program] instance to the specified binary file
    pub fn save_to_file(program: &Program, file_path: &Path) -> Result<(), ErrorDetail> {
        // attempt to open the file and write to it; create it if it does not exist and truncate if it does
        if let Ok(_) = fs::write(file_path, &program.program_data) {
            return Ok(());
        }
        // if we fall through to here, an error has occurred writing to the file
        return Err(ErrorDetail::FileError {
            file_path: file_path.to_str().unwrap_or_default().to_owned(),
        });
    }

    /// Sets the program data as per the specified byte vector.
    ///
    /// # Arguments
    ///
    /// * `data` - the byte vector containing the program data to use
    pub fn set_program_data(&mut self, data: Vec<u8>) -> Result<(), ErrorDetail> {
        self.program_data = data;
        Ok(())
    }

    /// Returns a reference to the program data held in this instance.
    pub fn program_data(&self) -> &Vec<u8> {
        &self.program_data
    }

    /// Returns the size of the instance's program data (in bytes).
    pub(crate) fn program_data_size(&self) -> usize {
        self.program_data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_program() -> Vec<u8> {
        vec![0xA1, 0x14, 0x0C, 0xFD, 0xA3]
    }

    #[test]
    fn test_program_data() {
        let mut program: Program = Program::default();
        let test_program: Vec<u8> = setup_test_program();
        program.set_program_data(test_program.clone()).unwrap();
        assert_eq!(program.program_data(), &test_program);
    }

    #[test]
    fn test_program_data_size() {
        let mut program: Program = Program::default();
        let test_program: Vec<u8> = setup_test_program();
        program.set_program_data(test_program.clone()).unwrap();
        assert_eq!(program.program_data_size(), test_program.len());
    }

    #[test]
    fn test_save_load() {
        const FILENAME: &str = "unit_test_save_load.ch8";
        let program: Program = Program {
            program_data: vec![0x3, 0xFF, 0x2, 0xA1],
        };
        Program::save_to_file(&program, Path::new(FILENAME)).unwrap();
        let new_program = Program::load_from_file(Path::new(FILENAME)).unwrap();
        assert_eq!(program, new_program);
        std::fs::remove_file(FILENAME).unwrap();
    }
}
