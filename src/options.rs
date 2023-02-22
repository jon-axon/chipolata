use crate::EmulationLevel;

/// The default CHIP-8 processor speed in hertz
const DEFAULT_PROCESSOR_SPEED_HERTZ: u64 = 720;
/// The default CHIP-8 program start address within memory.
const DEFAULT_PROGRAM_ADDRESS: u16 = 0x200;

/// A struct to allow specification of Chipolata start-up parameters.
///
/// Chipolata provides many configurable options, for example the (initial) processor speed and
/// a number of choices around how ambiguous instructions should be handled (so as to allow
/// fine-grained mimicking of specific historic CHIP-8 interpreters).  Configuration of these
/// options is done through the [Options] struct, an instance of which is passed to
/// [Processor::initialise_and_load()](crate::processor::Processor::initialise_and_load) when
/// instantiating [Processor](crate::Processor).
#[derive(Copy, Clone)]
pub struct Options {
    pub processor_speed_hertz: u64,
    pub program_start_address: u16,
    pub emulation_level: EmulationLevel,
}

impl Default for Options {
    /// Constructor that returns an [Options] instance using typical default settings.
    fn default() -> Self {
        Options {
            processor_speed_hertz: DEFAULT_PROCESSOR_SPEED_HERTZ,
            program_start_address: DEFAULT_PROGRAM_ADDRESS,
            emulation_level: EmulationLevel::Chip8,
        }
    }
}
