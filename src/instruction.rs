use crate::error::ErrorDetail;

/// An enum with a variant for each instruction within the CHIP-8 instruction set.
#[derive(Debug, PartialEq)]
pub(crate) enum Instruction {
    Op004B,                               // Turn on COSMAC VIP display
    Op00CN { n: u8 },                     // [SUPER-CHIP 1.1] Scroll N pixels down (N/2 in low res)
    Op00E0,                               // Clear screen
    Op00EE,                               // Subroutine (call)
    Op00FB,                               // [SUPER-CHIP 1.1] Scroll right 4 pixels (2 in low res)
    Op00FC,                               // [SUPER-CHIP 1.1] Scroll left 4 pixels (2 in low res)
    Op00FD,                               // [SUPER-CHIP 1.1] Exit the interpreter
    Op00FE,                               // [SUPER-CHIP 1.1] Disable high-resolution mode
    Op00FF,                               // [SUPER-CHIP 1.1] Enable high-resolution mode
    Op0NNN { nnn: u16 },                  // Execute machine language routine
    Op1NNN { nnn: u16 },                  // Jump to NNN
    Op2NNN { nnn: u16 },                  // Subroutine (return)
    Op3XNN { x: usize, nn: u8 },          // Skip (if Vx = NN)
    Op4XNN { x: usize, nn: u8 },          // Skip (if Vx != NN)
    Op5XY0 { x: usize, y: usize },        // Skip (if Vx = Vy)
    Op6XNN { x: usize, nn: u8 },          // Set register
    Op7XNN { x: usize, nn: u8 },          // Add (NN to Vx)
    Op8XY0 { x: usize, y: usize },        // Set
    Op8XY1 { x: usize, y: usize },        // Binary OR
    Op8XY2 { x: usize, y: usize },        // Binary AND
    Op8XY3 { x: usize, y: usize },        // Logical XOR
    Op8XY4 { x: usize, y: usize },        // Add (Vy to Vx)
    Op8XY5 { x: usize, y: usize },        // Subtract (Vx - Vy -> Vx)
    Op8XY6 { x: usize, y: usize },        // Vx = Vy then shift Vx >> 1, set Vf to shifted-out bit
    Op8XY7 { x: usize, y: usize },        // Subtract (Vy - Vx -> Vx)
    Op8XYE { x: usize, y: usize },        // Vx = Vy then shift Vx << 1, set Vf to shifted-out bit
    Op9XY0 { x: usize, y: usize },        // Skip (if Vx != Vy)
    OpANNN { nnn: u16 },                  // Set I = NNN
    OpBNNN { nnn: u16 },                  // Jump to NNN + V0
    OpCXNN { x: usize, nn: u8 },          // Rnd & NN, insert to Vx
    OpDXYN { x: usize, y: usize, n: u8 }, // Draw sprite
    OpEX9E { x: usize },                  // Skip if Vx key is pressed
    OpEXA1 { x: usize },                  // Skip if Vx key is not pressed
    OpFX07 { x: usize },                  // Vx = value of delay timer
    OpFX15 { x: usize },                  // value of delay timer = Vx
    OpFX18 { x: usize },                  // value of sound timer = Vx
    OpFX1E { x: usize },                  // I = I + Vx
    OpFX0A { x: usize },                  // Vx = blocks until keypress
    OpFX29 { x: usize },                  // Read char from Vx, set I to address of that font char
    OpFX30 { x: usize },                  // [SUPER-CHIP 1.1] as FX29 but for high-resolution font
    OpFX33 { x: usize },                  // Binary-coded decimal conversion
    OpFX55 { x: usize },                  // Store V registers to memory
    OpFX65 { x: usize },                  // Load V registers from memory
    OpFX75 { x: usize },                  // [SUPER-CHIP 1.1] Store V registers to RPL user flags
    OpFX85 { x: usize },                  // [SUPER-CHIP 1.1] Load V registers from RPL user flags
}

impl Instruction {
    /// Constructor/builder method that parses the supplied two-byte opcode and returns the
    /// corresponding [Instruction] enum variant.  Returns [ErrorDetail::UnknownInstruction] if
    /// the opcode cannot be parsed or recognised.
    ///
    /// # Arguments
    ///
    /// * `opcode` - a (big-endian) two-byte representation of the opcode to be parsed
    pub(crate) fn decode_from(opcode: u16) -> Result<Instruction, ErrorDetail> {
        // Divide the 16-bit opcode into four 4-bit nibbles, using bit shifting and masking
        let first_nibble: u16 = opcode >> 12;
        let second_nibble: u16 = (opcode & 0x0F00) >> 8;
        let third_nibble: u16 = (opcode & 0x00F0) >> 4;
        let fourth_nibble: u16 = opcode & 0x000F;
        // Pattern match on the nibbles as appropriate to identify the opcode and return
        // the corresponding enum variant
        match (first_nibble, second_nibble, third_nibble, fourth_nibble) {
            (0x0, 0x0, 0x4, 0xB) => Ok(Instruction::Op004B),
            (0x0, 0x0, 0xC, _) => Ok(Instruction::Op00CN {
                n: fourth_nibble as u8,
            }),
            (0x0, 0x0, 0xE, 0x0) => Ok(Instruction::Op00E0),
            (0x0, 0x0, 0xE, 0xE) => Ok(Instruction::Op00EE),
            (0x0, 0x0, 0xF, 0xB) => Ok(Instruction::Op00FB),
            (0x0, 0x0, 0xF, 0xC) => Ok(Instruction::Op00FC),
            (0x0, 0x0, 0xF, 0xD) => Ok(Instruction::Op00FD),
            (0x0, 0x0, 0xF, 0xE) => Ok(Instruction::Op00FE),
            (0x0, 0x0, 0xF, 0xF) => Ok(Instruction::Op00FF),
            (0x0, ..) => Ok(Instruction::Op0NNN {
                nnn: opcode & 0x0FFF,
            }),
            (0x1, ..) => Ok(Instruction::Op1NNN {
                nnn: opcode & 0x0FFF,
            }),
            (0x2, ..) => Ok(Instruction::Op2NNN {
                nnn: opcode & 0x0FFF,
            }),
            (0x3, ..) => Ok(Instruction::Op3XNN {
                x: second_nibble as usize,
                nn: (opcode & 0x00FF) as u8,
            }),
            (0x4, ..) => Ok(Instruction::Op4XNN {
                x: second_nibble as usize,
                nn: (opcode & 0x00ff) as u8,
            }),
            (0x5, ..) => Ok(Instruction::Op5XY0 {
                x: second_nibble as usize,
                y: third_nibble as usize,
            }),
            (0x6, ..) => Ok(Instruction::Op6XNN {
                x: second_nibble as usize,
                nn: (opcode & 0x00FF) as u8,
            }),
            (0x7, ..) => Ok(Instruction::Op7XNN {
                x: second_nibble as usize,
                nn: (opcode & 0x00FF) as u8,
            }),
            (0x8, _, _, 0x0) => Ok(Instruction::Op8XY0 {
                x: second_nibble as usize,
                y: third_nibble as usize,
            }),
            (0x8, _, _, 0x1) => Ok(Instruction::Op8XY1 {
                x: second_nibble as usize,
                y: third_nibble as usize,
            }),
            (0x8, _, _, 0x2) => Ok(Instruction::Op8XY2 {
                x: second_nibble as usize,
                y: third_nibble as usize,
            }),
            (0x8, _, _, 0x3) => Ok(Instruction::Op8XY3 {
                x: second_nibble as usize,
                y: third_nibble as usize,
            }),
            (0x8, _, _, 0x4) => Ok(Instruction::Op8XY4 {
                x: second_nibble as usize,
                y: third_nibble as usize,
            }),
            (0x8, _, _, 0x5) => Ok(Instruction::Op8XY5 {
                x: second_nibble as usize,
                y: third_nibble as usize,
            }),
            (0x8, _, _, 0x6) => Ok(Instruction::Op8XY6 {
                x: second_nibble as usize,
                y: third_nibble as usize,
            }),
            (0x8, _, _, 0x7) => Ok(Instruction::Op8XY7 {
                x: second_nibble as usize,
                y: third_nibble as usize,
            }),
            (0x8, _, _, 0xE) => Ok(Instruction::Op8XYE {
                x: second_nibble as usize,
                y: third_nibble as usize,
            }),
            (0x9, ..) => Ok(Instruction::Op9XY0 {
                x: second_nibble as usize,
                y: third_nibble as usize,
            }),
            (0xA, ..) => Ok(Instruction::OpANNN {
                nnn: opcode & 0x0FFF,
            }),
            (0xB, ..) => Ok(Instruction::OpBNNN {
                nnn: opcode & 0x0FFF,
            }),
            (0xC, ..) => Ok(Instruction::OpCXNN {
                x: second_nibble as usize,
                nn: (opcode & 0x00FF) as u8,
            }),
            (0xD, ..) => Ok(Instruction::OpDXYN {
                x: second_nibble as usize,
                y: third_nibble as usize,
                n: fourth_nibble as u8,
            }),
            (0xE, _, 0x9, 0xE) => Ok(Instruction::OpEX9E {
                x: second_nibble as usize,
            }),
            (0xE, _, 0xA, 0x1) => Ok(Instruction::OpEXA1 {
                x: second_nibble as usize,
            }),
            (0xF, _, 0x0, 0x7) => Ok(Instruction::OpFX07 {
                x: second_nibble as usize,
            }),
            (0xF, _, 0x1, 0x5) => Ok(Instruction::OpFX15 {
                x: second_nibble as usize,
            }),
            (0xF, _, 0x1, 0x8) => Ok(Instruction::OpFX18 {
                x: second_nibble as usize,
            }),
            (0xF, _, 0x1, 0xE) => Ok(Instruction::OpFX1E {
                x: second_nibble as usize,
            }),
            (0xF, _, 0x0, 0xA) => Ok(Instruction::OpFX0A {
                x: second_nibble as usize,
            }),
            (0xF, _, 0x2, 0x9) => Ok(Instruction::OpFX29 {
                x: second_nibble as usize,
            }),
            (0xF, _, 0x3, 0x0) => Ok(Instruction::OpFX30 {
                x: second_nibble as usize,
            }),
            (0xF, _, 0x3, 0x3) => Ok(Instruction::OpFX33 {
                x: second_nibble as usize,
            }),
            (0xF, _, 0x5, 0x5) => Ok(Instruction::OpFX55 {
                x: second_nibble as usize,
            }),
            (0xF, _, 0x6, 0x5) => Ok(Instruction::OpFX65 {
                x: second_nibble as usize,
            }),
            (0xF, _, 0x7, 0x5) => Ok(Instruction::OpFX75 {
                x: second_nibble as usize,
            }),
            (0xF, _, 0x8, 0x5) => Ok(Instruction::OpFX85 {
                x: second_nibble as usize,
            }),
            // If we have not matched by this point then we cannot identify the
            // instruction; return an Error
            _ => Err(ErrorDetail::UnknownInstruction { opcode }),
        }
    }

    /// Returns a textual representation of each enum variant.
    #[allow(dead_code)]
    pub(crate) fn name(&self) -> &str {
        match self {
            Instruction::Op004B => "004B",
            Instruction::Op00CN { .. } => "00CN",
            Instruction::Op00E0 => "00E0",
            Instruction::Op00EE => "00EE",
            Instruction::Op00FB => "00FB",
            Instruction::Op00FC => "00FC",
            Instruction::Op00FD => "00FD",
            Instruction::Op00FE => "00FE",
            Instruction::Op00FF => "00FF",
            Instruction::Op0NNN { .. } => "0NNN",
            Instruction::Op1NNN { .. } => "1NNN",
            Instruction::Op2NNN { .. } => "2NNN",
            Instruction::Op3XNN { .. } => "3XNN",
            Instruction::Op4XNN { .. } => "4XNN",
            Instruction::Op5XY0 { .. } => "5XY0",
            Instruction::Op6XNN { .. } => "6XNN",
            Instruction::Op7XNN { .. } => "7XNN",
            Instruction::Op8XY0 { .. } => "8XY0",
            Instruction::Op8XY1 { .. } => "8XY1",
            Instruction::Op8XY2 { .. } => "8XY2",
            Instruction::Op8XY3 { .. } => "8XY3",
            Instruction::Op8XY4 { .. } => "8XY4",
            Instruction::Op8XY5 { .. } => "8XY5",
            Instruction::Op8XY6 { .. } => "8XY6",
            Instruction::Op8XY7 { .. } => "8XY7",
            Instruction::Op8XYE { .. } => "8XYE",
            Instruction::Op9XY0 { .. } => "9XY0",
            Instruction::OpANNN { .. } => "ANNN",
            Instruction::OpBNNN { .. } => "BNNN",
            Instruction::OpCXNN { .. } => "CXNN",
            Instruction::OpDXYN { .. } => "DXYN",
            Instruction::OpEX9E { .. } => "EX9E",
            Instruction::OpEXA1 { .. } => "EXA1",
            Instruction::OpFX07 { .. } => "FX07",
            Instruction::OpFX15 { .. } => "FX15",
            Instruction::OpFX18 { .. } => "FX18",
            Instruction::OpFX1E { .. } => "FX1E",
            Instruction::OpFX0A { .. } => "FX0A",
            Instruction::OpFX29 { .. } => "FX29",
            Instruction::OpFX30 { .. } => "FX30",
            Instruction::OpFX33 { .. } => "FX33",
            Instruction::OpFX55 { .. } => "FX55",
            Instruction::OpFX65 { .. } => "FX65",
            Instruction::OpFX75 { .. } => "FX75",
            Instruction::OpFX85 { .. } => "FX85",
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;

    #[test]
    fn test_decode_004B() {
        assert_eq!(
            Instruction::decode_from(0x004B).unwrap(),
            Instruction::Op004B
        );
    }

    #[test]
    fn test_decode_00CN() {
        assert_eq!(
            Instruction::decode_from(0x00C5).unwrap(),
            Instruction::Op00CN { n: 0x5 }
        );
    }

    #[test]
    fn test_decode_00E0() {
        assert_eq!(
            Instruction::decode_from(0x00E0).unwrap(),
            Instruction::Op00E0
        );
    }

    #[test]
    fn test_decode_00EE() {
        assert_eq!(
            Instruction::decode_from(0x00EE).unwrap(),
            Instruction::Op00EE
        );
    }

    #[test]
    fn test_decode_00FB() {
        assert_eq!(
            Instruction::decode_from(0x00FB).unwrap(),
            Instruction::Op00FB
        );
    }

    #[test]
    fn test_decode_00FC() {
        assert_eq!(
            Instruction::decode_from(0x00FC).unwrap(),
            Instruction::Op00FC
        );
    }

    #[test]
    fn test_decode_00FD() {
        assert_eq!(
            Instruction::decode_from(0x00FD).unwrap(),
            Instruction::Op00FD
        );
    }

    #[test]
    fn test_decode_00FE() {
        assert_eq!(
            Instruction::decode_from(0x00FE).unwrap(),
            Instruction::Op00FE
        );
    }

    #[test]
    fn test_decode_00FF() {
        assert_eq!(
            Instruction::decode_from(0x00FF).unwrap(),
            Instruction::Op00FF
        );
    }

    #[test]
    fn test_decode_0NNN() {
        assert_eq!(
            Instruction::decode_from(0x016F).unwrap(),
            Instruction::Op0NNN { nnn: 0x16F }
        );
    }

    #[test]
    fn test_decode_1NNN() {
        assert_eq!(
            Instruction::decode_from(0x1D38).unwrap(),
            Instruction::Op1NNN { nnn: 0xD38 }
        );
    }

    #[test]
    fn test_decode_2NNN() {
        assert_eq!(
            Instruction::decode_from(0x21CD).unwrap(),
            Instruction::Op2NNN { nnn: 0x1CD }
        );
    }

    #[test]
    fn test_decode_3XNN() {
        assert_eq!(
            Instruction::decode_from(0x3C63).unwrap(),
            Instruction::Op3XNN { x: 0xC, nn: 0x63 }
        );
    }

    #[test]
    fn test_decode_4XNN() {
        assert_eq!(
            Instruction::decode_from(0x42A7).unwrap(),
            Instruction::Op4XNN { x: 0x2, nn: 0xA7 }
        );
    }

    #[test]
    fn test_decode_5XY0() {
        assert_eq!(
            Instruction::decode_from(0x5340).unwrap(),
            Instruction::Op5XY0 { x: 0x3, y: 0x4 }
        );
    }

    #[test]
    fn test_decode_6XNN() {
        assert_eq!(
            Instruction::decode_from(0x602E).unwrap(),
            Instruction::Op6XNN { x: 0x0, nn: 0x2E }
        );
    }

    #[test]
    fn test_decode_7XNN() {
        assert_eq!(
            Instruction::decode_from(0x7A9F).unwrap(),
            Instruction::Op7XNN { x: 0xA, nn: 0x9F }
        );
    }

    #[test]
    fn test_decode_8XY0() {
        assert_eq!(
            Instruction::decode_from(0x8270).unwrap(),
            Instruction::Op8XY0 { x: 0x2, y: 0x7 }
        );
    }

    #[test]
    fn test_decode_8XY1() {
        assert_eq!(
            Instruction::decode_from(0x8DE1).unwrap(),
            Instruction::Op8XY1 { x: 0xD, y: 0xE }
        );
    }

    #[test]
    fn test_decode_8XY2() {
        assert_eq!(
            Instruction::decode_from(0x8322).unwrap(),
            Instruction::Op8XY2 { x: 0x3, y: 0x2 }
        );
    }

    #[test]
    fn test_decode_8XY3() {
        assert_eq!(
            Instruction::decode_from(0x81F3).unwrap(),
            Instruction::Op8XY3 { x: 0x1, y: 0xF }
        );
    }

    #[test]
    fn test_decode_8XY4() {
        assert_eq!(
            Instruction::decode_from(0x8964).unwrap(),
            Instruction::Op8XY4 { x: 0x9, y: 0x6 }
        );
    }

    #[test]
    fn test_decode_8XY5() {
        assert_eq!(
            Instruction::decode_from(0x8B05).unwrap(),
            Instruction::Op8XY5 { x: 0xB, y: 0x0 }
        );
    }

    #[test]
    fn test_decode_8XY6() {
        assert_eq!(
            Instruction::decode_from(0x8246).unwrap(),
            Instruction::Op8XY6 { x: 0x2, y: 0x4 }
        );
    }

    #[test]
    fn test_decode_8XY7() {
        assert_eq!(
            Instruction::decode_from(0x8EF7).unwrap(),
            Instruction::Op8XY7 { x: 0xE, y: 0xF }
        );
    }

    #[test]
    fn test_decode_8XYE() {
        assert_eq!(
            Instruction::decode_from(0x816E).unwrap(),
            Instruction::Op8XYE { x: 0x1, y: 0x6 }
        );
    }

    #[test]
    fn test_decode_9XY0() {
        assert_eq!(
            Instruction::decode_from(0x9E20).unwrap(),
            Instruction::Op9XY0 { x: 0xE, y: 0x2 }
        );
    }

    #[test]
    fn test_decode_ANNN() {
        assert_eq!(
            Instruction::decode_from(0xA41C).unwrap(),
            Instruction::OpANNN { nnn: 0x41C }
        );
    }

    #[test]
    fn test_decode_BNNN() {
        assert_eq!(
            Instruction::decode_from(0xB2EA).unwrap(),
            Instruction::OpBNNN { nnn: 0x2EA }
        );
    }

    #[test]
    fn test_decode_CXNN() {
        assert_eq!(
            Instruction::decode_from(0xC4DE).unwrap(),
            Instruction::OpCXNN { x: 0x4, nn: 0xDE }
        );
    }

    #[test]
    fn test_decode_DXYN() {
        assert_eq!(
            Instruction::decode_from(0xD2FB).unwrap(),
            Instruction::OpDXYN {
                x: 0x2,
                y: 0xF,
                n: 0xB
            }
        );
    }

    #[test]
    fn test_decode_EX9E() {
        assert_eq!(
            Instruction::decode_from(0xE39E).unwrap(),
            Instruction::OpEX9E { x: 0x3 }
        );
    }

    #[test]
    fn test_decode_EXA1() {
        assert_eq!(
            Instruction::decode_from(0xEAA1).unwrap(),
            Instruction::OpEXA1 { x: 0xA }
        );
    }

    #[test]
    fn test_decode_FX07() {
        assert_eq!(
            Instruction::decode_from(0xFB07).unwrap(),
            Instruction::OpFX07 { x: 0xB }
        );
    }

    #[test]
    fn test_decode_FX15() {
        assert_eq!(
            Instruction::decode_from(0xF615).unwrap(),
            Instruction::OpFX15 { x: 0x6 }
        );
    }

    #[test]
    fn test_decode_FX18() {
        assert_eq!(
            Instruction::decode_from(0xFE18).unwrap(),
            Instruction::OpFX18 { x: 0xE }
        );
    }

    #[test]
    fn test_decode_FX1E() {
        assert_eq!(
            Instruction::decode_from(0xF51E).unwrap(),
            Instruction::OpFX1E { x: 0x5 }
        );
    }

    #[test]
    fn test_decode_FX0A() {
        assert_eq!(
            Instruction::decode_from(0xFC0A).unwrap(),
            Instruction::OpFX0A { x: 0xC }
        );
    }

    #[test]
    fn test_decode_FX29() {
        assert_eq!(
            Instruction::decode_from(0xF429).unwrap(),
            Instruction::OpFX29 { x: 0x4 }
        );
    }

    #[test]
    fn test_decode_FX30() {
        assert_eq!(
            Instruction::decode_from(0xF430).unwrap(),
            Instruction::OpFX30 { x: 0x4 }
        );
    }

    #[test]
    fn test_decode_FX33() {
        assert_eq!(
            Instruction::decode_from(0xFD33).unwrap(),
            Instruction::OpFX33 { x: 0xD }
        );
    }

    #[test]
    fn test_decode_FX55() {
        assert_eq!(
            Instruction::decode_from(0xF855).unwrap(),
            Instruction::OpFX55 { x: 0x8 }
        );
    }

    #[test]
    fn test_decode_FX65() {
        assert_eq!(
            Instruction::decode_from(0xFA65).unwrap(),
            Instruction::OpFX65 { x: 0xA }
        );
    }

    #[test]
    fn test_decode_FX75() {
        assert_eq!(
            Instruction::decode_from(0xFA75).unwrap(),
            Instruction::OpFX75 { x: 0xA }
        );
    }

    #[test]
    fn test_decode_FX85() {
        assert_eq!(
            Instruction::decode_from(0xFA85).unwrap(),
            Instruction::OpFX85 { x: 0xA }
        );
    }

    #[test]
    fn test_decode_unrecognised_opcode() {
        assert_eq!(
            Instruction::decode_from(0xFFFF).unwrap_err(),
            ErrorDetail::UnknownInstruction { opcode: 0xFFFF }
        );
    }
}
