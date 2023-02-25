use crate::EmulationLevel;

/// The original COSMAC VIP processor speed in hertz.  When instantiating an [Options] instance
/// to pass to Chipolata, this value should normally be supplied as the starting
/// [Options::processor_speed_hertz] choice when specifying [Options::use_variable_cycle_timings]
/// to be true.
pub const COSMAC_VIP_PROCESSOR_SPEED_HERTZ: u64 = 1760900;
/// The default CHIP-8 processor speed in hertz
const DEFAULT_PROCESSOR_SPEED_HERTZ: u64 = 720;
/// The default CHIP-8 program start address within memory.
const DEFAULT_PROGRAM_ADDRESS: u16 = 0x200;
/// The default CHIP-8 font start address within memory.
const DEFAULT_FONT_ADDRESS: u16 = 0x50;

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
    /// The number of complete fetch->decode->execute cycles Chipolata will carry out per second
    /// while in default fixed cycle timing mode.  When emulating the variable length instruction
    /// timings of the COSMAC VIP, this simulates the underlying original COSMAC processor speed
    /// (where each instruction may take make CPU cycles to fetch->decode->execute, not just one).
    pub processor_speed_hertz: u64,
    /// A flag to indicate whether to emulate fixed cycle timing or simulated original COSMAC
    /// variable instruction timing.
    pub use_variable_cycle_timings: bool,
    /// The location into memory at which the program should be loaded (and program counter set).
    pub program_start_address: u16,
    /// The location into memory at which the system font should be loaded.
    pub font_start_address: u16,
    /// Specification of the variant of CHIP-8 to emulate.
    pub emulation_level: EmulationLevel,
}

impl Default for Options {
    /// Constructor that returns an [Options] instance using typical default settings.
    fn default() -> Self {
        Options {
            processor_speed_hertz: DEFAULT_PROCESSOR_SPEED_HERTZ,
            use_variable_cycle_timings: false,
            program_start_address: DEFAULT_PROGRAM_ADDRESS,
            font_start_address: DEFAULT_FONT_ADDRESS,
            emulation_level: EmulationLevel::Chip8 {
                memory_limit_2k: false,
            },
        }
    }
}
