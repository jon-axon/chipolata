use super::*;
use crate::{program::Program, COSMAC_VIP_PROCESSOR_SPEED_HERTZ};
use std::time::{Duration, Instant};

fn get_variable_timing_options() -> Options {
    Options::new(
        COSMAC_VIP_PROCESSOR_SPEED_HERTZ,
        EmulationLevel::Chip8 {
            memory_limit_2k: false,
            variable_cycle_timing: true,
        },
    )
}
fn setup_test_processor_variable_timing() -> Processor {
    let program: Program = Program::default();
    Processor::initialise_and_load(program, get_variable_timing_options()).unwrap()
}

fn setup_test_processor_fixed_timing() -> Processor {
    let program: Program = Program::default();
    Processor::initialise_and_load(program, Options::default()).unwrap()
}

#[test]
#[ignore] // occasionally fails on CI, so ignored by default
fn test_processor_speed_fixed() {
    let processor_speed: u64 = 2000;
    let tolerance_percent: u64 = 3; // permitted difference between specified and calculated
    let program_data: Vec<u8> = vec![0xF0, 0x0A];
    let program: Program = Program::new(program_data);
    let mut options: Options = Options::default();
    options.processor_speed_hertz = processor_speed;
    let mut processor = Processor::initialise_and_load(program, options).unwrap();
    let start_time: Instant = Instant::now();
    let iterations: usize = 1000;
    for _ in 0..iterations {
        processor.execute_cycle().unwrap();
    }
    let execution_duration: u64 = start_time.elapsed().as_micros() as u64;
    let actual_processor_speed: u64 = (iterations as u64) * 1_000_000_u64 / execution_duration;
    let tolerance: u64 = tolerance_percent * processor_speed / 100;
    assert!(
        actual_processor_speed <= processor_speed + tolerance
            && actual_processor_speed >= processor_speed - tolerance
    );
}

#[test]
#[ignore] // occasionally fails on CI, so ignored by default
fn test_processor_speed_variable() {
    const COSMAC_CYCLES_PER_CYCLE: u64 = 19072;
    let tolerance_percent: u64 = 2; // permitted difference between specified and calculated
    let program_data: Vec<u8> = vec![0xF0, 0x0A];
    let program: Program = Program::new(program_data);
    let mut processor =
        Processor::initialise_and_load(program, get_variable_timing_options()).unwrap();
    let start_time: Instant = Instant::now();
    let iterations: usize = 25;
    for _ in 0..iterations {
        processor.execute_cycle().unwrap();
    }
    let execution_duration: u64 = start_time.elapsed().as_micros() as u64;
    let expected_duration: u64 =
        (iterations as u64) * COSMAC_CYCLES_PER_CYCLE * 8_u64 * 1_000_000_u64
            / processor.processor_speed_hertz;

    let tolerance: u64 = tolerance_percent * expected_duration / 100;
    assert!(
        execution_duration <= expected_duration + tolerance
            && execution_duration >= expected_duration - tolerance
    );
}

#[test]
fn test_calculate_cycle_duration_variable() {
    let processor = setup_test_processor_variable_timing();
    let expected_result: u64 = COSMAC_VIP_MACHINE_CYCLES_PER_CYCLE * 100_u64 * 1_000_000_u64
        / processor.processor_speed_hertz;
    assert_eq!(
        processor.calculate_cycle_duration(100),
        Duration::from_micros(expected_result)
    );
}

#[test]
fn test_calculate_cycle_duration_fixed() {
    let processor = setup_test_processor_fixed_timing();
    let expected_result: u64 = 1_000_000_u64 / processor.processor_speed_hertz;
    assert_eq!(
        processor.calculate_cycle_duration(100),
        Duration::from_micros(expected_result)
    );
}

#[test]
fn test_execute_00E0_timing() {
    const EXPECTED_CYCLES: u64 = 64;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_00E0().unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_00EE_timing() {
    const EXPECTED_CYCLES: u64 = 50;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.stack.push(0xB35E).unwrap();
    assert_eq!(processor.execute_00EE().unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_1NNN_timing() {
    const EXPECTED_CYCLES: u64 = 80;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_1NNN(0xEA5).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_2NNN_timing() {
    const EXPECTED_CYCLES: u64 = 94;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_2NNN(0xEA5).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_3XNN_true_timing() {
    const EXPECTED_CYCLES: u64 = 82;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.variable_registers[0x3] = 0xBB;
    assert_eq!(processor.execute_3XNN(0x3, 0xBB).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_3XNN_false_timing() {
    const EXPECTED_CYCLES: u64 = 78;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.variable_registers[0x3] = 0xBA;
    assert_eq!(processor.execute_3XNN(0x3, 0xBB).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_4XNN_true_timing() {
    const EXPECTED_CYCLES: u64 = 82;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.variable_registers[0x3] = 0xBA;
    assert_eq!(processor.execute_4XNN(0x3, 0xBB).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_4XNN_false_timing() {
    const EXPECTED_CYCLES: u64 = 78;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.variable_registers[0x3] = 0xBB;
    assert_eq!(processor.execute_4XNN(0x3, 0xBB).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_5XY0_true_timing() {
    const EXPECTED_CYCLES: u64 = 86;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.variable_registers[0x3] = 0xBA;
    processor.variable_registers[0xD] = 0xBA;
    assert_eq!(processor.execute_5XY0(0x3, 0xD).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_5XY0_false_timing() {
    const EXPECTED_CYCLES: u64 = 82;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.variable_registers[0x3] = 0xBA;
    processor.variable_registers[0xD] = 0xBB;
    assert_eq!(processor.execute_5XY0(0x3, 0xD).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_6XNN_timing() {
    const EXPECTED_CYCLES: u64 = 74;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_6XNN(0xB, 0x2F).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_7XNN_timing() {
    const EXPECTED_CYCLES: u64 = 78;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_7XNN(0x9, 0xE0).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_8XY0_timing() {
    const EXPECTED_CYCLES: u64 = 80;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_8XY0(0xE, 0x7).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_8XY1_timing() {
    const EXPECTED_CYCLES: u64 = 112;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_8XY1(0xE, 0x7).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_8XY2_timing() {
    const EXPECTED_CYCLES: u64 = 112;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_8XY2(0xE, 0x7).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_8XY3_timing() {
    const EXPECTED_CYCLES: u64 = 112;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_8XY3(0xE, 0x7).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_8XY4_timing() {
    const EXPECTED_CYCLES: u64 = 112;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_8XY4(0xE, 0x7).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_8XY5_timing() {
    const EXPECTED_CYCLES: u64 = 112;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_8XY5(0xE, 0x7).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_8XY6_timing() {
    const EXPECTED_CYCLES: u64 = 112;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_8XY6(0xE, 0x7).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_8XY7_timing() {
    const EXPECTED_CYCLES: u64 = 112;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_8XY7(0xE, 0x7).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_8XYE_timing() {
    const EXPECTED_CYCLES: u64 = 112;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_8XYE(0xE, 0x7).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_9XY0_true_timing() {
    const EXPECTED_CYCLES: u64 = 86;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.variable_registers[0x3] = 0xBA;
    processor.variable_registers[0xD] = 0xBB;
    assert_eq!(processor.execute_9XY0(0x3, 0xD).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_9XY0_false_timing() {
    const EXPECTED_CYCLES: u64 = 82;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.variable_registers[0x3] = 0xBA;
    processor.variable_registers[0xD] = 0xBA;
    assert_eq!(processor.execute_9XY0(0x3, 0xD).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_ANNN_timing() {
    const EXPECTED_CYCLES: u64 = 80;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_ANNN(0x0A5).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_BNNN_page_crossed_timing() {
    const EXPECTED_CYCLES: u64 = 92;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.program_counter = 0x113;
    processor.variable_registers[0] = 0x42;
    assert_eq!(processor.execute_BNNN(0x1F0).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_BNNN_page_not_crossed_timing() {
    const EXPECTED_CYCLES: u64 = 90;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.program_counter = 0x113;
    processor.variable_registers[0] = 0x42;
    assert_eq!(processor.execute_BNNN(0x110).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_CXNN_timing() {
    const EXPECTED_CYCLES: u64 = 104;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_CXNN(0x5, 0x0).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_DXYN_timing() {
    const MIN_CYCLES: u64 = 68 + 170;
    const MAX_CYCLES: u64 = MIN_CYCLES + 3812 - 170;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.vblank_status = VBlankStatus::ReadyToDraw; // assume we are ready to proceed
    let cycles: u64 = processor.execute_DXYN(0x3, 0xA, 1).unwrap();
    assert!(cycles >= MIN_CYCLES && cycles <= MAX_CYCLES);
}

#[test]
fn test_execute_EX9E_true_timing() {
    const EXPECTED_CYCLES: u64 = 86;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.variable_registers[0x9] = 0xA;
    processor.keystate.set_key_status(0xA, true).unwrap();
    assert_eq!(processor.execute_EX9E(0x9).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_EX9E_false_timing() {
    const EXPECTED_CYCLES: u64 = 82;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.variable_registers[0x9] = 0xA;
    processor.keystate.set_key_status(0xA, false).unwrap();
    assert_eq!(processor.execute_EX9E(0x9).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_EXA1_true_timing() {
    const EXPECTED_CYCLES: u64 = 86;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.variable_registers[0x9] = 0xA;
    processor.keystate.set_key_status(0xA, false).unwrap();
    assert_eq!(processor.execute_EXA1(0x9).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_EXA1_false_timing() {
    const EXPECTED_CYCLES: u64 = 82;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.variable_registers[0x9] = 0xA;
    processor.keystate.set_key_status(0xA, true).unwrap();
    assert_eq!(processor.execute_EXA1(0x9).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_FX07_timing() {
    const EXPECTED_CYCLES: u64 = 78;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_FX07(0x7).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_FX0A_timing() {
    const EXPECTED_CYCLES: u64 = 19072;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.status = ProcessorStatus::Running;
    assert_eq!(processor.execute_FX0A(0x3).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_FX15_timing() {
    const EXPECTED_CYCLES: u64 = 78;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_FX15(0x7).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_FX18_timing() {
    const EXPECTED_CYCLES: u64 = 78;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_FX18(0x7).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_FX1E_page_crossed_timing() {
    const EXPECTED_CYCLES: u64 = 92;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.index_register = 0xFA;
    processor.variable_registers[0xB] = 0xA2;
    assert_eq!(processor.execute_FX1E(0xB).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_FX1E_page_not_crossed_timing() {
    const EXPECTED_CYCLES: u64 = 84;
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.index_register = 0x3A;
    processor.variable_registers[0xB] = 0xA2;
    assert_eq!(processor.execute_FX1E(0xB).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_FX29_timing() {
    const EXPECTED_CYCLES: u64 = 88;
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_FX29(0x7).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_FX33_timing() {
    const CYCLES_BASE: u64 = 152;
    const CYCLES_INCREMENTAL: u64 = 16;
    const EXPECTED_CYCLES: u64 = CYCLES_BASE + (CYCLES_INCREMENTAL * 9); // 9 = 1 + 5 + 3
    let mut processor: Processor = setup_test_processor_variable_timing();
    processor.variable_registers[0x7] = 0x99; //153 in decimal
    assert_eq!(processor.execute_FX33(0x7).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_FX55_timing() {
    const CYCLES_BASE: u64 = 86;
    const CYCLES_INCREMENTAL: u64 = 14;
    const EXPECTED_CYCLES: u64 = CYCLES_BASE + (CYCLES_INCREMENTAL * 8); // 0x7 registers + 1
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_FX55(0x7).unwrap(), EXPECTED_CYCLES);
}

#[test]
fn test_execute_FX65_timing() {
    const CYCLES_BASE: u64 = 86;
    const CYCLES_INCREMENTAL: u64 = 14;
    const EXPECTED_CYCLES: u64 = CYCLES_BASE + (CYCLES_INCREMENTAL * 8); // 0x7 registers + 1
    let mut processor: Processor = setup_test_processor_variable_timing();
    assert_eq!(processor.execute_FX65(0x7).unwrap(), EXPECTED_CYCLES);
}
