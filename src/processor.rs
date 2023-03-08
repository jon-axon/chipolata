#![allow(non_snake_case)]

use super::display::Display;
use super::error::{ChipolataError, ErrorDetail};
use super::font::Font;
use super::instruction::Instruction;
use super::keystate::KeyState;
use super::memory::Memory;
use super::options::Options;
use super::program::Program;
use super::stack::Stack;
use rand::Rng;
use std::time::{Duration, Instant};

mod execute; // separate sub-module for all the instruction execution methods
#[cfg(test)]
mod tests; // functional unit tests
#[cfg(test)]
mod timing_tests; // non-functional (timing-related) unit tests

/// The number of ms that should pass inbetween decrements of delay and sound timers.
const TIMER_DECREMENT_INTERVAL_MICROSECONDS: u128 = 16666;
/// The number of ms that should pass inbetween vblank interrupts.
const VBLANK_INTERVAL_MICROSECONDS: u128 = 16666;
/// The number of variable registers available.
const VARIABLE_REGISTER_COUNT: usize = 16;
/// The number of RPL user flags; SUPER-CHIP 1.1 emulation mode only.
const RPL_REGISTER_COUNT: usize = 8;
/// The maximum sprite height (pixels).
const MAX_SPRITE_HEIGHT: u8 = 15;
/// The number of COSMAC VIP cycles used to execute one CHIP-8 interpreter cycle
/// (used when emulating original COSMAC VIP variable instruction timings)
const COSMAC_VIP_MACHINE_CYCLES_PER_CYCLE: u64 = 8;

/// An enum to indicate which extension of CHIP-8 is to be emulated.  See external
/// documentation for details of the differences in each case.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EmulationLevel {
    /// The original CHIP-8 interpreter for the RCA COSMAC VIP, optionally limited to 2k RAM
    /// and optionally set to simulate original COSMAC VIP cycles-per-instruction timings
    Chip8 {
        memory_limit_2k: bool,
        variable_cycle_timing: bool,
    },
    /// Re-implemented CHIP-8 interpreter for the HP48 graphing calculators
    Chip48,
    /// Version 1.1 of the SUPER-CHIP interpreter for HP48S and HP48SX graphing calculators
    /// Optionally includes OCTO-specific SCHIP instruction quirks
    SuperChip11 { octo_compatibility_mode: bool },
}

/// An enum used internally within the Chipolata crate to keep track of the processor
/// execution status.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ProcessorStatus {
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
    /// Execution copmpleted (program exited); SUPER-CHIP emulation mode only
    Completed,
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
#[derive(Debug, PartialEq)]
pub enum StateSnapshot {
    /// Minimal snapshot containing only the frame buffer state
    MinimalSnapshot {
        frame_buffer: Display,
        status: ProcessorStatus,
    },
    /// Extended snapshot containing the frame buffer state along with all registers,
    /// stack and memory
    ExtendedSnapshot {
        frame_buffer: Display,
        status: ProcessorStatus,
        stack: Stack,
        memory: Memory,
        program_counter: u16,
        index_register: u16,
        variable_registers: [u8; VARIABLE_REGISTER_COUNT],
        rpl_registers: [u8; RPL_REGISTER_COUNT],
        delay_timer: u8,
        sound_timer: u8,
        cycles: usize,
        high_resolution_mode: bool,
        emulation_level: EmulationLevel,
    },
}

/// An enum used to keep track of the state of the vertical blank interrupt, for accurate display
/// emulation in CHIP-8 mode
#[derive(Debug, PartialEq)]
pub enum VBlankStatus {
    // No display instruction has been processed yet this frame
    Idle,
    // A display instruction is queued, awaiting v-blank interrupt
    WaitingForVBlank,
    // THe v-blank interrupt has been set; drawing can proceed
    ReadyToDraw,
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
    rpl_registers: [u8; RPL_REGISTER_COUNT], // SUPER-CHIP 1.1 emulation mode only; RPL registers
    delay_timer: u8,      // Delay timer, decrements automatically at 60hz when non-zero
    sound_timer: u8,      // Sounds timer, decrements automatically at 60hz when non-zero
    cycles: usize,        // The number of processor cycles that have been executed
    high_resolution_mode: bool, // SUPER-CHIP 1.1 emulation mode only; true when when in high-res mode
    // ADDITIONAL STATE FIELDS
    keystate: KeyState, // A representation of the state (pressed/not pressed) of each key
    waiting_original_keystate: KeyState, // Keystate as at the start of an FX0A instruction
    keys_pressed_since_wait: Vec<u8>, // Keys pressed (but not released) during FX0A wait
    status: ProcessorStatus, // The current execution status of the processor
    last_timer_decrement: Instant, //  The moment the delay and sound timers were last decremented
    last_execution_cycle_complete: Instant, // The moment the execute cycle was last completed
    last_vblank_interrupt: Instant, // CHIP-8 emulation mode only; the last vblank interrupt time
    vblank_status: VBlankStatus, // CHIP-8 emulation mode only; state of v-blank interrupt
    // CONFIG AND SETUP FIELDS
    low_resolution_font: Font, // The font loaded into the processor (only used during initialisation)
    high_resolution_font: Option<Font>, // SUPER-CHIP 1.1 emulation mode only; the high resolution font data
    program: Program, // The program loaded into the processor (only used during initialisation)
    font_start_address: usize, // The start address in memory at which the font is loaded
    high_resolution_font_start_address: usize, // SUPER-CHIP 1.1 emulation mode only
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
    pub fn initialise_and_load(program: Program, options: Options) -> Result<Self, ChipolataError> {
        let low_res_font: Font = Font::default_low_resolution();
        let high_res_font: Option<Font> = match options.emulation_level {
            EmulationLevel::SuperChip11 {
                octo_compatibility_mode: true,
            } => Some(Font::octo_high_resolution()),
            EmulationLevel::SuperChip11 {
                octo_compatibility_mode: false,
            } => Some(Font::default_high_resolution()),
            _ => None,
        };
        let mut processor = Processor {
            frame_buffer: Display::new(options.emulation_level),
            stack: Stack::new(options.emulation_level),
            memory: Memory::new(options.emulation_level),
            program_counter: options.program_start_address,
            index_register: 0x0,
            variable_registers: [0x0; VARIABLE_REGISTER_COUNT],
            rpl_registers: [0x0; RPL_REGISTER_COUNT],
            delay_timer: 0x0,
            sound_timer: 0x0,
            cycles: 0,
            high_resolution_mode: false,
            keystate: KeyState::new(),
            waiting_original_keystate: KeyState::new(),
            keys_pressed_since_wait: Vec::new(),
            status: ProcessorStatus::StartingUp,
            last_timer_decrement: Instant::now(),
            last_execution_cycle_complete: Instant::now(),
            last_vblank_interrupt: Instant::now(),
            vblank_status: VBlankStatus::Idle,
            low_resolution_font: low_res_font,
            high_resolution_font: high_res_font,
            program: program,
            font_start_address: options.font_start_address as usize,
            high_resolution_font_start_address: 0x0,
            program_start_address: options.program_start_address as usize,
            processor_speed_hertz: options.processor_speed_hertz,
            emulation_level: options.emulation_level,
        };
        if let Err(e) = processor.load_font_data() {
            return Err(processor.crash(e));
        }
        processor.status = ProcessorStatus::Initialised;
        if let Err(e) = processor.load_program() {
            return Err(processor.crash(e));
        }
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
    pub fn export_state_snapshot(&self, verbosity: StateSnapshotVerbosity) -> StateSnapshot {
        match verbosity {
            StateSnapshotVerbosity::Minimal => StateSnapshot::MinimalSnapshot {
                frame_buffer: self.frame_buffer.clone(),
                status: self.status,
            },
            StateSnapshotVerbosity::Extended => StateSnapshot::ExtendedSnapshot {
                frame_buffer: self.frame_buffer.clone(),
                status: self.status,
                stack: self.stack.clone(),
                memory: self.memory.clone(),
                program_counter: self.program_counter,
                index_register: self.index_register,
                variable_registers: self.variable_registers,
                rpl_registers: self.rpl_registers,
                delay_timer: self.delay_timer,
                sound_timer: self.sound_timer,
                cycles: self.cycles,
                high_resolution_mode: self.high_resolution_mode,
                emulation_level: self.emulation_level,
            },
        }
    }

    /// Provides key press input to Chipolata, by setting the state of the specified key
    /// in the internal representation to pressed / not pressed as per supplied value.
    ///
    /// # Arguments
    ///
    /// * `key` - the hex ordinal of the key (valid range 0x0 to 0xF inclusive)
    /// * `status` - the value to set for the specified key (true means pressed)
    pub fn set_key_status(&mut self, key: u8, status: bool) -> Result<(), ChipolataError> {
        if let Err(e) = self.keystate.set_key_status(key, status) {
            return Err(self.crash(e));
        }
        Ok(())
    }

    /// Loads the processor's font data into memory.  If the size of the font data combined with
    /// the specified start location in memory would cause a write to unaddressable memory, then
    /// return an [ErrorDetail::MemoryAddressOutOfBounds].  This will always load the standard
    /// low-resolution CHIP-8 font into memory, however if in SUPER-CHIP 1.1 emulation mode this
    /// will also load the high-resolution SUPER-CHIP font as well
    fn load_font_data(&mut self) -> Result<(), ErrorDetail> {
        // Load the low-resolution font
        if self.font_start_address + self.low_resolution_font.font_data_size()
            >= self.program_start_address
        {
            return Err(ErrorDetail::MemoryAddressOutOfBounds {
                address: (self.font_start_address + self.low_resolution_font.font_data_size())
                    as u16,
            });
        }
        self.memory.write_bytes(
            self.font_start_address,
            self.low_resolution_font.font_data(),
        )?;
        // Load the high-resolution font, if present
        if let Some(high_resolution_font) = &self.high_resolution_font {
            self.high_resolution_font_start_address =
                self.font_start_address as usize + self.low_resolution_font.font_data_size();
            if self.high_resolution_font_start_address + high_resolution_font.font_data_size()
                >= self.program_start_address
            {
                return Err(ErrorDetail::MemoryAddressOutOfBounds {
                    address: (self.high_resolution_font_start_address
                        + high_resolution_font.font_data_size())
                        as u16,
                });
            }
            self.memory.write_bytes(
                self.high_resolution_font_start_address,
                high_resolution_font.font_data(),
            )?;
        }
        Ok(())
    }

    /// Loads the processor's program data into memory.  If the size of the program data combined
    /// with the specified start location in memory would cause a write to unaddressable memory,
    /// then return an [ErrorDetail::MemoryAddressOutOfBounds].
    fn load_program(&mut self) -> Result<(), ErrorDetail> {
        if self.program_start_address + self.program.program_data_size()
            >= self.memory.max_addressable_size()
        {
            return Err(ErrorDetail::MemoryAddressOutOfBounds {
                address: (self.program_start_address + self.program.program_data_size()) as u16,
            });
        }
        self.memory
            .write_bytes(self.program_start_address, self.program.program_data())?;
        Ok(())
    }

    /// Helper method that "crashes" the processor when an [ErrorDetail] instance is returned from a
    /// function call, and wraps this is in an appropriate [ChipolataError] instance before returning
    fn crash(&mut self, inner_error: ErrorDetail) -> ChipolataError {
        self.status = ProcessorStatus::Crashed;
        ChipolataError {
            state_snapshot_dump: self.export_state_snapshot(StateSnapshotVerbosity::Extended),
            inner_error,
        }
    }

    /// Executes one iteration of the Chipolata fetch -> decode -> execute cycle.  Returns a boolean
    /// indicating whether the display frame buffer was updated this cycle.
    pub fn execute_cycle(&mut self) -> Result<bool, ChipolataError> {
        // Change processor status if appropriate
        match self.status {
            ProcessorStatus::ProgramLoaded => self.status = ProcessorStatus::Running,
            ProcessorStatus::Running | ProcessorStatus::WaitingForKeypress => {
                // no change
            }
            ProcessorStatus::StartingUp
            | ProcessorStatus::Initialised
            | ProcessorStatus::Completed
            | ProcessorStatus::Crashed => {
                return Err(self.crash(ErrorDetail::UnknownError));
            }
        }
        // Increment the cycles counter
        self.cycles += 1;
        // Decrement the delay and sound timers, if appropriate
        self.decrement_timers();
        // Fetch two byte opcode from current Program Counter memory location
        let opcode: u16 = match self.memory.read_two_bytes(self.program_counter as usize) {
            Ok(opcode) => opcode,
            Err(e) => return Err(self.crash(e)),
        };
        // Increment Program Counter (by two bytes, as we have 16-bit opcodes)
        self.program_counter += 0x2;
        // Decode the opcode into an instruction, setting processor state to Crashed on error
        let instruction: Instruction = match Instruction::decode_from(opcode) {
            Ok(instruction) => instruction,
            Err(e) => return Err(self.crash(e)),
        };
        // If the instruction is one that updates the display, set a local flag to true
        let display_updated: bool = match instruction {
            Instruction::Op00E0 => true,
            Instruction::OpDXYN { .. } => true,
            _ => false,
        };
        // Execute the instruction, setting processor state to Crashed on error, and returning
        // the number of cycles the original COSMAC VIP interpreter would have used for this
        let cosmac_cycles: u64 = match self.execute(instruction) {
            Ok(timing) => timing,
            Err(e) => return Err(self.crash(e)),
        };
        // In order to simulate the configured processor speed, we now spin until the appropriate
        // time has passed since the last cycle completed
        let target_cycle_duration: Duration = self.calculate_cycle_duration(cosmac_cycles);
        while self.last_execution_cycle_complete.elapsed() < target_cycle_duration {
            // spin
        }
        self.last_execution_cycle_complete = Instant::now();
        // Return successfully, passing the flag indicating whether the display was updated this cycle
        return Ok(display_updated);
    }

    /// Internal helper function that returns the Duration a cycle should be emulated to take,
    /// based on the specified processor speed and emulation mode (fixed cycles vs COSMAC
    /// variable instruction timing).
    ///
    /// # Arguments
    ///
    /// * `cosmac_cycles` - if using COSMAC variable instruction timings, this is the number
    /// of COSMAC interpreter cycles taken to execute the instruction in question (returned
    /// by the relevant execute() method).  If using fixed cycle timings, this parameter is
    /// ignored by the function.
    fn calculate_cycle_duration(&self, cosmac_cycles: u64) -> Duration {
        let execution_duration: Duration;
        if let EmulationLevel::Chip8 {
            memory_limit_2k: _,
            variable_cycle_timing: true,
        } = self.emulation_level
        {
            // Define the cycle duration to be the COSMAC VIP original instruction timing
            // (in cycles) running at the specified processor speed
            execution_duration = Duration::from_micros(
                cosmac_cycles * COSMAC_VIP_MACHINE_CYCLES_PER_CYCLE * 1_000_000_u64
                    / self.processor_speed_hertz,
            );
        } else {
            // Drive the cycle duration purely from specified processor speed
            execution_duration = Duration::from_micros(1_000_000_u64 / self.processor_speed_hertz);
        }
        execution_duration
    }

    /// Checks if the required time has passed since the sound and delay timers were last decremented
    /// and if so, decrements them.  Also counts down to vblank interrupt.
    fn decrement_timers(&mut self) {
        // If in Chip8 emulation mode, check the vblank interrupt timer and set interrupt accordingly
        if let EmulationLevel::Chip8 {
            memory_limit_2k: _,
            variable_cycle_timing: _,
        } = self.emulation_level
        {
            if self.last_vblank_interrupt.elapsed().as_micros() >= VBLANK_INTERVAL_MICROSECONDS {
                if let VBlankStatus::WaitingForVBlank = self.vblank_status {
                    self.vblank_status = VBlankStatus::ReadyToDraw;
                }
                self.last_vblank_interrupt = Instant::now();
            }
        }
        // Nothing to do for delay and sound timers unless timers are running
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

    /// Executes the passed Instruction.  Returns [ErrorDetail::UnimplementedInstruction] if Chipolata is
    /// unable to process opcodes of this type.
    ///
    /// # Arguments
    ///
    /// * `instr` - the instruction to be executed
    fn execute(&mut self, instr: Instruction) -> Result<u64, ErrorDetail> {
        match instr {
            Instruction::Op004B => self.execute_004B(),
            Instruction::Op00CN { n } => self.execute_00CN(n),
            Instruction::Op00E0 => self.execute_00E0(),
            Instruction::Op00EE => self.execute_00EE(),
            Instruction::Op00FB => self.execute_00FB(),
            Instruction::Op00FC => self.execute_00FC(),
            Instruction::Op00FD => self.execute_00FD(),
            Instruction::Op00FE => self.execute_00FE(),
            Instruction::Op00FF => self.execute_00FF(),
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
            Instruction::OpFX30 { x } => self.execute_FX30(x),
            Instruction::OpFX33 { x } => self.execute_FX33(x),
            Instruction::OpFX55 { x } => self.execute_FX55(x),
            Instruction::OpFX65 { x } => self.execute_FX65(x),
            Instruction::OpFX75 { x } => self.execute_FX75(x),
            Instruction::OpFX85 { x } => self.execute_FX85(x),
        }
    }
}
