#![windows_subsystem = "windows"]

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

mod audio; // Sub-module for handling audio
mod event_handlers; // Sub-module holding all event-handling methods
mod render; // Sub-module containing all resource strings
mod resource_strings; // Sub-module holding all UI-rendering methods

/// The version of Chipolata, as defined in the `cargo.toml` file
const VERSION: &str = env!("CARGO_PKG_VERSION");
/// The initial width of the Chipolata UI window in pixels
const INITIAL_WIDTH: f32 = 940.;
/// The initial height of the Chipolata UI window in pixels
const INITIAL_HEIGHT: f32 = 540.;
/// A byte array (populated at compile-time) holding the Chipolata logo, for display in the taskbar
/// and app window
const ICON: &[u8; 4286] = include_bytes!("..\\assets\\chipolata.ico");
/// The minimum selectable Chipolata processor speed (for use in the UI's slider widget)
const MIN_SPEED: u64 = 100;
/// The maximum selectable Chipolata processor speed (for use in the UI's slider widget)
const MAX_SPEED: u64 = 10000;
/// The colour to use for any title text
const COLOUR_TITLE: Color32 = Color32::LIGHT_GRAY;
/// The colour to use for any heading text
const COLOUR_HEADING: Color32 = Color32::LIGHT_GRAY;
/// The colour to use for any label text
const COLOUR_LABEL: Color32 = Color32::LIGHT_GRAY;
/// The colour to use for any button text
const COLOUR_BUTTON: Color32 = Color32::LIGHT_GRAY;
/// The colour to use for any checkbox text
const COLOUR_CHECKBOX: Color32 = Color32::LIGHT_GRAY;
/// The colour to use for any error text
const COLOUR_ERROR: Color32 = Color32::RED;
/// The default colour to use for rendering Chipolata display foreground pixels
const COLOUR_DEFAULT_FOREGROUND: Color32 = egui::Color32::from_rgb(0, 220, 255);
/// The default colour to use for rendering Chipolata display background pixels
const COLOUR_DEFAULT_BACKGROUND: Color32 = egui::Color32::from_rgb(9, 73, 146);
/// The number of pixels to use for padding widgets at the top of containers
const UI_SPACER_TOP: f32 = 4.;
/// The number of pixels to use for padding widgets at the bottom of containers
const UI_SPACER_BOTTOM: f32 = 2.;
/// The number of pixels to use for padding text blocks within widgets
const UI_SPACER_TEXT: f32 = 8.;
/// The number of pixels to use for horizontal padding of containers/widgets
const UI_SPACER_HORIZONTAL: f32 = 100.;
/// The minimum amount by which the use can increment/decrement a DragValue widget's value
const DRAGVALUE_QUANTUM: f64 = 10.;

/// Entry point into the binary; uses eframe to start an instance of the Chipolata UI
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        icon_data: Some(load_icon()),
        initial_window_size: Some(Vec2::from((INITIAL_WIDTH, INITIAL_HEIGHT))),
        ..Default::default()
    };

    eframe::run_native(
        &format!("{} (v{})", TITLE_APP_WINDOW, VERSION),
        options,
        Box::new(|_cc| Box::new(ChipolataUi::default())),
    )
}

/// Helper function to create an [eframe::IconData] based on the const byte array [ICON]
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

/// An enum to represent the high-level current execution state of the hosted Chipolata instance
#[derive(PartialEq, Debug)]
enum ExecutionState {
    /// Not started, or crashed
    Stopped,
    /// Currently executing (even if the CPU itself is stalled e.g. waiting for input)
    Running,
    /// Paused by the user (no instructions will be executed in this state)
    Paused,
}

impl fmt::Display for ExecutionState {
    /// Formatter for [ExecutionState], to facilitate `to_string()` usage
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// An enum to represent messages passed from the UI thread to the worker thread hosting Chipolata
enum MessageToChipolata {
    /// The UI is ready to render a frame, and is requesting current state from Chipolata
    ReadyForStateSnapshot { verbosity: StateSnapshotVerbosity },
    /// The event of the user pressing or releasing a key
    KeyPressEvent { key: u8, pressed: bool },
    /// A change to the current Chipolata CPU speed
    SetProcessorSpeed { new_speed: u64 },
    /// Pause execution (if running)
    Pause,
    /// Resume execution (if paused)
    Resume,
    /// Kill the current Chipolata instance
    Terminate,
}

/// An enum to represent messages passed from the worker thread hosting Chipolata to the UI thread
enum MessageFromChipolata {
    /// A report of the current state of the Chipolata emulator (including frame buffer contents)
    StateSnapshotReport { snapshot: StateSnapshot },
    /// Surfacing an internal error generated by Chipolata
    ErrorReport { error: ChipolataError },
}

/// A struct that represents the overall Chipolata user interface
struct ChipolataUi {
    // Inter-thread communication channels
    message_to_chipolata_tx: Option<mpsc::Sender<MessageToChipolata>>, // sends messages to worker thread
    message_from_chipolata_rx: Option<mpsc::Receiver<MessageFromChipolata>>, // receives messages from worker thread
    // Static config
    roms_path: PathBuf,    // default folder from which to load program ROMs
    options_path: PathBuf, // default folder from which to load saved option set files
    // Dynamic config
    processor_speed: u64, // configured target Chipolata processor speed
    foreground_colour: egui::Color32, // colour with which to render Chipolata foreground fonts
    background_colour: egui::Color32, // colour with which to render Chipolata background fonts
    options: Options,     // emulation options currently defined
    new_options: Options, // new options being defined within the modal UI (but not yet applied)
    program_file_path: String, // file location of the loaded Chipolata ROM
    // State fields
    execution_state: ExecutionState, // Chipolata execution status
    last_error_string: String,       // holds the last error string, if an error has occurred
    cycles_completed: usize, // the total number of cycles completed (for speed calculation purposes)
    cycle_timer: Instant,    // the last moment cycles were counted (for speed calculation purposes)
    cycles_per_second: usize, // current actual processor speed (calculated from cycles completed)
    options_modal_open: bool, // boolean indicating whether the modal Options dialogue is open
    // Miscellaneous
    audio_stream: Option<Audio>, // audio stream for playing Chipolata sound
}

impl eframe::App for ChipolataUi {
    /// Top-level method called by eframe when UI update/repaint is required (~60 times per second)
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for key press events
        self.handle_input(ctx);
        // Render the Options modal dialogue, if required
        if self.options_modal_open {
            self.render_modal_options(ctx).open();
        }
        // Render the header panel
        self.render_header(ctx);
        // Render the footer panel
        self.render_footer(ctx);
        // If a program is currently running then ...
        if self.execution_state != ExecutionState::Stopped {
            // Inform Chipolata the UI is ready for a state snapshot update
            self.request_chipolata_update();
            // Process received state snapshot update from Chipolata
            if let Some(frame_buffer) = self.process_chipolata_update() {
                // Redraw the Chipolata frame buffer
                self.render_chipolata_frame_buffer(ctx, frame_buffer);
            }
        } else {
            // ... otherwise render the welcome screen
            self.render_welcome_screen(ctx);
        }
        // Update UI again as soon as possible
        ctx.request_repaint();
    }
}

impl Default for ChipolataUi {
    /// Constructor that returns a [ChipolataUi] instance using typical default settings
    fn default() -> Self {
        ChipolataUi {
            message_to_chipolata_tx: None,
            message_from_chipolata_rx: None,
            roms_path: std::env::current_dir()
                .unwrap()
                .join(PATH_RESOURCE_DIRECTORY_NAME)
                .join(PATH_ROMS_DIRECTORY_NAME),
            options_path: std::env::current_dir()
                .unwrap()
                .join(PATH_RESOURCE_DIRECTORY_NAME)
                .join(PATH_OPTIONS_DIRECTORY_NAME),
            processor_speed: 0,
            foreground_colour: COLOUR_DEFAULT_FOREGROUND,
            background_colour: COLOUR_DEFAULT_BACKGROUND,
            options: Options::default(),
            new_options: Options::default(),
            program_file_path: String::default(),
            execution_state: ExecutionState::Stopped,
            last_error_string: String::default(),
            cycles_completed: 0,
            cycle_timer: Instant::now(),
            cycles_per_second: 0,
            options_modal_open: false,
            audio_stream: None,
        }
    }
}

impl ChipolataUi {
    /// Instantiates and initialises Chipolata based on the passed [Program] and [Options],
    /// then spawns a new worker thread to own this instance and continually execute cycles,
    /// passing message to and from the UI thread using dedicated channels
    ///
    /// # Arguments
    ///
    /// * `program` - a [Program] instance holding the bytes of the ROM to be executed
    /// * `options` - an [Options] instance holding Chipolata start-up configuration information
    fn instantiate_chipolata(&mut self, program: Program, options: Options) {
        // If we already have a running/paused Chipolata instance then stop this before proceeding
        if self.execution_state != ExecutionState::Stopped {
            self.stop_chipolata();
        }
        // Instantiate a new Chipolata processor with passed options, and load passed program
        let mut processor: Processor;
        // It is possible an error can be generated even at this early stage, for example if the
        // emulation options specify a 2k memory limit but the specified program requires 4k
        match Processor::initialise_and_load(program, options) {
            Err(error) => {
                self.last_error_string = error.inner_error.to_string();
                self.stop_chipolata();
                return;
            }
            Ok(proc) => processor = proc,
        }
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
                        // An internal Chipolata error occurred; report this back to UI
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

    /// Instructs the worker thread to terminate the current instance of Chipolata, and resets
    /// all fields accordingly
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

    /// Instructs the worker thread to alter the processor speed of the current instance of Chipolata
    ///
    /// # Arguments
    ///
    /// * `new_speed` - the new target processor speed (cycles per second)
    fn set_chipolata_speed(&self, new_speed: u64) {
        if let Some(message_to_chipolata_tx) = &self.message_to_chipolata_tx {
            message_to_chipolata_tx
                .send(MessageToChipolata::SetProcessorSpeed { new_speed })
                .unwrap();
        }
    }

    /// Method to handle user keyboard input (passing relevant keystrokes on to Chipolata for processing)
    fn handle_input(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // we are only interested in key press input events (both press and release events)
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

    /// Helper function to inform worker thread of key presses to be handled by Chipolata
    fn send_key_press_event(&self, key: u8, pressed: bool) {
        if let Some(message_to_chipolata_tx) = &self.message_to_chipolata_tx {
            if let Err(_) =
                message_to_chipolata_tx.send(MessageToChipolata::KeyPressEvent { key, pressed })
            {
                // absorb the error; no need to handle
            }
        }
    }

    /// Helper function that encodes key emulation option information as a tuple of booleans,
    /// for easy access and matching
    ///
    /// First return bool - true if in CHIP-8 emulation mode
    /// Second return bool - true if in CHIP-48 emulation mode
    /// Third return bool - true if in SUPER-CHIP 1.1. emulation mode
    /// Fourth return bool - true in using variable cycle timing in CHIP-8 emulation mode
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

    /// Instantiates a new [Program] from the stored program file path
    fn get_program(&self) -> Program {
        let program: Program =
            Program::load_from_file(&Path::new(&self.program_file_path)).unwrap();
        program
    }

    /// Instructs the worked thread to notify the current instance of Chipolata that the UI is
    /// ready to receive a new state snapshot, including frame buffer for rendering
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

    /// Wait for the worker thread to supply an updated state snapshot from the hosted Chipolata
    /// instance, then process this to perform the following actions:
    ///
    /// * Keep track of Chipolata's reported target processor speed
    /// * Pause or resume audio as required
    /// * Recalculate the actual processor speed based on the timing of actual cycles completed
    /// * Return the state snapshot's frame buffer, to be rendered in the UI
    ///
    /// If the worker thread passes an error report instead of a state snapshot, then the error
    /// string is extracted and stored (for display in the UI) and the Chipolata instance is
    /// shut down
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
                        // An error has occurred; save the error message and shut down the running
                        // Chipolata instance
                        self.last_error_string = error.inner_error.to_string();
                        self.stop_chipolata();
                    }
                }
            }
        }
        return None;
    }
}
