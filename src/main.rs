use audio::Audio;
use chipolata::Options;
use chipolata::Processor;
use chipolata::Program;
use chipolata::StateSnapshot;
use chipolata::StateSnapshotVerbosity;
use eframe::egui;
use egui::*;
use image;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

mod audio;

const INITIAL_WIDTH: f32 = 960.;
const INITIAL_HEIGHT: f32 = 480.;

fn main() -> Result<(), eframe::Error> {
    let icon = image::open("F:\\Rust\\Projects\\chipolata\\assets\\chipolata.png")
        .expect("Failed to open icon path")
        .to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();
    let options = eframe::NativeOptions {
        icon_data: Some(eframe::IconData {
            rgba: icon.into_raw(),
            width: icon_width,
            height: icon_height,
        }),
        initial_window_size: Some(Vec2::from((INITIAL_WIDTH, INITIAL_HEIGHT))),
        ..Default::default()
    };

    eframe::run_native(
        "Chipolata: CHIP-8 emulator",
        options,
        Box::new(|_cc| Box::new(ChipolataApp::new())),
    )
}

enum MessageToChipolata {
    ReadyForStateSnapshot { verbosity: StateSnapshotVerbosity },
    KeyPressEvent { key: u8, pressed: bool },
    SetProcessorSpeed { new_speed: u64 },
    Pause,
    Resume,
}

enum MessageFromChipolata {
    StateSnapshotReport { snapshot: StateSnapshot },
}

struct ChipolataApp {
    message_to_chipolata_tx: mpsc::Sender<MessageToChipolata>,
    message_from_chipolata_rx: mpsc::Receiver<MessageFromChipolata>,
    audio_stream: Audio,
    processor_speed: u64,
}

impl ChipolataApp {
    pub fn new() -> Self {
        // let program_file: &str = "tests\\chip8-test-suite.ch8";
        // let program_file: &str = "superchip\\SPACEFIG.ch8";
        // let program_file: &str = "superchip\\knight.ch8";
        // let program_file: &str = "superchip\\binding.ch8";
        // let program_file: &str = "superchip\\JOUST.ch8";
        let program_file: &str = "demos\\Trip8 Demo (2008) [Revival Studios].ch8";
        let program: Program = Program::load_from_file(
            &Path::new("F:\\Rust\\Projects\\chipolata\\resources\\roms").join(program_file),
        )
        .unwrap();
        // let option_file: &str = "SCHIP-slow.json";
        // let option_file: &str = "SCHIP-fast.json";
        let option_file: &str = "SCHIP-octo.json";
        // let option_file: &str = "VIP-slow.json";
        // let option_file: &str = "VIP-fast.json";
        // let option_file: &str = "VIP-variable.json";
        let options: Options = Options::load_from_file(
            &Path::new("F:\\Rust\\Projects\\chipolata\\resources\\options").join(option_file),
        )
        .unwrap();
        let mut processor = Processor::initialise_and_load(program, options).unwrap();
        // Prepare cross-thread communication channels between UI and Chipolata
        let (message_to_chipolata_tx, message_to_chipolata_rx) = mpsc::channel();
        let (message_from_chipolata_tx, message_from_chipolata_rx) = mpsc::channel();

        let app = ChipolataApp {
            message_to_chipolata_tx,
            message_from_chipolata_rx,
            processor_speed: processor.processor_speed(),
            audio_stream: Audio::new(),
        };
        thread::spawn(move || loop {
            let mut ui_ready_for_update: bool = false;
            let mut snapshot_verbosity: StateSnapshotVerbosity = StateSnapshotVerbosity::Minimal;
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
                }
            }
            // Run a Chipolata processor cycle
            processor.execute_cycle().unwrap();
            // Send a state snapshot update back to UI if requested
            if ui_ready_for_update {
                let snapshot = processor.export_state_snapshot(snapshot_verbosity);
                message_from_chipolata_tx
                    .send(MessageFromChipolata::StateSnapshotReport { snapshot })
                    .unwrap();
            }
        });
        app
    }
}

impl eframe::App for ChipolataApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Inform Chipolata the UI is ready for a state snapshot update
        self.message_to_chipolata_tx
            .send(MessageToChipolata::ReadyForStateSnapshot {
                verbosity: StateSnapshotVerbosity::Minimal,
            })
            .unwrap();
        // Check for key press events
        self.handle_input(ctx);
        // Render the header panel
        self.render_header(ctx);
        // Process any received state snapshot update from Chipolata
        if let Ok(MessageFromChipolata::StateSnapshotReport { snapshot }) =
            self.message_from_chipolata_rx.try_recv()
        {
            if let StateSnapshot::MinimalSnapshot {
                frame_buffer,
                status: _,
                processor_speed,
                play_sound,
            } = snapshot
            {
                // Keep track of current processor speed
                self.processor_speed = processor_speed;
                // Pause / resume audio if required
                match (play_sound, self.audio_stream.is_paused()) {
                    (true, true) => self.audio_stream.play(),
                    (false, false) => self.audio_stream.pause(),
                    _ => (),
                }
                // Refresh the UI
                ChipolataApp::render_chipolata_ui(ctx, frame_buffer);
            }
        }
        // Update UI again as soon as possible
        ctx.request_repaint();
    }
}

impl ChipolataApp {
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

    fn send_key_press_event(&mut self, key: u8, pressed: bool) {
        self.message_to_chipolata_tx
            .send(MessageToChipolata::KeyPressEvent { key, pressed })
            .unwrap();
    }

    fn render_header(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Pause").clicked() {
                        self.message_to_chipolata_tx
                            .send(MessageToChipolata::Pause)
                            .unwrap();
                    }
                    if ui.button("Resume").clicked() {
                        self.message_to_chipolata_tx
                            .send(MessageToChipolata::Resume)
                            .unwrap();
                    }
                    if ui.button("Open").clicked() {
                        let open_file: String;
                        match tinyfiledialogs::open_file_dialog("Open", "password.txt", None) {
                            Some(file) => open_file = file,
                            None => open_file = "null".to_string(),
                        }
                    }
                })
            });
            let old_speed: u64 = self.processor_speed;
            ui.add(Slider::new(&mut self.processor_speed, 100..=20000).text("Speed"));
            if self.processor_speed != old_speed {
                self.message_to_chipolata_tx
                    .send(MessageToChipolata::SetProcessorSpeed {
                        new_speed: self.processor_speed,
                    })
                    .unwrap();
            }
        });
    }

    fn render_chipolata_ui(ctx: &egui::Context, frame_buffer: chipolata::Display) {
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
                        0 => egui::Color32::from_rgb(0x99, 0x66, 00),
                        _ => egui::Color32::from_rgb(0xFF, 0xCC, 00),
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
}
