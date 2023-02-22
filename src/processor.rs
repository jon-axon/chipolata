#![allow(non_snake_case)]

use super::display::Display;
use super::error::Error;
use super::font::Font;
use super::instruction::Instruction;
use super::keystate::KeyState;
use super::memory::Memory;
use super::options::Options;
use super::program::Program;
use super::stack::Stack;
use rand::Rng;
use std::time::{Duration, Instant};

#[cfg(test)]
mod tests;

/// The default CHIP-8 font start address within memory.
const DEFAULT_FONT_ADDRESS: usize = 0x50;
/// The number of ms that should pass inbetween decrements of delay and sound timers.
const TIMER_DECREMENT_INTERVAL_MICROSECONDS: u128 = 16667;
/// The number of variable registers available.
const VARIABLE_REGISTER_COUNT: usize = 16;
/// The maximum sprite height (pixels).
const MAX_SPRITE_HEIGHT: u8 = 15;
/// The number of font sprites.
const FONT_SPRITE_COUNT: u8 = 15;

/// An enum to indicate which extension of CHIP-8 is to be emulated.  See external
/// documentation for details of the differences in each case.
#[derive(Copy, Clone)]
pub enum EmulationLevel {
    /// The original CHIP-8 interpreter for the RCA COSMAC VIP
    Chip8,
    /// Re-implemented CHIP-8 interpreter for the HP48 graphing calculators
    Chip48,
    /// Version 1.1 of the SUPER-CHIP interpreter for HP48S and HP48SX graphing calculators
    SuperChip11,
}

/// An enum used internally within the Chipolata crate to keep track of the processor
/// execution status.
#[derive(Debug, PartialEq)]
enum ProcessorStatus {
    /// The processor has been instantiated but memory is empty
    StartingUp,
    /// The processor has been instantiated and font data loaded
    Initialised,
    /// A program has been loaded into the processor's memory
    ProgramLoaded,
    /// The program is being executed (the decode->fetch->execute cycle has begun)
    Running,
    /// The processor is stalled waiting for a keypress (instruction FX0A)
    WaitingForKeypress,
    /// The processor is in an error state, having generated an `Error`
    Crashed,
}

/// An enum used to indicate which variant of [StateSnapshot] should be returned when a call is
/// made to [Processor::export_state_snapshot()].
pub enum StateSnapshotVerbosity {
    /// Only the frame buffer state will be reported
    Minimal,
    /// The frame buffer, registers, stack and memory state will all be reported
    Extended,
}

/// An enum with variants representing the different Chipolata state snapshots that can be
/// returned to hosting applications for processing
pub enum StateSnapshot {
    /// Minimal snapshot containing only the frame buffer state
    MinimalSnapshot { frame_buffer: Display },
    /// Extended snapshot containing the frame buffer state along with all registers,
    /// stack and memory
    ExtendedSnapshot {
        frame_buffer: Display,
        stack: Stack,
        memory: Memory,
        program_counter: u16,
        index_register: u16,
        variable_registers: [u8; VARIABLE_REGISTER_COUNT],
        delay_timer: u8,
        sound_timer: u8,
        cycles: usize,
    },
}

/// An abstraction of the CHIP-8 processor, and the core public interface to the Chipolata crate.
///
/// This struct holds representations of all CHIP-8 sub-components, and exposes methods through which
/// a program can be loaded to memory and executed one cycle at a time, as well as methods for
/// supplying input to the processor (in the form of keypresses) and output to the host application
/// (in the form of a bitmapped display).
pub struct Processor {
    // CHIP-8 COMPONENT STATE FIELDS
    frame_buffer: Display, // The display frame buffer
    stack: Stack,          // The call stack (holds return addresses for subroutines)
    memory: Memory,        // The system memory
    program_counter: u16, // The program counter register (points to next opcode location in memory)
    index_register: u16,  // The index counter register (used to point to memory addresses)
    variable_registers: [u8; VARIABLE_REGISTER_COUNT], // General purposes registers
    delay_timer: u8,      // Delay timer, decrements automatically at 60hz when non-zero
    sound_timer: u8,      // Sounds timer, decrements automatically at 60hz when non-zero
    cycles: usize,        // The number of processor cycles that have been executed
    // ADDITIONAL STATE FIELDS
    keystate: KeyState, // A representation of the state (pressed/not pressed) of each key
    status: ProcessorStatus, // The current execution status of the processor
    last_timer_decrement: Instant, //  The moment the delay and sound timers were last decremented
    last_execution_cycle_complete: Instant, // The moment the execute cycle was last completed
    // CONFIG AND SETUP FIELDS
    font: Font, // The font loaded into the processor (only used during initialisation)
    program: Program, // The program loaded into the processor (only used during initialisation)
    font_start_address: usize, // The start address in memory at which the font is loaded
    program_start_address: usize, // The start address in memory at which the program is loaded
    processor_speed_hertz: u64, // Used to calculate the time between execute cycles
    emulation_level: EmulationLevel, // Component and instruction-compatibility configuration
}

impl Processor {
    /// Constructor/builder function that returns a freshly-initialised [Processor] instance
    /// with the supplied program data loaded into memory ready for execution.
    ///
    /// # Arguments
    ///
    /// * `program` - a [Program] instance holding the bytes of the ROM to be executed
    /// * `options` - an [Options] instance holding Chipolata start-up configuration information
    pub fn initialise_and_load(program: Program, options: Options) -> Result<Self, Error> {
        let mut processor = Processor {
            frame_buffer: Display::new(),
            stack: Stack::new(options.emulation_level),
            memory: Memory::new(options.emulation_level),
            program_counter: options.program_start_address,
            index_register: 0x0,
            variable_registers: [0x0; VARIABLE_REGISTER_COUNT],
            delay_timer: 0x0,
            sound_timer: 0x0,
            cycles: 0,
            keystate: KeyState::new(),
            status: ProcessorStatus::StartingUp,
            last_timer_decrement: Instant::now(),
            last_execution_cycle_complete: Instant::now(),
            font: Font::default(),
            program: program,
            font_start_address: DEFAULT_FONT_ADDRESS,
            program_start_address: options.program_start_address as usize,
            processor_speed_hertz: options.processor_speed_hertz,
            emulation_level: options.emulation_level,
        };
        processor.load_font_data()?;
        processor.status = ProcessorStatus::Initialised;
        processor.load_program()?;
        processor.status = ProcessorStatus::ProgramLoaded;
        Ok(processor)
    }

    /// Sets the current processor speed in hertz
    ///
    /// # Arguments
    ///
    /// * `speed_hertz` - the new processor speed
    pub fn set_processor_speed(&mut self, speed_hertz: u64) {
        self.processor_speed_hertz = speed_hertz;
    }

    /// Returns the current processor speed in hertz
    pub fn processor_speed(&self) -> u64 {
        self.processor_speed_hertz
    }

    /// Returns a copy of the current state of Chipolata.
    ///
    /// The minimal level of state reporting returns just a copy of the [Display] frame buffer
    /// instance, from which the bitmapped [Display::pixels] 2-D array can be interrogated
    /// for rendering purposes.
    ///
    /// The extended level of state reporting returns a copy of the [Display] frame buffer instance
    /// in addition to a copy of all registers and timers, the [Stack] and the [Memory].
    ///
    /// # Arguments
    ///
    /// * `verbosity` - the amount of state that should be returned
    pub fn export_state_snapshot(
        &self,
        verbosity: StateSnapshotVerbosity,
    ) -> Result<StateSnapshot, Error> {
        match verbosity {
            StateSnapshotVerbosity::Minimal => Ok(StateSnapshot::MinimalSnapshot {
                frame_buffer: self.frame_buffer.clone(),
            }),
            StateSnapshotVerbosity::Extended => Ok(StateSnapshot::ExtendedSnapshot {
                frame_buffer: self.frame_buffer.clone(),
                stack: self.stack.clone(),
                memory: self.memory.clone(),
                program_counter: self.program_counter,
                index_register: self.index_register,
                variable_registers: self.variable_registers,
                delay_timer: self.delay_timer,
                sound_timer: self.sound_timer,
                cycles: self.cycles,
            }),
        }
    }

    /// Provides key press input to Chipolata, by setting the state of the specified key
    /// in the internal representation to pressed / not pressed as per supplied value.
    ///
    /// # Arguments
    ///
    /// * `key` - the hex ordinal of the key (valid range 0x0 to 0xF inclusive)
    /// * `status` - the value to set for the specified key (true means pressed)
    pub fn set_key_status(&mut self, key: u8, status: bool) -> Result<(), Error> {
        Ok(self.keystate.set_key_status(key, status)?)
    }

    /// Loads the processor's font data into memory.  If the size of the font data combined with
    /// the specified start location in memory would cause a write to unaddressable memory, then
    /// return an [Error::MemoryAddressOutOfBounds].
    fn load_font_data(&mut self) -> Result<(), Error> {
        //
        if self.font_start_address + self.font.font_data_size() >= self.program_start_address {
            return Err(Error::MemoryAddressOutOfBounds);
        }
        self.memory
            .write_bytes(self.font_start_address, self.font.font_data())?;
        Ok(())
    }

    /// Loads the processor's program data into memory.  If the size of the program data combined
    /// with the specified start location in memory would cause a write to unaddressable memory,
    /// then return an [Error::MemoryAddressOutOfBounds].
    fn load_program(&mut self) -> Result<(), Error> {
        if self.program_start_address + self.program.program_data_size()
            >= self.memory.max_addressable_size()
        {
            return Err(Error::MemoryAddressOutOfBounds);
        }
        self.memory
            .write_bytes(self.program_start_address, self.program.program_data())?;
        Ok(())
    }

    /// Executes one iteration of the Chipolata fetch -> decode -> execute cycle.  Returns a boolean
    /// indicating whether the display frame buffer was updated this cycle.
    pub fn execute_cycle(&mut self) -> Result<bool, Error> {
        // Set processor state to Running
        self.status = ProcessorStatus::Running;
        // Increment the cycles counter
        self.cycles += 1;
        // Decrement the delay and sound timers, if appropriate
        self.decrement_timers();
        // Fetch two byte opcode from current Program Counter memory location
        let opcode: u16 = match self.memory.read_two_bytes(self.program_counter as usize) {
            Ok(opcode) => opcode,
            Err(e) => {
                self.status = ProcessorStatus::Crashed;
                return Err(e);
            }
        };
        // Increment Program Counter (by two bytes, as we have 16-bit opcodes)
        self.program_counter += 0x2;
        // Decode the opcode into an instruction, setting processor state to Crashed on error
        let instruction: Instruction = match Instruction::decode_from(opcode) {
            Ok(instruction) => instruction,
            Err(e) => {
                self.status = ProcessorStatus::Crashed;
                return Err(e);
            }
        };
        // If the instruction is one that updates the display, set a local flag to true
        let display_updated: bool = match instruction {
            Instruction::Op00E0 => true,
            Instruction::OpDXYN { .. } => true,
            _ => false,
        };
        // Execute the instruction, setting processor state to Crashed on error
        if let Err(e) = self.execute(instruction) {
            self.status = ProcessorStatus::Crashed;
            return Err(e);
        }
        // In order to simulate the configured processor speed, we now spin until the appropriate
        // time has passed since the last cycle completed
        while self.last_execution_cycle_complete.elapsed()
            < Duration::from_micros(1_000_000_u64 / self.processor_speed_hertz)
        {
            // spin
        }
        self.last_execution_cycle_complete = Instant::now();
        // Return successfully, passing the flag indicating whether the display was updated this cycle
        return Ok(display_updated);
    }

    /// Checks if the required time has passed since the timers were last decremented
    /// and if so, decrements them
    fn decrement_timers(&mut self) {
        // Nothing to do unless timers are running
        if (self.delay_timer | self.sound_timer) > 0x0 {
            // Check how long it has been since the timers were last decremented; if the interval
            // is greater than the specified threshold then we should decrement again
            if self.last_timer_decrement.elapsed().as_micros()
                >= TIMER_DECREMENT_INTERVAL_MICROSECONDS
            {
                self.last_timer_decrement = Instant::now(); // update the stored decrement instant to now
                if self.delay_timer > 0x0 {
                    self.delay_timer -= 1;
                }
                if self.sound_timer > 0x0 {
                    self.sound_timer -= 1;
                }
            }
        }
    }

    /// Returns true if the sound timer is active i.e. if the hosting application should play audio
    pub fn sound_timer_active(&self) -> bool {
        match self.sound_timer {
            0 => false,
            _ => true,
        }
    }

    /// Executes the passed Instruction.  Returns [Error::UnimplementedInstruction] if Chipolata is
    /// unable to process opcodes of this type.
    ///
    /// # Arguments
    ///
    /// * `instr` - the instruction to be executed
    fn execute(&mut self, instr: Instruction) -> Result<(), Error> {
        match instr {
            Instruction::Op00E0 => self.execute_00E0(),
            Instruction::Op00EE => self.execute_00EE(),
            Instruction::Op0NNN { nnn } => self.execute_0NNN(nnn),
            Instruction::Op1NNN { nnn } => self.execute_1NNN(nnn),
            Instruction::Op2NNN { nnn } => self.execute_2NNN(nnn),
            Instruction::Op3XNN { x, nn } => self.execute_3XNN(x, nn),
            Instruction::Op4XNN { x, nn } => self.execute_4XNN(x, nn),
            Instruction::Op5XY0 { x, y } => self.execute_5XY0(x, y),
            Instruction::Op6XNN { x, nn } => self.execute_6XNN(x, nn),
            Instruction::Op7XNN { x, nn } => self.execute_7XNN(x, nn),
            Instruction::Op8XY0 { x, y } => self.execute_8XY0(x, y),
            Instruction::Op8XY1 { x, y } => self.execute_8XY1(x, y),
            Instruction::Op8XY2 { x, y } => self.execute_8XY2(x, y),
            Instruction::Op8XY3 { x, y } => self.execute_8XY3(x, y),
            Instruction::Op8XY4 { x, y } => self.execute_8XY4(x, y),
            Instruction::Op8XY5 { x, y } => self.execute_8XY5(x, y),
            Instruction::Op8XY6 { x, y } => self.execute_8XY6(x, y),
            Instruction::Op8XY7 { x, y } => self.execute_8XY7(x, y),
            Instruction::Op8XYE { x, y } => self.execute_8XYE(x, y),
            Instruction::Op9XY0 { x, y } => self.execute_9XY0(x, y),
            Instruction::OpANNN { nnn } => self.execute_ANNN(nnn),
            Instruction::OpBNNN { nnn } => self.execute_BNNN(nnn),
            Instruction::OpCXNN { x, nn } => self.execute_CXNN(x, nn),
            Instruction::OpDXYN { x, y, n } => self.execute_DXYN(x, y, n),
            Instruction::OpEX9E { x } => self.execute_EX9E(x),
            Instruction::OpEXA1 { x } => self.execute_EXA1(x),
            Instruction::OpFX07 { x } => self.execute_FX07(x),
            Instruction::OpFX15 { x } => self.execute_FX15(x),
            Instruction::OpFX18 { x } => self.execute_FX18(x),
            Instruction::OpFX1E { x } => self.execute_FX1E(x),
            Instruction::OpFX0A { x } => self.execute_FX0A(x),
            Instruction::OpFX29 { x } => self.execute_FX29(x),
            Instruction::OpFX33 { x } => self.execute_FX33(x),
            Instruction::OpFX55 { x } => self.execute_FX55(x),
            Instruction::OpFX65 { x } => self.execute_FX65(x),
        }
    }

    /// Executes the 00E0 instruction - CLS
    /// Purpose: clear the display
    fn execute_00E0(&mut self) -> Result<(), Error> {
        Ok(self.frame_buffer.clear())
    }

    /// Executes the 00EE instruction - RET
    /// Purpose: return from a subroutine
    fn execute_00EE(&mut self) -> Result<(), Error> {
        let address: u16 = self.stack.pop()?; // pop the top address off the stack
        Ok(self.program_counter = address) // assign this address to the program counter
    }

    /// Executes the 0NNN instruction - SYS addr
    /// Purpose: jump to a machine code routine at NNN
    fn execute_0NNN(&mut self, _nnn: u16) -> Result<(), Error> {
        Err(Error::UnimplementedInstruction)
    }

    /// Executes the 1NNN instruction - JP addr
    /// Purpose: jump to location NNN
    fn execute_1NNN(&mut self, nnn: u16) -> Result<(), Error> {
        Ok(self.program_counter = nnn) // set the program counter to address NNN
    }

    /// Executes the 2NNN instruction - CALL addr
    /// Purpose: call subroutine at NNN
    fn execute_2NNN(&mut self, nnn: u16) -> Result<(), Error> {
        self.stack.push(self.program_counter)?; // push current program location to stack
        Ok(self.program_counter = nnn) // set the program counter to address NNN
    }

    /// Executes the 3XNN instruction - SE Vx, byte
    /// Purpose: skip next instruction if Vx = NN
    fn execute_3XNN(&mut self, x: usize, nn: u8) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        // Compare the value in register Vx to passed value NN
        if self.variable_registers[x] == nn {
            // If they are equal, increment the program counter by 2 bytes (1 opcode)
            self.program_counter += 2;
        }
        Ok(())
    }

    /// Executes the 4XNN instruction - SNE Vx, byte
    /// Purpose: skip next instruction if Vx != NN
    fn execute_4XNN(&mut self, x: usize, nn: u8) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        // Compare the value in register Vx to passed value NN
        if self.variable_registers[x] != nn {
            // If they are not equal, increment the program counter by 2 bytes (1 opcode)
            self.program_counter += 2;
        }
        Ok(())
    }

    /// Executes the 5XY0 instruction - SE Vx, Vy
    /// Purpose: skip next instruction if Vx = Vy
    fn execute_5XY0(&mut self, x: usize, y: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        // Compare the value in registers Vx and Vy
        if self.variable_registers[x] == self.variable_registers[y] {
            // If they are equal, increment the program counter by 2 bytes (1 opcode)
            self.program_counter += 2;
        }
        Ok(())
    }

    /// Executes the 6XNN instruction - LD Vx, byte
    /// Purpose: set Vx = NN
    fn execute_6XNN(&mut self, x: usize, nn: u8) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        Ok(self.variable_registers[x] = nn) // Set Vx = NN
    }

    /// Executes the 7XNN instruction - ADD Vx, byte
    /// Purpose: set Vx = Vx + NN
    fn execute_7XNN(&mut self, x: usize, nn: u8) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        // Set Vx equal to itself plus NN
        Ok(self.variable_registers[x] =
            (((self.variable_registers[x] as u16) + (nn as u16)) & 0xFF) as u8)
    }

    /// Executes the 8XY0 instruction - LD Vx, Vy
    /// Purpose: set Vx = Vy
    fn execute_8XY0(&mut self, x: usize, y: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        Ok(self.variable_registers[x] = self.variable_registers[y]) // set Vx = Vy
    }

    /// Executes the 8XY1 instruction - OR Vx, Vy
    /// Purpose: set Vx = Vx | Vy (bitwise OR)
    fn execute_8XY1(&mut self, x: usize, y: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        // Set Vx = Vx | Vy
        Ok(self.variable_registers[x] = self.variable_registers[x] | self.variable_registers[y])
    }

    /// Executes the 8XY2 instruction - AND Vx, Vy
    /// Purpose: set Vx = Vx & Vy (bitwise AND)
    fn execute_8XY2(&mut self, x: usize, y: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        // Set Vx = Vx & Vy
        Ok(self.variable_registers[x] = self.variable_registers[x] & self.variable_registers[y])
    }

    /// Executes the 8XY3 instruction - XOR Vx, Vy
    /// Purpose: set Vx = Vx ^ Vy (bitwise XOR)
    fn execute_8XY3(&mut self, x: usize, y: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        // Set Vx = Vx ^ Vy
        Ok(self.variable_registers[x] = self.variable_registers[x] ^ self.variable_registers[y])
    }

    /// Executes the 8XY4 instruction - ADD Vx, Vy
    /// Purpose: set Vx = Vx + Vy, set Vf = carry
    fn execute_8XY4(&mut self, x: usize, y: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        // Cast Vx and Vy as u16 (to allow overflow beyond u8 range), add, and store in temp variable
        let result: u16 = (self.variable_registers[x] as u16) + (self.variable_registers[y] as u16);
        // Check whether sum has overflowed beyond 8 bits; if so set Vf to 1 otherwise 0
        self.variable_registers[0xF] = match result > 0xFF {
            true => 1,
            false => 0,
        };
        Ok(self.variable_registers[x] = (result & 0xFF) as u8) // Save the low 8 bits of result to Vx
    }

    /// Executes the 8XY5 instruction - SUB Vx, Vy
    /// Purpose: set Vx = Vx - Vy, set Vf = NOT borrow
    fn execute_8XY5(&mut self, x: usize, y: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        // Cast Vx and Vy as i16 (to allow signed result), subtract, and store in temp variable
        let result: i16 = (self.variable_registers[x] as i16) - (self.variable_registers[y] as i16);
        // Check whether subtraction result is negative; if so set Vf to 0 otherwise 1
        self.variable_registers[0xF] = match result < 0x0 {
            true => 0,
            false => 1,
        };
        Ok(self.variable_registers[x] = (result & 0xFF) as u8) // Save the low 8 bits of result to Vx
    }

    /// Executes the 8XY6 instruction - SHR Vx {, Vy}
    /// Purpose: [CHIP-8] set Vx = Vy SHR 1, where SHR means bit-shift right
    ///          [CHIP-48 / SUPER-CHIP 1.1] set Vx = Vx SHR 1    
    fn execute_8XY6(&mut self, x: usize, y: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        match self.emulation_level {
            // CHIP-8 first sets Vx to Vy
            EmulationLevel::Chip8 => self.variable_registers[x] = self.variable_registers[y],
            // CHIP-48 and SUPER-CHIP 1.1 ignore Vy
            EmulationLevel::Chip48 | EmulationLevel::SuperChip11 => {}
        }
        // Check if least significant bit of Vx is 1; if so set Vf to 1 otherwise 0
        self.variable_registers[0xF] = match self.variable_registers[x] & 0x01 == 0x01 {
            true => 1,
            false => 0,
        };
        // Bitshift the value in Vx right by one bit (i.e. divide Vx by 2) then re-assign to Vx
        Ok(self.variable_registers[x] = self.variable_registers[x] >> 1)
    }

    /// Executes the 8XY7 instruction - SUBN Vx, Vy
    /// Purpose: set Vx = Vy - Vx, set Vf = NOT borrow
    fn execute_8XY7(&mut self, x: usize, y: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        // Cast Vx and Vy as i16 (to allow signed result), subtract, and store in temp variable
        let result: i16 = (self.variable_registers[y] as i16) - (self.variable_registers[x] as i16);
        // Check whether subtraction result is negative; if so set Vf to 0 otherwise 1
        self.variable_registers[0xF] = match result < 0x0 {
            true => 0,
            false => 1,
        };
        Ok(self.variable_registers[x] = (result & 0xFF) as u8) // Save the low 8 bits of result to Vx
    }

    /// Executes the 8XYE instruction - SHL Vx {, Vy}    
    /// Purpose: [CHIP-8] set Vx = Vy SHL 1, where SHL means bit-shift left
    ///          [CHIP-48 / SUPER-CHIP 1.1] set Vx = Vx SHL 1  
    fn execute_8XYE(&mut self, x: usize, y: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        match self.emulation_level {
            // CHIP-8 first sets Vx to Vy
            EmulationLevel::Chip8 => self.variable_registers[x] = self.variable_registers[y],
            // CHIP-48 and SUPER-CHIP 1.1 ignore Vy
            EmulationLevel::Chip48 | EmulationLevel::SuperChip11 => {}
        }
        // Check if most significant bit of Vx is 1; if so set Vf to 1 otherwise 0
        self.variable_registers[0xF] = match self.variable_registers[x] & 0x80 == 0x80 {
            true => 1,
            false => 0,
        };
        // Bitshift the value in Vx left by one bit (i.e. multiply Vx by 2) then assign to Vx
        Ok(self.variable_registers[x] = self.variable_registers[x] << 1)
    }

    /// Executes the 9XY0 instruction - SNE Vx, Vy
    /// Purpose: skip next instruction if Vx != Vy
    fn execute_9XY0(&mut self, x: usize, y: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        } else if self.variable_registers[x] != self.variable_registers[y] {
            // Compare the value in registers Vx and Vy.  If they are not equal, increment the
            // program counter by 2 bytes (1 opcode)
            self.program_counter += 2;
        }
        Ok(())
    }

    /// Executes the ANNN instruction - LD I, addr
    /// Purpose: set I = NNN
    fn execute_ANNN(&mut self, nnn: u16) -> Result<(), Error> {
        Ok(self.index_register = nnn) // set the index register to address NNN
    }

    /// Executes the BNNN instruction - JP V0, addr
    /// Purpose: [CHIP-8] jump to location NNN + V0
    ///          [CHIP-48 / SUPER-CHIP 1.1] jump to location xNN + Vx   
    fn execute_BNNN(&mut self, nnn: u16) -> Result<(), Error> {
        match self.emulation_level {
            EmulationLevel::Chip8 => {
                // Set the program counter to NNN plus the value in register V0
                Ok(self.program_counter = nnn + (self.variable_registers[0] as u16))
            }
            EmulationLevel::Chip48 | EmulationLevel::SuperChip11 => {
                // isolate the first hex digit
                let x: u16 = (nnn & 0x0F00) >> 8;
                // Set the program counter to XNN plus the value in register VX
                Ok(self.program_counter = nnn + (self.variable_registers[x as usize] as u16))
            }
        }
    }

    /// Executes the CXNN instruction - RND Vx, byte
    /// Purpose: set Vx = random byte & NN (bitwise AND)
    fn execute_CXNN(&mut self, x: usize, nn: u8) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        // Generate a random u8 value and store in temp variable
        let mut rng = rand::thread_rng();
        let rand: u8 = rng.gen();
        // Set Vx = bitwise AND of value NN and random value
        Ok(self.variable_registers[x] = nn & rand)
    }

    /// Executes the DXYN instruction - DRW Vx, Vy, nibble
    /// Purpose: display the N-byte sprite starting at memory location I at display
    /// coordinate (Vx, Vy), set Vf = collision
    fn execute_DXYN(&mut self, x: usize, y: usize, n: u8) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT || y >= VARIABLE_REGISTER_COUNT || n > MAX_SPRITE_HEIGHT {
            return Err(Error::OperandsOutOfBounds);
        }
        // Read the sprite to draw as an N-byte array slice at memory location
        // pointed to by the index register
        let sprite: &[u8] = self
            .memory
            .read_bytes(self.index_register as usize, n as usize)?;
        // Call into the Chipolata display to draw this sprite at location (Vx, Vy),
        // storing the result (i.e. whether collision occured) in a temp variable
        let any_pixel_turned_off: bool = self.frame_buffer.draw_sprite(
            self.variable_registers[x] as usize,
            self.variable_registers[y] as usize,
            sprite,
        )?;
        // Set Vf to 1 or 0 if collision did or did not occur, respectively
        self.variable_registers[0xF] = match any_pixel_turned_off {
            true => 0x1,
            false => 0x0,
        };
        Ok(())
    }

    /// Executes the EX9E instruction - SKP Vx
    /// Purpose: skip next instruction if the key with value Vx is pressed
    fn execute_EX9E(&mut self, x: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        let key: u8 = self.variable_registers[x]; // get the value stored in Vx
                                                  // Check whether the current keystate indicates the corresponding key is pressed
        let key_pressed: bool = self.keystate.is_key_pressed(key)?;
        if key_pressed {
            // If so, increment the program counter by 2 bytes (1 opcode)
            self.program_counter += 2;
            self.set_key_status(key, false)?; // Set key status to unpressed to prevent immediate repeats
        }
        Ok(())
    }

    /// Executes the EXA1 instruction - SKNP Vx
    /// Purpose: skip next instruction if the key with value Vx is not pressed
    fn execute_EXA1(&mut self, x: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        let key: u8 = self.variable_registers[x]; // get the value stored in Vx
                                                  // Check whether the current keystate indicates the corresponding key is pressed
        let key_pressed: bool = self.keystate.is_key_pressed(key)?;
        if !key_pressed {
            // If not, increment the program counter by 2 bytes (1 opcode)
            self.program_counter += 2;
        } else {
            self.set_key_status(key, false)?; // Set key status to unpressed to prevent immediate repeats
        }
        Ok(())
    }

    /// Executes the FX07 instruction - LD Vx, DT
    /// Purpose: set Vx = delay timer value
    fn execute_FX07(&mut self, x: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        Ok(self.variable_registers[x] = self.delay_timer) // set Vx = delay timer value
    }

    /// Executes the FX0A instruction - LD Vx, K
    /// Purpose: wait for a key press, store the key value in Vx
    fn execute_FX0A(&mut self, x: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        // Check whether any keys are currently pressed
        match self.keystate.get_keys_pressed() {
            Some(keys_pressed) => {
                // Store the (first) pressed key value in Vx
                self.variable_registers[x] = keys_pressed[0];
                self.status = ProcessorStatus::Running; // ensure processor state is "Running"
                Ok(())
            }
            None => {
                // Decrement the program counter by by 2 bytes (1 opcode)
                // i.e. keep repeating this instruction until a key press occurs
                self.program_counter -= 2;
                // Set processor state to "Waiting"
                self.status = ProcessorStatus::WaitingForKeypress;
                Ok(())
            }
        }
    }

    /// Executes the FX15 instruction - LD DT, Vx
    /// Purpose: set delay timer = Vx
    fn execute_FX15(&mut self, x: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        Ok(self.delay_timer = self.variable_registers[x]) // set delay timer = Vx
    }

    /// Executes the FX18 instruction - LD ST, Vx
    /// Purpose: set sound timer = Vx
    fn execute_FX18(&mut self, x: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        Ok(self.sound_timer = self.variable_registers[x]) // set sound timer = Vx
    }

    /// Executes the FX1E instruction - ADD I, Vx
    /// Purpose: set I = I + Vx.  Set Vf to 1 if result outside addressable memory
    fn execute_FX1E(&mut self, x: usize) -> Result<(), Error> {
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
                return Ok(self.index_register = result as u16);
            }
        }
        return Err(Error::OperandsOutOfBounds);
    }

    /// Executes the FX29 instruction - LD F, Vx
    /// Purpose: set I = location of font sprite for digit Vx
    fn execute_FX29(&mut self, x: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        // Fetch the character hex code in Vx and check it is within expected bounds
        let character = self.variable_registers[x];
        if character > FONT_SPRITE_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        // Calculate the corresponding font sprite location in memory based on the size per font
        // character (in bytes), the starting location of font data in memory, and the offset of
        // the requested character's ordinal within the range of font characters
        let character_memory_location: usize =
            (character as usize) * self.font.char_size() + self.font_start_address;
        Ok(self.index_register = character_memory_location as u16) // set index register to this address
    }

    /// Executes the FX33 instruction - LD V, Vx
    /// Purpose: converts value in Vx to decimal, and stores the digits in memory locations I, I+1 and I+2
    fn execute_FX33(&mut self, x: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        let hex_number: u8 = self.variable_registers[x]; // get the hex value in Vx
        let decimal_first_digit: u8 = hex_number / 100; // get the "hundreds" decimal digit
        let decimal_second_digit: u8 = (hex_number % 100) / 10; // get the "tens" decimal digit
        let decimal_third_digit: u8 = hex_number % 10; // get the "units" decimal digit
        let index: usize = self.index_register as usize; // get the memory address in the index register
        self.memory.write_byte(index, decimal_first_digit)?; // store the first digit at this address
        self.memory.write_byte(index + 1, decimal_second_digit)?; // store the second digit at the next address
        self.memory.write_byte(index + 2, decimal_third_digit)?; // store the third digit at the next address
        Ok(())
    }

    /// Executes the FX55 instruction - LD [I], Vx
    /// Purpose: store registers V0 to Vx in memory starting at the address in the index register    
    ///          [CHIP-8] also set I to I + x + 1
    ///          [CHIP-48] also set I to I + x
    ///          [SUPER-CHIP 1.1] do not modify I    
    fn execute_FX55(&mut self, x: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        let original_index_register: usize = self.index_register as usize;
        match self.emulation_level {
            EmulationLevel::Chip8 => {
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
        Ok(self
            .memory
            .write_bytes(original_index_register, &self.variable_registers[0..x + 1])?)
    }

    /// Executes the FX65 instruction - LD Vx, [I]
    /// Purpose: populate registers V0 to Vx from memory starting at the address in the index register
    ///          [CHIP-8] also set I to I + x + 1
    ///          [CHIP-48] also set I to I + x
    ///          [SUPER-CHIP 1.1] do not modify I
    fn execute_FX65(&mut self, x: usize) -> Result<(), Error> {
        if x >= VARIABLE_REGISTER_COUNT {
            return Err(Error::OperandsOutOfBounds);
        }
        let original_index_register: usize = self.index_register as usize;
        match self.emulation_level {
            EmulationLevel::Chip8 => {
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
        Ok(())
    }
}
