use crate::{EmulationLevel, ErrorDetail};
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;

/// The original COSMAC VIP processor speed in hertz.  When instantiating an [Options] instance
/// to pass to Chipolata, this value should normally be supplied as the starting
/// [Options::processor_speed_hertz] choice when specifying [EmulationLevel::Chip8] with
/// `variable_cycle_timing` set to true.
/// to be true.
pub const COSMAC_VIP_PROCESSOR_SPEED_HERTZ: u64 = 1760900;
/// The default CHIP-8 processor speed in hertz
const DEFAULT_PROCESSOR_SPEED_HERTZ: u64 = 1000;
/// The default CHIP-8 program start address within memory
const DEFAULT_PROGRAM_ADDRESS: u16 = 0x200;
/// The default CHIP-8 font start address within memory
const DEFAULT_FONT_ADDRESS: u16 = 0x50;

/// A struct to allow specification of Chipolata start-up parameters.
///
/// Chipolata provides many configurable options, for example the (initial) processor speed and
/// a number of choices around how ambiguous instructions should be handled (so as to allow
/// fine-grained mimicking of specific historic CHIP-8 interpreters).  Configuration of these
/// options is done through the [Options] struct, an instance of which is passed to
/// [Processor::initialise_and_load()](crate::processor::Processor::initialise_and_load) when
/// instantiating [Processor](crate::Processor).
#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq)]
pub struct Options {
    /// The number of complete fetch->decode->execute cycles Chipolata will carry out per second
    /// while in default fixed cycle timing mode.  When emulating the variable length instruction
    /// timings of the COSMAC VIP, this simulates the underlying original COSMAC processor speed
    /// (where each instruction may take make CPU cycles to fetch->decode->execute, not just one).
    pub processor_speed_hertz: u64,
    /// The location into memory at which the program should be loaded (and program counter set).
    pub program_start_address: u16,
    /// The location into memory at which the system font should be loaded.
    pub font_start_address: u16,
    /// Specification of the variant of CHIP-8 to emulate.
    pub emulation_level: EmulationLevel,
}

impl Options {
    /// Typical constructor that allows specification of processor speed and emulation level, but
    /// useful default values for less commonly set properties
    pub fn new(processor_speed_hertz: u64, emulation_level: EmulationLevel) -> Self {
        Options {
            processor_speed_hertz,
            emulation_level,
            program_start_address: DEFAULT_PROGRAM_ADDRESS,
            font_start_address: DEFAULT_FONT_ADDRESS,
        }
    }

    /// Builder method that instantiates Options from the specified JSON file
    pub fn load_from_file(file_path: &Path) -> Result<Options, ErrorDetail> {
        // attempt to open the file
        if let Ok(json_file) = File::open(file_path) {
            // parse the file as JSON and deserialise into an Options instance
            if let Ok(options) = serde_json::from_reader(json_file) {
                return Ok(options);
            }
        }
        // if we fall through to here, an error has occurred reading from the file
        return Err(ErrorDetail::FileError {
            file_path: file_path.to_str().unwrap_or_default().to_owned(),
        });
    }

    /// Method that serialises the passed [Options] instance to the specified JSON file
    pub fn save_to_file(options: &Options, file_path: &Path) -> Result<(), ErrorDetail> {
        // attempt to open the file; create it if it does not exist and truncate if it does
        if let Ok(_) = File::create(file_path) {
            if let Ok(serialised_options) = serde_json::to_string_pretty(options) {
                if std::fs::write(file_path, serialised_options).is_ok() {
                    return Ok(());
                }
            }
        }
        // if we fall through to here, an error has occurred writing to the file
        return Err(ErrorDetail::FileError {
            file_path: file_path.to_str().unwrap_or_default().to_owned(),
        });
    }
}

impl Default for Options {
    /// Constructor that returns an [Options] instance using typical default settings
    fn default() -> Self {
        Options {
            processor_speed_hertz: DEFAULT_PROCESSOR_SPEED_HERTZ,
            program_start_address: DEFAULT_PROGRAM_ADDRESS,
            font_start_address: DEFAULT_FONT_ADDRESS,
            emulation_level: EmulationLevel::SuperChip11 {
                octo_compatibility_mode: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_load() {
        const FILENAME: &str = "unit_test_save_load.json";
        let options: Options = Options::default();
        Options::save_to_file(&options, Path::new(FILENAME)).unwrap();
        let new_options = Options::load_from_file(Path::new(FILENAME)).unwrap();
        assert_eq!(options, new_options);
        std::fs::remove_file(FILENAME).unwrap();
    }
}
