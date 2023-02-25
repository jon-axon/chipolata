use crate::error::ErrorDetail;
use std::cmp;

/// The default CHIP-8 display size (64 x 32 pixels).
const DISPLAY_ROW_SIZE_PIXELS: usize = 64;
const DISPLAY_COLUMN_SIZE_PIXELS: usize = 32;

/// An abstraction of the CHIP-8 frame buffer.
///
/// This is only instantiated and written to from within the Chipolata crate, but is exposed
/// publically for read access by hosting applications so the display can be graphically rendered,
/// via a [StateSnapshot](crate::StateSnapshot) obtained from a call to
/// [Processor::export_state_snapshot()](crate::Processor::export_state_snapshot).
#[derive(Clone, Debug, PartialEq)]
pub struct Display {
    /// A two-dimensional array to hold the state of the display pixels (1 means on, 0 means off).
    ///
    /// Each inner array represents a row of the display, using one bit per pixel.  The outer
    /// array is the collection of rows.  A coordinate within the display is therefore accessed
    /// as `pixels[row][column]`. Note that (0, 0) is the top-left of the display, with positive
    /// coordinates extending right and down.
    pub pixels: [[u8; DISPLAY_ROW_SIZE_PIXELS / 8]; DISPLAY_COLUMN_SIZE_PIXELS],
}

impl Display {
    /// Constructor that returns a [Display] instance of default row and column size with all pixels
    /// set to off.
    pub(crate) fn new() -> Self {
        Self {
            pixels: [[0x0; DISPLAY_ROW_SIZE_PIXELS / 8]; DISPLAY_COLUMN_SIZE_PIXELS],
        }
    }

    /// Clears the display by recreating the pixel array with default size and all pixels set to off.
    pub(crate) fn clear(&mut self) {
        self.pixels = [[0x0; DISPLAY_ROW_SIZE_PIXELS / 8]; DISPLAY_COLUMN_SIZE_PIXELS];
    }

    /// Draws a sprite to the display as per the CHIP-8 specification.
    ///
    /// # Arguments
    ///
    /// * `x_start_pixel` - An zero-based integer giving the starting x coordinate of the sprite
    /// * `y_start_pixel` - An zero-based integer giving the starting y coordinate of the sprite
    /// * `sprite` - An array slice holding the bytes that make up the sprite
    pub(crate) fn draw_sprite(
        &mut self,
        x_start_pixel: usize,
        y_start_pixel: usize,
        sprite: &[u8],
    ) -> Result<bool, ErrorDetail> {
        // Sprites are one byte wide, so the height (in pixels) is the length of the sprite byte array,
        // however cap this if necessary so the sprite does not draw off the bottom of the display
        let pixel_rows: usize = cmp::min(
            sprite.len(),
            DISPLAY_COLUMN_SIZE_PIXELS - cmp::min(DISPLAY_COLUMN_SIZE_PIXELS, y_start_pixel),
        );
        // Calculate the offset (in pixels) of the sprite X position relative to the start of the byte
        let x_offset = x_start_pixel % 8;
        // Calculate which horizontal display byte the sprite starts in (allowing wrapping)
        let x_byte = (x_start_pixel / 8) % 8;
        // If the sprite does not align to the start of a display byte and does not fall within the
        // final byte of the display row then it will spil-over into the next display row byte
        let second_byte_needed: bool = (x_offset > 0) && x_byte < (DISPLAY_ROW_SIZE_PIXELS / 8) - 1;
        // Keep track of whether any pixels are turned off as a result of drawing the sprite, to return
        let mut any_pixel_turned_off: bool = false;
        // Loop for each row in the sprite
        for j in 0..pixel_rows {
            // Reference to the display byte affected
            let mut display_byte: &mut u8 = &mut self.pixels[y_start_pixel + j][x_byte];
            // Right bit-shift the sprite row to align with display byte
            let mut sprite_byte: u8 = sprite[j] >> (x_offset as u8);
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
                display_byte = &mut self.pixels[y_start_pixel + j][x_byte + 1];
                // Left-shift the sprite row to isolate and align the overspill portion
                sprite_byte = sprite[j] << (8 - x_offset as u8);
                // Apply bit turn-off check
                if (*display_byte & sprite_byte) > 0 {
                    any_pixel_turned_off = true;
                }
                // Carry out the XOR
                *display_byte ^= sprite_byte;
            }
        }
        Ok(any_pixel_turned_off)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_display() -> Display {
        let mut display: Display = Display::new();
        // Setup test display as follows:
        // 0000111101010101   (i.e. 0F 55 in hex)
        // 1111000010101010   (i.e. F0 AA in hex)
        // 0011001111001100   (i.e. 33 CC in hex)
        display.pixels[0][0] = 0x0F;
        display.pixels[0][1] = 0x55;
        display.pixels[1][0] = 0xF0;
        display.pixels[1][1] = 0xAA;
        display.pixels[2][0] = 0x33;
        display.pixels[2][1] = 0xCC;
        display
    }

    fn setup_test_sprite() -> [u8; 2] {
        // Setup test sprite as follows:
        // 10110110   (i.e. B6 in hex)
        // 11100011   (i.e. E3 in hex)
        let sprite: [u8; 2] = [0xB6, 0xE3];
        sprite
    }

    #[test]
    fn test_draw_sprite_aligned() {
        let mut display: Display = setup_test_display();
        let sprite: [u8; 2] = setup_test_sprite();
        // Draw sprite at coordinate (0, 0)
        let any_pixel_turned_off: bool = display.draw_sprite(0, 0, &sprite).unwrap();
        // Result should be:
        // 1011100101010101   (i.e. B9 55 in hex)
        // 0001001110101010   (i.e. 13 AA in hex)
        // 0011001111001100   (i.e. 33 CC in hex)
        assert!(
            any_pixel_turned_off == true
                && display.pixels[0][0] == 0xB9
                && display.pixels[0][1] == 0x55
                && display.pixels[1][0] == 0x13
                && display.pixels[1][1] == 0xAA
                && display.pixels[2][0] == 0x33
                && display.pixels[2][1] == 0xCC
        )
    }

    #[test]
    fn test_draw_sprite_unaligned() {
        let mut display: Display = setup_test_display();
        let sprite: [u8; 2] = setup_test_sprite();
        // Draw sprite at coordinate (3, 0)
        let any_pixel_turned_off: bool = display.draw_sprite(3, 0, &sprite).unwrap();
        // Result should be:
        // 0001100110010101   (i.e. 19 95 in hex)
        // 1110110011001010   (i.e. EC CA in hex)
        // 0011001111001100   (i.e. 33 CC in hex)
        assert!(
            any_pixel_turned_off == true
                && display.pixels[0][0] == 0x19
                && display.pixels[0][1] == 0x95
                && display.pixels[1][0] == 0xEC
                && display.pixels[1][1] == 0xCA
                && display.pixels[2][0] == 0x33
                && display.pixels[2][1] == 0xCC
        )
    }

    #[test]
    fn test_draw_sprite_unaligned_overflow_right() {
        let mut display: Display = setup_test_display();
        let sprite: [u8; 2] = setup_test_sprite();
        // Draw sprite at coordinate (9, 0)
        let any_pixel_turned_off: bool = display.draw_sprite(9, 0, &sprite).unwrap();
        // Result should be:
        // 0000111100001110   (i.e. 0F 0E in hex)
        // 1111000011011011   (i.e. F0 DB in hex)
        // 0011001111001100   (i.e. 33 CC in hex)
        assert!(
            any_pixel_turned_off == true
                && display.pixels[0][0] == 0x0F
                && display.pixels[0][1] == 0x0E
                && display.pixels[1][0] == 0xF0
                && display.pixels[1][1] == 0xDB
                && display.pixels[2][0] == 0x33
                && display.pixels[2][1] == 0xCC
        )
    }

    #[test]
    fn test_draw_sprite_aligned_overflow_bottom() {
        let mut display: Display = Display::new();
        // Setup test display as follows (at bottom of screen)
        // At row MAX-1:  0000111101010101   (i.e. 0F 55 in hex)
        // At row MAX:    1111000010101010   (i.e. F0 AA in hex)
        display.pixels[DISPLAY_COLUMN_SIZE_PIXELS - 2][0] = 0x0F;
        display.pixels[DISPLAY_COLUMN_SIZE_PIXELS - 2][1] = 0x55;
        display.pixels[DISPLAY_COLUMN_SIZE_PIXELS - 1][0] = 0xF0;
        display.pixels[DISPLAY_COLUMN_SIZE_PIXELS - 1][1] = 0xAA;
        let sprite: [u8; 2] = setup_test_sprite();
        // Draw sprite at coordinate (0, final row)
        let any_pixel_turned_off: bool = display
            .draw_sprite(0, DISPLAY_COLUMN_SIZE_PIXELS - 1, &sprite)
            .unwrap();
        // Result should be:
        // 0000111101010101   (i.e. 0F 55 in hex)
        // 0100011010101010   (i.e. 46 AA in hex)
        assert!(
            any_pixel_turned_off == true
                && display.pixels[DISPLAY_COLUMN_SIZE_PIXELS - 2][0] == 0x0F
                && display.pixels[DISPLAY_COLUMN_SIZE_PIXELS - 2][1] == 0x55
                && display.pixels[DISPLAY_COLUMN_SIZE_PIXELS - 1][0] == 0x46
                && display.pixels[DISPLAY_COLUMN_SIZE_PIXELS - 1][1] == 0xAA
        )
    }

    #[test]
    fn test_draw_sprite_no_pixels_unset() {
        let mut display: Display = setup_test_display();
        let sprite: [u8; 2] = [0x0, 0x0];
        // Draw sprite at coordinate (0, 0)
        let any_pixel_turned_off: bool = display.draw_sprite(0, 0, &sprite).unwrap();
        // Result should be:
        // 0000111101010101   (i.e. 0F 55 in hex)
        // 1111000010101010   (i.e. F0 AA in hex)
        // 0011001111001100   (i.e. 33 CC in hex)
        assert!(
            any_pixel_turned_off == false
                && display.pixels[0][0] == 0x0F
                && display.pixels[0][1] == 0x55
                && display.pixels[1][0] == 0xF0
                && display.pixels[1][1] == 0xAA
                && display.pixels[2][0] == 0x33
                && display.pixels[2][1] == 0xCC
        )
    }
}
