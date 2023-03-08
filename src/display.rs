use crate::{error::ErrorDetail, EmulationLevel};
use std::cmp;
use std::ops::{Index, IndexMut};

/// The default CHIP-8 display size (64 x 32 pixels).
const LOW_RES_ROW_SIZE_PIXELS: usize = 64;
const LOW_RES_COLUMN_SIZE_PIXELS: usize = 32;
/// The high-resolution SUPER-CHIP 1.1 display size (128 x 64 pixels).
const HIGH_RES_ROW_SIZE_PIXELS: usize = 128;
const HIGH_RES_COLUMN_SIZE_PIXELS: usize = 64;

/// An abstraction of the CHIP-8 frame buffer.
///
/// This is only instantiated and written to from within the Chipolata crate, but is exposed
/// publically for read access by hosting applications so the display can be graphically rendered,
/// via a [StateSnapshot](crate::StateSnapshot) obtained from a call to
/// [Processor::export_state_snapshot()](crate::Processor::export_state_snapshot).
#[derive(Clone, Debug, PartialEq)]
pub struct Display {
    /// Logically this is a two-dimensional array to hold the state of the display pixels
    /// (1 means on, 0 means off).  Physically, due to the fact the array size isn't know at compile
    /// time (as the display size varies depending on [Processor::EmulationLevel]), this is implemented
    /// as a heap-allocated one-dimensional byte array, with the [std::ops::Index] trait implemented
    /// so as to simulate the expected 2D array indexing.
    ///
    /// Each inner array of bytes represents a row of the display, using one bit per pixel.  The outer
    /// array is the collection of rows.  A coordinate within the display is therefore accessed
    /// as `display[row][column]`. Note that (0, 0) is the top-left of the display, with positive
    /// coordinates extending right and down.
    row_size_bytes: usize,
    column_size_pixels: usize,
    pixels: Box<[u8]>,
}

// Allow the 1D Box<[u8]> to be indexed as a 2D array
impl Index<usize> for Display {
    type Output = [u8];

    fn index(&self, index: usize) -> &Self::Output {
        &self.pixels[index * self.row_size_bytes..(index + 1) * self.row_size_bytes]
    }
}

// Allow the 1D Box<[u8]> to be indexed as a 2D array mutably
impl IndexMut<usize> for Display {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.pixels[index * self.row_size_bytes..(index + 1) * self.row_size_bytes]
    }
}

impl Display {
    /// Constructor that returns a [Display] instance of default row and column size with all pixels
    /// set to off.
    pub(crate) fn new(emulation_level: EmulationLevel) -> Self {
        let row_size: usize;
        let column_size: usize;
        let pixels: Box<[u8]>;
        (row_size, column_size) = match emulation_level {
            EmulationLevel::SuperChip11 { .. } => {
                (HIGH_RES_ROW_SIZE_PIXELS / 8, HIGH_RES_COLUMN_SIZE_PIXELS)
            }
            _ => (LOW_RES_ROW_SIZE_PIXELS / 8, LOW_RES_COLUMN_SIZE_PIXELS),
        };
        pixels = vec![0x0; row_size * column_size].into_boxed_slice();
        Self {
            row_size_bytes: row_size,
            column_size_pixels: column_size,
            pixels,
        }
    }

    /// Getter that returns the display row size in bytes
    pub fn get_row_size_bytes(&self) -> usize {
        self.row_size_bytes
    }

    /// Getter that returns the display column size in pixels
    pub fn get_column_size_pixels(&self) -> usize {
        self.column_size_pixels
    }

    /// Clears the display by recreating the pixel array with default size and all pixels set to off.
    pub(crate) fn clear(&mut self) {
        self.pixels = vec![0x0; self.row_size_bytes * self.column_size_pixels].into_boxed_slice();
    }

    /// Draws a sprite to the display as per the CHIP-8 specification.  Returns a tuple: the first u8
    /// is the number of rows that collide with another sprite, the second u8 is the number of rows
    /// that are clipped by the bottom of the screen
    ///
    /// # Arguments
    ///
    /// * `x_start_pixel` - An zero-based integer giving the starting x coordinate of the sprite
    /// * `y_start_pixel` - An zero-based integer giving the starting y coordinate of the sprite
    /// * `sprite` - An array slice holding the bytes that make up the sprite
    /// * `double_width_sprite` - A boolean set to true if the sprite is two bytes wide
    pub(crate) fn draw_sprite(
        &mut self,
        x_start_pixel: usize,
        y_start_pixel: usize,
        sprite: &[u8],
        double_width_sprite: bool,
    ) -> Result<(u8, u8), ErrorDetail> {
        // Determine the height of the sprite in pixels (based on the length of the sprite byte array
        // and whether the sprite is one or two bytes in width)
        let sprite_height: usize = match double_width_sprite {
            true => sprite.len() / 2,
            false => sprite.len(),
        };
        // Determine whether the sprite must be clipped due to overflowing the bottom of the display
        let y_start_pixel: usize = y_start_pixel % self.column_size_pixels;
        let pixel_rows_to_draw: usize = cmp::min(
            sprite_height,
            self.column_size_pixels - cmp::min(self.column_size_pixels, y_start_pixel),
        );
        // Number of rows clipped (to be returned from method) is sprite height minus rows to draw
        // Allegedly this is used by SUPER-CHIP 1.1 however testing and further investigation suggests
        // this is unintended, and this definitely causes issues with some games.  So, this is hardwired
        // to 0 for now
        //let rows_clipped: u8 = (sprite_height - pixel_rows_to_draw) as u8;
        let rows_clipped: u8 = 0_u8;
        // Calculate the offset (in pixels) of the sprite X position relative to the start of the byte
        let x_offset = x_start_pixel % 8;
        // Calculate which horizontal display byte the sprite starts in (allowing wrapping)
        let x_byte = (x_start_pixel / 8) % self.row_size_bytes;
        // Variables to determine whether we need to inspect second and third bytes on each row
        // for collisions (based on whether the sprite is one or two bytes wide)
        let second_byte_needed: bool;
        let third_byte_needed: bool;
        match double_width_sprite {
            true => {
                // If the sprite does not begin in the final column then a second byte will always be
                // needed for a double-width sprite
                if x_byte == self.row_size_bytes - 1 {
                    second_byte_needed = false;
                } else {
                    second_byte_needed = true;
                };
                // If the sprite does not align to the start of a display byte and does not begin in
                // the penultimate byte of the display row then it will spill-over into a third
                // display row byte
                third_byte_needed = (x_offset > 0) && x_byte < self.row_size_bytes - 2;
            }
            false => {
                // If the sprite does not align to the start of a display byte and does not begin in the
                // final byte of the display row then it will spill-over into the next display row byte
                second_byte_needed = (x_offset > 0) && x_byte < self.row_size_bytes - 1;
                third_byte_needed = false; // never required for single-byte-width sprites
            }
        }
        // Keep track of whether any pixels are turned off as a result of drawing each sprite row
        let mut any_pixel_turned_off: bool;
        let mut rows_with_collisions: u8 = 0;
        // Loop for each row in the sprite
        for j in 0..pixel_rows_to_draw {
            // Get the byte index of the sprite left-hand byte for this row (for a normal width sprite
            // this will just equal j, but for a double-width sprite it will be 2 * j)
            let byte_index: usize = match double_width_sprite {
                true => j * 2,
                false => j,
            };
            any_pixel_turned_off = false;
            // Reference to the display byte affected
            let mut display_byte: &mut u8 = &mut self[y_start_pixel + j][x_byte];
            // Right bit-shift the sprite row to align with display byte
            let mut sprite_byte: u8 = sprite[byte_index] >> (x_offset as u8);
            // Check if display bit will be turned off by this operation (i.e. if a display bit and
            // a corresponding sprite bit are both set to 1 prior to the XOR operation)
            if (*display_byte & sprite_byte) > 0 {
                any_pixel_turned_off = true;
            }
            // Carry out the XOR operation to apply the sprite byte to the display byte
            *display_byte ^= sprite_byte;
            // Check whether the sprite spills-over to the next display byte and if so, repeat
            if second_byte_needed {
                // Reference to the subsequent display byte
                display_byte = &mut self[y_start_pixel + j][x_byte + 1];
                // Left-shift the first sprite byte to isolate and align the overspill portion
                sprite_byte = match x_offset {
                    0 => 0x0, // no overspill from first byte
                    _ => sprite[byte_index] << (8 - x_offset as u8),
                };
                // If this is a double-width sprite then we must also bitwise OR into this the
                // corresponding portion of the second sprite byte
                if double_width_sprite {
                    sprite_byte = sprite_byte | sprite[byte_index + 1] >> (x_offset as u8);
                }
                // Apply bit turn-off check
                if (*display_byte & sprite_byte) > 0 {
                    any_pixel_turned_off = true;
                }
                // Carry out the XOR
                *display_byte ^= sprite_byte;
            }
            if third_byte_needed {
                // Reference to the subsequent display byte
                display_byte = &mut self[y_start_pixel + j][x_byte + 2];
                // Left-shift the second sprite byte to isolate and align the overspill portion
                sprite_byte = sprite[byte_index + 1] << (8 - x_offset as u8);
                // Apply bit turn-off check
                if (*display_byte & sprite_byte) > 0 {
                    any_pixel_turned_off = true;
                }
                // Carry out the XOR
                *display_byte ^= sprite_byte;
            }
            if any_pixel_turned_off {
                rows_with_collisions += 1;
            }
        }
        Ok((rows_with_collisions, rows_clipped))
    }

    /// Scrolls the display right by 4 pixels (4 pixels as per the high-resolution display mode i.e.
    /// if in low-resolution mode this is the equivalent of 2 low-resolution pixels)
    pub(crate) fn scroll_display_right(&mut self) -> Result<(), ErrorDetail> {
        let n: usize = self.get_row_size_bytes();
        // Iterate through each row in turn, shifting the bytes in that row
        for row_index in 0..self.get_column_size_pixels() {
            // For each byte except the first, carry out the scroll as follows:
            // consider two consecutive bytes: ABCD EFGH | IJKL MNOP.  To scroll the second we move
            // the first nibble of the second byte into the second nibble, then move the second nibble
            // of the first byte into the first nibble of the second byte i.e. ABCD EFGH | EFGH IJKL
            // This is achieved by i) right-shifting the second byte by 4 bits, then
            // ii) left-shifting the first byte by 4 bits, then
            // iii) combining the results into the first byte with a bitwise OR
            for column_index in (1..n).rev() {
                self[row_index][column_index] =
                    (self[row_index][column_index] >> 4) | (self[row_index][column_index - 1] << 4);
            }
            self[row_index][0] = self[row_index][0] >> 4;
        }
        Ok(())
    }

    /// Scrolls the display left by 4 pixels (4 pixels as per the high-resolution display mode i.e.
    /// if in low-resolution mode this is the equivalent of 2 low-resolution pixels)
    pub(crate) fn scroll_display_left(&mut self) -> Result<(), ErrorDetail> {
        let n: usize = self.get_row_size_bytes() - 1;
        // Iterate through each row in turn, shifting the bytes in that row
        for row_index in 0..self.get_column_size_pixels() {
            // For each byte except the last, carry out the scroll as follows:
            // consider two consecutive bytes: ABCD EFGH | IJKL MNOP.  To scroll the first we move
            // the second nibble of the first byte into the first nibble, then move the first nibble
            // of the second byte into the second nibble of the first byte i.e. EFGH IJKL | IJKL MNOP
            // This is achieved by i) left-shifting the first byte by 4 bits, then
            // ii) right-shifting the second byte by 4 bits, then
            // iii) combining the results into the first byte with a bitwise OR
            for column_index in 0..n {
                self[row_index][column_index] =
                    (self[row_index][column_index] << 4) | (self[row_index][column_index + 1] >> 4);
            }
            self[row_index][n] = self[row_index][n] << 4;
        }
        Ok(())
    }

    /// Scrolls the display down by N pixels (N pixels as per the high-resolution display mode i.e.
    /// if in low-resolution mode this is the equivalent of N/2 low-resolution pixels)
    ///
    /// # Arguments
    ///
    /// * `n` - The number of pixels by which to scroll down
    pub(crate) fn scroll_display_down(&mut self, n: u8) -> Result<(), ErrorDetail> {
        let n: usize = n as usize;
        // Iterate through each row of the display in reverse from the last row back to the (n+1)th row
        for row_index in (n..self.get_column_size_pixels()).rev() {
            // By exploiting offsets within the internal 1D array representing the 2D display,
            // Set each row's bytes to the bytes from that n rows earlier
            self.pixels.copy_within(
                (row_index - n) * self.row_size_bytes..(row_index + 1 - n) * self.row_size_bytes,
                row_index * self.row_size_bytes,
            );
        }
        // Finally, fill the top n rows with 0s (effectively newly-created rows to replace those scrolled
        // off the bottom of the display)
        for row_index in 0..n {
            self[row_index as usize].fill(0x00);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_display_low_res() -> Display {
        let mut display: Display = Display::new(EmulationLevel::Chip48);
        // Setup test display as follows:
        // 00001111 01010101   (i.e. 0F 55 in hex)
        // 11110000 10101010   (i.e. F0 AA in hex)
        // 00110011 11001100   (i.e. 33 CC in hex)
        display[0][0] = 0x0F;
        display[0][1] = 0x55;
        display[1][0] = 0xF0;
        display[1][1] = 0xAA;
        display[2][0] = 0x33;
        display[2][1] = 0xCC;
        display
    }

    fn setup_test_display_low_res_right() -> Display {
        let mut display: Display = Display::new(EmulationLevel::Chip48);
        // Setup test display as follows:
        // 00001111 01010101   (i.e. 0F 55 in hex)
        // 11110000 10101010   (i.e. F0 AA in hex)
        // 00110011 11001100   (i.e. 33 CC in hex)
        display[0][6] = 0x0F;
        display[0][7] = 0x55;
        display[1][6] = 0xF0;
        display[1][7] = 0xAA;
        display[2][6] = 0x33;
        display[2][7] = 0xCC;
        display
    }

    fn setup_test_display_low_res_bottom() -> Display {
        let mut display: Display = Display::new(EmulationLevel::Chip48);
        // Setup test display as follows (at bottom of screen)
        // At row MAX-1:  00001111 01010101   (i.e. 0F 55 in hex)
        // At row MAX:    11110000 10101010   (i.e. F0 AA in hex)
        display[LOW_RES_COLUMN_SIZE_PIXELS - 2][0] = 0x0F;
        display[LOW_RES_COLUMN_SIZE_PIXELS - 2][1] = 0x55;
        display[LOW_RES_COLUMN_SIZE_PIXELS - 1][0] = 0xF0;
        display[LOW_RES_COLUMN_SIZE_PIXELS - 1][1] = 0xAA;
        display
    }

    fn setup_test_display_high_res() -> Display {
        let mut display: Display = Display::new(EmulationLevel::SuperChip11 {
            octo_compatibility_mode: false,
        });
        // Setup test display as follows:
        // 00001111 01010101 11100010  (i.e. 0F 55 E2 in hex)
        // 11110000 10101010 00011101  (i.e. F0 AA 1D in hex)
        // 00110011 11001100 10110100  (i.e. 33 CC B4 in hex)
        display[0][0] = 0x0F;
        display[0][1] = 0x55;
        display[0][2] = 0xE2;
        display[1][0] = 0xF0;
        display[1][1] = 0xAA;
        display[1][2] = 0x1D;
        display[2][0] = 0x33;
        display[2][1] = 0xCC;
        display[2][2] = 0xB4;
        display
    }

    fn setup_test_display_high_res_right() -> Display {
        let mut display: Display = Display::new(EmulationLevel::SuperChip11 {
            octo_compatibility_mode: false,
        });
        // Setup test display as follows:
        // 00001111 01010101 11100010  (i.e. 0F 55 E2 in hex)
        // 11110000 10101010 00011101  (i.e. F0 AA 1D in hex)
        // 00110011 11001100 10110100  (i.e. 33 CC B4 in hex)
        display[0][13] = 0x0F;
        display[0][14] = 0x55;
        display[0][15] = 0xE2;
        display[1][13] = 0xF0;
        display[1][14] = 0xAA;
        display[1][15] = 0x1D;
        display[2][13] = 0x33;
        display[2][14] = 0xCC;
        display[2][15] = 0xB4;
        display
    }

    fn setup_test_display_high_res_bottom() -> Display {
        let mut display: Display = Display::new(EmulationLevel::SuperChip11 {
            octo_compatibility_mode: false,
        });
        // Setup test display as follows (at bottom of screen)
        // At row MAX-1:  00001111 01010101   (i.e. 0F 55 in hex)
        // At row MAX:    11110000 10101010   (i.e. F0 AA in hex)
        display[HIGH_RES_COLUMN_SIZE_PIXELS - 2][0] = 0x0F;
        display[HIGH_RES_COLUMN_SIZE_PIXELS - 2][1] = 0x55;
        display[HIGH_RES_COLUMN_SIZE_PIXELS - 1][0] = 0xF0;
        display[HIGH_RES_COLUMN_SIZE_PIXELS - 1][1] = 0xAA;
        display
    }

    fn setup_test_display_high_res_scroll_left() -> Display {
        let mut display: Display = Display::new(EmulationLevel::SuperChip11 {
            octo_compatibility_mode: false,
        });
        // Setup test display as follows, for every row:
        // 00011001 00011001 .. 00011001  (i.e. 19 19 .. 19)
        for i in 0..display.get_column_size_pixels() {
            for j in 0..display.get_row_size_bytes() {
                display[i][j] = 0x19;
            }
        }
        display
    }

    fn setup_test_display_high_res_scroll_right() -> Display {
        let mut display: Display = Display::new(EmulationLevel::SuperChip11 {
            octo_compatibility_mode: false,
        });
        // Setup test display as follows, for every row:
        // 01110100 01110100 .. 01110100  (i.e. 74 74 .. 74)
        for i in 0..display.get_column_size_pixels() {
            for j in 0..display.get_row_size_bytes() {
                display[i][j] = 0x74;
            }
        }
        display
    }

    fn setup_test_display_high_res_scroll_down() -> Display {
        let mut display: Display = Display::new(EmulationLevel::SuperChip11 {
            octo_compatibility_mode: false,
        });
        // Setup test display as follows.  First row has all pixels turned on i.e. all bytes are 0xFF
        // All other rows have all pixels turned off i.e. all bytes are 0x00
        // 11111111 11111111 .. 11111111    (i.e. FF FF .. FF)
        // 00000000 00000000 .. 00000000    (i.e. 00 00 .. 00)
        //    ..       ..          ..
        // 00000000 00000000 .. 00000000    (i.e. 00 00 .. 00)
        for i in 1..display.get_column_size_pixels() {
            for j in 0..display.get_row_size_bytes() {
                display[i][j] = 0x00;
            }
        }
        for j in 0..display.get_row_size_bytes() {
            display[0][j] = 0xFF;
        }
        display
    }

    fn setup_test_sprite() -> [u8; 2] {
        // Setup test sprite as follows:
        // 10110110   (i.e. B6 in hex)
        // 11100011   (i.e. E3 in hex)
        let sprite: [u8; 2] = [0xB6, 0xE3];
        sprite
    }

    fn setup_test_sprite_large() -> [u8; 32] {
        // Setup test sprite as follows:
        // 10110110 11000101  (i.e. B6 C5 in hex)
        // 11100011 00010011  (i.e. E3 13 in hex)
        // 01111011 11000001  (i.e. 7B C1 in hex)
        // 01001100 11000011  (i.e. 4C C3 in hex)
        // 10110110 11000101  (i.e. B6 C5 in hex)
        // 11100011 00010011  (i.e. E3 13 in hex)
        // 01111011 11000001  (i.e. 7B C1 in hex)
        // 01001100 11000011  (i.e. 4C C3 in hex)
        // 10110110 11000101  (i.e. B6 C5 in hex)
        // 11100011 00010011  (i.e. E3 13 in hex)
        // 01111011 11000001  (i.e. 7B C1 in hex)
        // 01001100 11000011  (i.e. 4C C3 in hex)
        // 10110110 11000101  (i.e. B6 C5 in hex)
        // 11100011 00010011  (i.e. E3 13 in hex)
        // 01111011 11000001  (i.e. 7B C1 in hex)
        // 01001100 11000011  (i.e. 4C C3 in hex)
        let sprite: [u8; 32] = [
            0xB6, 0xC5, 0xE3, 0x13, 0x7B, 0xC1, 0x4C, 0xC3, 0xB6, 0xC5, 0xE3, 0x13, 0x7B, 0xC1,
            0x4C, 0xC3, 0xB6, 0xC5, 0xE3, 0x13, 0x7B, 0xC1, 0x4C, 0xC3, 0xB6, 0xC5, 0xE3, 0x13,
            0x7B, 0xC1, 0x4C, 0xC3,
        ];
        sprite
    }

    #[test]
    fn test_draw_sprite_aligned() {
        let mut display: Display = setup_test_display_low_res();
        let sprite: [u8; 2] = setup_test_sprite();
        // Draw sprite at coordinate (0, 0)
        let (rows_with_collisions, rows_clipped) =
            display.draw_sprite(0, 0, &sprite, false).unwrap();
        // Result should be:
        // 10111001 01010101   (i.e. B9 55 in hex)
        // 00010011 10101010   (i.e. 13 AA in hex)
        // 00110011 11001100   (i.e. 33 CC in hex)
        assert!(
            rows_with_collisions == 2
                && rows_clipped == 0
                && display[0][0] == 0xB9
                && display[0][1] == 0x55
                && display[1][0] == 0x13
                && display[1][1] == 0xAA
                && display[2][0] == 0x33
                && display[2][1] == 0xCC
        )
    }

    #[test]
    fn test_draw_sprite_unaligned() {
        let mut display: Display = setup_test_display_low_res();
        let sprite: [u8; 2] = setup_test_sprite();
        // Draw sprite at coordinate (3, 0)
        let (rows_with_collisions, rows_clipped) =
            display.draw_sprite(3, 0, &sprite, false).unwrap();
        // Result should be:
        // 00011001 10010101   (i.e. 19 95 in hex)
        // 11101100 11001010   (i.e. EC CA in hex)
        // 00110011 11001100   (i.e. 33 CC in hex)
        assert!(
            rows_with_collisions == 2
                && rows_clipped == 0
                && display[0][0] == 0x19
                && display[0][1] == 0x95
                && display[1][0] == 0xEC
                && display[1][1] == 0xCA
                && display[2][0] == 0x33
                && display[2][1] == 0xCC
        )
    }

    #[test]
    fn test_draw_sprite_unaligned_overflow_right() {
        let mut display: Display = setup_test_display_low_res_right();
        let sprite: [u8; 2] = setup_test_sprite();
        // Draw sprite at coordinate (57, 0)
        let (rows_with_collisions, rows_clipped) =
            display.draw_sprite(57, 0, &sprite, false).unwrap();
        // Result should be:
        // 00001111 00001110   (i.e. 0F 0E in hex)
        // 11110000 11011011   (i.e. F0 DB in hex)
        // 00110011 11001100   (i.e. 33 CC in hex)
        assert!(
            rows_with_collisions == 2
                && rows_clipped == 0
                && display[0][6] == 0x0F
                && display[0][7] == 0x0E
                && display[1][6] == 0xF0
                && display[1][7] == 0xDB
                && display[2][6] == 0x33
                && display[2][7] == 0xCC
        )
    }

    #[test]
    fn test_draw_sprite_aligned_overflow_bottom() {
        let mut display: Display = setup_test_display_low_res_bottom();
        let sprite: [u8; 2] = setup_test_sprite();
        // Draw sprite at coordinate (0, final row)
        let (rows_with_collisions, rows_clipped) = display
            .draw_sprite(0, LOW_RES_COLUMN_SIZE_PIXELS - 1, &sprite, false)
            .unwrap();
        // Result should be:
        // 00001111 01010101   (i.e. 0F 55 in hex)
        // 01000110 10101010   (i.e. 46 AA in hex)
        assert!(
            rows_with_collisions == 1
                && rows_clipped == 0 // this is disabled; would be 1
                && display[LOW_RES_COLUMN_SIZE_PIXELS - 2][0] == 0x0F
                && display[LOW_RES_COLUMN_SIZE_PIXELS - 2][1] == 0x55
                && display[LOW_RES_COLUMN_SIZE_PIXELS - 1][0] == 0x46
                && display[LOW_RES_COLUMN_SIZE_PIXELS - 1][1] == 0xAA
        )
    }

    #[test]
    fn test_draw_sprite_large_aligned() {
        let mut display: Display = setup_test_display_high_res();
        let sprite: [u8; 32] = setup_test_sprite_large();
        // Draw sprite at coordinate (0, 0)
        let (rows_with_collisions, rows_clipped) =
            display.draw_sprite(0, 0, &sprite, true).unwrap();
        // Result should be:
        // 10111001 10010000 11100010   (i.e. B9 90 E2 in hex)
        // 00010011 10111001 00011101   (i.e. 13 B9 1D in hex)
        // 01001000 00001101 10110100   (i.e. 48 0D B4 in hex)
        // 01001100 11000011 00000000   (i.e. 4C C3 00 in hex)
        // 10110110 11000101 00000000   (i.e. B6 C5 00 in hex)
        // 11100011 00010011 00000000   (i.e. E3 13 00 in hex)
        // 01111011 11000001 00000000   (i.e. 7B C1 00 in hex)
        // 01001100 11000011 00000000   (i.e. 4C C3 00 in hex)
        // 10110110 11000101 00000000   (i.e. B6 C5 00 in hex)
        // 11100011 00010011 00000000   (i.e. E3 13 00 in hex)
        // 01111011 11000001 00000000   (i.e. 7B C1 00 in hex)
        // 01001100 11000011 00000000   (i.e. 4C C3 00 in hex)
        // 10110110 11000101 00000000   (i.e. B6 C5 00 in hex)
        // 11100011 00010011 00000000   (i.e. E3 13 00 in hex)
        // 01111011 11000001 00000000   (i.e. 7B C1 00 in hex)
        // 01001100 11000011 00000000   (i.e. 4C C3 00 in hex)
        assert!(
            rows_with_collisions == 3
                && rows_clipped == 0
                && display[0][0] == 0xB9
                && display[0][1] == 0x90
                && display[0][2] == 0xE2
                && display[1][0] == 0x13
                && display[1][1] == 0xB9
                && display[1][2] == 0x1D
                && display[2][0] == 0x48
                && display[2][1] == 0x0D
                && display[2][2] == 0xB4
                && display[3][0] == 0x4C
                && display[3][1] == 0xC3
                && display[3][2] == 0x00
                && display[4][0] == 0xB6
                && display[4][1] == 0xC5
                && display[4][2] == 0x00
                && display[5][0] == 0xE3
                && display[5][1] == 0x13
                && display[5][2] == 0x00
                && display[6][0] == 0x7B
                && display[6][1] == 0xC1
                && display[6][2] == 0x00
                && display[7][0] == 0x4C
                && display[7][1] == 0xC3
                && display[7][2] == 0x00
                && display[8][0] == 0xB6
                && display[8][1] == 0xC5
                && display[8][2] == 0x00
                && display[9][0] == 0xE3
                && display[9][1] == 0x13
                && display[9][2] == 0x00
                && display[10][0] == 0x7B
                && display[10][1] == 0xC1
                && display[10][2] == 0x00
                && display[11][0] == 0x4C
                && display[11][1] == 0xC3
                && display[11][2] == 0x00
                && display[12][0] == 0xB6
                && display[12][1] == 0xC5
                && display[12][2] == 0x00
                && display[13][0] == 0xE3
                && display[13][1] == 0x13
                && display[13][2] == 0x00
                && display[14][0] == 0x7B
                && display[14][1] == 0xC1
                && display[14][2] == 0x00
                && display[15][0] == 0x4C
                && display[15][1] == 0xC3
                && display[15][2] == 0x00
        )
    }

    #[test]
    fn test_draw_sprite_large_unaligned() {
        let mut display: Display = setup_test_display_high_res();
        let sprite: [u8; 32] = setup_test_sprite_large();
        // Draw sprite at coordinate (0, 0)
        let (rows_with_collisions, rows_clipped) =
            display.draw_sprite(3, 0, &sprite, true).unwrap();
        // Result should be:
        // 00011001 10001101 01000010   (i.e. 19 8D 42 in hex)
        // 11101100 11001000 01111101   (i.e. EC C8 7D in hex)
        // 00111100 10110100 10010100   (i.e. 3C B4 94 in hex)
        // 00001001 10011000 01100000   (i.e. 09 98 60 in hex)
        // 00010110 11011000 10100000   (i.e. 16 D8 A0 in hex)
        // 00011100 01100010 01100000   (i.e. 1C 62 60 in hex)
        // 00001111 01111000 00100000   (i.e. 0F 78 20 in hex)
        // 00001001 10011000 01100000   (i.e. 09 98 60 in hex)
        // 00010110 11011000 10100000   (i.e. 16 D8 A0 in hex)
        // 00011100 01100010 01100000   (i.e. 1C 62 60 in hex)
        // 00001111 01111000 00100000   (i.e. 0F 78 20 in hex)
        // 00001001 10011000 01100000   (i.e. 09 98 60 in hex)
        // 00010110 11011000 10100000   (i.e. 16 D8 A0 in hex)
        // 00011100 01100010 01100000   (i.e. 1C 62 60 in hex)
        // 00001111 01111000 00100000   (i.e. 0F 78 20 in hex)
        // 00001001 10011000 01100000   (i.e. 09 98 60 in hex)
        assert!(
            rows_with_collisions == 3
                && rows_clipped == 0
                && display[0][0] == 0x19
                && display[0][1] == 0x8D
                && display[0][2] == 0x42
                && display[1][0] == 0xEC
                && display[1][1] == 0xC8
                && display[1][2] == 0x7D
                && display[2][0] == 0x3C
                && display[2][1] == 0xB4
                && display[2][2] == 0x94
                && display[3][0] == 0x09
                && display[3][1] == 0x98
                && display[3][2] == 0x60
                && display[4][0] == 0x16
                && display[4][1] == 0xD8
                && display[4][2] == 0xA0
                && display[5][0] == 0x1C
                && display[5][1] == 0x62
                && display[5][2] == 0x60
                && display[6][0] == 0x0F
                && display[6][1] == 0x78
                && display[6][2] == 0x20
                && display[7][0] == 0x09
                && display[7][1] == 0x98
                && display[7][2] == 0x60
                && display[8][0] == 0x16
                && display[8][1] == 0xD8
                && display[8][2] == 0xA0
                && display[9][0] == 0x1C
                && display[9][1] == 0x62
                && display[9][2] == 0x60
                && display[10][0] == 0x0F
                && display[10][1] == 0x78
                && display[10][2] == 0x20
                && display[11][0] == 0x09
                && display[11][1] == 0x98
                && display[11][2] == 0x60
                && display[12][0] == 0x16
                && display[12][1] == 0xD8
                && display[12][2] == 0xA0
                && display[13][0] == 0x1C
                && display[13][1] == 0x62
                && display[13][2] == 0x60
                && display[14][0] == 0x0F
                && display[14][1] == 0x78
                && display[14][2] == 0x20
                && display[15][0] == 0x09
                && display[15][1] == 0x98
                && display[15][2] == 0x60
        )
    }

    #[test]
    fn test_draw_sprite_large_unaligned_overflow_right() {
        let mut display: Display = setup_test_display_high_res_right();
        let sprite: [u8; 32] = setup_test_sprite_large();
        // Draw sprite at coordinate (114, 0)
        let (rows_with_collisions, rows_clipped) =
            display.draw_sprite(114, 0, &sprite, true).unwrap();
        // Result should be:
        // 00001111 01111000 01010011   (i.e. 0F 78 53 in hex)
        // 11110000 10010010 11011001   (i.e. F0 92 D9 in hex)
        // 00110011 11010010 01000110   (i.e. 33 D2 44 in hex)
        // 00000000 00010011 00110000   (i.e. 00 13 30 in hex)
        // 00000000 00101101 10110001   (i.e. 00 2D B1 in hex)
        // 00000000 00111000 11000100   (i.e. 00 38 C4 in hex)
        // 00000000 00011110 11110000   (i.e. 00 1E F0 in hex)
        // 00000000 00010011 00110000   (i.e. 00 13 30 in hex)
        // 00000000 00101101 10110001   (i.e. 00 2D B1 in hex)
        // 00000000 00111000 11000100   (i.e. 00 38 C4 in hex)
        // 00000000 00011110 11110000   (i.e. 00 1E F0 in hex)
        // 00000000 00010011 00110000   (i.e. 00 13 30 in hex)
        // 00000000 00101101 10110001   (i.e. 00 2D B1 in hex)
        // 00000000 00111000 11000100   (i.e. 00 38 C4 in hex)
        // 00000000 00011110 11110000   (i.e. 00 1E F0 in hex)
        // 00000000 00010011 00110000   (i.e. 00 13 30 in hex)
        assert!(
            rows_with_collisions == 3
                && rows_clipped == 0
                && display[0][13] == 0x0F
                && display[0][14] == 0x78
                && display[0][15] == 0x53
                && display[1][13] == 0xF0
                && display[1][14] == 0x92
                && display[1][15] == 0xD9
                && display[2][13] == 0x33
                && display[2][14] == 0xD2
                && display[2][15] == 0x44
                && display[3][13] == 0x00
                && display[3][14] == 0x13
                && display[3][15] == 0x30
                && display[4][13] == 0x00
                && display[4][14] == 0x2D
                && display[4][15] == 0xB1
                && display[5][13] == 0x00
                && display[5][14] == 0x38
                && display[5][15] == 0xC4
                && display[6][13] == 0x00
                && display[6][14] == 0x1E
                && display[6][15] == 0xF0
                && display[7][13] == 0x00
                && display[7][14] == 0x13
                && display[7][15] == 0x30
                && display[8][13] == 0x00
                && display[8][14] == 0x2D
                && display[8][15] == 0xB1
                && display[9][13] == 0x00
                && display[9][14] == 0x38
                && display[9][15] == 0xC4
                && display[10][13] == 0x00
                && display[10][14] == 0x1E
                && display[10][15] == 0xF0
                && display[11][13] == 0x00
                && display[11][14] == 0x13
                && display[11][15] == 0x30
                && display[12][13] == 0x00
                && display[12][14] == 0x2D
                && display[12][15] == 0xB1
                && display[13][13] == 0x00
                && display[13][14] == 0x38
                && display[13][15] == 0xC4
                && display[14][13] == 0x00
                && display[14][14] == 0x1E
                && display[14][15] == 0xF0
                && display[15][13] == 0x00
                && display[15][14] == 0x13
                && display[15][15] == 0x30
        );
    }

    #[test]
    fn test_draw_sprite_large_aligned_overflow_bottom() {
        let mut display: Display = setup_test_display_high_res_bottom();
        let sprite: [u8; 32] = setup_test_sprite_large();
        // Draw sprite at coordinate (0, final row)
        let (rows_with_collisions, rows_clipped) = display
            .draw_sprite(0, HIGH_RES_COLUMN_SIZE_PIXELS - 1, &sprite, true)
            .unwrap();
        // Result should be:
        // 00001111 01010101    (i.e. 0F 55 in hex)
        // 01000110 01101111    (i.e. 46 6F in hex)

        assert!(
            rows_with_collisions == 1
                && rows_clipped == 0 // this is disabled; would be 15
                && display[HIGH_RES_COLUMN_SIZE_PIXELS - 2][0] == 0x0F
                && display[HIGH_RES_COLUMN_SIZE_PIXELS - 2][1] == 0x55
                && display[HIGH_RES_COLUMN_SIZE_PIXELS - 1][0] == 0x46
                && display[HIGH_RES_COLUMN_SIZE_PIXELS - 1][1] == 0x6F
        )
    }

    #[test]
    fn test_draw_sprite_no_pixels_unset() {
        let mut display: Display = setup_test_display_low_res();
        let sprite: [u8; 2] = [0x0, 0x0];
        // Draw sprite at coordinate (0, 0)
        let (rows_with_collisions, rows_clipped) =
            display.draw_sprite(0, 0, &sprite, false).unwrap();
        // Result should be:
        // 00001111 01010101   (i.e. 0F 55 in hex)
        // 11110000 10101010   (i.e. F0 AA in hex)
        // 00110011 11001100   (i.e. 33 CC in hex)
        assert!(
            rows_with_collisions == 0
                && rows_clipped == 0
                && display[0][0] == 0x0F
                && display[0][1] == 0x55
                && display[1][0] == 0xF0
                && display[1][1] == 0xAA
                && display[2][0] == 0x33
                && display[2][1] == 0xCC
        )
    }

    #[test]
    fn test_scroll_display_left() {
        let mut display: Display = setup_test_display_high_res_scroll_left();
        display.scroll_display_left().unwrap();
        let mut all_bytes_correct: bool = true;
        // Each byte should have scrolled from 00011001 (i.e. 0x19) to 10010001 (i.e. 0x91)
        // except for the last byte in each row, which will be 10010000 (i.e. 0x90)
        'outer: for i in 0..display.get_column_size_pixels() {
            for j in 0..display.get_row_size_bytes() - 1 {
                if display[i][j] != 0x91 {
                    all_bytes_correct = false;
                    break 'outer;
                }
            }
            if display[i][display.get_row_size_bytes() - 1] != 0x90 {
                all_bytes_correct = false;
                break 'outer;
            }
        }
        assert!(all_bytes_correct);
    }

    #[test]
    fn test_scroll_display_right() {
        let mut display: Display = setup_test_display_high_res_scroll_right();
        display.scroll_display_right().unwrap();
        let mut all_bytes_correct: bool = true;
        // Each byte should have scrolled from 01110100 (i.e. 0x74) to 01000111 (i.e. 0x47)
        // except for the first byte in each row, which will be 00000111 (i.e. 0x07)
        'outer: for i in 0..display.get_column_size_pixels() {
            for j in 1..display.get_row_size_bytes() {
                if display[i][j] != 0x47 {
                    all_bytes_correct = false;
                    break 'outer;
                }
            }
            if display[i][0] != 0x07 {
                all_bytes_correct = false;
                break 'outer;
            }
        }
        assert!(all_bytes_correct);
    }

    #[test]
    fn test_scroll_display_down() {
        let mut display: Display = setup_test_display_high_res_scroll_down();
        display.scroll_display_down(7).unwrap();
        let mut all_bytes_correct: bool = true;
        // Row 0 should be fully filled with 0x00 bytes
        // Row 6 should be fully filled with 0x00 bytes
        // Row 7 should be fully filled with 0xFF bytes
        // Row 8 should be fully filled with 0x00 bytes
        // except for the first byte in each row, which will be 00000111 (i.e. 0x07)
        for j in 0..display.get_row_size_bytes() {
            if display[0][j] != 0x00 {
                all_bytes_correct = false;
                break;
            }
            if display[6][j] != 0x00 {
                all_bytes_correct = false;
                break;
            }
            if display[7][j] != 0xFF {
                all_bytes_correct = false;
                break;
            }
            if display[8][j] != 0x00 {
                all_bytes_correct = false;
                break;
            }
        }
        assert!(all_bytes_correct);
    }
}
