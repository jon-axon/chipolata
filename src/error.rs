use crate::StateSnapshot;
use std::collections::HashMap;
use std::error;
use std::fmt;

/// An Error enum used throughout the Chipolata crate to communicate details of runtime errors
/// that have occurred.
///
/// Instances of [ErrorDetail] are bubbled-up to the hosting application through the public
/// API methods.
#[derive(Debug, PartialEq)]
pub enum ErrorDetail {
    /// An unrecognised opcode was read from memory
    UnknownInstruction { opcode: u16 },
    /// A valid opcode was read from memory but which is not implemented by Chipolata
    UnimplementedInstruction { opcode: u16 },
    /// One or more operands fall outside expected ranges and cannot be safely used
    /// The HashMap field holds the name of each potential faulty operand and its value
    OperandsOutOfBounds { operands: HashMap<String, usize> },
    /// An attempt was made to pop an item off the Chipolata stack while it is empty
    PopEmptyStack,
    /// An attempt was made to push an item on to the Chipolata stack while it is full
    PushFullStack,
    /// An attempt was made to read/write from an address outside the addressable range
    MemoryAddressOutOfBounds { address: u16 },
    /// A key ordinal was referenced that is outside the valid CHIP-8 keypad range (0x0 to 0xF)
    InvalidKey { key: u8 },
    /// Error used for any file I/O issues
    FileError { file_path: String },
    /// General bucket for any unknown issues (to return *something* rather than panicking)
    UnknownError,
}

impl error::Error for ErrorDetail {}

impl fmt::Display for ErrorDetail {
    /// Returns a textual description of each enum variant for display purposes.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorDetail::UnknownInstruction { opcode } => {
                write!(f, "an unrecognised opcode {:#X} was decoded", opcode)
            }
            ErrorDetail::UnimplementedInstruction { opcode } => {
                write!(f, "an unimplemented opcode {} was executed", opcode)
            }
            ErrorDetail::OperandsOutOfBounds { operands } => {
                write!(f, "an opcode contains invalid operands: {:?}", operands)
            }
            ErrorDetail::PopEmptyStack => {
                write!(f, "an attempt was made to pop the stack while empty")
            }
            ErrorDetail::PushFullStack => {
                write!(f, "an attempt was made to push to the stack while full")
            }
            ErrorDetail::MemoryAddressOutOfBounds { address } => {
                write!(f, "invalid memory address {} was accessed", address)
            }
            ErrorDetail::InvalidKey { key } => {
                write!(f, "invalid key {} was specified", key)
            }
            ErrorDetail::FileError { file_path } => {
                write!(
                    f,
                    "invalid file path {} was specified",
                    file_path.to_string()
                )
            }
            ErrorDetail::UnknownError => {
                write!(f, "an unknown error occurred")
            }
        }
    }
}

/// An Error struct used to bubble up Chipolata errors to the hosting application.  This wraps
/// the more specific [ErrorDetail] error enum, and provides overall processor state context
/// at the point of the failure
#[derive(Debug, PartialEq)]
pub struct ChipolataError {
    pub state_snapshot_dump: StateSnapshot,
    pub inner_error: ErrorDetail,
}

impl error::Error for ChipolataError {}

impl fmt::Display for ChipolataError {
    /// Returns a textual description of the error
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let StateSnapshot::ExtendedSnapshot {
            frame_buffer: _,
            status: _,
            stack: _,
            memory: _,
            program_counter,
            index_register: _,
            variable_registers: _,
            rpl_registers: _,
            delay_timer: _,
            sound_timer: _,
            cycles,
            high_resolution_mode: _,
            emulation_level: _,
        } = &self.state_snapshot_dump
        {
            write!(
                f,
                "an error occurred on cycle {}, with program_counter {}",
                cycles, program_counter
            )?;
        }
        self.inner_error.fmt(f)
    }
}
