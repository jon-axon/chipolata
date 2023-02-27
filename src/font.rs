/// The size of each character of the default CHIP-8 font in bytes.
const CHIP8_CHAR_SIZE: usize = 5;
/// The sprites of the default CHIP-8 font, where each character is one byte wide
/// and `CHIP8_CHAR_SIZE` bytes tall.  Each bit represents one pixel in the sprite.
const CHIP8_FONT_DATA: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
/// The size of each character of the default SUPER-CHIP 1.1 font in bytes.
const SUPERCHIP11_CHAR_SIZE: usize = 10;
/// The sprites of the default SUPER-CHIP 1.1 font, where each character is one byte wide
/// and `SUPERCHIP11_CHAR_SIZE` bytes tall.  Each bit represents one pixel in the sprite.
const SUPERCHIP11_FONT_DATA: [u8; 100] = [
    0x3C, 0x7E, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0x7E, 0x3C, // 0
    0x18, 0x38, 0x58, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, // 1
    0x3E, 0x7F, 0xC3, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xFF, 0xFF, // 2
    0x3C, 0x7E, 0xC3, 0x03, 0x0E, 0x0E, 0x03, 0xC3, 0x7E, 0x3C, // 3
    0x06, 0x0E, 0x1E, 0x36, 0x66, 0xC6, 0xFF, 0xFF, 0x06, 0x06, // 4
    0xFF, 0xFF, 0xC0, 0xC0, 0xFC, 0xFE, 0x03, 0xC3, 0x7E, 0x3C, // 5
    0x3E, 0x7C, 0xC0, 0xC0, 0xFC, 0xFE, 0xC3, 0xC3, 0x7E, 0x3C, // 6
    0xFF, 0xFF, 0x03, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x60, 0x60, // 7
    0x3C, 0x7E, 0xC3, 0xC3, 0x7E, 0x7E, 0xC3, 0xC3, 0x7E, 0x3C, // 8
    0x3C, 0x7E, 0xC3, 0xC3, 0x7F, 0x3F, 0x03, 0x03, 0x3E, 0x7C, // 9
];

/// An abstraction of the Chipolata font (prior to loading to memory).
pub(crate) struct Font {
    /// The size of each character in the font in bytes.
    char_size: usize,
    /// A vector containing the font sprite data.
    font_data: Vec<u8>,
}

impl Font {
    /// Constructor that returns the default CHIP-8 font data
    pub fn default_low_resolution() -> Self {
        Font {
            char_size: CHIP8_CHAR_SIZE,
            font_data: Vec::from(CHIP8_FONT_DATA),
        }
    }

    /// Constructor that returns the default SUPER_CHIP 1.1 high-resolution font data
    pub fn default_high_resolution() -> Self {
        Font {
            char_size: SUPERCHIP11_CHAR_SIZE,
            font_data: Vec::from(SUPERCHIP11_FONT_DATA),
        }
    }

    /// Returns a reference to the font data vector.
    pub(crate) fn font_data(&self) -> &Vec<u8> {
        &self.font_data
    }

    /// Returns the length of the font data vector.
    pub(crate) fn font_data_size(&self) -> usize {
        self.font_data.len()
    }

    /// Returns the size of each character in the font in bytes.
    pub(crate) fn char_size(&self) -> usize {
        self.char_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_data_low_resolution() {
        let font: Font = Font::default_low_resolution();
        assert_eq!(font.font_data()[4], CHIP8_FONT_DATA[4]);
    }

    #[test]
    fn test_font_data_high_resolution() {
        let font: Font = Font::default_high_resolution();
        assert_eq!(font.font_data()[4], SUPERCHIP11_FONT_DATA[4]);
    }

    #[test]
    fn test_font_data_size_low_resolution() {
        let font: Font = Font::default_low_resolution();
        assert_eq!(font.font_data_size(), CHIP8_FONT_DATA.len());
    }

    #[test]
    fn test_font_data_size_high_resolution() {
        let font: Font = Font::default_high_resolution();
        assert_eq!(font.font_data_size(), SUPERCHIP11_FONT_DATA.len());
    }

    #[test]
    fn test_char_size_low_resolution() {
        let font: Font = Font::default_low_resolution();
        assert_eq!(font.char_size, CHIP8_CHAR_SIZE);
    }

    #[test]
    fn test_char_size_high_resolution() {
        let font: Font = Font::default_high_resolution();
        assert_eq!(font.char_size, SUPERCHIP11_CHAR_SIZE);
    }
}
