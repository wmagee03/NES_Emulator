#![allow(unused)]

pub use super::opcodes::{
  AddressingMode,
  OpCode,
  OPCODES_MAP,
  STATUS_FLAGS
};

// // CPU Core Registers (Global)
// static mut A: u8 = 0x00; // Accumulator Reg
// static mut X: u8 = 0x00; // X Reg
// static mut Y: u8 = 0x00; // Y Reg
// static mut STKP: u8 = 0x00; // Stack Pointer Reg (points to location on Bus)
// static mut PC: u16 = 0x0000; // Program Counter Reg
// static mut STATUS: u8 = 0x00; // Status Reg

const STACK: u16 = 0x0100;
const STACK_RESET: u8 = 0xFD;



pub struct CPU {
  pub register_a: u8,
  pub register_x: u8,
  pub register_y: u8,
  pub status: u8,
  pub stack_pointer: u8,
  pub program_counter: u16,
  memory: [u8; 0xFFFF]
}

impl CPU {
  // CPU constructor
  pub fn new() -> Self {
    CPU {
      register_a: 0,
      register_x: 0,
      register_y: 0,
      status: 0b0010_0100,
      stack_pointer: STACK_RESET,
      program_counter: 0,
      memory: [0x00; 0xFFFF]
    }
  }

  // Read from Memory
  fn mem_read(&self, addr: u16) -> u8 {
    self.memory[addr as usize]
  }
  // Read from Memory in little endian format
  fn mem_read_u16(&self, memory_pos: u16) -> u16 {
    let lo = self.mem_read(memory_pos) as u16;
    let hi = self.mem_read(memory_pos + 1) as u16;

    (hi << 8) | (lo as u16)
  }

  // Write to Memory
  fn mem_write(&mut self, addr: u16, data: u8) {
    self.memory[addr as usize] = data;
  }
  // Write to Memory in little endian format
  fn mem_write_u16(&mut self, memory_pos: u16, data: u16) {
    let hi = (data >> 8) as u8;
    let lo = (data & 0xff) as u8;

    self.mem_write(memory_pos, lo);
    self.mem_write(memory_pos + 1, hi);
  }

  // Returns true if the given status flag is set
  fn check_status_flag_set(&self, flag: &'static str) -> bool {
    let status_flag = (*STATUS_FLAGS.get(flag)
      .expect(&format!(
        "Bruh this flag doesn't exist: {flag}"
      )));
    status_flag & self.status != 0
  }

  // Helper function that sets status flags
  fn set_status_flag(&mut self, flag: &'static str) {
    let status_flag = (*STATUS_FLAGS.get(flag)
      .expect(&format!(
        "Bruh this flag doesn't exist: {flag}"
      )));
    self.status |= status_flag;
  }

  // Helper function that unsets status flags
  fn unset_status_flag(&mut self, flag: &'static str) {
    let status_flag = (*STATUS_FLAGS.get(flag)
      .expect(&format!(
        "Bruh this flag doesn't exist: {flag}"
      )));
    
    self.status &= !status_flag;
  }

  // Helper function that adds value to register A
  fn add_to_register_a(&mut self, value: u8) {
    let sum = self.register_a as u16
      + value as u16
      + (if self.check_status_flag_set("CARRY") { 1 } else { 0 });

    let should_carry = sum > 0xFF;
    if should_carry {
      self.set_status_flag("CARRY");
    }
    else {
      self.unset_status_flag("CARRY");
    }
  }

  // Determine what register to return based on Addressing Mode
  fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
    match mode {
      AddressingMode::Absolute => {
        self.mem_read_u16(self.program_counter)
      },
      AddressingMode::Absolute_X => {
        let base = self.mem_read_u16(self.program_counter);
        let addr = base.wrapping_add(self.register_x as u16);
        addr
      },
      AddressingMode::Absolute_Y => {
        let base = self.mem_read_u16(self.program_counter);
        let addr = base.wrapping_add(self.register_y as u16);
        addr
      },
      AddressingMode::Immediate => {
        self.program_counter
      },
      AddressingMode::Indirect_X => {
        let base = self.mem_read(self.program_counter);
        let ptr: u8 = (base as u8).wrapping_add(self.register_x);

        let lo = self.mem_read(ptr as u16) as u16;
        let hi = self.mem_read(ptr.wrapping_add(1) as u16) as u16;

        (hi << 8) | lo
      },
      AddressingMode::Indirect_Y => {
        let base = self.mem_read(self.program_counter);
        
        let lo = self.mem_read(base as u16) as u16;
        let hi = self.mem_read((base as u8).wrapping_add(1) as u16) as u16;

        let deref_base = (hi << 8) | lo;
        let derefed = deref_base.wrapping_add(self.register_y as u16);
        derefed
      },
      AddressingMode::ZeroPage => {
        self.mem_read(self.program_counter) as u16
      },
      AddressingMode::ZeroPage_X => {
        let pos = self.mem_read(self.program_counter);
        let addr = pos.wrapping_add(self.register_x) as u16;
        addr
      },
      AddressingMode::ZeroPage_Y => {
        let pos = self.mem_read(self.program_counter);
        let addr = pos.wrapping_add(self.register_y) as u16;
        addr
      },
      AddressingMode::NoneAddressing => {
        panic!("mode {:?} is not supported", mode);
      }
    }
  }

  // Update zero and negative flags based on results of an operation
  fn update_zero_and_negative_flags(&mut self, result: u8) {
    if result == 0 { self.status |= 0b0000_0010; }
    else { self.status &= 0b1111_1101; }

    if result & 0b1000_0000 != 0 { self.status |= 0b1000_0000; }
    else { self.status &= 0b0111_1111; }
  }

  /* Opcode Functions */
  // Force Interrupt
  fn brk(&mut self) {
    self.status |= 0b0001_0100;
  }
  // Increment register X by 1
  fn inx(&mut self) {
    self.register_x = self.register_x.wrapping_add(1);
    self.update_zero_and_negative_flags(self.register_x);
  }

  // Load accumulator
  fn lda(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);
    println!("addr: {}", addr);
    println!("value: {}", value);
    self.register_a = value;
    self.update_zero_and_negative_flags(self.register_a);
  }

  // Store accumulator
  fn sta(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    self.mem_write(addr, self.register_a);
  }

  // Transfer accumulator to register X
  fn tax(&mut self) {
    self.register_x = self.register_a;
    self.update_zero_and_negative_flags(self.register_x);
  }

  // Add memory contents to accumulator with carry bit (set carry if overflow)
  fn adc(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    let current_accumulator_value = self.register_a;
    let carry_bit = if self.status & 0b0000_0001 != 0 { 1 } else { 0 } as u8;

    let result = self.register_a.wrapping_add(value).wrapping_add(carry_bit);
    self.register_a = result;

    if result <= current_accumulator_value { self.status |= 0b0100_0001; }
    else { self.status &= 0b1011_1110; }

    self.update_zero_and_negative_flags(result);
  }

  // Subtract memory contents to accumulator with negated carry bit (clear carry if overflow)
  fn sbc(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.mem_read(addr);

    let current_accumulator_value = self.register_a;
    let carry_bit = if self.status & 0b0000_0001 != 0 { 1 } else { 0 } as u8;

    let result = self.register_a.wrapping_sub(value).wrapping_sub(1 - carry_bit);
    self.register_a = result;

    // if result >= current_accumulator_value { self.status |= }
  }
  /* End of Opcode Functions */

  pub fn load_and_run(&mut self, program: Vec<u8>) {
    self.load(program);
    self.reset();
    self.run();
  }

  pub fn reset(&mut self) {
    self.register_a = 0;
    self.register_x = 0;
    self.register_y = 0;

    self.stack_pointer = STACK_RESET;
    self.status = 0b0010_0100;

    self.program_counter = self.mem_read_u16(0xFFFC);
  }

  pub fn load(&mut self, program: Vec<u8>) {
    self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
    self.mem_write_u16(0xFFFC, 0x8000);
  }

  pub fn run(&mut self) {
    loop {
      let register = self.mem_read(self.program_counter);
      self.program_counter += 1;
      let current_prog_state = self.program_counter;

      let OpCode(
        name,
        byte_count,
        cycle_count,
        mode
      ) = *OPCODES_MAP
        .get(&register)
        .expect(&format!(
          "Ah shit this opcode {:x} don't exist...",
          register
        ));
      // println!("name {}", name);
      // println!("mode {:?}", mode);

      match name {
        "ADC" => { self.adc(&mode); }, // Add with carry
        "AND" => { todo!(); }, // Logical AND
        "ASL" => { todo!(); }, // Arithmetic shift left
        "BCC" => { todo!(); }, // Branch if carry flag is clear
        "BCS" => { todo!(); }, // Branch if carry flag is set
        "BEQ" => { todo!(); }, // Branch if equal
        "BIT" => { todo!(); }, // Bit test
        "BMI" => { todo!(); }, // Branch if negative flag is set
        "BNE" => { todo!(); }, // Branch if not equal
        "BRK" => { self.brk(); }, // Force interrupt
        "BVC" => { todo!(); }, // Branch if overflow flag is clear
        "BVS" => { todo!(); }, // Branch if overflow flag is set
        "CLC" => { todo!(); }, // Clear carry flag
        "CLD" => { todo!(); }, // Clear decimal mode flag
        "CLI" => { todo!(); }, // Clear interrupt disable flag
        "CLV" => { todo!(); }, // Clear overflow flag
        "CMP" => { todo!(); }, // Compare value in register A with value in memory location
        "CPX" => { todo!(); }, // Compare value in register X with value in memory location
        "CPY" => { todo!(); }, // Compare value in register Y with value in memory location
        "DEC" => { todo!(); }, // Decrement value in memory location
        "DEX" => { todo!(); }, // Decrement value in register X
        "DEY" => { todo!(); }, // Decrement value in register Y
        "EOR" => { todo!(); }, // Logical XOR
        "INC" => { todo!(); }, // Increment value in memory location
        "INX" => { self.inx(); }, // Increment value in register X
        "INY" => { todo!(); }, // Increment value in register Y
        "JMP" => { todo!(); }, // Sets program counter to address specified by operand
        "JSR" => { todo!(); }, // Jump to subroutine
        "LDA" => { self.lda(&mode); }, // Load value into register A
        "LDX" => { todo!(); }, // Load value into register X
        "LDY" => { todo!(); }, // Load value into register Y
        "LSR" => { todo!(); }, // Logicial shift right
        "NOP" => { todo!(); }, // No operation to be made
        "ORA" => { todo!(); }, // Logical OR
        "PHA" => { todo!(); }, // Push copy of value in register A onto stack
        "PHP" => { todo!(); }, // Push copy of processor status onto stack
        "PLA" => { todo!(); }, // Pull 8 bit value from stack and loads it into register A
        "PLP" => { todo!(); }, // Pull 8 bit value from stack and sets processor status to be said value
        "ROL" => { todo!(); }, // Shift register A or memory location's value's bits to the left such that the 0th bit is set to be the carry flag's value and then the carry flag's value is set to be the old 7th bit value
        "ROR" => { todo!(); }, // Same as ROR instruction except shift right (7th bit gets set to carry flag value and carry flag value gets set to old 0tth bit value)
        "RTI" => { todo!(); }, // Return from processing routine interrupt, and pull and set processor status flags and program counter from stack
        "RTS" => { todo!(); }, // Return from end of subroutine to routine that called it and pull and set program counter (minus 1) from stack
        "SBC" => { self.sbc(&mode); }, // Subtract contents of memory location from register A with the NOT of the carry flag (if overflow, clear carry bit)
        "SEC" => { todo!(); }, // Set carry flag to 1
        "SED" => { todo!(); }, // Set decimal flag to 1
        "SEI" => { todo!(); }, // Set interrupt disable flag to 1
        "STA" => { self.sta(&mode); }, // Store register A value in memory location
        "STX" => { todo!(); }, // Store register X value in memory location
        "STY" => { todo!(); }, // Store registter Y value in memory location
        "TAX" => { self.tax(); }, // Copy value in register A and store it in register X
        "TAY" => { todo!(); }, // Copy value in register A and store it in register Y
        "TSX" => { todo!(); }, // Copy value in stack register and store it in register X
        "TXA" => { todo!(); }, // Copy value in register X and store it in register A
        "TXS" => { todo!(); }, // Copy value in register X and store it in stack register
        "TYA" => { todo!(); }, // Copy value in register Y and store it in register A
        _ => {
          !todo!()
        }
      }

      if (current_prog_state == self.program_counter) {
        self.program_counter += (byte_count - 1) as u16;
      }
      if self.status & 0b0000_0100 != 0 {
        return;
      }
    }
  }
}


#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_0xa9_lda_immediate_load_data() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
    assert_eq!(cpu.register_a, 5);
    assert!(cpu.status & 0b0000_0010 == 0b00);
    assert!(cpu.status & 0b1000_0000 == 0);
  }

  #[test]
  fn test_0xaa_tax_move_a_to_x() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xaa, 0x00]);
    cpu.reset();
    cpu.register_a = 10;
    cpu.run();

    assert_eq!(cpu.register_x, 10);
  }

  #[test]
  fn test_5_ops_working_together() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
    assert_eq!(cpu.register_x, 0xc1);
  }

  #[test]
  fn test_inx_overflow() {
    let mut cpu = CPU::new();
    cpu.load(vec![0xe8, 0xe8, 0x00]);
    cpu.reset();
    cpu.register_x = 0xff;
    cpu.run();

    assert_eq!(cpu.register_x, 1);
  }

  #[test]
  fn test_lda_from_mem() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x10, 0x55);
    cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

    assert_eq!(cpu.register_a, 0x55);
  }
}