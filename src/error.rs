use std::error;
use std::fmt;

/// An Error enum used throughout the Chipolata crate to communicate runtime errors
/// that have occurred.
///
/// Instances of [Error] are bubbled-up to the hosting application through the public
/// API methods.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// An unrecognised opcode was read from memory
    UnknownInstruction,
    /// A valid opcode was read from memory but which is not implemented by Chipolata
    UnimplementedInstruction,
    /// One or more operands fall outside expected ranges and cannot be safely used
    OperandsOutOfBounds,
    /// An attempt was made to pop an item off the Chipolata stack while it is empty
    PopEmptyStack,
    /// An attempt was made to push an item on to the Chipolata stack while it is full
    PushFullStack,
    /// An attempt was made to read/write from an address outside the addressable range
    MemoryAddressOutOfBounds,
    /// A key ordinal was referenced that is outside the valid CHIP-8 keypad range (0x0 to 0xF)
    InvalidKey,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    /// Returns a textual description of each enum variant for display purposes.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownInstruction => {
                write!(f, "an unrecognised opcode was decoded")
            }
            Error::UnimplementedInstruction => {
                write!(f, "an unimplemented opcode was executed")
            }
            Error::OperandsOutOfBounds => {
                write!(f, "an opcode contains invalid operands")
            }
            Error::PopEmptyStack => {
                write!(f, "an attempt was made to pop the stack while empty")
            }
            Error::PushFullStack => {
                write!(f, "an attempt was made to push to the stack while full")
            }
            Error::MemoryAddressOutOfBounds => {
                write!(f, "memory was accessed out of bounds")
            }
            Error::InvalidKey => {
                write!(f, "an invalid key was specified")
            }
        }
    }
}
