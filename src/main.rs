use chipolata::EmulationLevel;
use chipolata::COSMAC_VIP_PROCESSOR_SPEED_HERTZ;
// #![allow(unused)]
use chipolata::Options;
use chipolata::Processor;
use chipolata::Program;
use chipolata::StateSnapshot;
use chipolata::StateSnapshotVerbosity;
use eframe::egui;
use egui::*;
use std::fs;
use std::sync::mpsc;
use std::thread;

fn main() -> Result<(), eframe::Error> {
    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(Vec2::from((1920., 960.)));
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
        let program_data =
            fs::read("roms\\demos\\Trip8 Demo (2008) [Revival Studios].ch8").unwrap();
        let program: Program = Program::new(program_data);
        let mut options: Options = Options::default();
        options.processor_speed_hertz = 2500;
        options.emulation_level = EmulationLevel::SuperChip11;
        // options.processor_speed_hertz = COSMAC_VIP_PROCESSOR_SPEED_HERTZ;
        // options.use_variable_cycle_timings = true;
        // options.emulation_level = EmulationLevel::Chip8 {
        //     memory_limit_2k: false,
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
        thread::spawn(move || {
            //let mut time = std::time::Instant::now();
            loop {
                //if time.elapsed() >= std::time::Duration::from_micros(400) {
                //time = std::time::Instant::now();
                for (received, pressed) in proc_input_rx.try_iter() {
                    processor.set_key_status(received, pressed).unwrap();
                }
                processor.execute_cycle().unwrap();
                if proc_ready_rx.try_recv().is_ok() {
                    proc_output_tx
                        .send(processor.export_state_snapshot(StateSnapshotVerbosity::Minimal))
                        .unwrap();
                }
                //}
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
            if let StateSnapshot::MinimalSnapshot { frame_buffer } = disp {
                for i in 0..64 {
                    for j in 0..32 {
                        let colour: egui::Color32 =
                            match frame_buffer.pixels[j][i / 8] & (128 >> (i % 8)) {
                                0 => egui::Color32::BLACK,
                                _ => egui::Color32::WHITE,
                            };
                        painter.rect_filled(
                            egui::Rect::from_two_pos(
                                Pos2::from((i as f32 * 30., j as f32 * 30.)),
                                Pos2::from(((i + 1) as f32 * 30., (j + 1) as f32 * 30.)),
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
