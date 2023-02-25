use super::*;
use crate::program::Program;
use std::time::{Duration, Instant};

fn setup_test_processor_chip8() -> Processor {
    let program: Program = Program::default();
    Processor::initialise_and_load(program, Options::default()).unwrap()
}

fn setup_test_processor_chip48() -> Processor {
    let program: Program = Program::default();
    let mut options: Options = Options::default();
    options.emulation_level = EmulationLevel::Chip48;
    Processor::initialise_and_load(program, options).unwrap()
}

fn setup_test_processor_superchip11() -> Processor {
    let program: Program = Program::default();
    let mut options: Options = Options::default();
    options.emulation_level = EmulationLevel::SuperChip11;
    Processor::initialise_and_load(program, options).unwrap()
}

#[test]
fn test_load_font_data() {
    let mut processor: Processor = setup_test_processor_chip8();
    let stored_font: Vec<u8> = Vec::from(
        processor
            .memory
            .read_bytes(
                processor.font_start_address,
                processor.font.font_data_size(),
            )
            .unwrap(),
    );
    assert!(processor.load_font_data().is_ok());
    assert_eq!(stored_font, *processor.font.font_data());
}

#[test]
fn test_load_font_data_overflow_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.font_start_address = processor.memory.max_addressable_size() - 0x1;
    assert_eq!(
        processor.load_font_data().unwrap_err(),
        Error::MemoryAddressOutOfBounds
    );
}

#[test]
fn test_load_program() {
    let program_data: Vec<u8> = vec![0xFF, 0x0A, 0x12, 0xC4, 0xD1];
    let program: Program = Program::new(program_data.clone());
    let processor: Processor = Processor::initialise_and_load(program, Options::default()).unwrap();
    assert_eq!(
        program_data,
        processor
            .memory
            .read_bytes(processor.program_start_address, program_data.len())
            .unwrap()
    );
}

#[test]
fn test_load_program_overflow_error() {
    let program_data: Vec<u8> = vec![0xFF, 0x0A, 0x12, 0xC4, 0xD1];
    let program: Program = Program::new(program_data);
    let mut processor: Processor =
        Processor::initialise_and_load(program, Options::default()).unwrap();
    processor.program_start_address = processor.memory.max_addressable_size() - 0x1;
    assert_eq!(
        processor.load_program().unwrap_err(),
        Error::MemoryAddressOutOfBounds
    );
}

#[test]
fn test_export_state_snapshot_minimal() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.frame_buffer.pixels[0][0] = 0xC3;
    let state_snapshot: StateSnapshot = processor
        .export_state_snapshot(StateSnapshotVerbosity::Minimal)
        .unwrap();
    assert!(
        matches!(state_snapshot, StateSnapshot::MinimalSnapshot { .. })
            && match state_snapshot {
                StateSnapshot::MinimalSnapshot { frame_buffer } =>
                    frame_buffer.pixels[0][0] == 0xC3,
                _ => false,
            }
    );
}

#[test]
fn test__state_snapshot_verbose() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.frame_buffer.pixels[0][0] = 0xC3;
    processor.program_counter = 0x1DF1;
    processor.index_register = 0x3CC2;
    processor.variable_registers[0x4] = 0xB2;
    processor.delay_timer = 0x3;
    processor.sound_timer = 0x4;
    processor.stack.push(0x30E1).unwrap();
    processor.memory.bytes[0x33] = 0x44;
    processor.cycles = 16473;
    let state_snapshot: StateSnapshot = processor
        .export_state_snapshot(StateSnapshotVerbosity::Extended)
        .unwrap();
    assert!(
        matches!(state_snapshot, StateSnapshot::ExtendedSnapshot { .. })
            && match state_snapshot {
                StateSnapshot::ExtendedSnapshot {
                    frame_buffer,
                    program_counter,
                    index_register,
                    variable_registers,
                    delay_timer,
                    sound_timer,
                    mut stack,
                    memory,
                    cycles,
                } =>
                    frame_buffer.pixels[0][0] == 0xC3
                        && program_counter == 0x1DF1
                        && index_register == 0x3CC2
                        && variable_registers[0x4] == 0xB2
                        && delay_timer == 0x3
                        && sound_timer == 0x4
                        && stack.pop().unwrap() == 0x30E1
                        && memory.bytes[0x33] == 0x44
                        && cycles == 16473,
                _ => false,
            }
    );
}

#[test]
fn test_execute_cycle() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x0BC1;
    let instruction: [u8; 2] = [0xA1, 0x11];
    processor.memory.write_bytes(0x0BC1, &instruction).unwrap();
    assert!(processor.execute_cycle().is_ok() && processor.program_counter == 0x0BC3);
}

#[test]
fn test_check_sound_timer() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.sound_timer = 0;
    let result_one: bool = processor.sound_timer_active();
    processor.sound_timer = 1;
    let result_two: bool = processor.sound_timer_active();
    assert!(!result_one && result_two);
}

#[test]
fn test_decrement_timers() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.delay_timer = 0x1B;
    processor.sound_timer = 0xEC;
    let duration: Duration =
        Duration::from_micros(100 + TIMER_DECREMENT_INTERVAL_MICROSECONDS as u64);
    let last_time: Instant = Instant::now() - duration;
    processor.last_timer_decrement = last_time;
    processor.decrement_timers();
    assert!(
        processor.delay_timer == 0x1A
            && processor.sound_timer == 0xEB
            && processor
                .last_timer_decrement
                .duration_since(last_time)
                .as_nanos()
                > 0
    );
}

#[test]
fn test_decrement_timers_too_early() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.delay_timer = 0x1B;
    processor.sound_timer = 0xEC;
    let last_time: Instant = Instant::now();
    processor.last_timer_decrement = last_time;
    processor.decrement_timers();
    assert!(
        processor.delay_timer == 0x1B
            && processor.sound_timer == 0xEC
            && processor
                .last_timer_decrement
                .duration_since(last_time)
                .as_nanos()
                == 0
    );
}

#[test]
fn test_decrement_timers_stopped() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.delay_timer = 0x00;
    processor.sound_timer = 0x00;
    let duration: Duration =
        Duration::from_micros(100 + TIMER_DECREMENT_INTERVAL_MICROSECONDS as u64);
    let last_time: Instant = Instant::now() - duration;
    processor.last_timer_decrement = last_time;
    processor.decrement_timers();
    assert!(processor.delay_timer == 0x0 && processor.sound_timer == 0x0);
}

#[test]
fn test_execute_004B() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_004B().unwrap_err(),
        Error::UnimplementedInstruction
    );
}

#[test]
fn test_execute_00E0() {
    let mut processor: Processor = setup_test_processor_chip8();
    // Set every pixel to 1
    for row in &mut processor.frame_buffer.pixels {
        for col in &mut *row {
            *col = 0xFF;
        }
    }
    // Now execute the instruction to clear the display
    processor.execute_00E0().unwrap();
    // Now check that every pixel is 0
    let mut pixel_is_set: bool = false;
    'outer: for row in &processor.frame_buffer.pixels {
        for col in row {
            if *col > 0x00 {
                pixel_is_set = true;
                break 'outer;
            }
        }
    }
    assert!(!pixel_is_set);
}

#[test]
fn test_execute_00EE() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.stack.push(0xB35E).unwrap();
    assert!(
        processor.execute_00EE().is_ok()
            && processor.stack.pop().is_err()
            && processor.program_counter == 0xB35E
    );
}

#[test]
fn test_execute_00EE_empty_stack_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(processor.execute_00EE().unwrap_err(), Error::PopEmptyStack);
}

#[test]
fn test_execute_0NNN() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_0NNN(0x2F5).unwrap_err(),
        Error::UnimplementedInstruction
    );
}

#[test]
fn test_execute_1NNN() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert!(processor.execute_1NNN(0xEA5).is_ok() && processor.program_counter == 0xEA5);
}

#[test]
fn test_execute_2NNN() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0xF03;
    assert!(
        processor.execute_2NNN(0x44F).is_ok()
            && processor.stack.pop().unwrap() == 0xF03
            && processor.program_counter == 0x44F
    );
}

#[test]
fn test_execute_3XNN() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0x3] = 0xBB;
    assert!(processor.execute_3XNN(0x3, 0xBB).is_ok() && processor.program_counter == 0x15);
}

#[test]
fn test_execute_3XNN_no_action() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0x3] = 0xBA;
    assert!(processor.execute_3XNN(0x3, 0xBB).is_ok() && processor.program_counter == 0x13);
}

#[test]
fn test_execute_3XNN_invalid_register_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_3XNN(0x10, 0x2F).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_4XNN() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0x3] = 0xBB;
    assert!(processor.execute_4XNN(0x3, 0xBB).is_ok() && processor.program_counter == 0x13);
}

#[test]
fn test_execute_4XNN_no_action() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0x3] = 0xBA;
    assert!(processor.execute_4XNN(0x3, 0xBB).is_ok() && processor.program_counter == 0x15);
}

#[test]
fn test_execute_4XNN_invalid_register_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_4XNN(0x10, 0x2F).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_5XY0() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0x3] = 0xBA;
    processor.variable_registers[0xD] = 0xBA;
    assert!(processor.execute_5XY0(0x3, 0xD).is_ok() && processor.program_counter == 0x15);
}

#[test]
fn test_execute_5XY0_no_action() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0x3] = 0xBA;
    processor.variable_registers[0xD] = 0xBB;
    assert!(processor.execute_5XY0(0x3, 0xD).is_ok() && processor.program_counter == 0x13);
}

#[test]
fn test_execute_5XY0_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_5XY0(0x10, 0xD).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_5XY0_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_5XY0(0x3, 0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_6XNN() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert!(processor.execute_6XNN(0xB, 0x2F).is_ok() && processor.variable_registers[0xB] == 0x2F);
}

#[test]
fn test_execute_6XNN_invalid_register_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_6XNN(0x10, 0x2F).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_7XNN() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0x9] = 0x3;
    assert!(processor.execute_7XNN(0x9, 0xE0).is_ok() && processor.variable_registers[0x9] == 0xE3);
}

#[test]
fn test_execute_7XNN_overflow() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0x1] = 0xFF;
    assert!(processor.execute_7XNN(0x1, 0x05).is_ok() && processor.variable_registers[0x1] == 0x04);
}

#[test]
fn test_execute_7XNN_invalid_register_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_7XNN(0x10, 0x1E).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XY0() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xE] = 0x01;
    processor.variable_registers[0x7] = 0x51;
    assert!(processor.execute_8XY0(0xE, 0x7).is_ok() && processor.variable_registers[0xE] == 0x51);
}

#[test]
fn test_execute_8XY0_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XY0(0x10, 0xD).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XY0_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XY0(0x3, 0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XY1() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xE] = 0x9B;
    processor.variable_registers[0x7] = 0xAA;
    assert!(processor.execute_8XY1(0xE, 0x7).is_ok() && processor.variable_registers[0xE] == 0xBB);
}

#[test]
fn test_execute_8XY1_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XY1(0x10, 0xD).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XY1_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XY1(0x3, 0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XY2() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xE] = 0x9B;
    processor.variable_registers[0x7] = 0xAA;
    assert!(processor.execute_8XY2(0xE, 0x7).is_ok() && processor.variable_registers[0xE] == 0x8A);
}

#[test]
fn test_execute_8XY2_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XY2(0x10, 0xD).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XY2_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XY2(0x3, 0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XY3() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xE] = 0x9B;
    processor.variable_registers[0x7] = 0xAA;
    assert!(processor.execute_8XY3(0xE, 0x7).is_ok() && processor.variable_registers[0xE] == 0x31);
}

#[test]
fn test_execute_8XY3_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XY3(0x10, 0xD).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XY3_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XY3(0x3, 0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XY4() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xE] = 0xF2;
    processor.variable_registers[0x7] = 0x06;
    processor.variable_registers[0xF] = 0x44;
    assert!(
        processor.execute_8XY4(0xE, 0x7).is_ok()
            && processor.variable_registers[0xE] == 0xF8
            && processor.variable_registers[0xF] == 0x00
    );
}

#[test]
fn test_execute_8XY4_overflow() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xE] = 0xF2;
    processor.variable_registers[0x7] = 0x16;
    processor.variable_registers[0xF] = 0x44;
    assert!(
        processor.execute_8XY4(0xE, 0x7).is_ok()
            && processor.variable_registers[0xE] == 0x08
            && processor.variable_registers[0xF] == 0x01
    );
}

#[test]
fn test_execute_8XY4_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XY4(0x10, 0xD).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XY4_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XY4(0x3, 0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XY5() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xE] = 0xF2;
    processor.variable_registers[0x7] = 0x06;
    processor.variable_registers[0xF] = 0x44;
    assert!(
        processor.execute_8XY5(0xE, 0x7).is_ok()
            && processor.variable_registers[0xE] == 0xEC
            && processor.variable_registers[0xF] == 0x01
    );
}

#[test]
fn test_execute_8XY5_underflow() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xE] = 0x16;
    processor.variable_registers[0x7] = 0xF2;
    processor.variable_registers[0xF] = 0x44;
    assert!(
        processor.execute_8XY5(0xE, 0x7).is_ok()
            && processor.variable_registers[0xE] == 0x24
            && processor.variable_registers[0xF] == 0x00
    );
}

#[test]
fn test_execute_8XY5_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XY5(0x10, 0xD).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XY5_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XY5(0x3, 0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XY6_1_shifted() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xE] = 0x06;
    processor.variable_registers[0x7] = 0xD9;
    processor.variable_registers[0xF] = 0x44;
    assert!(
        processor.execute_8XY6(0xE, 0x7).is_ok()
            && processor.variable_registers[0xE] == 0x6C
            && processor.variable_registers[0xF] == 0x01
    );
}

#[test]
fn test_execute_8XY6_0_shifted() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xE] = 0x06;
    processor.variable_registers[0x7] = 0x62;
    processor.variable_registers[0xF] = 0x44;
    assert!(
        processor.execute_8XY6(0xE, 0x7).is_ok()
            && processor.variable_registers[0xE] == 0x31
            && processor.variable_registers[0xF] == 0x00
    );
}

#[test]
fn test_execute_8XY6_1_shifted_chip48_mode() {
    let mut processor: Processor = setup_test_processor_chip48();
    processor.variable_registers[0xE] = 0xD9;
    processor.variable_registers[0x7] = 0x06;
    processor.variable_registers[0xF] = 0x44;
    assert!(
        processor.execute_8XY6(0xE, 0x7).is_ok()
            && processor.variable_registers[0xE] == 0x6C
            && processor.variable_registers[0xF] == 0x01
    );
}

#[test]
fn test_execute_8XY6_0_shifted_chip48_mode() {
    let mut processor: Processor = setup_test_processor_chip48();
    processor.variable_registers[0xE] = 0x62;
    processor.variable_registers[0x7] = 0x06;
    processor.variable_registers[0xF] = 0x44;
    assert!(
        processor.execute_8XY6(0xE, 0x7).is_ok()
            && processor.variable_registers[0xE] == 0x31
            && processor.variable_registers[0xF] == 0x00
    );
}

#[test]
fn test_execute_8XY6_1_shifted_superchip11_mode() {
    let mut processor: Processor = setup_test_processor_superchip11();
    processor.variable_registers[0xE] = 0xD9;
    processor.variable_registers[0x7] = 0x06;
    processor.variable_registers[0xF] = 0x44;
    assert!(
        processor.execute_8XY6(0xE, 0x7).is_ok()
            && processor.variable_registers[0xE] == 0x6C
            && processor.variable_registers[0xF] == 0x01
    );
}

#[test]
fn test_execute_8XY6_0_shifted_superchip11_mode() {
    let mut processor: Processor = setup_test_processor_superchip11();
    processor.variable_registers[0xE] = 0x62;
    processor.variable_registers[0x7] = 0x06;
    processor.variable_registers[0xF] = 0x44;
    assert!(
        processor.execute_8XY6(0xE, 0x7).is_ok()
            && processor.variable_registers[0xE] == 0x31
            && processor.variable_registers[0xF] == 0x00
    );
}

#[test]
fn test_execute_8XY6_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XY6(0x10, 0xD).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XY6_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XY6(0x3, 0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XY7() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xE] = 0x06;
    processor.variable_registers[0x7] = 0xF2;
    processor.variable_registers[0xF] = 0x44;
    assert!(
        processor.execute_8XY7(0xE, 0x7).is_ok()
            && processor.variable_registers[0xE] == 0xEC
            && processor.variable_registers[0xF] == 0x01
    );
}

#[test]
fn test_execute_8XY7_underflow() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xE] = 0xF2;
    processor.variable_registers[0x7] = 0x16;
    processor.variable_registers[0xF] = 0x44;
    assert!(
        processor.execute_8XY7(0xE, 0x7).is_ok()
            && processor.variable_registers[0xE] == 0x24
            && processor.variable_registers[0xF] == 0x00
    );
}

#[test]
fn test_execute_8XY7_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XY7(0x10, 0xD).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XY7_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XY7(0x3, 0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XYE_1_shifted() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xE] = 0x06;
    processor.variable_registers[0x7] = 0x9B;
    processor.variable_registers[0xF] = 0x44;
    assert!(
        processor.execute_8XYE(0xE, 0x7).is_ok()
            && processor.variable_registers[0xE] == 0x36
            && processor.variable_registers[0xF] == 0x01
    );
}

#[test]
fn test_execute_8XYE_0_shifted() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xE] = 0x06;
    processor.variable_registers[0x7] = 0x62;
    processor.variable_registers[0xF] = 0x44;
    assert!(
        processor.execute_8XYE(0xE, 0x7).is_ok()
            && processor.variable_registers[0xE] == 0xC4
            && processor.variable_registers[0xF] == 0x00
    );
}

#[test]
fn test_execute_8XYE_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XYE(0x10, 0xD).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_8XYE_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_8XYE(0x3, 0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_9XY0() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0x3] = 0xBA;
    processor.variable_registers[0xD] = 0xBB;
    assert!(processor.execute_9XY0(0x3, 0xD).is_ok() && processor.program_counter == 0x15);
}

#[test]
fn test_execute_9XY0_no_action() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0x3] = 0xBA;
    processor.variable_registers[0xD] = 0xBA;
    assert!(processor.execute_9XY0(0x3, 0xD).is_ok() && processor.program_counter == 0x13);
}

#[test]
fn test_execute_9XY0_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_9XY0(0x10, 0xD).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_9XY0_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_9XY0(0x3, 0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_ANNN() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert!(processor.execute_ANNN(0x0A5).is_ok() && processor.index_register == 0x0A5);
}

#[test]
fn test_execute_BNNN() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0] = 0x42;
    processor.variable_registers[3] = 0x1B;
    assert!(processor.execute_BNNN(0x3A5).is_ok() && processor.program_counter == 0x3E7);
}

#[test]
fn test_execute_BNNN_chip48_mode() {
    let mut processor: Processor = setup_test_processor_chip48();
    processor.program_counter = 0x13;
    processor.variable_registers[0] = 0x42;
    processor.variable_registers[3] = 0x1B;
    assert!(processor.execute_BNNN(0x3A5).is_ok() && processor.program_counter == 0x3C0);
}

#[test]
fn test_execute_BNNN_superchip11_mode() {
    let mut processor: Processor = setup_test_processor_superchip11();
    processor.program_counter = 0x13;
    processor.variable_registers[0] = 0x42;
    processor.variable_registers[3] = 0x1B;
    assert!(processor.execute_BNNN(0x3A5).is_ok() && processor.program_counter == 0x3C0);
}

#[test]
fn test_execute_CXNN_0_operand() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0x5] = 0xF;
    assert!(processor.execute_CXNN(0x5, 0x0).is_ok() && processor.variable_registers[0x5] == 0x0);
}

#[test]
fn test_execute_CXNN_0_nondeterministic() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0x5] = 0xF;
    processor.execute_CXNN(0x5, 0xFF).unwrap();
    let result_one: u8 = processor.variable_registers[0x5];
    processor.execute_CXNN(0x5, 0xFF).unwrap();
    let result_two: u8 = processor.variable_registers[0x5];
    assert_ne!(result_one, result_two);
}

#[test]
fn test_execute_CXNN_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_CXNN(0x10, 0xD).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

fn fill_row(display: &mut Display, y: usize) {
    for i in &mut display.pixels[y] {
        *i = 0xFF;
    }
}

#[test]
fn test_execute_DXYN_pixel_turned_off() {
    let mut processor: Processor = setup_test_processor_chip8();
    fill_row(&mut processor.frame_buffer, 0x1); // all display pixels on in second row
    processor.frame_buffer.pixels[0x1][0x0] = 0x0; // turn off first byte of pixels only
    processor.variable_registers[0xF] = 0x2; // only possible values later are 0x0 and 0x1
    processor.index_register = processor.font_start_address as u16;
    let sprite: [u8; 1] = [0xFF]; // create single-byte sprite with all pixels on
    processor
        .memory
        .write_bytes(processor.font_start_address, &sprite)
        .unwrap(); // write sprite to memory at default font location
    processor.variable_registers[0x3] = 0x8; // set V3 to 0 (X coordinate)
    processor.variable_registers[0xA] = 0x1; // set V10 to 1 (Y coordinate)
    processor.execute_DXYN(0x3, 0xA, 1).unwrap();
    assert_eq!(processor.variable_registers[0xF], 0x1); // at least one pixel will flip if successful
}

#[test]
fn test_execute_DXYN_no_pixel_turned_off() {
    let mut processor: Processor = setup_test_processor_chip8();
    fill_row(&mut processor.frame_buffer, 0x1); // all display pixels on in second row
    processor.variable_registers[0xF] = 0x2; // only possible values later are 0x0 and 0x1
    processor.index_register = processor.font_start_address as u16;
    let sprite: [u8; 1] = [0x0]; // create single-byte sprite with all pixels off
    processor
        .memory
        .write_bytes(processor.font_start_address, &sprite)
        .unwrap(); // write sprite to memory at default font location
    processor.variable_registers[0x3] = 0x0; // set V3 to 0 (X coordinate)
    processor.variable_registers[0xA] = 0x1; // set V10 to 1 (Y coordinate)
    processor.execute_DXYN(0x3, 0xA, 1).unwrap();
    assert_eq!(processor.variable_registers[0xF], 0x0); // no pixel will flip if successful
}

#[test]
fn test_execute_DXYN_invalid_x_register_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_DXYN(0x10, 0x2, 0x5).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_DXYN_invalid_y_register_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_DXYN(0x2, 0x10, 0x5).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_DXYN_invalid_n_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_DXYN(0x2, 0x5, 0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_EX9E_pressed() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0x9] = 0xA;
    processor.keystate.set_key_status(0xA, true).unwrap();
    assert!(processor.execute_EX9E(0x9).is_ok() && processor.program_counter == 0x15);
}

#[test]
fn test_execute_EX9E_not_pressed() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0x9] = 0xA;
    processor.keystate.set_key_status(0xA, false).unwrap();
    assert!(processor.execute_EX9E(0x9).is_ok() && processor.program_counter == 0x13);
}

#[test]
fn test_execute_EX9E_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_EX9E(0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_EX9E_invalid_key_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0x9] = 0x10;
    assert_eq!(processor.execute_EX9E(0x9).unwrap_err(), Error::InvalidKey);
}

#[test]
fn test_execute_EXA1_pressed() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0x9] = 0xA;
    processor.keystate.set_key_status(0xA, true).unwrap();
    assert!(processor.execute_EXA1(0x9).is_ok() && processor.program_counter == 0x13);
}

#[test]
fn test_execute_EXA1_not_pressed() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0x9] = 0xA;
    processor.keystate.set_key_status(0xA, false).unwrap();
    assert!(processor.execute_EXA1(0x9).is_ok() && processor.program_counter == 0x15);
}

#[test]
fn test_execute_EXA1_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_EXA1(0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_EXA1_invalid_key_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0x9] = 0x10;
    assert_eq!(processor.execute_EXA1(0x9).unwrap_err(), Error::InvalidKey);
}

#[test]
fn test_execute_FX07() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.delay_timer = 0xF3;
    assert!(processor.execute_FX07(0x7).is_ok() && processor.variable_registers[0x7] == 0xF3);
}

#[test]
fn test_execute_FX07_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_FX07(0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_FX0A_block() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0xC5;
    processor.execute_FX0A(0x3).unwrap();
    assert!(
        processor.status == ProcessorStatus::WaitingForKeypress
            && processor.program_counter == 0xC3
    );
}

#[test]
fn test_execute_FX0A_no_block() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.keystate.set_key_status(0xB, true).unwrap();
    processor.status = ProcessorStatus::Running;
    processor.program_counter = 0xC5;
    processor.execute_FX0A(0x3).unwrap();
    assert!(
        processor.status == ProcessorStatus::Running
            && processor.program_counter == 0xC5
            && processor.variable_registers[0x3] == 0xB
    );
}

#[test]
fn test_execute_FX0A_resume() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.keystate.set_key_status(0xB, true).unwrap();
    processor.status = ProcessorStatus::WaitingForKeypress;
    processor.execute_FX0A(0x3).unwrap();
    assert!(
        processor.status == ProcessorStatus::Running && processor.variable_registers[0x3] == 0xB
    );
}

#[test]
fn test_execute_FX0A_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_FX0A(0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_FX15() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0x7] = 0xF3;
    assert!(processor.execute_FX15(0x7).is_ok() && processor.delay_timer == 0xF3);
}

#[test]
fn test_execute_FX15_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_FX15(0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_FX18() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0x7] = 0xF3;
    assert!(processor.execute_FX18(0x7).is_ok() && processor.sound_timer == 0xF3);
}

#[test]
fn test_execute_FX18_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_FX18(0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_FX1E() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.index_register = 0x3A;
    processor.variable_registers[0xB] = 0xA2;
    assert!(
        processor.execute_FX1E(0xB).is_ok()
            && processor.index_register == 0xDC
            && processor.variable_registers[0xF] == 0
    );
}

#[test]
fn test_execute_FX1E_outside_memory() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.index_register = 0x0FF2;
    processor.variable_registers[0xB] = 0xA2;
    assert!(
        processor.execute_FX1E(0xB).is_ok()
            && processor.index_register == 0x1094
            && processor.variable_registers[0xF] == 1
    );
}

#[test]
fn test_execute_FX1E_overflow_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.index_register = 0x0FF2;
    assert_eq!(
        processor.execute_FX1E(0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_FX1E_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_FX1E(0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_FX29() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0x7] = 0xA;
    assert!(processor.execute_FX29(0x7).is_ok() && processor.index_register == 0x82);
}

#[test]
fn test_execute_FX29_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_FX29(0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_FX29_invalid_register_x_value_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0x7] = 0x10;
    assert_eq!(
        processor.execute_FX29(0x7).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_FX33_one_digit() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.index_register = 0x025A;
    processor.variable_registers[0x3] = 0x06;
    assert!(
        processor.execute_FX33(0x3).is_ok()
            && processor.memory.read_byte(0x025A).unwrap() == 0
            && processor.memory.read_byte(0x025B).unwrap() == 0
            && processor.memory.read_byte(0x025C).unwrap() == 6
    )
}

#[test]
fn test_execute_FX33_two_digits() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.index_register = 0x025A;
    processor.variable_registers[0x3] = 0x5B;
    assert!(
        processor.execute_FX33(0x3).is_ok()
            && processor.memory.read_byte(0x025A).unwrap() == 0
            && processor.memory.read_byte(0x025B).unwrap() == 9
            && processor.memory.read_byte(0x025C).unwrap() == 1
    )
}

#[test]
fn test_execute_FX33_three_digits() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.index_register = 0x025A;
    processor.variable_registers[0x3] = 0x9C;
    assert!(
        processor.execute_FX33(0x3).is_ok()
            && processor.memory.read_byte(0x025A).unwrap() == 1
            && processor.memory.read_byte(0x025B).unwrap() == 5
            && processor.memory.read_byte(0x025C).unwrap() == 6
    )
}

#[test]
fn test_execute_FX33_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_FX33(0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_FX55_one_register() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.index_register = 0x025A;
    processor.variable_registers[0x0] = 0x3C;
    processor.variable_registers[0x1] = 0x12;
    processor.variable_registers[0x2] = 0xF4;
    processor.variable_registers[0x3] = 0x2D;
    processor.variable_registers[0x4] = 0x07;
    assert!(
        processor.execute_FX55(0x0).is_ok()
            && processor.memory.read_byte(0x025A).unwrap() == 0x3C
            && processor.memory.read_byte(0x025B).unwrap() == 0x0
    );
}

#[test]
fn test_execute_FX55_multiple_registers() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.index_register = 0x025A;
    processor.variable_registers[0x0] = 0x3C;
    processor.variable_registers[0x1] = 0x12;
    processor.variable_registers[0x2] = 0xF4;
    processor.variable_registers[0x3] = 0x2D;
    processor.variable_registers[0x4] = 0x07;
    assert!(
        processor.execute_FX55(0x03).is_ok()
            && processor.memory.read_byte(0x025A).unwrap() == 0x3C
            && processor.memory.read_byte(0x025B).unwrap() == 0x12
            && processor.memory.read_byte(0x025C).unwrap() == 0xF4
            && processor.memory.read_byte(0x025D).unwrap() == 0x2D
            && processor.memory.read_byte(0x025E).unwrap() == 0x0
            && processor.index_register == 0x025E
    );
}

#[test]
fn test_execute_FX55_multiple_registers_chip48_mode() {
    let mut processor: Processor = setup_test_processor_chip48();
    processor.index_register = 0x025A;
    processor.variable_registers[0x0] = 0x3C;
    processor.variable_registers[0x1] = 0x12;
    processor.variable_registers[0x2] = 0xF4;
    processor.variable_registers[0x3] = 0x2D;
    processor.variable_registers[0x4] = 0x07;
    assert!(
        processor.execute_FX55(0x03).is_ok()
            && processor.memory.read_byte(0x025A).unwrap() == 0x3C
            && processor.memory.read_byte(0x025B).unwrap() == 0x12
            && processor.memory.read_byte(0x025C).unwrap() == 0xF4
            && processor.memory.read_byte(0x025D).unwrap() == 0x2D
            && processor.memory.read_byte(0x025E).unwrap() == 0x0
            && processor.index_register == 0x025D
    );
}

#[test]
fn test_execute_FX55_multiple_registers_superchip11_mode() {
    let mut processor: Processor = setup_test_processor_superchip11();
    processor.index_register = 0x025A;
    processor.variable_registers[0x0] = 0x3C;
    processor.variable_registers[0x1] = 0x12;
    processor.variable_registers[0x2] = 0xF4;
    processor.variable_registers[0x3] = 0x2D;
    processor.variable_registers[0x4] = 0x07;
    assert!(
        processor.execute_FX55(0x03).is_ok()
            && processor.memory.read_byte(0x025A).unwrap() == 0x3C
            && processor.memory.read_byte(0x025B).unwrap() == 0x12
            && processor.memory.read_byte(0x025C).unwrap() == 0xF4
            && processor.memory.read_byte(0x025D).unwrap() == 0x2D
            && processor.memory.read_byte(0x025E).unwrap() == 0x0
            && processor.index_register == 0x025A
    );
}

#[test]
fn test_execute_FX55_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_FX55(0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}

#[test]
fn test_execute_FX65_one_register() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.index_register = 0x025A;
    processor.memory.write_byte(0x025A, 0x3C).unwrap();
    processor.memory.write_byte(0x025B, 0x12).unwrap();
    processor.memory.write_byte(0x025C, 0xF4).unwrap();
    processor.memory.write_byte(0x025D, 0x2D).unwrap();
    processor.memory.write_byte(0x025E, 0x07).unwrap();
    assert!(
        processor.execute_FX65(0x0).is_ok()
            && processor.variable_registers[0x0] == 0x3C
            && processor.variable_registers[0x1] == 0x0
    );
}

#[test]
fn test_execute_FX65_multiple_registers() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.index_register = 0x025A;
    processor.memory.write_byte(0x025A, 0x3C).unwrap();
    processor.memory.write_byte(0x025B, 0x12).unwrap();
    processor.memory.write_byte(0x025C, 0xF4).unwrap();
    processor.memory.write_byte(0x025D, 0x2D).unwrap();
    processor.memory.write_byte(0x025E, 0x07).unwrap();
    assert!(
        processor.execute_FX65(0x03).is_ok()
            && processor.variable_registers[0x0] == 0x3C
            && processor.variable_registers[0x1] == 0x12
            && processor.variable_registers[0x2] == 0xF4
            && processor.variable_registers[0x3] == 0x2D
            && processor.variable_registers[0x4] == 0x0
            && processor.index_register == 0x025E
    );
}

#[test]
fn test_execute_FX65_multiple_registers_chip48_mode() {
    let mut processor: Processor = setup_test_processor_chip48();
    processor.index_register = 0x025A;
    processor.memory.write_byte(0x025A, 0x3C).unwrap();
    processor.memory.write_byte(0x025B, 0x12).unwrap();
    processor.memory.write_byte(0x025C, 0xF4).unwrap();
    processor.memory.write_byte(0x025D, 0x2D).unwrap();
    processor.memory.write_byte(0x025E, 0x07).unwrap();
    assert!(
        processor.execute_FX65(0x03).is_ok()
            && processor.variable_registers[0x0] == 0x3C
            && processor.variable_registers[0x1] == 0x12
            && processor.variable_registers[0x2] == 0xF4
            && processor.variable_registers[0x3] == 0x2D
            && processor.variable_registers[0x4] == 0x0
            && processor.index_register == 0x025D
    );
}

#[test]
fn test_execute_FX65_multiple_registers_superchip11_mode() {
    let mut processor: Processor = setup_test_processor_superchip11();
    processor.index_register = 0x025A;
    processor.memory.write_byte(0x025A, 0x3C).unwrap();
    processor.memory.write_byte(0x025B, 0x12).unwrap();
    processor.memory.write_byte(0x025C, 0xF4).unwrap();
    processor.memory.write_byte(0x025D, 0x2D).unwrap();
    processor.memory.write_byte(0x025E, 0x07).unwrap();
    assert!(
        processor.execute_FX65(0x03).is_ok()
            && processor.variable_registers[0x0] == 0x3C
            && processor.variable_registers[0x1] == 0x12
            && processor.variable_registers[0x2] == 0xF4
            && processor.variable_registers[0x3] == 0x2D
            && processor.variable_registers[0x4] == 0x0
            && processor.index_register == 0x025A
    );
}

#[test]
fn test_execute_FX65_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_FX65(0x10).unwrap_err(),
        Error::OperandsOutOfBounds
    );
}
