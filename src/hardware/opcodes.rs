#![allow(unused)]
/* The following code defines the opCodes for each instruction */

#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
   Immediate,
   ZeroPage,
   ZeroPage_X,
   ZeroPage_Y,
   Absolute,
   Absolute_X,
   Absolute_Y,
   Indirect_X,
   Indirect_Y,
   NoneAddressing,
}

// type aliases for readability
pub type ByteCount = u8;
pub type CycleCount = u8;

// OpCode args = name, byte count, cycle count, addressing mode
pub struct OpCode (
  pub &'static str,
  pub ByteCount,
  pub CycleCount,
  pub AddressingMode
);
impl From<u8> for OpCode {
  fn from(value: u8) -> Self {
    match value {
      // Load Accumulator
      0xAD => OpCode("LDA", 3, 4, AddressingMode::Absolute),
      0xBD => OpCode("LDA", 3, 4, AddressingMode::Absolute_X),
      0xB9 => OpCode("LDA", 3, 4, AddressingMode::Absolute_Y),
      0xA9 => OpCode("LDA", 2, 2, AddressingMode::Immediate),
      0xA1 => OpCode("LDA", 2, 6, AddressingMode::Indirect_X),
      0xB1 => OpCode("LDA", 2, 5, AddressingMode::Indirect_Y),
      0xA5 => OpCode("LDA", 2, 3, AddressingMode::ZeroPage),
      0xB5 => OpCode("LDA", 2, 4, AddressingMode::ZeroPage_X),
      // Store Accumulator
      0x8D => OpCode("STA", 3, 4, AddressingMode::Absolute),
      0x9D => OpCode("STA", 3, 5, AddressingMode::Absolute_X),
      0x99 => OpCode("STA", 3, 5, AddressingMode::Absolute_Y),
      0x81 => OpCode("STA", 2, 6, AddressingMode::Indirect_X),
      0x91 => OpCode("STA", 2, 6, AddressingMode::Indirect_Y),
      0x85 => OpCode("STA", 2, 3, AddressingMode::ZeroPage),
      0x95 => OpCode("STA", 2, 4, AddressingMode::ZeroPage_X),
      // Add with Carry
      0x6D => OpCode("ADC", 3, 4, AddressingMode::Absolute),
      0x7D => OpCode("ADC", 3, 4, AddressingMode::Absolute_X), // +1 cycle if page crossed
      0x79 => OpCode("ADC", 3, 4, AddressingMode::Absolute_Y), // +1 cycle if page crossed
      0x69 => OpCode("ADC", 2, 2, AddressingMode::Immediate),
      0x61 => OpCode("ADC", 2, 6, AddressingMode::Indirect_X),
      0x71 => OpCode("ADC", 2, 5, AddressingMode::Indirect_Y), // +1 cycle if page crossed
      0x65 => OpCode("ADC", 2, 3, AddressingMode::ZeroPage),
      0x75 => OpCode("ADC", 2, 4, AddressingMode::ZeroPage_X),
      // Others
      0x00 => OpCode("BRK", 1, 7, AddressingMode::NoneAddressing),
      0xAA => OpCode("TAX", 1, 2, AddressingMode::NoneAddressing),
      0xE8 => OpCode("INX", 1, 2, AddressingMode::NoneAddressing),
      // PANIC!!
      _ => panic!("no operation exists for the given value {:?}", value)
    }
  }
}

use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
  pub static ref OPCODES_MAP: HashMap<u8, OpCode> = {
    let valid_codes: Vec<u8> = vec![
      0xAD, 0xBD, 0xB9, 0xA9, 0xA1, 0xB1, 0xA5, 0xB5, // LDA
      0x8D, 0x9D, 0x99, 0x81, 0x91, 0x85, 0x95, // STA
      0x6D, 0x7D, 0x79, 0x69, 0x61, 0x71, 0x65, 0x75, // ADC
      0x00, // BRK
      0xAA, // TAX
      0xE8, // INX
    ];
    let mut map = HashMap::new();
    for code in valid_codes {
      map.insert(code, OpCode::from(code));
    }

    map
  };
  
  pub static ref STATUS_FLAGS: HashMap<&'static str, u8> = HashMap::from([
    ("CARRY", 0b0000_0001),
    ("ZERO", 0b0000_0010),
    ("INTERRUPT_DISABLE", 0b0000_0100),
    ("DECIMAL_MODE", 0b0000_1000),
    ("BREAK", 0b0001_0000),
    ("BREAK2", 0b0010_0000),
    ("OVERFLOW", 0b0100_0000),
    ("NEGATIVE", 0b1000_0000)
  ]);
}
