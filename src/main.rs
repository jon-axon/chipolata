use chipolata::EmulationLevel;
use chipolata::Options;
use chipolata::Processor;
use chipolata::Program;
use chipolata::StateSnapshot;
use chipolata::StateSnapshotVerbosity;
use chipolata::COSMAC_VIP_PROCESSOR_SPEED_HERTZ;
use eframe::egui;
use egui::*;
use std::fs;
use std::sync::mpsc;
use std::thread;

const WIDTH: f32 = 960.;
const HEIGHT: f32 = 480.;

fn main() -> Result<(), eframe::Error> {
    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(Vec2::from((WIDTH, HEIGHT)));
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
}

impl ChipolataApp {
    pub fn new() -> Self {
        let program_data = fs::read("roms\\tests\\chip8-test-suite.ch8").unwrap();
        let program: Program = Program::new(program_data);
        let mut options: Options = Options::default();
        // options.processor_speed_hertz = 2500;
        // options.emulation_level = EmulationLevel::SuperChip11;
        options.processor_speed_hertz = 2500;
        options.emulation_level = EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: false,
        };
        // options.processor_speed_hertz = COSMAC_VIP_PROCESSOR_SPEED_HERTZ;
        // options.emulation_level = EmulationLevel::Chip8 {
        //     memory_limit_2k: false,
        //     variable_cycle_timing: true,
        // };
        let mut processor = Processor::initialise_and_load(program, options).unwrap();
        let (proc_input_tx, proc_input_rx) = mpsc::channel();
        let (proc_output_tx, proc_output_rx) = mpsc::channel();
        let (proc_ready_tx, proc_ready_rx) = mpsc::channel();
        let app = ChipolataApp {
            proc_input_tx: proc_input_tx,
            proc_output_rx: proc_output_rx,
            proc_ready_tx: proc_ready_tx,
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.proc_ready_tx
                .send(StateSnapshotVerbosity::Minimal)
                .unwrap();
            self.handle_input(ctx);
            //self.play_sound(ctx);
            self.render_ui(ui);
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

    fn play_sound(&mut self, ctx: &egui::Context) {
        todo!();
    }

    fn render_ui(&mut self, ui: &mut Ui) {
        let painter = ui.painter();
        if let Ok(disp) = self.proc_output_rx.try_recv() {
            if let StateSnapshot::MinimalSnapshot {
                frame_buffer,
                status: _,
            } = disp
            {
                let row_pixels: usize = frame_buffer.get_row_size_bytes() * 8;
                let column_pixels: usize = frame_buffer.get_column_size_pixels();
                let pixel_size: f32 = (WIDTH as usize / row_pixels) as f32;
                for i in 0..row_pixels {
                    for j in 0..column_pixels {
                        let colour: egui::Color32 = match frame_buffer[j][i / 8] & (128 >> (i % 8))
                        {
                            0 => egui::Color32::KHAKI,
                            _ => egui::Color32::DARK_GRAY,
                        };
                        painter.rect_filled(
                            egui::Rect::from_two_pos(
                                Pos2::from((i as f32 * pixel_size, j as f32 * pixel_size)),
                                Pos2::from((
                                    (i + 1) as f32 * pixel_size,
                                    (j + 1) as f32 * pixel_size,
                                )),
                            ),
                            egui::Rounding::none(),
                            colour,
                        );
                    }
                }
            }
        }
    }
}
