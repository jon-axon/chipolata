use crate::{EmulationLevel, ErrorDetail};
use rand::Rng;

/// The default memory size for all system variants (in bytes).
const CHIPOLATA_MEMORY_SIZE_BYTES: usize = 0x1000;
// The COSMAC VIP had either 2048 bytes or 4096 bytes of RAM; we allow this to be configured.
// From this, the last 352 bytes are reserved
const CHIP8_SMALL_ADDRESSABLE_MEMORY_BYTES: usize = 0x6A0;
const CHIP8_LARGE_ADDRESSABLE_MEMORY_BYTES: usize = 0xEA0;
// For CHIP-48 the full 4096 bytes are addressable
const CHIP48_ADDRESSABLE_MEMORY_BYTES: usize = 0x1000;
// For SUPER-CHIP 1.1 the final byte is reserved (presumably by mistake), so 4095 are addressable
const SUPERCHIP11_ADDRESSABLE_MEMORY_BYTES: usize = 0xFFF;

/// An abstraction of the CHIP-8 memory space.
#[derive(Clone, Debug, PartialEq)]
pub struct Memory {
    /// A stack-allocated array of bytes representing the entire CHIP-8 memory space
    pub bytes: [u8; CHIPOLATA_MEMORY_SIZE_BYTES],
    /// The number of addressable memory slots
    address_limit: usize,
}

impl Memory {
    /// Constructor that returns a [Memory] instance initialised with all bytes 0x00.  If
    /// the emulation level is [EmulationLevel::SuperChip11] then the memory will instead
    /// be randomised on startup, mirroring original behaviour.
    ///
    /// The addressable memory space will be (soft) limited depending on emulation level.
    ///
    /// # Arguments
    ///
    /// * `emulation_level` - the CHIP-8 variant to be emulated (impacts addressable memory)
    pub(crate) fn new(emulation_level: EmulationLevel) -> Self {
        let mut bytes: [u8; CHIPOLATA_MEMORY_SIZE_BYTES] = [0x0; CHIPOLATA_MEMORY_SIZE_BYTES];
        // For SUPER-CHIP 1.1 emulation, assign each memory slot a random byte value
        if let EmulationLevel::SuperChip11 { .. } = emulation_level {
            rand::thread_rng().fill(&mut bytes[..]);
        }
        Self {
            bytes,
            address_limit: match emulation_level {
                EmulationLevel::Chip8 {
                    memory_limit_2k: true,
                    variable_cycle_timing: _,
                } => CHIP8_SMALL_ADDRESSABLE_MEMORY_BYTES,
                EmulationLevel::Chip8 { .. } => CHIP8_LARGE_ADDRESSABLE_MEMORY_BYTES,
                EmulationLevel::Chip48 => CHIP48_ADDRESSABLE_MEMORY_BYTES,
                EmulationLevel::SuperChip11 { .. } => SUPERCHIP11_ADDRESSABLE_MEMORY_BYTES,
            },
        }
    }

    /// Returns a copy of the byte in memory at the specified address.  If the address
    /// is outside the addressable range, returns
    /// [ErrorDetail::MemoryAddressOutOfBounds](crate::error::ErrorDetail::MemoryAddressOutOfBounds).
    ///
    /// # Arguments
    ///
    /// * `address` - the memory address at which the byte should be read
    pub fn read_byte(&self, address: usize) -> Result<u8, ErrorDetail> {
        if address >= self.address_limit {
            return Err(ErrorDetail::MemoryAddressOutOfBounds {
                address: address as u16,
            });
        }
        Ok(self.bytes[address])
    }

    /// Writes the passed byte to the specified memory address.  If the address is
    /// outside the addressable range, returns
    /// [ErrorDetail::MemoryAddressOutOfBounds].
    ///
    /// # Arguments
    ///
    /// * `address` - the memory address at which the byte should be written
    /// * `value` - the byte value to be written
    pub(crate) fn write_byte(&mut self, address: usize, value: u8) -> Result<(), ErrorDetail> {
        if address >= self.address_limit {
            return Err(ErrorDetail::MemoryAddressOutOfBounds {
                address: address as u16,
            });
        }
        Ok(self.bytes[address] = value)
    }

    /// Returns an array slice from memory as per the specified start address and
    /// number of bytes.  If the operands are such that the array slice would extend beyond
    /// addressable memory then returns [ErrorDetail::MemoryAddressOutOfBounds].
    ///
    /// # Arguments
    ///
    /// * `start_address` - the memory address at the start of the range from which to read
    /// * `num_bytes` - the number of bytes to read from memory
    pub fn read_bytes(&self, start_address: usize, num_bytes: usize) -> Result<&[u8], ErrorDetail> {
        let final_address: usize = start_address + num_bytes - 1;
        // Check that the start address plus number of bytes to read does not exceed the
        // addressable memory space
        if final_address >= self.address_limit {
            return Err(ErrorDetail::MemoryAddressOutOfBounds {
                address: final_address as u16,
            });
        }
        Ok(&self.bytes[start_address..(final_address + 1)])
    }

    /// Returns a 16-bit unsigned integer constructed by reading two consecutive bytes from memory
    /// starting from the specified address.  The construction is big-endian.  In the unlikely
    /// event that the second byte would fall outside the addressable memory space, this returns
    /// [ErrorDetail::MemoryAddressOutOfBounds].
    ///
    /// The method is generally used as a convenience for reading opcodes from memory, as
    /// CHIP-8 opcodes are 16-bits in size.
    ///
    /// # Arguments
    ///
    /// * `start_address` - the memory address of the first (most significant) byte to read
    pub fn read_two_bytes(&self, start_address: usize) -> Result<u16, ErrorDetail> {
        if start_address + 1 >= self.address_limit {
            return Err(ErrorDetail::MemoryAddressOutOfBounds {
                address: 1 + start_address as u16,
            });
        }
        // Construct the u16 from the two u8s through bit shifting and a bitwise OR
        Ok(((self.bytes[start_address] as u16) << 8) | self.bytes[start_address + 1] as u16)
    }

    /// Writes the passed byte array slice to memory starting at the specified address.
    /// If the operands are such that the operation would write to addresses extending beyond
    /// the addressable memory then returns [ErrorDetail::MemoryAddressOutOfBounds].
    ///
    /// # Arguments
    ///
    /// * `start_address` - the memory address at the start of the range to which to write
    /// * `bytes_to_write` - the array slice containing the bytes to write to memory
    pub(crate) fn write_bytes(
        &mut self,
        start_address: usize,
        bytes_to_write: &[u8],
    ) -> Result<(), ErrorDetail> {
        let final_address: usize = start_address + bytes_to_write.len() - 1;
        // Check that the start address plus size of the byte array slice to write does not
        // exceed the number of bytes to read does not exceed the addressable memory space
        if final_address >= self.address_limit {
            return Err(ErrorDetail::MemoryAddressOutOfBounds {
                address: final_address as u16,
            });
        }
        // Iterate through the passed array slice writing the bytes in turn to successive
        // memory addresses beginning at the specified starting location
        for (i, x) in bytes_to_write.iter().enumerate() {
            self.bytes[start_address + i] = *x;
        }
        Ok(())
    }

    /// Returns the size of the addressable memory space in bytes
    pub fn max_addressable_size(&self) -> usize {
        self.address_limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_initialisation_chip8() {
        let instance_one_first_byte: u8 = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        })
        .read_byte(0x0)
        .unwrap();
        let instance_two_first_byte: u8 = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        })
        .read_byte(0x0)
        .unwrap();
        assert_eq!(instance_one_first_byte, instance_two_first_byte);
    }

    #[test]
    fn test_zero_initialisation_chip48() {
        let instance_one_first_byte: u8 =
            Memory::new(EmulationLevel::Chip48).read_byte(0x0).unwrap();
        let instance_two_first_byte: u8 =
            Memory::new(EmulationLevel::Chip48).read_byte(0x0).unwrap();
        assert_eq!(instance_one_first_byte, instance_two_first_byte);
    }

    #[test]
    fn test_random_initialisation_superchip11() {
        let instance_one_first_byte: u8 = Memory::new(EmulationLevel::SuperChip11 {
            octo_compatibility_mode: false,
        })
        .read_byte(0x0)
        .unwrap();
        let instance_two_first_byte: u8 = Memory::new(EmulationLevel::SuperChip11 {
            octo_compatibility_mode: false,
        })
        .read_byte(0x0)
        .unwrap();
        assert_ne!(instance_one_first_byte, instance_two_first_byte);
    }

    #[test]
    fn test_read_byte() {
        let mut memory = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        });
        memory.bytes[0x3] = 0xF2;
        assert_eq!(memory.read_byte(0x3).unwrap(), 0xF2);
    }

    #[test]
    fn test_read_byte_out_of_bounds_chip8_small_error() {
        let memory = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: true,
            variable_cycle_timing: false,
        });
        assert_eq!(
            memory
                .read_byte(CHIP8_SMALL_ADDRESSABLE_MEMORY_BYTES)
                .unwrap_err(),
            ErrorDetail::MemoryAddressOutOfBounds {
                address: CHIP8_SMALL_ADDRESSABLE_MEMORY_BYTES as u16
            }
        );
    }

    #[test]
    fn test_read_byte_out_of_bounds_chip8_large_error() {
        let memory = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        });
        assert_eq!(
            memory
                .read_byte(CHIP8_LARGE_ADDRESSABLE_MEMORY_BYTES)
                .unwrap_err(),
            ErrorDetail::MemoryAddressOutOfBounds {
                address: CHIP8_LARGE_ADDRESSABLE_MEMORY_BYTES as u16
            }
        );
    }

    #[test]
    fn test_read_byte_out_of_bounds_error_chip48_mode() {
        let memory = Memory::new(EmulationLevel::Chip48);
        assert_eq!(
            memory
                .read_byte(CHIP48_ADDRESSABLE_MEMORY_BYTES)
                .unwrap_err(),
            ErrorDetail::MemoryAddressOutOfBounds {
                address: CHIP48_ADDRESSABLE_MEMORY_BYTES as u16
            }
        );
    }

    #[test]
    fn test_read_byte_out_of_bounds_error_superchip11_mode() {
        let memory = Memory::new(EmulationLevel::SuperChip11 {
            octo_compatibility_mode: false,
        });
        assert_eq!(
            memory
                .read_byte(SUPERCHIP11_ADDRESSABLE_MEMORY_BYTES)
                .unwrap_err(),
            ErrorDetail::MemoryAddressOutOfBounds {
                address: SUPERCHIP11_ADDRESSABLE_MEMORY_BYTES as u16
            }
        );
    }

    #[test]
    fn test_read_two_bytes() {
        let mut memory = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        });
        memory.bytes[0x3] = 0xF2;
        memory.bytes[0x4] = 0x1C;
        assert_eq!(memory.read_two_bytes(0x3).unwrap(), 0xF21C);
    }

    #[test]
    fn test_read_two_bytes_out_of_bounds_chip8_small_error() {
        let memory = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: true,
            variable_cycle_timing: false,
        });
        assert_eq!(
            memory
                .read_two_bytes(CHIP8_SMALL_ADDRESSABLE_MEMORY_BYTES - 1)
                .unwrap_err(),
            ErrorDetail::MemoryAddressOutOfBounds {
                address: CHIP8_SMALL_ADDRESSABLE_MEMORY_BYTES as u16
            }
        );
    }

    #[test]
    fn test_read_two_bytes_out_of_bounds_chip8_large_error() {
        let memory = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        });
        assert_eq!(
            memory
                .read_two_bytes(CHIP8_LARGE_ADDRESSABLE_MEMORY_BYTES - 1)
                .unwrap_err(),
            ErrorDetail::MemoryAddressOutOfBounds {
                address: CHIP8_LARGE_ADDRESSABLE_MEMORY_BYTES as u16
            }
        );
    }

    #[test]
    fn test_write_byte() {
        let mut memory = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        });
        assert!(memory.write_byte(0x3, 0xF2).is_ok() && memory.bytes[0x3] == 0xF2);
    }

    #[test]
    fn test_write_byte_out_of_bounds_chip8_small_error() {
        let mut memory = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: true,
            variable_cycle_timing: false,
        });
        assert_eq!(
            memory
                .write_byte(CHIP8_SMALL_ADDRESSABLE_MEMORY_BYTES, 0xF2)
                .unwrap_err(),
            ErrorDetail::MemoryAddressOutOfBounds {
                address: CHIP8_SMALL_ADDRESSABLE_MEMORY_BYTES as u16
            }
        );
    }

    #[test]
    fn test_write_byte_out_of_bounds_chip8_large_error() {
        let mut memory = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        });
        assert_eq!(
            memory
                .write_byte(CHIP8_LARGE_ADDRESSABLE_MEMORY_BYTES, 0xF2)
                .unwrap_err(),
            ErrorDetail::MemoryAddressOutOfBounds {
                address: CHIP8_LARGE_ADDRESSABLE_MEMORY_BYTES as u16
            }
        );
    }

    #[test]
    fn test_read_bytes() {
        let mut memory = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        });
        memory.bytes[0x3] = 0xF2;
        memory.bytes[0x4] = 0x18;
        memory.bytes[0x5] = 0xCC;
        let mem_slice: &[u8] = memory.read_bytes(0x3, 3).unwrap();
        assert!(mem_slice[0] == 0xF2 && mem_slice[1] == 0x18 && mem_slice[2] == 0xCC);
    }

    #[test]
    fn test_read_bytes_out_of_bounds_chip8_small_error() {
        let memory = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: true,
            variable_cycle_timing: false,
        });
        assert_eq!(
            memory
                .read_bytes(CHIP8_SMALL_ADDRESSABLE_MEMORY_BYTES - 1, 2)
                .unwrap_err(),
            ErrorDetail::MemoryAddressOutOfBounds {
                address: CHIP8_SMALL_ADDRESSABLE_MEMORY_BYTES as u16
            }
        );
    }

    #[test]
    fn test_read_bytes_out_of_bounds_chip8_large_error() {
        let memory = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        });
        assert_eq!(
            memory
                .read_bytes(CHIP8_LARGE_ADDRESSABLE_MEMORY_BYTES - 1, 2)
                .unwrap_err(),
            ErrorDetail::MemoryAddressOutOfBounds {
                address: CHIP8_LARGE_ADDRESSABLE_MEMORY_BYTES as u16
            }
        );
    }

    #[test]
    fn test_write_bytes() {
        let mut memory = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        });
        let bytes_to_write: [u8; 3] = [0xF2, 0x18, 0xCC];
        memory.write_bytes(0x3, &bytes_to_write).unwrap();
        assert!(
            memory.bytes[0x3] == 0xF2 && memory.bytes[0x4] == 0x18 && memory.bytes[0x5] == 0xCC
        );
    }

    #[test]
    fn test_write_bytes_out_of_bounds_chip8_small_error() {
        let mut memory = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: true,
            variable_cycle_timing: false,
        });
        let bytes_to_write: [u8; 2] = [0xF2, 0x18];
        assert_eq!(
            memory
                .write_bytes(CHIP8_SMALL_ADDRESSABLE_MEMORY_BYTES - 1, &bytes_to_write)
                .unwrap_err(),
            ErrorDetail::MemoryAddressOutOfBounds {
                address: CHIP8_SMALL_ADDRESSABLE_MEMORY_BYTES as u16
            }
        );
    }

    #[test]
    fn test_write_bytes_out_of_bounds_chip8_large_error() {
        let mut memory = Memory::new(EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        });
        let bytes_to_write: [u8; 2] = [0xF2, 0x18];
        assert_eq!(
            memory
                .write_bytes(CHIP8_LARGE_ADDRESSABLE_MEMORY_BYTES - 1, &bytes_to_write)
                .unwrap_err(),
            ErrorDetail::MemoryAddressOutOfBounds {
                address: CHIP8_LARGE_ADDRESSABLE_MEMORY_BYTES as u16
            }
        );
    }
}
