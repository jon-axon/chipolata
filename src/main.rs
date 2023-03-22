#![windows_subsystem = "windows"]

mod audio;
mod resource_strings;

use audio::Audio;
use chipolata::{
    ChipolataError, Display, EmulationLevel, Options, Processor, Program, StateSnapshot,
    StateSnapshotVerbosity, COSMAC_VIP_PROCESSOR_SPEED_HERTZ,
};
use core::fmt;
use eframe::egui;
use egui::*;
use egui_modal::*;
use image;
use resource_strings::*;
use rfd::*;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const INITIAL_WIDTH: f32 = 960.;
const INITIAL_HEIGHT: f32 = 540.;
const ICON: &[u8; 4286] = include_bytes!("..\\assets\\chipolata.ico");
const MIN_SPEED: u64 = 100;
const MAX_SPEED: u64 = 10000;
const COLOUR_TITLE: Color32 = Color32::LIGHT_GRAY;
const COLOUR_HEADING: Color32 = Color32::LIGHT_GRAY;
const COLOUR_LABEL: Color32 = Color32::LIGHT_GRAY;
const COLOUR_BUTTON: Color32 = Color32::LIGHT_GRAY;
const COLOUR_CHECKBOX: Color32 = Color32::LIGHT_GRAY;
const COLOUR_ERROR: Color32 = Color32::RED;
const COLOUR_DEFAULT_FOREGROUND: Color32 = egui::Color32::from_rgb(0, 220, 255);
const COLOUR_DEFAULT_BACKGROUND: Color32 = egui::Color32::from_rgb(9, 73, 146);
const UI_SPACER_TOP: f32 = 4.;
const UI_SPACER_BOTTOM: f32 = 2.;
const UI_SPACER_TEXT: f32 = 8.;
const UI_SPACER_HORIZONTAL: f32 = 100.;
const DRAGVALUE_QUANTUM: f64 = 10.;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        icon_data: Some(load_icon()),
        initial_window_size: Some(Vec2::from((INITIAL_WIDTH, INITIAL_HEIGHT))),
        ..Default::default()
    };

    eframe::run_native(
        &format!("{} (v{})", TITLE_APP_WINDOW, VERSION),
        options,
        Box::new(|_cc| Box::new(ChipolataApp::default())),
    )
}

fn load_icon() -> eframe::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(ICON).unwrap().into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    eframe::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

#[derive(PartialEq, Debug)]
enum ExecutionState {
    Stopped,
    Running,
    Paused,
}

impl fmt::Display for ExecutionState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

enum MessageToChipolata {
    ReadyForStateSnapshot { verbosity: StateSnapshotVerbosity },
    KeyPressEvent { key: u8, pressed: bool },
    SetProcessorSpeed { new_speed: u64 },
    Pause,
    Resume,
    Terminate,
}

enum MessageFromChipolata {
    StateSnapshotReport { snapshot: StateSnapshot },
    ErrorReport { error: ChipolataError },
}

struct ChipolataApp {
    message_to_chipolata_tx: Option<mpsc::Sender<MessageToChipolata>>,
    message_from_chipolata_rx: Option<mpsc::Receiver<MessageFromChipolata>>,
    audio_stream: Option<Audio>,
    program_file_path: String,
    processor_speed: u64,
    execution_state: ExecutionState,
    options: Options,
    new_options: Options,
    foreground_colour: egui::Color32,
    background_colour: egui::Color32,
    roms_path: PathBuf,
    options_path: PathBuf,
    last_error_string: String,
    cycles_completed: usize,
    cycle_timer: Instant,
    cycles_per_second: usize,
}

impl eframe::App for ChipolataApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for key press events
        self.handle_input(ctx);
        // Render the header panel
        self.render_header(ctx);
        // Render the footer panel
        self.render_footer(ctx);
        if self.execution_state != ExecutionState::Stopped {
            // Inform Chipolata the UI is ready for a state snapshot update
            self.request_chipolata_update();
            // Process received state snapshot update from Chipolata
            if let Some(frame_buffer) = self.process_chipolata_update() {
                // Refresh the UI
                self.render_chipolata_ui(ctx, frame_buffer);
            }
        } else {
            // Render the welcome screen
            self.render_welcome_screen(ctx);
        }
        // Update UI again as soon as possible
        ctx.request_repaint();
    }
}

impl Default for ChipolataApp {
    fn default() -> Self {
        ChipolataApp {
            message_to_chipolata_tx: None,
            message_from_chipolata_rx: None,
            audio_stream: None,
            program_file_path: String::default(),
            processor_speed: 0,
            execution_state: ExecutionState::Stopped,
            options: Options::default(),
            new_options: Options::default(),
            foreground_colour: COLOUR_DEFAULT_FOREGROUND,
            background_colour: COLOUR_DEFAULT_BACKGROUND,
            roms_path: std::env::current_dir()
                .unwrap()
                .join(PATH_RESOURCE_DIRECTORY_NAME)
                .join(PATH_ROMS_DIRECTORY_NAME),
            options_path: std::env::current_dir()
                .unwrap()
                .join(PATH_RESOURCE_DIRECTORY_NAME)
                .join(PATH_OPTIONS_DIRECTORY_NAME),
            last_error_string: String::default(),
            cycles_completed: 0,
            cycle_timer: Instant::now(),
            cycles_per_second: 0,
        }
    }
}

impl ChipolataApp {
    fn instantiate_chipolata(&mut self, program: Program, options: Options) {
        // If we already have a running/paused Chipolata instance then stop this before proceeding
        if self.execution_state != ExecutionState::Stopped {
            self.stop_chipolata();
        }
        // Instantiate a new Chipolata processor with passed options, and load passed program
        let mut processor = Processor::initialise_and_load(program, options).unwrap();
        // Prepare cross-thread communication channels between UI and Chipolata
        let (message_to_chipolata_tx, message_to_chipolata_rx) = mpsc::channel();
        let (message_from_chipolata_tx, message_from_chipolata_rx) = mpsc::channel();
        self.message_to_chipolata_tx = Some(message_to_chipolata_tx);
        self.message_from_chipolata_rx = Some(message_from_chipolata_rx);
        // Prepare other app fields
        self.audio_stream = Some(Audio::new());
        self.processor_speed = processor.processor_speed();
        self.cycles_completed = 0;
        self.cycle_timer = Instant::now();
        self.cycles_per_second = 0;
        self.last_error_string = String::default();
        // Spawn a new thread to host the Chipolata processor and continually execute cycles,
        // handling communication with the UI app via the previously created channels
        thread::spawn(move || 'outer: {
            let mut crashed: bool = false;
            loop {
                let mut ui_ready_for_update: bool = false;
                let mut snapshot_verbosity: StateSnapshotVerbosity =
                    StateSnapshotVerbosity::Minimal;
                // Process any messages waiting from UI
                for message_to_chipolata in message_to_chipolata_rx.try_iter() {
                    match message_to_chipolata {
                        MessageToChipolata::KeyPressEvent { key, pressed } => {
                            processor.set_key_status(key, pressed).unwrap()
                        }
                        MessageToChipolata::ReadyForStateSnapshot { verbosity } => {
                            ui_ready_for_update = true;
                            snapshot_verbosity = verbosity;
                        }
                        MessageToChipolata::SetProcessorSpeed { new_speed } => {
                            processor.set_processor_speed(new_speed);
                        }
                        MessageToChipolata::Pause => processor.pause_execution().unwrap(),
                        MessageToChipolata::Resume => processor.resume_execution().unwrap(),
                        MessageToChipolata::Terminate => break 'outer,
                    }
                }
                // Run a Chipolata processor cycle
                if !crashed {
                    if let Err(error) = processor.execute_cycle() {
                        crashed = true;
                        message_from_chipolata_tx
                            .send(MessageFromChipolata::ErrorReport { error })
                            .unwrap();
                    }
                }
                // Send a state snapshot update back to UI if requested
                if ui_ready_for_update {
                    let snapshot = processor.export_state_snapshot(snapshot_verbosity);
                    message_from_chipolata_tx
                        .send(MessageFromChipolata::StateSnapshotReport { snapshot })
                        .unwrap();
                }
            }
        });
        self.execution_state = ExecutionState::Running;
    }

    fn stop_chipolata(&mut self) {
        self.execution_state = ExecutionState::Stopped;
        self.audio_stream = None;
        if let Some(message_to_chipolata_tx) = &self.message_to_chipolata_tx {
            message_to_chipolata_tx
                .send(MessageToChipolata::Terminate)
                .unwrap();
        }
        self.message_from_chipolata_rx = None;
        self.message_to_chipolata_tx = None;
        self.processor_speed = 0;
        self.cycles_per_second = 0;
    }

    fn pause_chipolata(&mut self) {
        self.execution_state = ExecutionState::Paused;
        if let Some(message_to_chipolata_tx) = &self.message_to_chipolata_tx {
            message_to_chipolata_tx
                .send(MessageToChipolata::Pause)
                .unwrap();
        }
    }

    fn resume_chipolata(&mut self) {
        self.execution_state = ExecutionState::Running;
        if let Some(message_to_chipolata_tx) = &self.message_to_chipolata_tx {
            message_to_chipolata_tx
                .send(MessageToChipolata::Resume)
                .unwrap();
        }
    }

    fn set_chipolata_speed(&self, new_speed: u64) {
        if let Some(message_to_chipolata_tx) = &self.message_to_chipolata_tx {
            message_to_chipolata_tx
                .send(MessageToChipolata::SetProcessorSpeed { new_speed })
                .unwrap();
        }
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            let key_events: Vec<(&Key, &bool)> = i
                .events
                .iter()
                .filter_map(|e| match e {
                    Event::Key { key, pressed, .. } => Some((key, pressed)),
                    _ => None,
                })
                .collect();
            for (key, state) in key_events {
                match key {
                    Key::Num1 => self.send_key_press_event(0x1, *state),
                    Key::Num2 => self.send_key_press_event(0x2, *state),
                    Key::Num3 => self.send_key_press_event(0x3, *state),
                    Key::Num4 => self.send_key_press_event(0xC, *state),
                    Key::Q => self.send_key_press_event(0x4, *state),
                    Key::W => self.send_key_press_event(0x5, *state),
                    Key::E => self.send_key_press_event(0x6, *state),
                    Key::R => self.send_key_press_event(0xD, *state),
                    Key::A => self.send_key_press_event(0x7, *state),
                    Key::S => self.send_key_press_event(0x8, *state),
                    Key::D => self.send_key_press_event(0x9, *state),
                    Key::F => self.send_key_press_event(0xE, *state),
                    Key::Z => self.send_key_press_event(0xA, *state),
                    Key::X => self.send_key_press_event(0x0, *state),
                    Key::C => self.send_key_press_event(0xB, *state),
                    Key::V => self.send_key_press_event(0xF, *state),
                    _ => (),
                }
            }
        });
    }

    fn send_key_press_event(&self, key: u8, pressed: bool) {
        if let Some(message_to_chipolata_tx) = &self.message_to_chipolata_tx {
            if let Err(_) =
                message_to_chipolata_tx.send(MessageToChipolata::KeyPressEvent { key, pressed })
            {
                // absorb the error; no need to handle
            }
        }
    }

    fn render_header(&mut self, ctx: &egui::Context) {
        let modal: Modal = self.render_modal_options(ctx);
        TopBottomPanel::top(ID_TOP_PANEL).show(ctx, |ui| {
            ui.add_space(UI_SPACER_TOP);
            ui.horizontal(|ui| {
                if ui
                    .button(RichText::new(CAPTION_BUTTON_LOAD_PROGRAM).color(COLOUR_BUTTON))
                    .on_hover_text(TOOLTIP_BUTTON_LOAD_PROGRAM)
                    .clicked()
                {
                    if let Some(file) = FileDialog::new()
                        .set_title(TITLE_LOAD_PROGRAM_WINDOW)
                        .add_filter(FILTER_CHIP8, &["ch8"])
                        .add_filter(FILTER_ALL, &["*"])
                        .set_directory(&self.roms_path)
                        .pick_file()
                    {
                        self.program_file_path = file.display().to_string();
                        modal.open();
                    }
                }
                if ui
                    .add_enabled(
                        self.program_file_path != String::default(),
                        Button::new(RichText::new(CAPTION_BUTTON_OPTIONS).color(COLOUR_BUTTON)),
                    )
                    .on_hover_text(TOOLTIP_BUTTON_OPTIONS)
                    .on_disabled_hover_text(TOOLTIP_BUTTON_OPTIONS_DISABLED)
                    .clicked()
                {
                    self.new_options = self.options.clone();
                    modal.open();
                }
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    ui.color_edit_button_srgba(&mut self.background_colour)
                        .on_hover_text(TOOLTIP_COLOUR_PICKER_BACKGROUND);
                    ui.label(RichText::new(CAPTION_LABEL_BACKGROUND_COLOUR).color(COLOUR_LABEL));
                    ui.color_edit_button_srgba(&mut self.foreground_colour)
                        .on_hover_text(TOOLTIP_COLOUR_PICKER_FOREGROUND);
                    ui.label(RichText::new(CAPTION_LABEL_FOREGROUND_COLOUR).color(COLOUR_LABEL));
                });
            });
            ui.add_space(UI_SPACER_BOTTOM);
        });
    }

    fn render_footer(&mut self, ctx: &egui::Context) {
        TopBottomPanel::bottom(ID_BOTTOM_PANEL).show(ctx, |ui| {
            ui.add_space(UI_SPACER_TOP);
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
            ui.horizontal(|ui| {
                match self.execution_state {
                    ExecutionState::Paused => {
                        if ui
                            .button(RichText::new(CAPTION_BUTTON_RUN).color(COLOUR_BUTTON))
                            .on_hover_text(TOOLTIP_BUTTON_RUN)
                            .clicked()
                        {
                            self.resume_chipolata();
                        }
                    }
                    ExecutionState::Running => {
                        if ui
                            .button(RichText::new(CAPTION_BUTTON_PAUSE).color(COLOUR_BUTTON))
                            .on_hover_text(TOOLTIP_BUTTON_PAUSE)
                            .clicked()
                        {
                            self.pause_chipolata();
                        }
                    }
                    ExecutionState::Stopped => {
                        ui.add_enabled(
                            false,
                            Button::new(RichText::new(CAPTION_BUTTON_RUN).color(COLOUR_BUTTON)),
                        )
                        .on_disabled_hover_text(TOOLTIP_BUTTON_RUN_DISABLED);
                    }
                }
                let can_restart: bool = match self.execution_state {
                    ExecutionState::Stopped => self.program_file_path != String::default(),
                    ExecutionState::Paused | ExecutionState::Running => true,
                };
                if ui
                    .add_enabled(
                        can_restart,
                        Button::new(RichText::new(CAPTION_BUTTON_RESTART).color(COLOUR_BUTTON)),
                    )
                    .on_hover_text(TOOLTIP_BUTTON_RESTART)
                    .on_disabled_hover_text(TOOLTIP_BUTTON_RESTART_DISABLED)
                    .clicked()
                {
                    self.instantiate_chipolata(self.get_program(), self.options);
                };
                match self.execution_state {
                    ExecutionState::Paused | ExecutionState::Running => {
                        if ui
                            .button(RichText::new(CAPTION_BUTTON_STOP).color(COLOUR_BUTTON))
                            .on_hover_text(TOOLTIP_BUTTON_STOP)
                            .clicked()
                        {
                            self.stop_chipolata();
                            self.program_file_path = String::default();
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

                let old_speed: u64 = self.processor_speed;
                ui.label(RichText::new(CAPTION_LABEL_PROCESSOR_SPEED).color(COLOUR_LABEL));
                match self.options.emulation_level {
                    EmulationLevel::Chip8 {
                        memory_limit_2k: _,
                        variable_cycle_timing: true,
                    } => {
                        ui.add_enabled(
                            false,
                            Slider::new(&mut self.processor_speed, old_speed..=old_speed)
                                .text(CAPTION_PROCESSOR_SPEED_SUFFIX),
                        )
                        .on_disabled_hover_text(TOOLTIP_SLIDER_PROCESSOR_SPEED_DISABLED);
                    }
                    _ => {
                        ui.add(
                            Slider::new(&mut self.processor_speed, MIN_SPEED..=MAX_SPEED)
                                .text(CAPTION_PROCESSOR_SPEED_SUFFIX),
                        )
                        .on_hover_text(TOOLTIP_SLIDER_PROCESSOR_SPEED);
                    }
                }
                if self.processor_speed != old_speed {
                    self.set_chipolata_speed(self.processor_speed);
                }
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

    fn render_modal_options(&mut self, ctx: &egui::Context) -> Modal {
        let modal_style: ModalStyle = ModalStyle {
            default_width: Some(0.), // seems necessary to force window to adjust to sensible width
            ..Default::default()
        };
        let modal = Modal::new(ctx, ID_OPTIONS_MODAL).with_style(&modal_style);
        let (emulate_chip8, emulate_chip48, emulate_superchip, variable_cycle_timing): (
            bool,
            bool,
            bool,
            bool,
        ) = self.check_emulation_level();
        modal.show(|ui| {
            modal.title(ui, RichText::new(TITLE_OPTIONS_WINDOW).color(COLOUR_TITLE));
            // Standard options (all emulation levels)
            ui.heading(RichText::new(CAPTION_HEADING_OPTIONS_COMMON).color(COLOUR_HEADING));
            egui::Grid::new(ID_OPTIONS_MODAL_GRID).show(ui, |ui| {
                ui.label(RichText::new(CAPTION_LABEL_PROCESSOR_SPEED).color(COLOUR_LABEL));
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
                ui.label(RichText::new(CAPTION_LABEL_PROGRAM_ADDRESS).color(COLOUR_LABEL));
                ui.add(
                    egui::DragValue::new(&mut self.new_options.program_start_address)
                        .clamp_range(0x0..=0xFFFF)
                        .hexadecimal(1, false, true),
                )
                .on_hover_text(TOOLTIP_SLIDER_PROGRAM_ADDRESS);
                ui.end_row();
                ui.label(RichText::new(CAPTION_LABEL_FONT_ADDRESS).color(COLOUR_LABEL));
                ui.add(
                    egui::DragValue::new(&mut self.new_options.font_start_address)
                        .clamp_range(0x0..=0x1FF)
                        .hexadecimal(1, false, true),
                )
                .on_hover_text(TOOLTIP_SLIDER_FONT_ADDRESS);
                ui.end_row();
            });
            ui.separator();
            ui.heading(RichText::new(CAPTION_HEADING_EMULATION_MODE).color(COLOUR_HEADING));
            // Selectable labels for selection of emulation level
            ui.horizontal(|ui| {
                if ui
                    .add(egui::SelectableLabel::new(
                        emulate_chip8,
                        CAPTION_RADIO_CHIP8,
                    ))
                    .on_hover_text(TOOLTIP_SELECTABLE_CHIP8)
                    .clicked()
                {
                    self.new_options.emulation_level = EmulationLevel::Chip8 {
                        memory_limit_2k: false,
                        variable_cycle_timing: false,
                    };
                }
                if ui
                    .add(egui::SelectableLabel::new(
                        emulate_chip48,
                        CAPTION_RADIO_CHIP48,
                    ))
                    .on_hover_text(TOOLTIP_SELECTABLE_CHIP48)
                    .clicked()
                {
                    self.new_options.emulation_level = EmulationLevel::Chip48;
                }
                if ui
                    .add(egui::SelectableLabel::new(
                        emulate_superchip,
                        CAPTION_RADIO_SCHIP,
                    ))
                    .on_hover_text(TOOLTIP_SELECTABLE_SUPERCHIP)
                    .clicked()
                {
                    self.new_options.emulation_level = EmulationLevel::SuperChip11 {
                        octo_compatibility_mode: false,
                    };
                }
            });
            // Emulation-level-specific options
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
                EmulationLevel::Chip48 => (),
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
            // Load and save buttons
            ui.heading(RichText::new(CAPTION_HEADING_OPTIONS_LOAD_SAVE).color(COLOUR_HEADING));
            ui.horizontal(|ui| {
                if ui
                    .button(RichText::new(CAPTION_BUTTON_LOAD_OPTIONS).color(COLOUR_BUTTON))
                    .on_hover_text(TOOLTIP_BUTTON_LOAD_OPTIONS)
                    .clicked()
                {
                    if let Some(file) = FileDialog::new()
                        .set_title(TITLE_LOAD_OPTIONS_WINDOW)
                        .add_filter(FILTER_JSON, &["json"])
                        .add_filter(FILTER_ALL, &["*"])
                        .set_directory(&self.options_path)
                        .pick_file()
                    {
                        if let Ok(options) =
                            Options::load_from_file(&Path::new(&file.display().to_string()))
                        {
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
                if ui
                    .button(RichText::new(CAPTION_BUTTON_SAVE_OPTIONS).color(COLOUR_BUTTON))
                    .on_hover_text(TOOLTIP_BUTTON_SAVE_OPTIONS)
                    .clicked()
                {
                    if let Some(file) = FileDialog::new()
                        .set_title(TITLE_SAVE_OPTIONS_WINDOW)
                        .add_filter(FILTER_JSON, &["json"])
                        .add_filter(FILTER_ALL, &["*"])
                        .set_directory(&self.options_path)
                        .save_file()
                    {
                        if let Err(_) = Options::save_to_file(
                            &self.new_options,
                            &Path::new(&file.display().to_string()),
                        ) {
                            MessageDialog::new()
                                .set_level(MessageLevel::Error)
                                .set_title(TITLE_SAVE_OPTIONS_ERROR_WINDOW)
                                .set_description(ERROR_SAVE_OPTIONS)
                                .set_buttons(MessageButtons::Ok)
                                .show();
                        }
                    }
                }
            });
            // Buttons to close modal dialogue box
            modal.buttons(ui, |ui| {
                if self.execution_state != ExecutionState::Stopped
                    || self.last_error_string != String::default()
                {
                    modal
                        .button(ui, CAPTION_BUTTON_CANCEL)
                        .on_hover_text(TOOLTIP_BUTTON_OPTIONS_CANCEL);
                }
                if modal
                    .button(ui, CAPTION_BUTTON_OK)
                    .on_hover_text(TOOLTIP_BUTTON_OPTIONS_OK)
                    .clicked()
                {
                    self.options = self.new_options.clone();
                    self.instantiate_chipolata(self.get_program(), self.options);
                };
            });
        });
        modal
    }

    fn check_emulation_level(&self) -> (bool, bool, bool, bool) {
        match self.new_options.emulation_level {
            EmulationLevel::Chip8 {
                memory_limit_2k: _,
                variable_cycle_timing: true,
            } => return (true, false, false, true),
            EmulationLevel::Chip8 {
                memory_limit_2k: _,
                variable_cycle_timing: false,
            } => return (true, false, false, false),
            EmulationLevel::Chip48 => return (false, true, false, false),
            EmulationLevel::SuperChip11 { .. } => return (false, false, true, false),
        };
    }

    fn get_program(&self) -> Program {
        let program: Program =
            Program::load_from_file(&Path::new(&self.program_file_path)).unwrap();
        program
    }

    fn request_chipolata_update(&self) {
        if let Some(message_to_chipolata_tx) = &self.message_to_chipolata_tx {
            if let Err(_) =
                message_to_chipolata_tx.send(MessageToChipolata::ReadyForStateSnapshot {
                    verbosity: StateSnapshotVerbosity::Minimal,
                })
            {
                // absorb the error; no need to handle
            }
        }
    }

    fn process_chipolata_update(&mut self) -> Option<Display> {
        if let Some(message_from_chipolata_rx) = &self.message_from_chipolata_rx {
            if let Ok(message) = message_from_chipolata_rx.recv() {
                match message {
                    MessageFromChipolata::StateSnapshotReport { snapshot } => {
                        if let StateSnapshot::MinimalSnapshot {
                            frame_buffer,
                            status: _,
                            processor_speed,
                            play_sound,
                            cycles,
                        } = snapshot
                        {
                            // Keep track of current processor speed
                            self.processor_speed = processor_speed;
                            // Pause / resume audio if required
                            if let Some(audio_stream) = &self.audio_stream {
                                match (play_sound, audio_stream.is_paused()) {
                                    (true, true) => audio_stream.play(),
                                    (false, false) => audio_stream.pause(),
                                    _ => (),
                                }
                            }
                            // Recalculate cycles per second
                            let millis_elapsed: u128 = self.cycle_timer.elapsed().as_millis();
                            if millis_elapsed >= 1000 {
                                self.cycles_per_second = (cycles - self.cycles_completed) * 1000
                                    / millis_elapsed as usize;
                                self.cycles_completed = cycles;
                                self.cycle_timer = Instant::now();
                            }
                            // Return frame buffer, for rendering
                            return Some(frame_buffer);
                        }
                    }
                    MessageFromChipolata::ErrorReport { error } => {
                        self.last_error_string = error.inner_error.to_string();
                        self.stop_chipolata();
                    }
                }
            }
        }
        return None;
    }

    fn render_chipolata_ui(&self, ctx: &egui::Context, frame_buffer: chipolata::Display) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            let row_pixels: usize = frame_buffer.get_row_size_bytes() * 8;
            let column_pixels: usize = frame_buffer.get_column_size_pixels();
            let pixel_width: f32 = ui.available_width() / (row_pixels as f32);
            let pixel_height: f32 = ui.available_height() / (column_pixels as f32);
            let min_x: f32 = ui.min_rect().min[0];
            let min_y: f32 = ui.min_rect().min[1];
            for i in 0..row_pixels {
                for j in 0..column_pixels {
                    let colour: egui::Color32 = match frame_buffer[j][i / 8] & (128 >> (i % 8)) {
                        0 => self.background_colour,
                        _ => self.foreground_colour,
                    };
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

    fn render_welcome_screen(&self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.heading(CAPTION_HEADING_GETTING_STARTED);
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
                ui.vertical(|ui| {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading(CAPTION_HEADING_KEYBOARD_CONTROLS);
                            ui.add_space(UI_SPACER_TEXT);
                            ui.label(CAPTION_LABEL_KEYBOARD_CONTROLS_1);
                            ui.add_space(UI_SPACER_TEXT);
                            ui.add_space(UI_SPACER_TEXT);
                            ui.horizontal(|ui| {
                                ui.add_space(UI_SPACER_HORIZONTAL);
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
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.heading(CAPTION_HEADING_ABOUT);
                            ui.add_space(UI_SPACER_TEXT);
                            ui.horizontal(|ui| {
                                ui.label(CAPTION_LABEL_ABOUT_1);
                                ui.label(
                                    RichText::new(&format!("v{}", VERSION)).color(COLOUR_LABEL),
                                );
                            });
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
