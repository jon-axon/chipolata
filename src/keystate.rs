#![allow(non_snake_case)]

use crate::error::ErrorDetail;

/// The default number of keys in the CHIP-8 keypad.
const NUMBER_OF_KEYS: u8 = 16;

/// An abstraction of the state of each key on the CHIP-8 keypad
/// (pressed / not pressed).
pub(crate) struct KeyState {
    /// Array holding a boolean for each key (true means pressed, false means not pressed).
    keys_pressed: [bool; NUMBER_OF_KEYS as usize],
}

impl KeyState {
    /// Constructor that returns a [KeyState] instance with no keys pressed.
    pub(crate) fn new() -> Self {
        KeyState {
            keys_pressed: [false; NUMBER_OF_KEYS as usize],
        }
    }

    /// Returns true if the specified key is pressed, false if the specified key is not
    /// pressed, and returns an [ErrorDetail::InvalidKey](crate::error::ErrorDetail::InvalidKey) if
    /// the specified key is invalid.
    ///
    /// # Arguments
    ///
    /// * `key` - the hex ordinal of the key (valid range 0x0 to 0xF inclusive)
    pub(crate) fn is_key_pressed(&self, key: u8) -> Result<bool, ErrorDetail> {
        match key {
            n if n < NUMBER_OF_KEYS => Ok(self.keys_pressed[n as usize]),
            _ => Err(ErrorDetail::InvalidKey { key }),
        }
    }

    /// Sets the state of the specified key; returns an [ErrorDetail::InvalidKey] if the
    /// specified key is invalid.
    ///
    /// # Arguments
    ///
    /// * `key` - the hex ordinal of the key (valid range 0x0 to 0xF inclusive)
    /// * `status` - boolean representing key state (true meaning pressed)
    pub(crate) fn set_key_status(&mut self, key: u8, status: bool) -> Result<(), ErrorDetail> {
        match key {
            n if n < NUMBER_OF_KEYS => Ok(self.keys_pressed[n as usize] = status),
            _ => Err(ErrorDetail::InvalidKey { key }),
        }
    }

    /// Returns a byte vector holding the hex ordinals of all keys currently pressed.
    pub(crate) fn get_keys_pressed(&self) -> Option<Vec<u8>> {
        let mut keys: Vec<u8> = Vec::new();
        // Iterate through each key, adding to the output vector if pressed
        for i in 0..NUMBER_OF_KEYS {
            if self.is_key_pressed(i).unwrap() {
                keys.push(i);
            }
        }
        // If the output vector contains at least one key then return it, otherwise `None`
        if keys.len() > 0 {
            return Some(keys);
        } else {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_key_pressed_yes() {
        let mut keys: KeyState = KeyState::new();
        keys.keys_pressed[0x2] = true;
        assert!(keys.is_key_pressed(0x2).unwrap());
    }

    #[test]
    fn test_is_key_pressed_no() {
        let mut keys: KeyState = KeyState::new();
        keys.keys_pressed[0x2] = false;
        assert!(!keys.is_key_pressed(0x2).unwrap());
    }

    #[test]
    fn test_is_key_pressed_error() {
        let keys: KeyState = KeyState::new();
        assert_eq!(
            keys.is_key_pressed(NUMBER_OF_KEYS).unwrap_err(),
            ErrorDetail::InvalidKey {
                key: NUMBER_OF_KEYS
            }
        );
    }

    #[test]
    fn test_set_key_status() {
        let mut keys: KeyState = KeyState::new();
        keys.set_key_status(0x2, true).unwrap();
        assert!(keys.keys_pressed[0x2] == true);
    }

    #[test]
    fn test_set_key_status_error() {
        let mut keys: KeyState = KeyState::new();
        assert_eq!(
            keys.set_key_status(NUMBER_OF_KEYS, true).unwrap_err(),
            ErrorDetail::InvalidKey {
                key: NUMBER_OF_KEYS
            }
        );
    }

    #[test]
    fn test_get_keys_pressed() {
        let mut keys: KeyState = KeyState::new();
        keys.keys_pressed[0x2] = true;
        keys.keys_pressed[0x7] = true;
        keys.keys_pressed[0xF] = true;
        let key_vector: Vec<u8> = keys.get_keys_pressed().unwrap();
        assert_eq!(key_vector, vec![0x2, 0x7, 0xF]);
    }

    #[test]
    fn test_get_keys_pressed_none() {
        let keys: KeyState = KeyState::new();
        assert!(keys.get_keys_pressed().is_none());
    }
}
