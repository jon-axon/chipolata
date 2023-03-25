use super::*;

impl ChipolataUi {
    /// Rendering function to display the header panel at the top of the Chipolata UI
    pub(crate) fn render_header(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top(ID_TOP_PANEL).show(ctx, |ui| {
            ui.add_space(UI_SPACER_TOP);
            // The entire panel is in horizontal layout (thin strip at top of screen)
            ui.horizontal(|ui| {
                // Render the "Load Program" button and delegate click event
                if ui
                    .button(RichText::new(CAPTION_BUTTON_LOAD_PROGRAM).color(COLOUR_BUTTON))
                    .on_hover_text(TOOLTIP_BUTTON_LOAD_PROGRAM)
                    .clicked()
                {
                    self.on_click_load_program();
                }
                // Render the "Options" button and delegate click event
                if ui
                    .add_enabled(
                        // Only enabled if we have a program file specified
                        self.program_file_path != String::default(),
                        Button::new(RichText::new(CAPTION_BUTTON_OPTIONS).color(COLOUR_BUTTON)),
                    )
                    .on_hover_text(TOOLTIP_BUTTON_OPTIONS)
                    .on_disabled_hover_text(TOOLTIP_BUTTON_OPTIONS_DISABLED)
                    .clicked()
                {
                    self.on_click_options();
                }
                // Render the foreground and background colour picker widgets, aligned to the right
                // of the panel
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    ui.color_edit_button_srgba(&mut self.background_colour)
                        .on_hover_text(TOOLTIP_COLOUR_PICKER_BACKGROUND);
                    ui.label(RichText::new(CAPTION_LABEL_BACKGROUND_COLOUR).color(COLOUR_LABEL));
                    ui.color_edit_button_srgba(&mut self.foreground_colour)
                        .on_hover_text(TOOLTIP_COLOUR_PICKER_FOREGROUND);
                    ui.label(RichText::new(CAPTION_LABEL_FOREGROUND_COLOUR).color(COLOUR_LABEL));
                });
            });
            // Some padding at the bottom of the panel
            ui.add_space(UI_SPACER_BOTTOM);
        });
    }

    /// Rendering function to display the footer panel at the top of the Chipolata UI
    pub(crate) fn render_footer(&mut self, ctx: &egui::Context) {
        TopBottomPanel::bottom(ID_BOTTOM_PANEL).show(ctx, |ui| {
            ui.add_space(UI_SPACER_TOP);
            // If an error has occurred then we render an extra horizontal section at the top
            // of the footer panel, to display the error message
            if self.last_error_string != String::default() {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(CAPTION_LABEL_ERROR).color(COLOUR_ERROR));
                    ui.label(
                        RichText::new(&self.last_error_string)
                            .color(COLOUR_ERROR)
                            .monospace(),
                    );
                });
                ui.separator();
            }
            // The entire panel is in horizontal layout (thin strip at bottom of screen)
            ui.horizontal(|ui| {
                // If program execution is paused, then render a Play button.
                // If program execution is paused, then render a Pause button instead.
                // If program execution is stopped then render a Play button, but in a disabled state
                match self.execution_state {
                    ExecutionState::Paused => {
                        // Render the "Play" button and delegate click event
                        if ui
                            .button(RichText::new(CAPTION_BUTTON_RUN).color(COLOUR_BUTTON))
                            .on_hover_text(TOOLTIP_BUTTON_RUN)
                            .clicked()
                        {
                            self.on_click_play();
                        }
                    }
                    ExecutionState::Running => {
                        // Render the "Pause" button and delegate click event
                        if ui
                            .button(RichText::new(CAPTION_BUTTON_PAUSE).color(COLOUR_BUTTON))
                            .on_hover_text(TOOLTIP_BUTTON_PAUSE)
                            .clicked()
                        {
                            self.on_click_pause();
                        }
                    }
                    // Render the "Play" button in a disabled state (cannot be clicked)
                    ExecutionState::Stopped => {
                        ui.add_enabled(
                            false,
                            Button::new(RichText::new(CAPTION_BUTTON_RUN).color(COLOUR_BUTTON)),
                        )
                        .on_disabled_hover_text(TOOLTIP_BUTTON_RUN_DISABLED);
                    }
                }
                // Check whether the user can decide to restart execution; this is possible either if
                // the program is currently executing (regardless of whether running or paused), or if
                // the program is stopped but a program file path is already specified within the UI.
                // If the program is stopped and no program file is already known then the button is
                // disabled, and the user must first load a program
                let can_restart: bool = match self.execution_state {
                    ExecutionState::Stopped => self.program_file_path != String::default(),
                    ExecutionState::Paused | ExecutionState::Running => true,
                };
                // Render the "Restart" button if the required conditions are met, and delegate click event
                if ui
                    .add_enabled(
                        can_restart,
                        Button::new(RichText::new(CAPTION_BUTTON_RESTART).color(COLOUR_BUTTON)),
                    )
                    .on_hover_text(TOOLTIP_BUTTON_RESTART)
                    .on_disabled_hover_text(TOOLTIP_BUTTON_RESTART_DISABLED)
                    .clicked()
                {
                    self.on_click_restart();
                };
                // If a program is executing (Running or Paused) then render the "Stop" button and
                // delegate click event.  If program is already stopped then render the "Stop" button
                // in a disabled state (cannot be clicked)
                match self.execution_state {
                    ExecutionState::Paused | ExecutionState::Running => {
                        if ui
                            .button(RichText::new(CAPTION_BUTTON_STOP).color(COLOUR_BUTTON))
                            .on_hover_text(TOOLTIP_BUTTON_STOP)
                            .clicked()
                        {
                            self.on_click_stop();
                        };
                    }
                    ExecutionState::Stopped => {
                        ui.add_enabled(
                            false,
                            Button::new(RichText::new(CAPTION_BUTTON_STOP).color(COLOUR_BUTTON)),
                        )
                        .on_disabled_hover_text(TOOLTIP_BUTTON_STOP_DISABLED);
                    }
                }
                // Render the target processor speed slider as long as the emulation options allow this
                // to be controlled by the user
                let old_speed: u64 = self.processor_speed; // temporarily store current speed
                ui.label(RichText::new(CAPTION_LABEL_PROCESSOR_SPEED).color(COLOUR_LABEL));
                match self.options.emulation_level {
                    // In CHIP-8 emulation mode, if emulation options specify to use variable cycle timing,
                    // then the processor speed slider must be disabled (as speed is fixed)
                    EmulationLevel::Chip8 {
                        memory_limit_2k: _,
                        variable_cycle_timing: true,
                    } => {
                        // Render the slider, but in a disabled state (value cannot be modified)
                        ui.add_enabled(
                            false,
                            Slider::new(&mut self.processor_speed, old_speed..=old_speed)
                                .text(CAPTION_PROCESSOR_SPEED_SUFFIX),
                        )
                        .on_disabled_hover_text(TOOLTIP_SLIDER_PROCESSOR_SPEED_DISABLED);
                    }
                    // Otherwise, render the slider, binding its value directly to the processor_speed
                    // field of the Chipolata UI struct.  If the value is modified
                    _ => {
                        if ui
                            .add(
                                Slider::new(&mut self.processor_speed, MIN_SPEED..=MAX_SPEED)
                                    .text(CAPTION_PROCESSOR_SPEED_SUFFIX),
                            )
                            .on_hover_text(TOOLTIP_SLIDER_PROCESSOR_SPEED)
                            .changed()
                        {
                            self.on_changed_speed_slider();
                        };
                    }
                }
                // Render current execution status and actual reported processor speed, aligned to the
                // right of the panel
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let state_colour: Color32 = match self.execution_state {
                        ExecutionState::Stopped => Color32::RED,
                        ExecutionState::Paused => Color32::YELLOW,
                        ExecutionState::Running => Color32::GREEN,
                    };
                    ui.label(RichText::new(&self.execution_state.to_string()).color(state_colour));
                    ui.label(RichText::new(CAPTION_LABEL_EXECUTION_STATUS).color(COLOUR_LABEL));
                    ui.label(RichText::new(
                        self.cycles_per_second.to_string() + " " + CAPTION_PROCESSOR_SPEED_SUFFIX,
                    ));
                    ui.label(RichText::new(CAPTION_LABEL_CYCLES_PER_SECOND).color(COLOUR_LABEL));
                });
            });
            ui.add_space(UI_SPACER_BOTTOM);
        });
    }

    /// Rendering function to display the modal Options dialogue box
    pub(crate) fn render_modal_options(&mut self, ctx: &egui::Context) -> Modal {
        // Initial setup and preparation of helper variables
        let modal_style: ModalStyle = ModalStyle {
            default_width: Some(0.), // seems necessary to force window to auto-adjust to sensible width
            ..Default::default()
        };
        let modal = Modal::new(ctx, ID_OPTIONS_MODAL).with_style(&modal_style);
        let (emulate_chip8, emulate_chip48, emulate_superchip, variable_cycle_timing): (
            bool,
            bool,
            bool,
            bool,
        ) = self.check_emulation_level();
        // Rendering code
        modal.show(|ui| {
            // Render overall window title
            modal.title(ui, RichText::new(TITLE_OPTIONS_WINDOW).color(COLOUR_TITLE));
            // Render heading for common/shared option section
            ui.heading(RichText::new(CAPTION_HEADING_OPTIONS_COMMON).color(COLOUR_HEADING));
            // Render this portion of the UI as 3-row grid, with descriptive labels in the first
            // column and corresponding user-editable DragValue widgets in the second column
            egui::Grid::new(ID_OPTIONS_MODAL_GRID).show(ui, |ui| {
                // Render the target CPU label and DragValue widgets
                ui.label(RichText::new(CAPTION_LABEL_PROCESSOR_SPEED).color(COLOUR_LABEL));
                // In CHIP-8 emulation mode, if emulation options specify to use variable cycle timing,
                // then the processor speed DragValue widget must be disabled (as speed is fixed)
                if variable_cycle_timing {
                    self.new_options.processor_speed_hertz = COSMAC_VIP_PROCESSOR_SPEED_HERTZ;
                    ui.add_enabled(
                        false,
                        egui::DragValue::new(&mut self.new_options.processor_speed_hertz)
                            .clamp_range(
                                COSMAC_VIP_PROCESSOR_SPEED_HERTZ..=COSMAC_VIP_PROCESSOR_SPEED_HERTZ,
                            )
                            .fixed_decimals(0),
                    )
                    .on_disabled_hover_text(TOOLTIP_SLIDER_PROCESSOR_SPEED_DISABLED);
                // Otherwise, render the DragValue, binding its value directly to the processor_speed_hertz
                // field in the new Options struct
                } else {
                    ui.add(
                        egui::DragValue::new(&mut self.new_options.processor_speed_hertz)
                            .clamp_range(MIN_SPEED..=MAX_SPEED)
                            .fixed_decimals(0)
                            .speed(DRAGVALUE_QUANTUM),
                    )
                    .on_hover_text(TOOLTIP_SLIDER_PROCESSOR_SPEED);
                }
                ui.label(RichText::new(CAPTION_PROCESSOR_SPEED_SUFFIX));
                ui.end_row();
                // Render the program start address label and DragValue widgets
                ui.label(RichText::new(CAPTION_LABEL_PROGRAM_ADDRESS).color(COLOUR_LABEL));
                ui.add(
                    // Bind the DragValue directly to the program_start_address field in the new Options
                    // struct
                    egui::DragValue::new(&mut self.new_options.program_start_address)
                        .clamp_range(0x0..=0xFFFF)
                        .hexadecimal(1, false, true),
                )
                .on_hover_text(TOOLTIP_SLIDER_PROGRAM_ADDRESS);
                ui.end_row();
                // Render the font start address label and DragValue widgets
                ui.label(RichText::new(CAPTION_LABEL_FONT_ADDRESS).color(COLOUR_LABEL));
                ui.add(
                    // Bind the DragValue directly to the font_start_address field in the new Options struct
                    egui::DragValue::new(&mut self.new_options.font_start_address)
                        .clamp_range(0x0..=0x1FF)
                        .hexadecimal(1, false, true),
                )
                .on_hover_text(TOOLTIP_SLIDER_FONT_ADDRESS);
                ui.end_row();
            });
            ui.separator();
            // Render heading for emulation mode section
            ui.heading(RichText::new(CAPTION_HEADING_EMULATION_MODE).color(COLOUR_HEADING));
            // Use selectable labels in a horizontal arrangements for choosing between emulation modes
            // and delegate click events
            ui.horizontal(|ui| {
                if ui
                    .add(egui::SelectableLabel::new(
                        emulate_chip8,
                        CAPTION_RADIO_CHIP8,
                    ))
                    .on_hover_text(TOOLTIP_SELECTABLE_CHIP8)
                    .clicked()
                {
                    self.on_click_chip8_label();
                }
                if ui
                    .add(egui::SelectableLabel::new(
                        emulate_chip48,
                        CAPTION_RADIO_CHIP48,
                    ))
                    .on_hover_text(TOOLTIP_SELECTABLE_CHIP48)
                    .clicked()
                {
                    self.on_click_chip48_label();
                }
                if ui
                    .add(egui::SelectableLabel::new(
                        emulate_superchip,
                        CAPTION_RADIO_SCHIP,
                    ))
                    .on_hover_text(TOOLTIP_SELECTABLE_SUPERCHIP)
                    .clicked()
                {
                    self.on_click_superchip11_label();
                }
            });
            // Depending on which selectable label is active, display any mode-specific options the
            // use may also configure.  This is done via a mutable reference to the emulation_level
            // enum instance in the new Options struct, so the user can toggle these options on/off
            // directly
            match &mut self.new_options.emulation_level {
                EmulationLevel::Chip8 {
                    memory_limit_2k,
                    variable_cycle_timing,
                } => {
                    ui.label(
                        RichText::new(CAPTION_LABEL_MODE_SPECIFIC_OPTIONS).color(COLOUR_LABEL),
                    );
                    ui.group(|ui| {
                        ui.checkbox(
                            memory_limit_2k,
                            RichText::new(CAPTION_CHECKBOX_MEMORY_LIMIT).color(COLOUR_CHECKBOX),
                        )
                        .on_hover_text(TOOLTIP_CHECKBOX_MEMORY_LIMIT);
                        ui.checkbox(
                            variable_cycle_timing,
                            RichText::new(CAPTION_CHECKBOX_CYCLE_TIMING).color(COLOUR_CHECKBOX),
                        )
                        .on_hover_text(TOOLTIP_CHECKBOX_VARIABLE_CYCLE_TIMING);
                    });
                }
                EmulationLevel::Chip48 => (), // no additional options in this mode
                EmulationLevel::SuperChip11 {
                    octo_compatibility_mode,
                } => {
                    ui.label(
                        RichText::new(CAPTION_LABEL_MODE_SPECIFIC_OPTIONS).color(COLOUR_LABEL),
                    );
                    ui.group(|ui| {
                        ui.checkbox(
                            octo_compatibility_mode,
                            RichText::new(CAPTION_CHECKBOX_OCTO_COMPATIBILITY)
                                .color(COLOUR_CHECKBOX),
                        )
                        .on_hover_text(TOOLTIP_CHECKBOX_OCTO_COMPATIBILITY);
                    });
                }
            };
            ui.separator();
            // Render heading for load and save button section
            ui.heading(RichText::new(CAPTION_HEADING_OPTIONS_LOAD_SAVE).color(COLOUR_HEADING));
            // The buttons are rendered in a horizontal layout
            ui.horizontal(|ui| {
                // Render the "Load From File" button and delegate click event
                if ui
                    .button(RichText::new(CAPTION_BUTTON_LOAD_OPTIONS).color(COLOUR_BUTTON))
                    .on_hover_text(TOOLTIP_BUTTON_LOAD_OPTIONS)
                    .clicked()
                {
                    self.on_click_load_options();
                }
                // Render the "Load From File" button and delegate click event
                if ui
                    .button(RichText::new(CAPTION_BUTTON_SAVE_OPTIONS).color(COLOUR_BUTTON))
                    .on_hover_text(TOOLTIP_BUTTON_SAVE_OPTIONS)
                    .clicked()
                {
                    self.on_click_save_options();
                }
            });
            // Render bottom of dialogue box, with buttons to close modal window
            modal.buttons(ui, |ui| {
                // If execution is stopped and no error message is present, then this must be
                // the first time the modal dialogue has been opened i.e. corresponds to the user
                // first loading a program.  In this situation it should not be possible for the user
                // to cancel - they MUST select a set of Options in order to proceed, otherwise it
                // is ambiguous as to in which mode Chipolata should run.  So, in this situation,
                // the "Cancel" button is not rendered
                if self.execution_state != ExecutionState::Stopped
                    || self.last_error_string != String::default()
                {
                    if modal
                        .button(ui, CAPTION_BUTTON_CANCEL)
                        .on_hover_text(TOOLTIP_BUTTON_OPTIONS_CANCEL)
                        .clicked()
                    {
                        self.on_click_cancel_options();
                    };
                }
                // Render the "OK" button and delegate click event
                if modal
                    .button(ui, CAPTION_BUTTON_OK)
                    .on_hover_text(TOOLTIP_BUTTON_OPTIONS_OK)
                    .clicked()
                {
                    self.on_click_ok_options();
                };
            });
        });
        modal
    }

    /// Rendering function to redraw the Chipolata frame buffer
    pub(crate) fn render_chipolata_frame_buffer(
        &self,
        ctx: &egui::Context,
        frame_buffer: chipolata::Display,
    ) {
        // Render this as a central panel, taking up all remaining space around the header and footer panels
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            // Determine the number of screen pixels to use to represent each Chipolata pixel, based
            // on the available screen size and the number of Chipolata pixels in the frame buffer
            let row_pixels: usize = frame_buffer.get_row_size_bytes() * 8;
            let column_pixels: usize = frame_buffer.get_column_size_pixels();
            let pixel_width: f32 = ui.available_width() / (row_pixels as f32);
            let pixel_height: f32 = ui.available_height() / (column_pixels as f32);
            // Determine the top left and top right pixel locations within the UI (as an anchor coordinate
            // from which to render)
            let min_x: f32 = ui.min_rect().min[0];
            let min_y: f32 = ui.min_rect().min[1];
            // Iterate through each column of Chipolata pixels in the frame buffer
            for i in 0..row_pixels {
                // Iterate through each row of Chipolata pixels in the frame buffer
                for j in 0..column_pixels {
                    // Retrieve the corresponding bit from the bitmapped frame buffer, and examine its
                    // state (1 or 0) to determine whether this pixels is "on" or "off"; set to the
                    // background or foreground colour accordingly
                    let colour: egui::Color32 = match frame_buffer[j][i / 8] & (128 >> (i % 8)) {
                        0 => self.background_colour,
                        _ => self.foreground_colour,
                    };
                    // Draw the pixel (as a rectangle) using the calculated colour, size and coordinates
                    let stroke: egui::Stroke = Stroke::new(1., colour);
                    painter.rect(
                        egui::Rect::from_two_pos(
                            Pos2::from((
                                min_x + i as f32 * pixel_width,
                                min_y + j as f32 * pixel_height,
                            )),
                            Pos2::from((
                                min_x + (i + 1) as f32 * pixel_width,
                                min_y + (j + 1) as f32 * pixel_height,
                            )),
                        ),
                        egui::Rounding::none(),
                        colour,
                        stroke,
                    );
                }
            }
        });
    }

    /// Rendering function for the "welcome screen" displayed when no program is executing
    pub(crate) fn render_welcome_screen(&self, ctx: &egui::Context) {
        // Render this as a central panel, taking up all remaining space around the header and footer panels
        egui::CentralPanel::default().show(ctx, |ui| {
            // This screen consists of two large containers, side-by-side in a horizontal arrangement
            // The right-hand container is itself split into two groups in a vertical arrangement
            ui.horizontal(|ui| {
                // The left-hand container displays the "getting started" text, as a series of vertical labels
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        // Render heading for "getting started" section
                        ui.heading(CAPTION_HEADING_GETTING_STARTED);
                        // Render all the body text labels, separated by spacing as required
                        ui.add_space(UI_SPACER_TEXT);
                        ui.label(CAPTION_LABEL_GETTING_STARTED_1);
                        ui.add_space(UI_SPACER_TEXT);
                        ui.label(CAPTION_LABEL_GETTING_STARTED_2);
                        ui.add_space(UI_SPACER_TEXT);
                        ui.label(CAPTION_LABEL_GETTING_STARTED_3);
                        ui.add_space(UI_SPACER_TEXT);
                        ui.label(CAPTION_LABEL_GETTING_STARTED_4);
                        ui.add_space(UI_SPACER_TEXT);
                        ui.label(CAPTION_LABEL_GETTING_STARTED_5);
                        ui.add_space(UI_SPACER_TEXT);
                        ui.label(CAPTION_LABEL_GETTING_STARTED_6);
                    });
                });
                ui.add_space(UI_SPACER_TEXT);
                // The right-hand container holds two groups in a vertical arrangement.  The top group displays
                // the "keyboard controls" information
                ui.vertical(|ui| {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            // Render heading for "getting started" section
                            ui.heading(CAPTION_HEADING_KEYBOARD_CONTROLS);
                            // Render introductory text
                            ui.add_space(UI_SPACER_TEXT);
                            ui.label(CAPTION_LABEL_KEYBOARD_CONTROLS_1);
                            ui.add_space(UI_SPACER_TEXT);
                            ui.add_space(UI_SPACER_TEXT);
                            ui.horizontal(|ui| {
                                ui.add_space(UI_SPACER_HORIZONTAL);
                                // Render a representation of the original CHIP-8 keypad, as a grid of characters
                                egui::Grid::new(ID_KEYBOARD_CONTROLS_GRID_1)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.label("1");
                                        ui.label("2");
                                        ui.label("3");
                                        ui.label("C");
                                        ui.end_row();
                                        ui.label("4");
                                        ui.label("5");
                                        ui.label("6");
                                        ui.label("D");
                                        ui.end_row();
                                        ui.label("7");
                                        ui.label("8");
                                        ui.label("9");
                                        ui.label("E");
                                        ui.end_row();
                                        ui.label("A");
                                        ui.label("0");
                                        ui.label("B");
                                        ui.label("F");
                                        ui.end_row();
                                    });
                            });
                            ui.add_space(UI_SPACER_TEXT);
                            ui.add_space(UI_SPACER_TEXT);
                            ui.label(CAPTION_LABEL_KEYBOARD_CONTROLS_2);
                            ui.add_space(UI_SPACER_TEXT);
                            ui.add_space(UI_SPACER_TEXT);
                            ui.horizontal(|ui| {
                                ui.add_space(UI_SPACER_HORIZONTAL);
                                // Render a representation of the Chipolata keys, as a grid of characters
                                egui::Grid::new(ID_KEYBOARD_CONTROLS_GRID_2)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.label("1");
                                        ui.label("2");
                                        ui.label("3");
                                        ui.label("4");
                                        ui.end_row();
                                        ui.label("Q");
                                        ui.label("W");
                                        ui.label("E");
                                        ui.label("R");
                                        ui.end_row();
                                        ui.label("A");
                                        ui.label("S");
                                        ui.label("D");
                                        ui.label("F");
                                        ui.end_row();
                                        ui.label("Z");
                                        ui.label("X");
                                        ui.label("C");
                                        ui.label("V");
                                        ui.end_row();
                                    });
                            });
                        });
                    });
                    // The bottom group in the right-hand container displays the "about" information
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            // Render heading for "about" section
                            ui.heading(CAPTION_HEADING_ABOUT);
                            ui.add_space(UI_SPACER_TEXT);
                            // Render the current version number
                            ui.horizontal(|ui| {
                                ui.label(CAPTION_LABEL_ABOUT_1);
                                ui.label(
                                    RichText::new(&format!("v{}", VERSION)).color(COLOUR_LABEL),
                                );
                            });
                            // Render a link to the GitHub page
                            ui.label(CAPTION_LABEL_ABOUT_2);
                            ui.add_space(UI_SPACER_TEXT);
                            ui.add(egui::Hyperlink::new(LINK_GITHUB));
                        });
                    });
                });
            });
        });
    }
}
