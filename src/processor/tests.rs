use super::*;
use std::collections::HashMap;

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
    options.emulation_level = EmulationLevel::SuperChip11 {
        octo_compatibility_mode: false,
    };
    Processor::initialise_and_load(program, options).unwrap()
}

fn setup_test_processor_superchip11_octo() -> Processor {
    let program: Program = Program::default();
    let mut options: Options = Options::default();
    options.emulation_level = EmulationLevel::SuperChip11 {
        octo_compatibility_mode: true,
    };
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
                processor.low_resolution_font.font_data_size(),
            )
            .unwrap(),
    );
    assert!(processor.load_font_data().is_ok());
    assert_eq!(stored_font, *processor.low_resolution_font.font_data());
}

#[test]
fn test_load_font_data_overflow_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.font_start_address = processor.memory.max_addressable_size() - 0x1;
    assert_eq!(
        processor.load_font_data().unwrap_err(),
        ErrorDetail::MemoryAddressOutOfBounds {
            address: (processor.font_start_address + processor.low_resolution_font.font_data_size())
                as u16
        }
    );
}

#[test]
fn test_load_font_data_superchip11_low_resolution() {
    let mut processor: Processor = setup_test_processor_superchip11();
    let stored_font: Vec<u8> = Vec::from(
        processor
            .memory
            .read_bytes(
                processor.font_start_address,
                processor.low_resolution_font.font_data_size(),
            )
            .unwrap(),
    );
    assert!(processor.load_font_data().is_ok());
    assert_eq!(stored_font, *processor.low_resolution_font.font_data());
}

#[test]
fn test_load_font_data_superchip11_high_resolution() {
    let mut processor: Processor = setup_test_processor_superchip11();
    let stored_font: Vec<u8> = Vec::from(
        processor
            .memory
            .read_bytes(
                processor.high_resolution_font_start_address,
                processor
                    .high_resolution_font
                    .as_ref()
                    .unwrap()
                    .font_data_size(),
            )
            .unwrap(),
    );
    assert!(processor.load_font_data().is_ok());
    assert_eq!(
        stored_font,
        *processor.high_resolution_font.unwrap().font_data()
    );
}

#[test]
fn test_load_font_data_superchip11_high_resolution_overflow_error() {
    let mut processor: Processor = setup_test_processor_superchip11();
    // This leaves space for the low-resolution font, but not the high-resolution one
    processor.font_start_address = 0x1AF;
    assert_eq!(
        processor.load_font_data().unwrap_err(),
        ErrorDetail::MemoryAddressOutOfBounds {
            address: (processor.high_resolution_font_start_address
                + processor
                    .high_resolution_font
                    .as_ref()
                    .unwrap()
                    .font_data_size()) as u16
        }
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
        ErrorDetail::MemoryAddressOutOfBounds {
            address: (processor.program_start_address + processor.program.program_data_size())
                as u16
        }
    );
}

#[test]
fn test_export_state_snapshot_minimal() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.frame_buffer[0][0] = 0xC3;
    let state_snapshot: StateSnapshot =
        processor.export_state_snapshot(StateSnapshotVerbosity::Minimal);
    assert!(
        matches!(state_snapshot, StateSnapshot::MinimalSnapshot { .. })
            && match state_snapshot {
                StateSnapshot::MinimalSnapshot {
                    frame_buffer,
                    status: _,
                    play_sound: _,
                } => frame_buffer[0][0] == 0xC3,
                _ => false,
            }
    );
}

#[test]
fn test_state_snapshot_verbose() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.frame_buffer[0][0] = 0xC3;
    processor.status = ProcessorStatus::Running;
    processor.program_counter = 0x1DF1;
    processor.index_register = 0x3CC2;
    processor.variable_registers[0x4] = 0xB2;
    processor.rpl_registers[0x2] = 0x13;
    processor.delay_timer = 0x3;
    processor.sound_timer = 0x4;
    processor.stack.push(0x30E1).unwrap();
    processor.memory.bytes[0x33] = 0x44;
    processor.cycles = 16473;
    processor.high_resolution_mode = true;
    let state_snapshot: StateSnapshot =
        processor.export_state_snapshot(StateSnapshotVerbosity::Extended);
    assert!(
        matches!(state_snapshot, StateSnapshot::ExtendedSnapshot { .. })
            && match state_snapshot {
                StateSnapshot::ExtendedSnapshot {
                    frame_buffer,
                    status,
                    play_sound: _,
                    program_counter,
                    index_register,
                    variable_registers,
                    rpl_registers,
                    delay_timer,
                    sound_timer,
                    mut stack,
                    memory,
                    cycles,
                    high_resolution_mode,
                    emulation_level,
                } =>
                    frame_buffer[0][0] == 0xC3
                        && status == ProcessorStatus::Running
                        && program_counter == 0x1DF1
                        && index_register == 0x3CC2
                        && variable_registers[0x4] == 0xB2
                        && rpl_registers[0x2] == 0x13
                        && delay_timer == 0x3
                        && sound_timer == 0x4
                        && stack.pop().unwrap() == 0x30E1
                        && memory.bytes[0x33] == 0x44
                        && cycles == 16473
                        && high_resolution_mode == true
                        && emulation_level
                            == EmulationLevel::Chip8 {
                                memory_limit_2k: false,
                                variable_cycle_timing: false
                            },
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
fn test_execute_cycle_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x0BC1;
    let instruction: [u8; 2] = [0xFF, 0xFF]; // invalid instruction
    processor.memory.write_bytes(0x0BC1, &instruction).unwrap();
    assert_eq!(
        processor.execute_cycle().unwrap_err().inner_error,
        ErrorDetail::UnknownInstruction { opcode: 0xFFFF }
    );
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
fn test_decrement_vblankinterrupt() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.vblank_status = VBlankStatus::WaitingForVBlank;
    let mut duration: Duration = Duration::from_micros(VBLANK_INTERVAL_MICROSECONDS as u64 - 100);
    let mut last_time: Instant = Instant::now() - duration;
    processor.last_vblank_interrupt = last_time;
    processor.decrement_timers();
    assert_eq!(processor.vblank_status, VBlankStatus::WaitingForVBlank);
    duration = Duration::from_micros(VBLANK_INTERVAL_MICROSECONDS as u64 + 100);
    last_time = Instant::now() - duration;
    processor.last_vblank_interrupt = last_time;
    processor.decrement_timers();
    assert_eq!(processor.vblank_status, VBlankStatus::ReadyToDraw);
}

#[test]
fn test_execute_004B() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_004B().unwrap_err(),
        ErrorDetail::UnimplementedInstruction { opcode: 0x4B }
    );
}

#[test]
fn test_execute_00CN_superchip11() {
    let mut processor: Processor = setup_test_processor_superchip11();
    // Set the first byte of the first row to be 11111111 (i.e. 0xFF) and the first byte of the
    // 10th row to be 00000000 (i.e. 0x00)
    processor.frame_buffer[0][0] = 0xFF;
    processor.frame_buffer[9][0] = 0x00;
    // When scrolled down by 9 pixels, this first byte of the 10th row should become 111111111 (i.e. 0xFF)
    assert!(
        processor.frame_buffer.scroll_display_down(9).is_ok()
            && processor.frame_buffer[9][0] == 0xFF
    );
}

#[test]
fn test_execute_00CN_chip8_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_00CN(2).unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0x00C2 }
    );
}

#[test]
fn test_execute_00CN_chip48_error() {
    let mut processor: Processor = setup_test_processor_chip48();
    assert_eq!(
        processor.execute_00CN(2).unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0x00C2 }
    );
}

#[test]
fn test_execute_00E0() {
    let mut processor: Processor = setup_test_processor_chip8();
    // Set every pixel to 1
    for column in 0..processor.frame_buffer.get_row_size_bytes() {
        for row in 0..processor.frame_buffer.get_column_size_pixels() {
            processor.frame_buffer[row][column] = 0xFF;
        }
    }
    // Now execute the instruction to clear the display
    processor.execute_00E0().unwrap();
    // Now check that every pixel is 0
    let mut pixel_is_set: bool = false;
    'outer: for column in 0..processor.frame_buffer.get_row_size_bytes() {
        for row in 0..processor.frame_buffer.get_column_size_pixels() {
            if processor.frame_buffer[row][column] > 0x00 {
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
    assert_eq!(
        processor.execute_00EE().unwrap_err(),
        ErrorDetail::PopEmptyStack
    );
}

#[test]
fn test_execute_00FB_superchip11() {
    let mut processor: Processor = setup_test_processor_superchip11();
    let row_size: usize = processor.frame_buffer.get_row_size_bytes();
    // Set the last byte of the first row to be 11110000 (i.e. 0xF0)
    processor.frame_buffer[0][row_size - 1] = 0xF0;
    // When scrolled right by 4 pixels, this last byte should become 00001111 (i.e. 0x0F)
    assert!(
        processor.frame_buffer.scroll_display_right().is_ok()
            && processor.frame_buffer[0][row_size - 1] == 0x0F
    );
}

#[test]
fn test_execute_00FB_chip8_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_00FB().unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0x00FB }
    );
}

#[test]
fn test_execute_00FB_chip48_error() {
    let mut processor: Processor = setup_test_processor_chip48();
    assert_eq!(
        processor.execute_00FB().unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0x00FB }
    );
}

#[test]
fn test_execute_00FC_superchip11() {
    let mut processor: Processor = setup_test_processor_superchip11();
    // Set the first byte of the first row to be 00001111 (i.e. 0x0F)
    processor.frame_buffer[0][0] = 0x0F;
    // When scrolled left by 4 pixels, this first byte should become 11110000 (i.e. 0xF0)
    assert!(
        processor.frame_buffer.scroll_display_left().is_ok()
            && processor.frame_buffer[0][0] == 0xF0
    );
}

#[test]
fn test_execute_00FC_chip8_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_00FC().unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0x00FC }
    );
}

#[test]
fn test_execute_00FC_chip48_error() {
    let mut processor: Processor = setup_test_processor_chip48();
    assert_eq!(
        processor.execute_00FC().unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0x00FC }
    );
}

#[test]
fn test_execute_00FD() {
    let mut processor: Processor = setup_test_processor_superchip11();
    assert!(processor.execute_00FD().is_ok() && processor.status == ProcessorStatus::Completed);
}

#[test]
fn test_execute_00FD_chip8_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_00FD().unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0x00FD }
    );
}

#[test]
fn test_execute_00FD_chip48_error() {
    let mut processor: Processor = setup_test_processor_chip48();
    assert_eq!(
        processor.execute_00FD().unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0x00FD }
    );
}

#[test]
fn test_execute_00FE() {
    let mut processor: Processor = setup_test_processor_superchip11();
    processor.high_resolution_mode = true;
    processor.frame_buffer[0][0] = 0xFF; // set some pixels
    assert!(
        processor.execute_00FE().is_ok()
            && processor.high_resolution_mode == false
            && processor.frame_buffer[0][0] == 0xFF // check the pixels are intact
    );
}

#[test]
fn test_execute_00FE_octo() {
    let mut processor: Processor = setup_test_processor_superchip11_octo();
    processor.high_resolution_mode = true;
    processor.frame_buffer[0][0] = 0xFF; // set some pixels
    assert!(
        processor.execute_00FE().is_ok()
            && processor.high_resolution_mode == false
            && processor.frame_buffer[0][0] == 0x0 // check the pixels are reset
    );
}

#[test]
fn test_execute_00FE_chip8_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_00FE().unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0x00FE }
    );
}

#[test]
fn test_execute_00FE_chip48_error() {
    let mut processor: Processor = setup_test_processor_chip48();
    assert_eq!(
        processor.execute_00FE().unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0x00FE }
    );
}

#[test]
fn test_execute_00FF() {
    let mut processor: Processor = setup_test_processor_superchip11();
    processor.high_resolution_mode = false;
    processor.frame_buffer[0][0] = 0xFF; // set some pixels
    assert!(
        processor.execute_00FF().is_ok()
            && processor.high_resolution_mode == true
            && processor.frame_buffer[0][0] == 0xFF // check the pixels are intact
    );
}

#[test]
fn test_execute_00FF_octo() {
    let mut processor: Processor = setup_test_processor_superchip11_octo();
    processor.high_resolution_mode = false;
    processor.frame_buffer[0][0] = 0xFF; // set some pixels
    assert!(
        processor.execute_00FF().is_ok()
            && processor.high_resolution_mode == true
            && processor.frame_buffer[0][0] == 0x0 // check the pixels are reset
    );
}

#[test]
fn test_execute_00FF_chip8_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_00FF().unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0x00FF }
    );
}

#[test]
fn test_execute_00FF_chip48_error() {
    let mut processor: Processor = setup_test_processor_chip48();
    assert_eq!(
        processor.execute_00FF().unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0x00FF }
    );
}

#[test]
fn test_execute_0NNN() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_0NNN(0x2F5).unwrap_err(),
        ErrorDetail::UnimplementedInstruction { opcode: 0x02F5 }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_3XNN(0x10, 0x2F).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_4XNN(0x10, 0x2F).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    operands.insert("y".to_string(), 0xD);
    assert_eq!(
        processor.execute_5XY0(0x10, 0xD).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_5XY0_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x3);
    operands.insert("y".to_string(), 0x10);
    assert_eq!(
        processor.execute_5XY0(0x3, 0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_6XNN(0x10, 0x2F).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_7XNN(0x10, 0x1E).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    operands.insert("y".to_string(), 0xD);
    assert_eq!(
        processor.execute_8XY0(0x10, 0xD).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_8XY0_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x3);
    operands.insert("y".to_string(), 0x10);
    assert_eq!(
        processor.execute_8XY0(0x3, 0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    operands.insert("y".to_string(), 0xD);
    assert_eq!(
        processor.execute_8XY1(0x10, 0xD).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_8XY1_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x3);
    operands.insert("y".to_string(), 0x10);
    assert_eq!(
        processor.execute_8XY1(0x3, 0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    operands.insert("y".to_string(), 0xD);
    assert_eq!(
        processor.execute_8XY2(0x10, 0xD).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_8XY2_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x3);
    operands.insert("y".to_string(), 0x10);
    assert_eq!(
        processor.execute_8XY2(0x3, 0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x3);
    operands.insert("y".to_string(), 0x10);
    assert_eq!(
        processor.execute_8XY3(0x3, 0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_8XY3_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x3);
    operands.insert("y".to_string(), 0x10);
    assert_eq!(
        processor.execute_8XY3(0x3, 0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
fn test_execute_8XY4_flag_order() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xF] = 0xF2;
    processor.variable_registers[0x7] = 0x16;
    assert!(processor.execute_8XY4(0xF, 0x7).is_ok() && processor.variable_registers[0xF] == 0x01);
}

#[test]
fn test_execute_8XY4_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    operands.insert("y".to_string(), 0xD);
    assert_eq!(
        processor.execute_8XY4(0x10, 0xD).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_8XY4_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x3);
    operands.insert("y".to_string(), 0x10);
    assert_eq!(
        processor.execute_8XY4(0x3, 0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
fn test_execute_8XY5_flag_order() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xF] = 0xF2;
    processor.variable_registers[0x7] = 0x16;
    assert!(processor.execute_8XY5(0xF, 0x7).is_ok() && processor.variable_registers[0xF] == 0x01);
}

#[test]
fn test_execute_8XY5_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    operands.insert("y".to_string(), 0xD);
    assert_eq!(
        processor.execute_8XY5(0x10, 0xD).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_8XY5_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x3);
    operands.insert("y".to_string(), 0x10);
    assert_eq!(
        processor.execute_8XY5(0x3, 0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
fn test_execute_8XY6_flag_order() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xF] = 0x44;
    processor.variable_registers[0x7] = 0xFF;
    assert!(processor.execute_8XY6(0xF, 0x7).is_ok() && processor.variable_registers[0xF] == 0x01);
}

#[test]
fn test_execute_8XY6_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    operands.insert("y".to_string(), 0xD);
    assert_eq!(
        processor.execute_8XY6(0x10, 0xD).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_8XY6_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x3);
    operands.insert("y".to_string(), 0x10);
    assert_eq!(
        processor.execute_8XY6(0x3, 0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
fn test_execute_8XY7_flag_order() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xF] = 0x44;
    processor.variable_registers[0x7] = 0xFF;
    assert!(processor.execute_8XY7(0xF, 0x7).is_ok() && processor.variable_registers[0xF] == 0x01);
}

#[test]
fn test_execute_8XY7_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    operands.insert("y".to_string(), 0xD);
    assert_eq!(
        processor.execute_8XY7(0x10, 0xD).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_8XY7_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x3);
    operands.insert("y".to_string(), 0x10);
    assert_eq!(
        processor.execute_8XY7(0x3, 0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
fn test_execute_8XYE_flag_order() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.variable_registers[0xF] = 0x44;
    processor.variable_registers[0x7] = 0xFF;
    assert!(processor.execute_8XYE(0xF, 0x7).is_ok() && processor.variable_registers[0xF] == 0x01);
}

#[test]
fn test_execute_8XYE_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    operands.insert("y".to_string(), 0xD);
    assert_eq!(
        processor.execute_8XYE(0x10, 0xD).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_8XYE_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x3);
    operands.insert("y".to_string(), 0x10);
    assert_eq!(
        processor.execute_8XYE(0x3, 0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    operands.insert("y".to_string(), 0xD);
    assert_eq!(
        processor.execute_9XY0(0x10, 0xD).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_9XY0_invalid_register_y_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x3);
    operands.insert("y".to_string(), 0x10);
    assert_eq!(
        processor.execute_9XY0(0x3, 0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_CXNN(0x10, 0xD).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

fn fill_row(display: &mut Display, y: usize) {
    for i in &mut display[y] {
        *i = 0xFF;
    }
}

#[test]
fn test_execute_DXYN_Idle_to_Waiting() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.vblank_status = VBlankStatus::Idle;
    processor.last_vblank_interrupt = Instant::now();
    processor.execute_DXYN(0x3, 0xA, 1).unwrap();
    assert_eq!(processor.vblank_status, VBlankStatus::WaitingForVBlank);
}

#[test]
fn test_execute_DXYN_Waiting_to_Waiting() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.vblank_status = VBlankStatus::WaitingForVBlank;
    processor.last_vblank_interrupt = Instant::now();
    processor.execute_DXYN(0x3, 0xA, 1).unwrap();
    assert_eq!(processor.vblank_status, VBlankStatus::WaitingForVBlank);
}

#[test]
fn test_execute_DXYN_Ready_To_Idle() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.index_register = processor.font_start_address as u16;
    let sprite: [u8; 1] = [0xFF]; // create single-byte sprite with all pixels on
    processor
        .memory
        .write_bytes(processor.font_start_address, &sprite)
        .unwrap(); // write sprite to memory at default font location
    processor.variable_registers[0x3] = 0x8; // set V3 to 0 (X coordinate)
    processor.variable_registers[0xA] = 0x1; // set V10 to 1 (Y coordinate)
    processor.vblank_status = VBlankStatus::ReadyToDraw;
    processor.last_vblank_interrupt = Instant::now();
    processor.execute_DXYN(0x3, 0xA, 1).unwrap();
    assert_eq!(processor.vblank_status, VBlankStatus::Idle);
}

#[test]
fn test_execute_DXYN_pixel_turned_off() {
    let mut processor: Processor = setup_test_processor_chip8();
    fill_row(&mut processor.frame_buffer, 0x1); // all display pixels on in second row
    processor.frame_buffer[0x1][0x0] = 0x0; // turn off first byte of pixels only
    processor.variable_registers[0xF] = 0x2; // only possible values later are 0x0 and 0x1
    processor.index_register = processor.font_start_address as u16;
    let sprite: [u8; 1] = [0xFF]; // create single-byte sprite with all pixels on
    processor
        .memory
        .write_bytes(processor.font_start_address, &sprite)
        .unwrap(); // write sprite to memory at default font location
    processor.variable_registers[0x3] = 0x8; // set V3 to 0 (X coordinate)
    processor.variable_registers[0xA] = 0x1; // set V10 to 1 (Y coordinate)
    processor.vblank_status = VBlankStatus::ReadyToDraw;
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
    processor.vblank_status = VBlankStatus::ReadyToDraw;
    processor.execute_DXYN(0x3, 0xA, 1).unwrap();
    assert_eq!(processor.variable_registers[0xF], 0x0); // no pixel will flip if successful
}

#[test]
fn test_duplicate_bits() {
    let (a, b) = Processor::duplicate_bits(0b10110101);
    assert!(a == 0b11001111 && b == 0b00110011);
    let (a, b) = Processor::duplicate_bits(0b11100110);
    assert!(a == 0b11111100 && b == 0b00111100);
    let (a, b) = Processor::duplicate_bits(0b11111111);
    assert!(a == 0b11111111 && b == 0b11111111);
}

#[test]
fn test_execute_DXYN_superchip11_low_res_trivial_sprite() {
    let mut processor: Processor = setup_test_processor_superchip11();
    processor.high_resolution_mode = false;
    processor.variable_registers[0xF] = 0x1; // set Vf to 1
    processor.index_register = processor.font_start_address as u16;
    // Create a trivial 1-bit sprite: 10000000
    let sprite: [u8; 1] = [0x80];
    // Write sprite to memory at default font location
    processor
        .memory
        .write_bytes(processor.font_start_address, &sprite)
        .unwrap();
    // Set V3 to 1 (X coordinate) and V10 to 1 (Y coordinate)
    processor.variable_registers[0x3] = 0x1;
    processor.variable_registers[0xA] = 0x1;
    // The following should execute a low-res SUPER-CHIP draw, which should up-scale the sprite to 2x2
    processor.execute_DXYN(0x3, 0xA, 1).unwrap();
    assert!(
        processor.variable_registers[0xF] == 0x0 // no collisions occurred
            && processor.frame_buffer[0][0] == 0x00 // 00000000
            && processor.frame_buffer[1][0] == 0x00 // 00000000
            && processor.frame_buffer[2][0] == 0x30 // 00110000
            && processor.frame_buffer[3][0] == 0x30 // 00110000
    );
}

#[test]
fn test_execute_DXYN_superchip11_low_res_wide_sprite() {
    let mut processor: Processor = setup_test_processor_superchip11();
    processor.high_resolution_mode = false;
    processor.variable_registers[0xF] = 0x1; // set Vf to 1
    processor.index_register = processor.font_start_address as u16;
    // Create a full-width sprite: 10011011
    let sprite: [u8; 1] = [0x9B];
    // Write sprite to memory at default font location
    processor
        .memory
        .write_bytes(processor.font_start_address, &sprite)
        .unwrap();
    // Set V3 to 2 (X coordinate) and V10 to 2 (Y coordinate)
    processor.variable_registers[0x3] = 0x2;
    processor.variable_registers[0xA] = 0x2;
    // The following should execute a low-res SUPER-CHIP draw, which should up-scale the sprite 2x
    processor.execute_DXYN(0x3, 0xA, 1).unwrap();
    assert!(
        processor.variable_registers[0xF] == 0x0 // no collisions occurred
            && processor.frame_buffer[0][0] == 0x00 // 00000000
            && processor.frame_buffer[0][1] == 0x00 // 00000000
            && processor.frame_buffer[0][2] == 0x00 // 00000000
            && processor.frame_buffer[1][0] == 0x00 // 00000000
            && processor.frame_buffer[1][1] == 0x00 // 00000000
            && processor.frame_buffer[1][2] == 0x00 // 00000000
            && processor.frame_buffer[2][0] == 0x00 // 00000000
            && processor.frame_buffer[2][1] == 0x00 // 00000000
            && processor.frame_buffer[2][2] == 0x00 // 00000000
            && processor.frame_buffer[3][0] == 0x00 // 00000000
            && processor.frame_buffer[3][1] == 0x00 // 00000000
            && processor.frame_buffer[3][2] == 0x00 // 00000000
            && processor.frame_buffer[4][0] == 0x0C // 00001100
            && processor.frame_buffer[4][1] == 0x3C // 00111100
            && processor.frame_buffer[4][2] == 0xF0 // 11110000
            && processor.frame_buffer[5][0] == 0x0C // 00001100
            && processor.frame_buffer[5][1] == 0x3C // 00111100
            && processor.frame_buffer[5][2] == 0xF0 // 11110000
    );
}

#[test]
fn test_execute_DXY0_superchip11() {
    let mut processor: Processor = setup_test_processor_superchip11();
    processor.high_resolution_mode = true;
    let display_rows: usize = processor.frame_buffer.get_column_size_pixels();
    fill_row(&mut processor.frame_buffer, display_rows - 2); // all display pixels on in penultimate row
    fill_row(&mut processor.frame_buffer, display_rows - 1); // all display pixels on in final row
    processor.variable_registers[0xF] = 0x0; // set Vf to 0
    processor.index_register = processor.font_start_address as u16;
    let sprite: [u8; 32] = [0xFF; 32]; // create 32-byte sprite with all pixels on
    processor
        .memory
        .write_bytes(processor.font_start_address, &sprite)
        .unwrap(); // write sprite to memory at default font location
    processor.variable_registers[0x3] = 0x8; // set V3 to 0 (X coordinate)
                                             // set V10 (Y coord) to 3rd final row // execute a DXY0 instruction
    processor.variable_registers[0xA] = (display_rows - 3) as u8;
    // This operation should cause pixel collison on two rows (penultimate and final but not third last)
    // and should also cause clipping of 13 rows (16-byte high sprite with only 3 rows on-screen)
    // however clipping is currently disabled by design, so 0 for this component
    assert!(
        processor.execute_DXYN(0x3, 0xA, 0).unwrap() == 0
            && processor.variable_registers[0xF] == 0x2 // 2 (if not disabled would be 2 + 13 = 15 = 0xF)
    );
}

#[test]
fn test_execute_DXYN_invalid_x_register_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    operands.insert("y".to_string(), 0x2);
    operands.insert("n".to_string(), 0x5);
    assert_eq!(
        processor.execute_DXYN(0x10, 0x2, 0x5).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_DXYN_invalid_y_register_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x2);
    operands.insert("y".to_string(), 0x10);
    operands.insert("n".to_string(), 0x5);
    assert_eq!(
        processor.execute_DXYN(0x2, 0x10, 0x5).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_DXYN_invalid_n_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x2);
    operands.insert("y".to_string(), 0x5);
    operands.insert("n".to_string(), 0x10);
    assert_eq!(
        processor.execute_DXYN(0x2, 0x5, 0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_EX9E(0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_EX9E_invalid_key_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0x9] = 0x10;
    assert_eq!(
        processor.execute_EX9E(0x9).unwrap_err(),
        ErrorDetail::InvalidKey { key: 0x10 }
    );
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_EXA1(0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_EXA1_invalid_key_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0x13;
    processor.variable_registers[0x9] = 0x10;
    assert_eq!(
        processor.execute_EXA1(0x9).unwrap_err(),
        ErrorDetail::InvalidKey { key: 0x10 }
    );
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_FX07(0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_FX0A_block() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.program_counter = 0xC5;
    processor.status = ProcessorStatus::Running;
    processor.execute_FX0A(0x3).unwrap();
    assert!(
        processor.status == ProcessorStatus::WaitingForKeypress
            && processor.program_counter == 0xC3
    );
}

#[test]
fn test_execute_FX0A_press_and_release() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.status = ProcessorStatus::Running;
    processor.program_counter = 0xC5;
    processor.execute_FX0A(0x3).unwrap();
    assert_eq!(processor.status, ProcessorStatus::WaitingForKeypress);
    processor.keystate.set_key_status(0xB, true).unwrap(); // Simulate key press
    processor.execute_FX0A(0x3).unwrap();
    assert_eq!(processor.status, ProcessorStatus::WaitingForKeypress);
    processor.keystate.set_key_status(0xB, false).unwrap(); // Simulate key release
    processor.execute_FX0A(0x3).unwrap();
    assert!(
        processor.status == ProcessorStatus::Running
            && processor.program_counter == 0xC1
            && processor.variable_registers[0x3] == 0xB
    );
}

#[test]
fn test_execute_FX0A_press_and_release_multiple() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.status = ProcessorStatus::Running;
    processor.program_counter = 0xC5;
    processor.execute_FX0A(0x3).unwrap();
    assert_eq!(processor.status, ProcessorStatus::WaitingForKeypress);
    processor.keystate.set_key_status(0xA, true).unwrap(); // Simulate key press
    processor.keystate.set_key_status(0xB, true).unwrap(); // Simulate key press
    processor.execute_FX0A(0x3).unwrap();
    assert_eq!(processor.status, ProcessorStatus::WaitingForKeypress);
    processor.keystate.set_key_status(0xB, false).unwrap(); // Simulate key release
    processor.execute_FX0A(0x3).unwrap();
    assert!(
        processor.status == ProcessorStatus::Running
            && processor.program_counter == 0xC1
            && processor.variable_registers[0x3] == 0xB
    );
}

#[test]
fn test_execute_FX0A_press_and_release_existing_keys() {
    let mut processor: Processor = setup_test_processor_chip8();
    processor.status = ProcessorStatus::Running;
    processor.set_key_status(0x5, true).unwrap();
    processor.set_key_status(0x9, true).unwrap();
    processor.program_counter = 0xC5;
    processor.execute_FX0A(0x3).unwrap();
    assert_eq!(processor.status, ProcessorStatus::WaitingForKeypress);
    processor.keystate.set_key_status(0xB, true).unwrap(); // Simulate key press
    processor.execute_FX0A(0x3).unwrap();
    assert_eq!(processor.status, ProcessorStatus::WaitingForKeypress);
    processor.keystate.set_key_status(0xB, false).unwrap(); // Simulate key release
    processor.execute_FX0A(0x3).unwrap();
    assert!(
        processor.status == ProcessorStatus::Running
            && processor.program_counter == 0xC1
            && processor.variable_registers[0x3] == 0xB
    );
}

#[test]
fn test_execute_FX0A_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_FX0A(0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_FX15(0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_FX18(0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    processor.index_register = 0x0FF2;
    assert_eq!(
        processor.execute_FX1E(0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_FX1E_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_FX1E(0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_FX29(0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_FX29_invalid_register_x_value_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("character".to_string(), 0x10);
    processor.variable_registers[0x7] = 0x10;
    assert_eq!(
        processor.execute_FX29(0x7).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_FX30() {
    let mut processor: Processor = setup_test_processor_superchip11();
    processor.variable_registers[0x7] = 0x02;
    assert!(processor.execute_FX30(0x7).is_ok() && processor.index_register == 0xB4);
}

#[test]
fn test_execute_FX30_octo() {
    let mut processor: Processor = setup_test_processor_superchip11_octo();
    processor.variable_registers[0x7] = 0x0A;
    assert!(processor.execute_FX30(0x7).is_ok() && processor.index_register == 0x104);
}

#[test]
fn test_execute_FX30_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_superchip11();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_FX30(0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_FX30_invalid_register_x_value_error() {
    let mut processor: Processor = setup_test_processor_superchip11();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("character".to_string(), 0x0A);
    processor.variable_registers[0x7] = 0x0A;
    assert_eq!(
        processor.execute_FX30(0x7).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_FX30_chip8_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_FX30(0x3).unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0xF330 }
    );
}

#[test]
fn test_execute_FX30_chip48_error() {
    let mut processor: Processor = setup_test_processor_chip48();
    assert_eq!(
        processor.execute_FX30(0x3).unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0xF330 }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_FX33(0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
            && processor.index_register == 0x025A
    );
}

#[test]
fn test_execute_FX55_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_FX55(0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
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
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x10);
    assert_eq!(
        processor.execute_FX65(0x10).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_FX75_one_register() {
    let mut processor: Processor = setup_test_processor_superchip11();
    processor.variable_registers[0x0] = 0x3C;
    processor.variable_registers[0x1] = 0x12;
    processor.variable_registers[0x2] = 0xF4;
    processor.variable_registers[0x3] = 0x2D;
    processor.variable_registers[0x4] = 0x07;
    assert!(
        processor.execute_FX75(0x0).is_ok()
            && processor.rpl_registers[0x0] == 0x3C
            && processor.rpl_registers[0x1] == 0x0
    );
}

#[test]
fn test_execute_FX75_multiple_registers() {
    let mut processor: Processor = setup_test_processor_superchip11();
    processor.variable_registers[0x0] = 0x3C;
    processor.variable_registers[0x1] = 0x12;
    processor.variable_registers[0x2] = 0xF4;
    processor.variable_registers[0x3] = 0x2D;
    processor.variable_registers[0x4] = 0x07;
    assert!(
        processor.execute_FX75(0x03).is_ok()
            && processor.rpl_registers[0x0] == 0x3C
            && processor.rpl_registers[0x1] == 0x12
            && processor.rpl_registers[0x2] == 0xF4
            && processor.rpl_registers[0x3] == 0x2D
            && processor.rpl_registers[0x4] == 0x0
    );
}

#[test]
fn test_execute_FX75_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_superchip11();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x8);
    assert_eq!(
        processor.execute_FX75(0x8).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_FX75_chip8_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_FX75(0x3).unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0xF375 }
    );
}

#[test]
fn test_execute_FX75_chip48_error() {
    let mut processor: Processor = setup_test_processor_chip48();
    assert_eq!(
        processor.execute_FX75(0x3).unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0xF375 }
    );
}

#[test]
fn test_execute_FX85_one_register() {
    let mut processor: Processor = setup_test_processor_superchip11();
    processor.rpl_registers[0x0] = 0x3C;
    processor.rpl_registers[0x1] = 0x12;
    processor.rpl_registers[0x2] = 0xF4;
    processor.rpl_registers[0x3] = 0x2D;
    processor.rpl_registers[0x4] = 0x07;
    assert!(
        processor.execute_FX85(0x0).is_ok()
            && processor.variable_registers[0x0] == 0x3C
            && processor.variable_registers[0x1] == 0x0
    );
}

#[test]
fn test_execute_FX85_multiple_registers() {
    let mut processor: Processor = setup_test_processor_superchip11();
    processor.rpl_registers[0x0] = 0x3C;
    processor.rpl_registers[0x1] = 0x12;
    processor.rpl_registers[0x2] = 0xF4;
    processor.rpl_registers[0x3] = 0x2D;
    processor.rpl_registers[0x4] = 0x07;
    assert!(
        processor.execute_FX85(0x03).is_ok()
            && processor.variable_registers[0x0] == 0x3C
            && processor.variable_registers[0x1] == 0x12
            && processor.variable_registers[0x2] == 0xF4
            && processor.variable_registers[0x3] == 0x2D
            && processor.variable_registers[0x4] == 0x0
    );
}

#[test]
fn test_execute_FX85_invalid_register_x_error() {
    let mut processor: Processor = setup_test_processor_superchip11();
    let mut operands: HashMap<String, usize> = HashMap::new();
    operands.insert("x".to_string(), 0x8);
    assert_eq!(
        processor.execute_FX85(0x8).unwrap_err(),
        ErrorDetail::OperandsOutOfBounds { operands: operands }
    );
}

#[test]
fn test_execute_FX85_chip8_error() {
    let mut processor: Processor = setup_test_processor_chip8();
    assert_eq!(
        processor.execute_FX85(0x3).unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0xF385 }
    );
}

#[test]
fn test_execute_FX85_chip48_error() {
    let mut processor: Processor = setup_test_processor_chip48();
    assert_eq!(
        processor.execute_FX85(0x3).unwrap_err(),
        ErrorDetail::UnknownInstruction { opcode: 0xF385 }
    );
}
