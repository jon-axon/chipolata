use super::*;

impl ChipolataUi {
    /// Event handler for "Load Program" button
    pub(crate) fn on_click_load_program(&mut self) {
        // Open a file load dialogue with appropriate settings, and
        // save results in program_file_path field.
        if let Some(file) = FileDialog::new()
            .set_title(TITLE_LOAD_PROGRAM_WINDOW)
            .add_filter(FILTER_CHIP8, &["ch8"])
            .add_filter(FILTER_ALL, &["*"])
            .set_directory(&self.roms_path)
            .pick_file()
        {
            self.program_file_path = file.display().to_string();
            // Mark the Options model dialogue as open for rendering, as we should
            // immediately prompt the user for emulation opens before running program.
            // Clone existing options settings into a temporary, working new option set
            self.new_options = self.options.clone();
            self.options_modal_open = true;
        }
    }

    /// Event handler for "Options" button
    pub(crate) fn on_click_options(&mut self) {
        // Mark the Options model dialogue as open for rendering.
        // Clone existing options settings into a temporary, working new option set
        self.new_options = self.options.clone();
        self.options_modal_open = true;
    }

    /// Event handler for "Stop" button
    pub(crate) fn on_click_stop(&mut self) {
        // Stop Chipolata, and clear stored program file path
        self.stop_chipolata();
        self.program_file_path = String::default();
    }

    /// Event handler for "Pause" button
    pub(crate) fn on_click_pause(&mut self) {
        // Instruct the worker thread to pause execution of the current instance of Chipolata and
        // set execution status to Paused
        self.execution_state = ExecutionState::Paused;
        if let Some(message_to_chipolata_tx) = &self.message_to_chipolata_tx {
            message_to_chipolata_tx
                .send(MessageToChipolata::Pause)
                .unwrap();
        }
    }

    /// Event handler for "Play" button    
    pub(crate) fn on_click_play(&mut self) {
        // Instruct the worker thread to resume execution of the current instance of Chipolata and
        // set execution status to Running
        self.execution_state = ExecutionState::Running;
        if let Some(message_to_chipolata_tx) = &self.message_to_chipolata_tx {
            message_to_chipolata_tx
                .send(MessageToChipolata::Resume)
                .unwrap();
        }
    }

    /// Event handler for "Restart" button    
    pub(crate) fn on_click_restart(&mut self) {
        // Re-instantiate Chipolata
        self.instantiate_chipolata(self.get_program(), self.options);
    }

    /// Event handler for target processor speed slider
    pub(crate) fn on_changed_speed_slider(&mut self) {
        // Change Chipolata's speed
        self.set_chipolata_speed(self.processor_speed);
    }

    /// Event handler for CHIP-8 emulation mode selectable label
    pub(crate) fn on_click_chip8_label(&mut self) {
        // Set emulation_level field of new Options struct, using appropriate defaults
        self.new_options.emulation_level = EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        };
    }

    /// Event handler for CHIP-48 emulation mode selectable label
    pub(crate) fn on_click_chip48_label(&mut self) {
        // Set emulation_level field of new Options struct
        self.new_options.emulation_level = EmulationLevel::Chip48;
    }

    /// Event handler for SUPER-CHIP 1.1 emulation mode selectable label
    pub(crate) fn on_click_superchip11_label(&mut self) {
        // Set emulation_level field of new Options struct, using appropriate defaults
        self.new_options.emulation_level = EmulationLevel::SuperChip11 {
            octo_compatibility_mode: false,
        };
    }

    /// Event handler for "OK" options button
    pub(crate) fn on_click_ok_options(&mut self) {
        // Copy the new options over to the main Chipolata Options struct
        self.options = self.new_options.clone();
        // Instantiate Chipolata using these new options
        self.instantiate_chipolata(self.get_program(), self.options);
        // Mark the modal dialogue as ready to close
        self.options_modal_open = false;
    }

    /// Event handler for "Cancel" options button
    pub(crate) fn on_click_cancel_options(&mut self) {
        // Mark the modal dialogue as ready to close
        self.options_modal_open = false;
    }

    /// Event handler for for modal Options "Load From File"button
    pub(crate) fn on_click_load_options(&mut self) {
        // Open a file load dialogue with appropriate settings, and instantiate an Options struct
        // from the contents of the user-selected file
        if let Some(file) = FileDialog::new()
            .set_title(TITLE_LOAD_OPTIONS_WINDOW)
            .add_filter(FILTER_JSON, &["json"])
            .add_filter(FILTER_ALL, &["*"])
            .set_directory(&self.options_path)
            .pick_file()
        {
            if let Ok(options) = Options::load_from_file(&Path::new(&file.display().to_string())) {
                self.new_options = options;
            } else {
                MessageDialog::new()
                    .set_level(MessageLevel::Error)
                    .set_title(TITLE_LOAD_OPTIONS_ERROR_WINDOW)
                    .set_description(ERROR_LOAD_OPTIONS)
                    .set_buttons(MessageButtons::Ok)
                    .show();
            }
        }
    }

    /// Event handler for for modal Options "Save To File"button
    pub(crate) fn on_click_save_options(&mut self) {
        // Open a file save dialogue with appropriate settings, and serialise the new Options struct
        // to this file as JSON
        if let Some(file) = FileDialog::new()
            .set_title(TITLE_SAVE_OPTIONS_WINDOW)
            .add_filter(FILTER_JSON, &["json"])
            .add_filter(FILTER_ALL, &["*"])
            .set_directory(&self.options_path)
            .save_file()
        {
            if let Err(_) =
                Options::save_to_file(&self.new_options, &Path::new(&file.display().to_string()))
            {
                MessageDialog::new()
                    .set_level(MessageLevel::Error)
                    .set_title(TITLE_SAVE_OPTIONS_ERROR_WINDOW)
                    .set_description(ERROR_SAVE_OPTIONS)
                    .set_buttons(MessageButtons::Ok)
                    .show();
            }
        }
    }
}
