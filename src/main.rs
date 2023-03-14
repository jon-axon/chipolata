use audio::Audio;
use chipolata::Options;
use chipolata::Processor;
use chipolata::Program;
use chipolata::StateSnapshot;
use chipolata::StateSnapshotVerbosity;
use eframe::egui;
use egui::*;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

mod audio;

const INITIAL_WIDTH: f32 = 960.;
const INITIAL_HEIGHT: f32 = 480.;

fn main() -> Result<(), eframe::Error> {
    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(Vec2::from((INITIAL_WIDTH, INITIAL_HEIGHT)));
    eframe::run_native(
        "Chipolata: CHIP-8 emulator",
        options,
        Box::new(|_cc| Box::new(ChipolataApp::new())),
    )
}

struct ChipolataApp {
    proc_input_tx: mpsc::Sender<(u8, bool)>,
    proc_output_rx: mpsc::Receiver<StateSnapshot>,
    proc_ready_tx: mpsc::Sender<StateSnapshotVerbosity>,
    audio_stream: Audio,
}

impl ChipolataApp {
    pub fn new() -> Self {
        let program_file: &str = "tests\\chip8-test-suite.ch8";
        // let program_file: &str = "superchip\\SPACEFIG.ch8";
        // let program_file: &str = "superchip\\knight.ch8";
        // let program_file: &str = "superchip\\binding.ch8";
        // let program_file: &str = "superchip\\JOUST.ch8";
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
        let (proc_input_tx, proc_input_rx) = mpsc::channel();
        let (proc_output_tx, proc_output_rx) = mpsc::channel();
        let (proc_ready_tx, proc_ready_rx) = mpsc::channel();

        let app = ChipolataApp {
            proc_input_tx,
            proc_output_rx,
            proc_ready_tx,
            audio_stream: Audio::new(),
        };
        thread::spawn(move || loop {
            for (received, pressed) in proc_input_rx.try_iter() {
                processor.set_key_status(received, pressed).unwrap();
            }
            processor.execute_cycle().unwrap();
            if proc_ready_rx.try_recv().is_ok() {
                proc_output_tx
                    .send(processor.export_state_snapshot(StateSnapshotVerbosity::Minimal))
                    .unwrap();
            }
        });
        app
    }
}

impl eframe::App for ChipolataApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.proc_ready_tx
                .send(StateSnapshotVerbosity::Minimal)
                .unwrap();
            self.handle_input(ctx);
            if let Ok(disp) = self.proc_output_rx.try_recv() {
                if let StateSnapshot::MinimalSnapshot {
                    frame_buffer,
                    status: _,
                    play_sound,
                } = disp
                {
                    match (play_sound, self.audio_stream.is_paused()) {
                        (true, true) => self.audio_stream.play(),
                        (false, false) => self.audio_stream.pause(),
                        _ => (),
                    }
                    ChipolataApp::render_ui(frame_buffer, ui, frame);
                }
            }
        });
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
                    Key::Num1 => self.proc_input_tx.send((0x1, *state)).unwrap(),
                    Key::Num2 => self.proc_input_tx.send((0x2, *state)).unwrap(),
                    Key::Num3 => self.proc_input_tx.send((0x3, *state)).unwrap(),
                    Key::Num4 => self.proc_input_tx.send((0xC, *state)).unwrap(),
                    Key::Q => self.proc_input_tx.send((0x4, *state)).unwrap(),
                    Key::W => self.proc_input_tx.send((0x5, *state)).unwrap(),
                    Key::E => self.proc_input_tx.send((0x6, *state)).unwrap(),
                    Key::R => self.proc_input_tx.send((0xD, *state)).unwrap(),
                    Key::A => self.proc_input_tx.send((0x7, *state)).unwrap(),
                    Key::S => self.proc_input_tx.send((0x8, *state)).unwrap(),
                    Key::D => self.proc_input_tx.send((0x9, *state)).unwrap(),
                    Key::F => self.proc_input_tx.send((0xE, *state)).unwrap(),
                    Key::Z => self.proc_input_tx.send((0xA, *state)).unwrap(),
                    Key::X => self.proc_input_tx.send((0x0, *state)).unwrap(),
                    Key::C => self.proc_input_tx.send((0xB, *state)).unwrap(),
                    Key::V => self.proc_input_tx.send((0xF, *state)).unwrap(),
                    _ => (),
                }
            }
        });
    }

    fn render_ui(frame_buffer: chipolata::Display, ui: &mut Ui, frame: &eframe::Frame) {
        let painter = ui.painter();
        let row_pixels: usize = frame_buffer.get_row_size_bytes() * 8;
        let column_pixels: usize = frame_buffer.get_column_size_pixels();
        let pixel_width: f32 = frame.info().window_info.size[0] / (row_pixels as f32);
        let pixel_height: f32 = frame.info().window_info.size[1] / (column_pixels as f32);
        for i in 0..row_pixels {
            for j in 0..column_pixels {
                let colour: egui::Color32 = match frame_buffer[j][i / 8] & (128 >> (i % 8)) {
                    0 => egui::Color32::from_rgb(0x99, 0x66, 00),
                    _ => egui::Color32::from_rgb(0xFF, 0xCC, 00),
                };
                let stroke: egui::Stroke = Stroke::new(1., colour);
                painter.rect(
                    egui::Rect::from_two_pos(
                        Pos2::from((i as f32 * pixel_width, j as f32 * pixel_height)),
                        Pos2::from(((i + 1) as f32 * pixel_width, (j + 1) as f32 * pixel_height)),
                    ),
                    egui::Rounding::none(),
                    colour,
                    stroke,
                );
            }
        }
    }
}
