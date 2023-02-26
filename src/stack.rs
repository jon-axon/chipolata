use crate::{error::ErrorDetail, EmulationLevel};

/// The default stack size for all system variants (in terms of u16 values).
const CHIPOLATA_STACK_DEPTH: usize = 16;
const CHIP8_STACK_DEPTH: usize = 12;
const CHIP48_STACK_DEPTH: usize = 16;
const SUPERCHIP11_STACK_DEPTH: usize = 16;

/// An abstraction of the CHIP-8 stack, used for holding return addresses from function calls.
#[derive(Clone, Debug, PartialEq)]
pub struct Stack {
    /// A stack-allocated array of 16-bit values representing the entire CHIP-8 stack.
    pub bytes: [u16; CHIPOLATA_STACK_DEPTH],
    /// A pointer to the current top of the stack (i.e. the next available empty slot).
    pub pointer: usize,
    /// A "soft" stack size limit, which may be smaller than the actual array allocated.
    stack_size_limit: usize,
}

impl Stack {
    /// Constructor that returns a [Stack] instance, initialised to zero entries.  The stack size
    /// will be (soft) limited depending on emulation level.
    ///
    /// # Arguments
    ///
    /// * `emulation_level` - the CHIP-8 variant to be emulated (impacts permitted stack entries)
    pub(crate) fn new(emulation_level: EmulationLevel) -> Self {
        Stack {
            bytes: [0x0; CHIPOLATA_STACK_DEPTH],
            pointer: 0,
            stack_size_limit: match emulation_level {
                EmulationLevel::Chip8 { .. } => CHIP8_STACK_DEPTH,
                EmulationLevel::Chip48 => CHIP48_STACK_DEPTH,
                EmulationLevel::SuperChip11 => SUPERCHIP11_STACK_DEPTH,
            },
        }
    }

    /// Pushes the specified 16-bit value on to the top of the stack.  If the stack is already
    /// full, returns [ErrorDetail::PushFullStack].
    ///
    /// # Arguments
    ///
    /// * `value` - the value to push on to the stack
    pub fn push(&mut self, value: u16) -> Result<(), ErrorDetail> {
        if self.pointer >= self.stack_size_limit {
            return Err(ErrorDetail::PushFullStack);
        }
        self.bytes[self.pointer] = value;
        // Increment the stack pointer to point to the next free slot
        Ok(self.pointer += 1)
    }

    /// Pops the top entry off the stack and returns it.  If the stack is already empty, returns
    /// [ErrorDetail::PopEmptyStack].
    pub fn pop(&mut self) -> Result<u16, ErrorDetail> {
        if self.pointer <= 0 {
            return Err(ErrorDetail::PopEmptyStack);
        }
        // Decrement the stack pointer (before accessing the item at this index)
        self.pointer -= 1;
        Ok(self.bytes[self.pointer])
    }

    /// Returns the maximum permitted stack size (number of entries)
    pub fn max_stack_size(&self) -> usize {
        self.stack_size_limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pop() {
        let mut stack: Stack = Stack::new(EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        });
        stack.bytes[0] = 0xC4;
        stack.pointer = 1;
        assert!(stack.pop().unwrap() == 0xC4 && stack.pointer == 0);
    }

    #[test]
    fn test_pop_empty_error() {
        let mut stack: Stack = Stack::new(EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        });
        assert_eq!(stack.pop().unwrap_err(), ErrorDetail::PopEmptyStack);
    }

    #[test]
    fn test_push() {
        let mut stack: Stack = Stack::new(EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        });
        stack.bytes[0] = 0xC4;
        stack.pointer = 1;
        assert!(stack.push(0xFF).is_ok() && stack.bytes[1] == 0xFF && stack.pointer == 2);
    }

    #[test]
    fn test_push_full_chip8_mode_error() {
        let mut stack: Stack = Stack::new(EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        });
        stack.pointer = CHIP8_STACK_DEPTH;
        assert_eq!(stack.push(0xFF).unwrap_err(), ErrorDetail::PushFullStack);
    }

    #[test]
    fn test_push_full_chip48_mode_error() {
        let mut stack: Stack = Stack::new(EmulationLevel::Chip48);
        stack.pointer = CHIP48_STACK_DEPTH;
        assert_eq!(stack.push(0xFF).unwrap_err(), ErrorDetail::PushFullStack);
    }

    #[test]
    fn test_push_full_superchip11_mode_error() {
        let mut stack: Stack = Stack::new(EmulationLevel::SuperChip11);
        stack.pointer = SUPERCHIP11_STACK_DEPTH;
        assert_eq!(stack.push(0xFF).unwrap_err(), ErrorDetail::PushFullStack);
    }
}
