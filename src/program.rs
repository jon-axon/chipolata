use crate::error::ErrorDetail;

/// An abstraction of a CHIP-8 ROM, ready for loading into the Chipolata emulator.
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
    /// Constructor that returns [Program] instance representing the passed program data.
    pub fn new(data: Vec<u8>) -> Self {
        Program { program_data: data }
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
}
