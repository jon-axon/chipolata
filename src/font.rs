/// The size of each character of the default Chipolata font in bytes.
const DEFAULT_CHAR_SIZE: usize = 5;
/// The sprites of the default Chipolata font, where each character is one byte wide
/// and `DEFAULT_CHAR_SIZE` bytes tall.  Each bit represents one pixel in the sprite.
const DEFAULT_FONT_DATA: [u8; 80] = [
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

/// An abstraction of the CHIP-8 font (prior to loading to memory).
pub(crate) struct Font {
    /// The size of each character in the font in bytes.
    char_size: usize,
    /// A vector containing the font sprite data.
    font_data: Vec<u8>,
}

impl Default for Font {
    /// Constructor that returns a [Font] instance using the default Chipolata font.
    fn default() -> Self {
        Font {
            char_size: DEFAULT_CHAR_SIZE,
            font_data: Vec::from(DEFAULT_FONT_DATA),
        }
    }
}

impl Font {
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
    fn test_font_data() {
        let font: Font = Font::default();
        assert_eq!(font.font_data()[4], DEFAULT_FONT_DATA[4]);
    }

    #[test]
    fn test_font_data_size() {
        let font: Font = Font::default();
        assert_eq!(font.font_data_size(), DEFAULT_FONT_DATA.len());
    }

    #[test]
    fn test_char_size() {
        let font: Font = Font::default();
        assert_eq!(font.char_size, DEFAULT_CHAR_SIZE);
    }
}
