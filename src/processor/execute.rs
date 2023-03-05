use super::*;

impl Processor {
    /// Executes the 004B instruction - [turn on COSMAC VIP display]
    /// Purpose: switch on COSMAC VIP display
    pub(super) fn execute_004B(&mut self) -> Result<u64, ErrorDetail> {
        // Execution time would be 48 cycles if implemented
        Err(ErrorDetail::UnimplementedInstruction { opcode: 0x004B })
    }

    /// Executes the 00CN instruction - SCD nibble
    /// Purpose: [SUPER-CHIP 1.1] scroll display N pixels down (N/2 in low-resolution mode)
    ///          [CHIP-8 / CHIP-48] this will error as an [ErrorDetail::UnknownInstruction]
    pub(super) fn execute_00CN(&mut self, n: u8) -> Result<u64, ErrorDetail> {
        match self.emulation_level {
            EmulationLevel::SuperChip11 => {
                self.frame_buffer.scroll_display_down(n)?;
                Ok(0)
            }
            EmulationLevel::Chip8 { .. } | EmulationLevel::Chip48 => {
                let opcode: u16 = 0x00C0 | (n as u16);
                Err(ErrorDetail::UnknownInstruction { opcode })
            }
        }
    }

    /// Executes the 00E0 instruction - CLS
    /// Purpose: clear the display
    pub(super) fn execute_00E0(&mut self) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 64;
        self.frame_buffer.clear();
        Ok(CYCLES)
    }

    /// Executes the 00EE instruction - RET
    /// Purpose: return from a subroutine
    pub(super) fn execute_00EE(&mut self) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 50;
        let address: u16 = self.stack.pop()?;
        self.program_counter = address;
        Ok(CYCLES)
    }

    /// Executes the 00FB instruction - SCR
    /// Purpose: [SUPER-CHIP 1.1] scroll right by 4 pixels (2 in low-resolution mode)
    ///          [CHIP-8 / CHIP-48] this will error as an [ErrorDetail::UnknownInstruction]
    pub(super) fn execute_00FB(&mut self) -> Result<u64, ErrorDetail> {
        match self.emulation_level {
            EmulationLevel::SuperChip11 => {
                self.frame_buffer.scroll_display_right()?;
                Ok(0)
            }
            EmulationLevel::Chip8 { .. } | EmulationLevel::Chip48 => {
                Err(ErrorDetail::UnknownInstruction { opcode: 0x00FB })
            }
        }
    }

    /// Executes the 00FC instruction - SCL
    /// Purpose: [SUPER-CHIP 1.1] scroll left by 4 pixels (2 in low-resolution mode)
    ///          [CHIP-8 / CHIP-48] this will error as an [ErrorDetail::UnknownInstruction]
    pub(super) fn execute_00FC(&mut self) -> Result<u64, ErrorDetail> {
        match self.emulation_level {
            EmulationLevel::SuperChip11 => {
                self.frame_buffer.scroll_display_left()?;
                Ok(0)
            }
            EmulationLevel::Chip8 { .. } | EmulationLevel::Chip48 => {
                Err(ErrorDetail::UnknownInstruction { opcode: 0x00FC })
            }
        }
    }

    /// Executes the 00FD instruction - EXIT
    /// Purpose: [SUPER-CHIP 1.1] exit the interpreter (set status to [ProcessorStatus::Complete])
    ///          [CHIP-8 / CHIP-48] this will error as an [ErrorDetail::UnknownInstruction]
    pub(super) fn execute_00FD(&mut self) -> Result<u64, ErrorDetail> {
        match self.emulation_level {
            EmulationLevel::SuperChip11 => {
                self.status = ProcessorStatus::Completed;
                Ok(0)
            }
            EmulationLevel::Chip8 { .. } | EmulationLevel::Chip48 => {
                Err(ErrorDetail::UnknownInstruction { opcode: 0x00FD })
            }
        }
    }

    /// Executes the 00FE instruction - LOW
    /// Purpose: [SUPER-CHIP 1.1] disable high-resolution mode
    ///          [CHIP-8 / CHIP-48] this will error as an [ErrorDetail::UnknownInstruction]
    pub(super) fn execute_00FE(&mut self) -> Result<u64, ErrorDetail> {
        match self.emulation_level {
            EmulationLevel::SuperChip11 => {
                self.high_resolution_mode = false;
                Ok(0)
            }
            EmulationLevel::Chip8 { .. } | EmulationLevel::Chip48 => {
                Err(ErrorDetail::UnknownInstruction { opcode: 0x00FE })
            }
        }
    }

    /// Executes the 00FF instruction - HIGH
    /// Purpose: [SUPER-CHIP 1.1] enable high-resolution mode
    ///          [CHIP-8 / CHIP-48] this will error as an [ErrorDetail::UnknownInstruction]
    pub(super) fn execute_00FF(&mut self) -> Result<u64, ErrorDetail> {
        match self.emulation_level {
            EmulationLevel::SuperChip11 => {
                self.high_resolution_mode = true;
                Ok(0)
            }
            EmulationLevel::Chip8 { .. } | EmulationLevel::Chip48 => {
                Err(ErrorDetail::UnknownInstruction { opcode: 0x00FF })
            }
        }
    }

    /// Executes the 0NNN instruction - SYS addr
    /// Purpose: jump to a machine code routine at NNN
    pub(super) fn execute_0NNN(&mut self, nnn: u16) -> Result<u64, ErrorDetail> {
        Err(ErrorDetail::UnimplementedInstruction { opcode: nnn })
    }

    /// Executes the 1NNN instruction - JP addr
    /// Purpose: jump to location NNN
    pub(super) fn execute_1NNN(&mut self, nnn: u16) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 80;
        self.program_counter = nnn;
        Ok(CYCLES)
    }

    /// Executes the 2NNN instruction - CALL addr
    /// Purpose: call subroutine at NNN
    pub(super) fn execute_2NNN(&mut self, nnn: u16) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 94;
        self.stack.push(self.program_counter)?;
        self.program_counter = nnn;
        Ok(CYCLES)
    }

    /// Executes the 3XNN instruction - SE Vx, byte
    /// Purpose: skip next instruction if Vx = NN
    pub(super) fn execute_3XNN(&mut self, x: usize, nn: u8) -> Result<u64, ErrorDetail> {
        const CYCLES_IF_TRUE: u64 = 82;
        const CYCLES_IF_FALSE: u64 = 78;
        if x >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        // Compare the value in register Vx to passed value NN
        if self.variable_registers[x] == nn {
            // If they are equal, increment the program counter by 2 bytes (1 opcode)
            self.program_counter += 2;
            Ok(CYCLES_IF_TRUE)
        } else {
            Ok(CYCLES_IF_FALSE)
        }
    }

    /// Executes the 4XNN instruction - SNE Vx, byte
    /// Purpose: skip next instruction if Vx != NN
    pub(super) fn execute_4XNN(&mut self, x: usize, nn: u8) -> Result<u64, ErrorDetail> {
        const CYCLES_IF_TRUE: u64 = 82;
        const CYCLES_IF_FALSE: u64 = 78;
        if x >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        // Compare the value in register Vx to passed value NN
        if self.variable_registers[x] != nn {
            // If they are not equal, increment the program counter by 2 bytes (1 opcode)
            self.program_counter += 2;
            Ok(CYCLES_IF_TRUE)
        } else {
            Ok(CYCLES_IF_FALSE)
        }
    }

    /// Executes the 5XY0 instruction - SE Vx, Vy
    /// Purpose: skip next instruction if Vx = Vy
    pub(super) fn execute_5XY0(&mut self, x: usize, y: usize) -> Result<u64, ErrorDetail> {
        const CYCLES_IF_TRUE: u64 = 86;
        const CYCLES_IF_FALSE: u64 = 82;
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            operands.insert("y".to_string(), y);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        // Compare the value in registers Vx and Vy
        if self.variable_registers[x] == self.variable_registers[y] {
            // If they are equal, increment the program counter by 2 bytes (1 opcode)
            self.program_counter += 2;
            Ok(CYCLES_IF_TRUE)
        } else {
            Ok(CYCLES_IF_FALSE)
        }
    }

    /// Executes the 6XNN instruction - LD Vx, byte
    /// Purpose: set Vx = NN
    pub(super) fn execute_6XNN(&mut self, x: usize, nn: u8) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 74;
        if x >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        self.variable_registers[x] = nn;
        Ok(CYCLES)
    }

    /// Executes the 7XNN instruction - ADD Vx, byte
    /// Purpose: set Vx = Vx + NN
    pub(super) fn execute_7XNN(&mut self, x: usize, nn: u8) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 78;
        if x >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        // Set Vx equal to itself plus NN
        self.variable_registers[x] =
            (((self.variable_registers[x] as u16) + (nn as u16)) & 0xFF) as u8;
        Ok(CYCLES)
    }

    /// Executes the 8XY0 instruction - LD Vx, Vy
    /// Purpose: set Vx = Vy
    pub(super) fn execute_8XY0(&mut self, x: usize, y: usize) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 80;
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            operands.insert("y".to_string(), y);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        self.variable_registers[x] = self.variable_registers[y];
        Ok(CYCLES)
    }

    /// Executes the 8XY1 instruction - OR Vx, Vy
    /// Purpose: set Vx = Vx | Vy (bitwise OR)
    pub(super) fn execute_8XY1(&mut self, x: usize, y: usize) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 112;
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            operands.insert("y".to_string(), y);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        // Set Vx = Vx | Vy
        self.variable_registers[x] = self.variable_registers[x] | self.variable_registers[y];
        if let EmulationLevel::Chip8 { .. } = self.emulation_level {
            self.variable_registers[0xF] = 0;
        }
        Ok(CYCLES)
    }

    /// Executes the 8XY2 instruction - AND Vx, Vy
    /// Purpose: set Vx = Vx & Vy (bitwise AND)
    pub(super) fn execute_8XY2(&mut self, x: usize, y: usize) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 112;
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            operands.insert("y".to_string(), y);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        // Set Vx = Vx & Vy
        self.variable_registers[x] = self.variable_registers[x] & self.variable_registers[y];
        if let EmulationLevel::Chip8 { .. } = self.emulation_level {
            self.variable_registers[0xF] = 0;
        }
        Ok(CYCLES)
    }

    /// Executes the 8XY3 instruction - XOR Vx, Vy
    /// Purpose: set Vx = Vx ^ Vy (bitwise XOR)
    pub(super) fn execute_8XY3(&mut self, x: usize, y: usize) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 112;
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            operands.insert("y".to_string(), y);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        // Set Vx = Vx ^ Vy
        self.variable_registers[x] = self.variable_registers[x] ^ self.variable_registers[y];
        if let EmulationLevel::Chip8 { .. } = self.emulation_level {
            self.variable_registers[0xF] = 0;
        }
        Ok(CYCLES)
    }

    /// Executes the 8XY4 instruction - ADD Vx, Vy
    /// Purpose: set Vx = Vx + Vy, set Vf = carry
    pub(super) fn execute_8XY4(&mut self, x: usize, y: usize) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 112;
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            operands.insert("y".to_string(), y);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        // Cast Vx and Vy as u16 (to allow overflow beyond u8 range), add, and store in temp variable
        let result: u16 = (self.variable_registers[x] as u16) + (self.variable_registers[y] as u16);
        // Check whether sum has overflowed beyond 8 bits; if so set Vf to 1 otherwise 0
        self.variable_registers[0xF] = match result > 0xFF {
            true => 1,
            false => 0,
        };
        // Save the low 8 bits of result to Vx
        self.variable_registers[x] = (result & 0xFF) as u8;
        Ok(CYCLES)
    }

    /// Executes the 8XY5 instruction - SUB Vx, Vy
    /// Purpose: set Vx = Vx - Vy, set Vf = NOT borrow
    pub(super) fn execute_8XY5(&mut self, x: usize, y: usize) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 112;
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            operands.insert("y".to_string(), y);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        // Cast Vx and Vy as i16 (to allow signed result), subtract, and store in temp variable
        let result: i16 = (self.variable_registers[x] as i16) - (self.variable_registers[y] as i16);
        // Check whether subtraction result is negative; if so set Vf to 0 otherwise 1
        self.variable_registers[0xF] = match result < 0x0 {
            true => 0,
            false => 1,
        };
        // Save the low 8 bits of result to Vx
        self.variable_registers[x] = (result & 0xFF) as u8;
        Ok(CYCLES)
    }

    /// Executes the 8XY6 instruction - SHR Vx {, Vy}
    /// Purpose: [CHIP-8] set Vx = Vy SHR 1, where SHR means bit-shift right
    ///          [CHIP-48 / SUPER-CHIP 1.1] set Vx = Vx SHR 1    
    pub(super) fn execute_8XY6(&mut self, x: usize, y: usize) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 112;
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            operands.insert("y".to_string(), y);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        match self.emulation_level {
            // CHIP-8 first sets Vx to Vy
            EmulationLevel::Chip8 { .. } => self.variable_registers[x] = self.variable_registers[y],
            // CHIP-48 and SUPER-CHIP 1.1 ignore Vy
            EmulationLevel::Chip48 | EmulationLevel::SuperChip11 => {}
        }
        // Check if least significant bit of Vx is 1; if so at the end we set Vf to 1 otherwise 0
        let flag_value: u8 = match self.variable_registers[x] & 0x01 == 0x01 {
            true => 1,
            false => 0,
        };
        // Bitshift the value in Vx right by one bit (i.e. divide Vx by 2) then re-assign to Vx
        self.variable_registers[x] = self.variable_registers[x] >> 1;
        self.variable_registers[0xF] = flag_value;
        Ok(CYCLES)
    }

    /// Executes the 8XY7 instruction - SUBN Vx, Vy
    /// Purpose: set Vx = Vy - Vx, set Vf = NOT borrow
    pub(super) fn execute_8XY7(&mut self, x: usize, y: usize) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 112;
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            operands.insert("y".to_string(), y);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        // Cast Vx and Vy as i16 (to allow signed result), subtract, and store in temp variable
        let result: i16 = (self.variable_registers[y] as i16) - (self.variable_registers[x] as i16);
        // Check whether subtraction result is negative; if so set Vf to 0 otherwise 1
        self.variable_registers[0xF] = match result < 0x0 {
            true => 0,
            false => 1,
        };
        // Save the low 8 bits of result to Vx
        self.variable_registers[x] = (result & 0xFF) as u8;
        Ok(CYCLES)
    }

    /// Executes the 8XYE instruction - SHL Vx {, Vy}    
    /// Purpose: [CHIP-8] set Vx = Vy SHL 1, where SHL means bit-shift left
    ///          [CHIP-48 / SUPER-CHIP 1.1] set Vx = Vx SHL 1  
    pub(super) fn execute_8XYE(&mut self, x: usize, y: usize) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 112;
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            operands.insert("y".to_string(), y);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        match self.emulation_level {
            // CHIP-8 first sets Vx to Vy
            EmulationLevel::Chip8 { .. } => self.variable_registers[x] = self.variable_registers[y],
            // CHIP-48 and SUPER-CHIP 1.1 ignore Vy
            EmulationLevel::Chip48 | EmulationLevel::SuperChip11 => {}
        }
        // Check if most significant bit of Vx is 1; if so at the end we set Vf to 1 otherwise 0
        let flag_value: u8 = match self.variable_registers[x] & 0x80 == 0x80 {
            true => 1,
            false => 0,
        };
        // Bitshift the value in Vx left by one bit (i.e. multiply Vx by 2) then assign to Vx
        self.variable_registers[x] = self.variable_registers[x] << 1;
        self.variable_registers[0xF] = flag_value;
        Ok(CYCLES)
    }

    /// Executes the 9XY0 instruction - SNE Vx, Vy
    /// Purpose: skip next instruction if Vx != Vy
    pub(super) fn execute_9XY0(&mut self, x: usize, y: usize) -> Result<u64, ErrorDetail> {
        const CYCLES_IF_TRUE: u64 = 86;
        const CYCLES_IF_FALSE: u64 = 82;
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            operands.insert("y".to_string(), y);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        } else if self.variable_registers[x] != self.variable_registers[y] {
            // Compare the value in registers Vx and Vy.  If they are not equal, increment the
            // program counter by 2 bytes (1 opcode)
            self.program_counter += 2;
            Ok(CYCLES_IF_TRUE)
        } else {
            Ok(CYCLES_IF_FALSE)
        }
    }

    /// Executes the ANNN instruction - LD I, addr
    /// Purpose: set I = NNN
    pub(super) fn execute_ANNN(&mut self, nnn: u16) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 80;
        self.index_register = nnn;
        Ok(CYCLES)
    }

    /// Executes the BNNN instruction - JP V0, addr
    /// Purpose: [CHIP-8] jump to location NNN + V0
    ///          [CHIP-48 / SUPER-CHIP 1.1] jump to location xNN + Vx   
    pub(super) fn execute_BNNN(&mut self, nnn: u16) -> Result<u64, ErrorDetail> {
        const CYCLES_IF_PAGE_CROSSED: u64 = 92;
        const CYCLES_IF_PAGE_NOT_CROSSED: u64 = 90;
        // Check if the jump is across page boundaries, by comparing the 3rd least significant
        // nibble of the jump address and current program counters
        let page_boundary_crossed: bool =
            ((nnn + (self.variable_registers[0] as u16)) & 0xF00) != (self.program_counter & 0xF00);
        self.program_counter = match self.emulation_level {
            EmulationLevel::Chip8 { .. } => {
                // Set the program counter to NNN plus the value in register V0
                nnn + (self.variable_registers[0] as u16)
            }
            EmulationLevel::Chip48 | EmulationLevel::SuperChip11 => {
                // isolate the first hex digit
                let x: u16 = (nnn & 0x0F00) >> 8;
                // Set the program counter to XNN plus the value in register VX
                nnn + (self.variable_registers[x as usize] as u16)
            }
        };
        if page_boundary_crossed {
            Ok(CYCLES_IF_PAGE_CROSSED)
        } else {
            Ok(CYCLES_IF_PAGE_NOT_CROSSED)
        }
    }

    /// Executes the CXNN instruction - RND Vx, byte
    /// Purpose: set Vx = random byte & NN (bitwise AND)
    pub(super) fn execute_CXNN(&mut self, x: usize, nn: u8) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 104;
        if x >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        // Generate a random u8 value and store in temp variable
        let mut rng = rand::thread_rng();
        let rand: u8 = rng.gen();
        // Set Vx = bitwise AND of value NN and random value
        self.variable_registers[x] = nn & rand;
        Ok(CYCLES)
    }

    /// Executes the DXYN instruction - DRW Vx, Vy, nibble
    /// Purpose: display the N-byte sprite starting at memory location I at display
    /// coordinate (Vx, Vy)
    ///          [CHIP-8 / CHIP-48] set Vf = 1 if collision
    ///          [SUPER-CHIP 1.1] separate implementation for higher resolution mode:
    ///                           special implementation of DXY0 also set Vf = n where
    ///                           n is rows that collide or clip screen bottom
    pub(super) fn execute_DXYN(&mut self, x: usize, y: usize, n: u8) -> Result<u64, ErrorDetail> {
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT || n > MAX_SPRITE_HEIGHT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            operands.insert("y".to_string(), y);
            operands.insert("n".to_string(), n as usize);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        match self.emulation_level {
            EmulationLevel::Chip8 { .. } | EmulationLevel::Chip48 => {
                self.execute_DXYN_chip8(x, y, n) // delegate to standard CHIP-8 method
            }
            EmulationLevel::SuperChip11 => {
                match (self.high_resolution_mode, n) {
                    (true, 0) => self.execute_DXY0_superchip11(x, y), // special behaviour where n = 0
                    (true, ..) => self.execute_DXYN_chip8(x, y, n), // delegate to standard CHIP-8 method
                    (false, ..) => self.execute_DXYN_superchip11_low_res(x, y, n),
                }
            }
        }
    }

    // Private function to execute DXYN for CHIP-8 / CHIP-48 emulation level
    fn execute_DXYN_chip8(&mut self, x: usize, y: usize, n: u8) -> Result<u64, ErrorDetail> {
        // Base timing is decode time plus lowest possible execute and lowest possible idle
        const BASE_CYCLES: u64 = 68 + 170 + 2355;
        const MAX_EXTRA_EXECUTE_CYCLES: u64 = 3812 - 170;
        const MAX_EXTRA_IDLE_CYCLES: u64 = 3666 - 2355;
        // Read the sprite to draw as an N-byte array slice at memory location
        // pointed to by the index register
        let sprite: &[u8] = self
            .memory
            .read_bytes(self.index_register as usize, n as usize)?;
        // Call into the Chipolata display to draw this sprite at location (Vx, Vy),
        // storing the results (i.e. collision and clip row counts) in temp variables
        let (rows_with_collisions, rows_clipped) = self.frame_buffer.draw_sprite(
            self.variable_registers[x] as usize,
            self.variable_registers[y] as usize,
            sprite,
            false,
        )?;
        // If in high-resolution mode for SUPER-CHIP 1.1 emulation level, set Vf to the number
        // of rows that either underwent collision or were clipped off the bottom of the screen
        // Otherwise, set Vf to 1 if collision occurred in at least one row, and 0 if it did not.
        self.variable_registers[0xF] = match (self.emulation_level, self.high_resolution_mode) {
            (EmulationLevel::SuperChip11, true) => rows_with_collisions + rows_clipped,
            _ => {
                if rows_with_collisions > 0 {
                    0x1 // collisions occurred
                } else {
                    0x0 // collisions did not occur
                }
            }
        };
        // Now calculate a randomised cycle execution value within possible range
        let mut rng = rand::thread_rng();
        Ok(BASE_CYCLES
            + rng.gen_range(0..=MAX_EXTRA_EXECUTE_CYCLES)
            + rng.gen_range(0..=MAX_EXTRA_IDLE_CYCLES))
    }

    // Private function to execute low-DXYN for SUPER-CHIP 1.1 emulation level
    fn execute_DXYN_superchip11_low_res(
        &mut self,
        x: usize,
        y: usize,
        n: u8,
    ) -> Result<u64, ErrorDetail> {
        // To simulate low-resolution mode whilst at the SUPER-CHIP 1.1 emulation level we use the
        // normal display draw_sprite() method, but must explode every pixel to a 2x2 pixel.
        // First get the low-resolution sprite like normal
        let sprite: &[u8] = self
            .memory
            .read_bytes(self.index_register as usize, n as usize)?;
        // Now declare two vectors to represent the left and right portions of the high-res sprite
        let mut sprite_left: Vec<u8> = Vec::new();
        let mut sprite_right: Vec<u8> = Vec::new();
        // Iterate through each byte in the original sprite, duplicating bits in each row and assigning
        // the two new bytes in each case to left and right sprite vector accordingly. Add each value
        // to the new sprite vectors TWICE, as we are creating two rows per original row (2x2)
        for byte in sprite {
            let (left_byte, right_byte) = Processor::duplicate_bits(*byte);
            sprite_left.push(left_byte);
            sprite_left.push(left_byte);
            sprite_right.push(right_byte);
            sprite_right.push(right_byte);
        }
        // Now draw each of these two new sprites in turn at twice the specified X and Y coords
        // The right-hand sprite should start 8 pixels further right i.e. X + 8
        let (rows_with_collisions_left, _) = self.frame_buffer.draw_sprite(
            self.variable_registers[x] as usize * 2,
            self.variable_registers[y] as usize * 2,
            &sprite_left,
            false,
        )?;
        // We cannot draw the right-hand sprite if it will wrap; instead we must clip
        let mut rows_with_collisions_right: u8 = 0;
        if ((self.variable_registers[x] as usize * 2 / 8) + 1)
            % self.frame_buffer.get_row_size_bytes()
            != 0
        {
            (rows_with_collisions_right, _) = self.frame_buffer.draw_sprite(
                (self.variable_registers[x] as usize * 2) + 8,
                self.variable_registers[y] as usize * 2,
                &sprite_right,
                false,
            )?;
        }
        // Finally, set Vf according to whether any collisions occurred
        self.variable_registers[0xF] =
            match rows_with_collisions_left + rows_with_collisions_right > 0 {
                true => 0x1,
                false => 0x0,
            };
        Ok(0)
    }

    // Helper function that takes a byte and duplicates each bit next to the original bit,
    // returning the results as two new bytes
    pub(crate) fn duplicate_bits(byte: u8) -> (u8, u8) {
        let mut y: u16 = byte as u16;
        // Black magic bit manipulation ensues ...
        y = (y | (y << 4)) & 0x0F0F;
        y = (y | (y << 2)) & 0x3333;
        y = (y | (y << 1)) & 0x5555;
        y = y | (y << 1);
        ((y >> 8) as u8, (y & 0xFF) as u8)
    }

    // Private function to execute DXY0 for SUPER-CHIP 1.1 emulation level (draws a 2-byte wide by 16-byte
    // high sprite, instead of the usual 1*N sprite)
    fn execute_DXY0_superchip11(&mut self, x: usize, y: usize) -> Result<u64, ErrorDetail> {
        // Read the sprite to draw as a 32-byte array slice at memory location
        // pointed to by the index register
        let sprite: &[u8] = self.memory.read_bytes(self.index_register as usize, 32)?;
        let (rows_with_collisions, rows_clipped) = self.frame_buffer.draw_sprite(
            self.variable_registers[x] as usize,
            self.variable_registers[y] as usize,
            sprite,
            true,
        )?;
        // Set Vf to the number of rows that either underwent collision or were clipped off the bottom
        // of the screen
        self.variable_registers[0xF] = rows_with_collisions + rows_clipped;
        Ok(0)
    }

    /// Executes the EX9E instruction - SKP Vx
    /// Purpose: skip next instruction if the key with value Vx is pressed
    pub(super) fn execute_EX9E(&mut self, x: usize) -> Result<u64, ErrorDetail> {
        const CYCLES_IF_TRUE: u64 = 86;
        const CYCLES_IF_FALSE: u64 = 82;
        if x >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        let key: u8 = self.variable_registers[x]; // get the value stored in Vx
                                                  // Check whether the current keystate indicates the corresponding key is pressed
        let key_pressed: bool = self.keystate.is_key_pressed(key)?;
        if key_pressed {
            // If so, increment the program counter by 2 bytes (1 opcode)
            self.program_counter += 2;
            self.keystate.set_key_status(key, false)?; // Set key status to unpressed to prevent immediate repeats
            Ok(CYCLES_IF_TRUE)
        } else {
            Ok(CYCLES_IF_FALSE)
        }
    }

    /// Executes the EXA1 instruction - SKNP Vx
    /// Purpose: skip next instruction if the key with value Vx is not pressed
    pub(super) fn execute_EXA1(&mut self, x: usize) -> Result<u64, ErrorDetail> {
        const CYCLES_IF_TRUE: u64 = 86;
        const CYCLES_IF_FALSE: u64 = 82;
        if x >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        let key: u8 = self.variable_registers[x]; // get the value stored in Vx
                                                  // Check whether the current keystate indicates the corresponding key is pressed
        let key_pressed: bool = self.keystate.is_key_pressed(key)?;
        if !key_pressed {
            // If not, increment the program counter by 2 bytes (1 opcode)
            self.program_counter += 2;
            Ok(CYCLES_IF_TRUE)
        } else {
            self.keystate.set_key_status(key, false)?; // Set key status to unpressed to prevent immediate repeats
            Ok(CYCLES_IF_FALSE)
        }
    }

    /// Executes the FX07 instruction - LD Vx, DT
    /// Purpose: set Vx = delay timer value
    pub(super) fn execute_FX07(&mut self, x: usize) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 78;
        if x >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        self.variable_registers[x] = self.delay_timer;
        Ok(CYCLES)
    }

    /// Executes the FX0A instruction - LD Vx, K
    /// Purpose: wait for a key press, store the key value in Vx
    pub(super) fn execute_FX0A(&mut self, x: usize) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 19072;
        if x >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        // Check whether any keys are currently pressed
        match self.keystate.get_keys_pressed() {
            Some(keys_pressed) => {
                // Store the (first) pressed key value in Vx
                self.variable_registers[x] = keys_pressed[0];
                self.status = ProcessorStatus::Running; // ensure processor state is "Running"
            }
            None => {
                // Decrement the program counter by by 2 bytes (1 opcode)
                // i.e. keep repeating this instruction until a key press occurs
                self.program_counter -= 2;
                // Set processor state to "Waiting"
                self.status = ProcessorStatus::WaitingForKeypress;
            }
        }
        Ok(CYCLES)
    }

    /// Executes the FX15 instruction - LD DT, Vx
    /// Purpose: set delay timer = Vx
    pub(super) fn execute_FX15(&mut self, x: usize) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 78;
        if x >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        self.delay_timer = self.variable_registers[x];
        Ok(CYCLES)
    }

    /// Executes the FX18 instruction - LD ST, Vx
    /// Purpose: set sound timer = Vx
    pub(super) fn execute_FX18(&mut self, x: usize) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 78;
        if x >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        self.sound_timer = self.variable_registers[x];
        Ok(CYCLES)
    }

    /// Executes the FX1E instruction - ADD I, Vx
    /// Purpose: set I = I + Vx.  Set Vf to 1 if result outside addressable memory
    pub(super) fn execute_FX1E(&mut self, x: usize) -> Result<u64, ErrorDetail> {
        const CYCLES_IF_PAGE_CROSSED: u64 = 92;
        const CYCLES_IF_PAGE_NOT_CROSSED: u64 = 84;
        if x < VARIABLE_REGISTER_COUNT {
            // Cast Vx and I as u32 (to allow overflow beyond u16 range), add, and store in temp variable
            let result: u32 = (self.index_register as u32) + (self.variable_registers[x] as u32);
            if result <= 0xFFFF {
                // if result is outside u16 range then fall through to return error
                // Check if result is outside addressable memory space and set Vf to 1 if so, 0 otherwise
                self.variable_registers[0xF] =
                    match result <= (self.memory.max_addressable_size() as u32) {
                        true => 0,
                        false => 1,
                    };
                // Check if the jump is across page boundaries, by comparing the 3rd least significant
                // nibble of the jump address and current program counters
                let page_boundary_crossed: bool =
                    (result as u16 & 0xF00) != (self.index_register & 0xF00);
                self.index_register = result as u16;
                if page_boundary_crossed {
                    return Ok(CYCLES_IF_PAGE_CROSSED);
                } else {
                    return Ok(CYCLES_IF_PAGE_NOT_CROSSED);
                }
            }
        }
        let mut operands: HashMap<String, usize> = HashMap::new();
        operands.insert("x".to_string(), x);
        return Err(ErrorDetail::OperandsOutOfBounds { operands });
    }

    /// Executes the FX29 instruction - LD F, Vx
    /// Purpose: set I = location of font sprite for digit Vx
    pub(super) fn execute_FX29(&mut self, x: usize) -> Result<u64, ErrorDetail> {
        const CYCLES: u64 = 88;
        if x >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        // Fetch the character hex code in Vx and check it is within expected bounds
        let character = self.variable_registers[x];
        let font: &Font = &self.low_resolution_font;
        if character >= (font.font_data_size() / font.char_size()) as u8 {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("character".to_string(), character as usize);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        // Calculate the corresponding font sprite location in memory based on the size per font
        // character (in bytes), the starting location of font data in memory, and the offset of
        // the requested character's ordinal within the range of font characters
        let character_memory_location: usize =
            (character as usize) * font.char_size() + self.font_start_address;
        self.index_register = character_memory_location as u16;
        Ok(CYCLES)
    }

    /// Executes the FX30 instruction - LD HF, Vx
    /// Purpose: [SUPER-CHIP 1.1] point I to 10-byte font sprite for digit Vx
    ///          [CHIP-8 / CHIP-48] this will error as an [ErrorDetail::UnknownInstruction]
    pub(super) fn execute_FX30(&mut self, x: usize) -> Result<u64, ErrorDetail> {
        match self.emulation_level {
            EmulationLevel::SuperChip11 => {
                if x >= VARIABLE_REGISTER_COUNT {
                    let mut operands: HashMap<String, usize> = HashMap::new();
                    operands.insert("x".to_string(), x);
                    return Err(ErrorDetail::OperandsOutOfBounds { operands });
                }
                // Fetch the character hex code in Vx and check it is within expected bounds
                let character = self.variable_registers[x];
                let font: &Font = self.high_resolution_font.as_ref().unwrap();
                if character >= (font.font_data_size() / font.char_size()) as u8 {
                    let mut operands: HashMap<String, usize> = HashMap::new();
                    operands.insert("character".to_string(), character as usize);
                    return Err(ErrorDetail::OperandsOutOfBounds { operands });
                }
                // Calculate the corresponding font sprite location in memory based on the size per font
                // character (in bytes), the starting location of font data in memory, and the offset of
                // the requested character's ordinal within the range of font characters
                let character_memory_location: usize = (character as usize) * font.char_size()
                    + self.high_resolution_font_start_address;
                self.index_register = character_memory_location as u16;
                Ok(0)
            }
            EmulationLevel::Chip8 { .. } | EmulationLevel::Chip48 => {
                let opcode: u16 = 0xF030 | ((x as u16) << 8);
                Err(ErrorDetail::UnknownInstruction { opcode })
            }
        }
    }

    /// Executes the FX33 instruction - LD V, Vx
    /// Purpose: converts value in Vx to decimal, and stores the digits in memory locations I, I+1 and I+2
    pub(super) fn execute_FX33(&mut self, x: usize) -> Result<u64, ErrorDetail> {
        const CYCLES_BASE: u64 = 152;
        const CYCLES_INCREMENTAL: u64 = 16;
        if x >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        let hex_number: u8 = self.variable_registers[x]; // get the hex value in Vx
        let decimal_first_digit: u8 = hex_number / 100; // get the "hundreds" decimal digit
        let decimal_second_digit: u8 = (hex_number % 100) / 10; // get the "tens" decimal digit
        let decimal_third_digit: u8 = hex_number % 10; // get the "units" decimal digit
        let index: usize = self.index_register as usize; // get the memory address in the index register
        self.memory.write_byte(index, decimal_first_digit)?; // store the first digit at this address
        self.memory.write_byte(index + 1, decimal_second_digit)?; // store the second digit at the next address
        self.memory.write_byte(index + 2, decimal_third_digit)?; // store the third digit at the next address
        let digit_sum: u64 =
            (decimal_first_digit + decimal_second_digit + decimal_third_digit) as u64;
        // Timing is calculated as base amount plus an increment multiplied by the sum of all digits
        Ok(CYCLES_BASE + (CYCLES_INCREMENTAL * digit_sum))
    }

    /// Executes the FX55 instruction - LD [I], Vx
    /// Purpose: store registers V0 to Vx in memory starting at the address in I   
    ///          [CHIP-8] also set I to I + x + 1
    ///          [CHIP-48] also set I to I + x
    ///          [SUPER-CHIP 1.1] do not modify I    
    pub(super) fn execute_FX55(&mut self, x: usize) -> Result<u64, ErrorDetail> {
        const CYCLES_BASE: u64 = 86;
        const CYCLES_INCREMENTAL: u64 = 14;
        if x >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        let original_index_register: usize = self.index_register as usize;
        match self.emulation_level {
            EmulationLevel::Chip8 { .. } => {
                // Original CHIP-8 behaviour incremented index register after each assignment
                self.index_register = (original_index_register + x + 1) as u16;
            }
            EmulationLevel::Chip48 => {
                // CHIP-48 increments index register by one less than it should
                self.index_register = (original_index_register + x) as u16;
            }
            EmulationLevel::SuperChip11 => {
                // SUPER-CHIP 1.1 does not increment the index register at all; do nothing here
            }
        }
        // Construct an appropriate array slice from the variable register array and write to memory
        self.memory
            .write_bytes(original_index_register, &self.variable_registers[0..x + 1])?;
        let variable_count: u64 = (x + 1) as u64;
        // Timing is calculated as base amount plus an increment multiplied by every variable stored
        Ok(CYCLES_BASE + (CYCLES_INCREMENTAL * variable_count))
    }

    /// Executes the FX65 instruction - LD Vx, [I]
    /// Purpose: populate registers V0 to Vx from memory starting at the address in I
    ///          [CHIP-8] also set I to I + x + 1
    ///          [CHIP-48] also set I to I + x
    ///          [SUPER-CHIP 1.1] do not modify I
    pub(super) fn execute_FX65(&mut self, x: usize) -> Result<u64, ErrorDetail> {
        const CYCLES_BASE: u64 = 86;
        const CYCLES_INCREMENTAL: u64 = 14;
        if x >= VARIABLE_REGISTER_COUNT {
            let mut operands: HashMap<String, usize> = HashMap::new();
            operands.insert("x".to_string(), x);
            return Err(ErrorDetail::OperandsOutOfBounds { operands });
        }
        let original_index_register: usize = self.index_register as usize;
        match self.emulation_level {
            EmulationLevel::Chip8 { .. } => {
                // Original CHIP-8 behaviour incremented index register after each assignment
                self.index_register = (original_index_register + x + 1) as u16;
            }
            EmulationLevel::Chip48 => {
                // CHIP-48 increments index register by one less than it should
                self.index_register = (original_index_register + x) as u16;
            }
            EmulationLevel::SuperChip11 => {
                // SUPER-CHIP 1.1 does not increment the index register at all; do nothing here
            }
        }
        // Iterate through the appropriate portion of the variable register array
        for i in 0..(x + 1) {
            // Set the new value by reading the appropriate byte from memory
            self.variable_registers[i] =
                self.memory.read_byte(original_index_register + i).unwrap();
        }
        let variable_count: u64 = (x + 1) as u64;
        // Timing is calculated as base amount plus an increment multiplied by every variable stored
        Ok(CYCLES_BASE + (CYCLES_INCREMENTAL * variable_count))
    }

    /// Executes the FX75 instruction - LD R, Vx
    /// Purpose: [SUPER-CHIP 1.1] store registers V0 to Vx in RPL user flags starting at address in I
    ///          [CHIP-8 / CHIP-48] this will error as an [ErrorDetail::UnknownInstruction]
    pub(super) fn execute_FX75(&mut self, x: usize) -> Result<u64, ErrorDetail> {
        match self.emulation_level {
            EmulationLevel::SuperChip11 => {
                if x >= RPL_REGISTER_COUNT {
                    let mut operands: HashMap<String, usize> = HashMap::new();
                    operands.insert("x".to_string(), x);
                    return Err(ErrorDetail::OperandsOutOfBounds { operands });
                }
                // Iterate through the appropriate portion of the variable register array
                self.rpl_registers[0..=x].copy_from_slice(&self.variable_registers[0..=x]);
                Ok(0)
            }
            EmulationLevel::Chip8 { .. } | EmulationLevel::Chip48 => {
                let opcode: u16 = 0xF075 | ((x as u16) << 8);
                Err(ErrorDetail::UnknownInstruction { opcode })
            }
        }
    }

    /// Executes the FX85 instruction - LD Vx, R
    /// Purpose: [SUPER-CHIP 1.1] populate registers V0 to Vx from RPL user flags starting at address in I
    ///          [CHIP-8 / CHIP-48] this will error as an [ErrorDetail::UnknownInstruction]
    pub(super) fn execute_FX85(&mut self, x: usize) -> Result<u64, ErrorDetail> {
        match self.emulation_level {
            EmulationLevel::SuperChip11 => {
                if x >= RPL_REGISTER_COUNT {
                    let mut operands: HashMap<String, usize> = HashMap::new();
                    operands.insert("x".to_string(), x);
                    return Err(ErrorDetail::OperandsOutOfBounds { operands });
                }
                // Iterate through the appropriate portion of the variable register array
                self.variable_registers[0..=x].copy_from_slice(&self.rpl_registers[0..=x]);
                Ok(0)
            }
            EmulationLevel::Chip8 { .. } | EmulationLevel::Chip48 => {
                let opcode: u16 = 0xF085 | ((x as u16) << 8);
                Err(ErrorDetail::UnknownInstruction { opcode })
            }
        }
    }
}
