// 6502 implementation
/// # Status Register (P) http://wiki.nesdev.com/w/index.php/Status_flags
///
///  7 6 5 4 3 2 1 0
///  N V _ B D I Z C
///  | |   | | | | +--- Carry Flag
///  | |   | | | +----- Zero Flag
///  | |   | | +------- Interrupt Disable
///  | |   | +--------- Decimal Mode (not used on NES)
///  | |   +----------- Break Command
///  | +--------------- Overflow Flag
///  +----------------- Negative Flag
///
pub struct CPU {
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub status: u8,
    pub pc: u16,
    pub stk_ptr: u8,
    memory: [u8; 0xFFFF],
}
const STK: u16 = 0x0100;
const STK_RESET: u8 = 0xfd;

trait Mem {
    fn mem_read(&self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }
    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}

#[derive(Debug)]
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

impl Mem for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
}

impl CPU {
    //constructor i.e. associated function
    pub fn new() -> Self {
        CPU {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            status: 0,
            pc: 0,
            stk_ptr: STK_RESET,
            memory: [0; 0xFFFF],
        }
    }

    pub fn reset(&mut self) {
        self.reg_x = 0;
        self.reg_a = 0;
        self.status = 0;
        self.stk_ptr = STK_RESET;
        self.pc = self.mem_read_u16(0xFFFC);
    }
    fn stk_push(&mut self, data: u8) {
        self.mem_write((STK as u16) + self.stk_ptr as u16, data);
        self.stk_ptr = self.stk_ptr.wrapping_sub(1);
    }
    fn stk_push_u16(&mut self, data: u8) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.stk_push(hi);
        self.stk_push(lo);
    }
    fn stk_pop(&mut self) -> u8 {
        self.stk_ptr = self.stk_ptr.wrapping_add(1);
        self.mem_read((STK as u16) + self.stk_ptr as u16)
    }
    fn stk_pop_u16(&mut self) -> u16 {
        let lo = self.stk_pop() as u16;
        let hi = self.stk_pop() as u16;
        hi << 8 | lo
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.mem_read(addr);
        self.reg_a = val;
        self.update_zero_and_neg_flag(self.reg_a);
    }
    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.mem_read(addr);
        self.reg_x = val;
        self.update_zero_and_neg_flag(self.reg_x);
    }
    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.mem_read(addr);
        self.reg_y = val;
        self.update_zero_and_neg_flag(self.reg_y);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.reg_a = self.reg_a & data;
        self.update_zero_and_neg_flag(self.reg_a);
    }
    fn asl(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        self.update_carry_flag_asl(data);
        data = data << 1;
        self.mem_write(addr, data);
        self.update_zero_and_neg_flag(data);
        data
    }
    fn lsr(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        self.update_carry_flag_lsr(data);
        data = data >> 1;
        self.mem_write(addr, data);
        self.update_zero_and_neg_flag(data);
        data
    }
    fn branch(&mut self, condition: bool) {
        if condition {
            let jump: i8 = self.mem_read(self.pc) as i8;
            let jump_addr = self.pc.wrapping_add(1).wrapping_add(jump as u16);
            self.pc = jump_addr;
        }
    }
    fn compare(&mut self, mode: &AddressingMode, cmp_with: u8) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        if data <= cmp_with {
            self.status = self.status | 0b0000_0001;
        } else {
            self.status = self.status & 0b1111_1110;
        }
        self.update_zero_and_neg_flag(cmp_with.wrapping_sub(data));
    }

    fn pla(&mut self) {
        let data = self.stk_pop();
        self.reg_a = data;
        self.update_zero_and_neg_flag(self.reg_a);
    }
    fn plp(&mut self) {
        self.stk_ptr = self.stk_ptr.wrapping_add(1);
        let value = self.mem_read(STK + self.stk_ptr as u16);
        self.status = value | 0b0010_0000;
    }

    fn inc(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        data = data.wrapping_add(1);
        self.mem_write(addr, data);
        self.update_zero_and_neg_flag(data);
        data
    }

    fn incx(&mut self) {
        self.reg_x = self.reg_x.wrapping_add(1);
        self.update_zero_and_neg_flag(self.reg_x);
    }

    fn incy(&mut self) {
        self.reg_y = self.reg_y.wrapping_add(1);
        self.update_zero_and_neg_flag(self.reg_y);
    }

    fn dec(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        data = data.wrapping_sub(1);
        self.mem_write(addr, data);
        self.update_zero_and_neg_flag(data);
        data
    }
    fn dex(&mut self) {
        self.reg_x = self.reg_x.wrapping_sub(1);
        self.update_zero_and_neg_flag(self.reg_x);
    }
    fn dey(&mut self) {
        self.reg_y = self.reg_y.wrapping_sub(1);
        self.update_zero_and_neg_flag(self.reg_y);
    }
    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.reg_a = data ^ self.reg_a;
        self.update_zero_and_neg_flag(self.reg_a);
    }
    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.reg_a = data | self.reg_a;
        self.update_zero_and_neg_flag(self.reg_a);
    }

    fn rol_acc(&mut self) {
        let carry_in = if self.get_flag_carry() { 1 } else { 0 };
        let old_val = self.reg_a;
        let new_val = (old_val << 1) | carry_in;
        self.set_flag_carry(old_val & 0x80 != 0);
        self.reg_a = new_val;
        self.update_zero_and_neg_flag(self.reg_a);
    }
    fn rol_mem(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let carry_in = if self.get_flag_carry() { 1 } else { 0 };
        let old_val = self.mem_read(addr);
        let new_val = (old_val << 1) | carry_in;
        self.set_flag_carry(old_val & 0x80 != 0);
        self.mem_write(addr, new_val);
        self.update_zero_and_neg_flag(new_val);
    }
    fn ror_acc(&mut self) {
        let carry_in = if self.get_flag_carry() { 0x80 } else { 0 };
        let old_val = self.reg_a;
        let new_val = (old_val >> 1) | carry_in;
        self.set_flag_carry(old_val & 0x01 != 0);
        self.reg_a = new_val;
        self.update_zero_and_neg_flag(self.reg_a);
    }
    fn ror_mem(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let carry_in = if self.get_flag_carry() { 0x80 } else { 0 };
        let old_val = self.mem_read(addr);
        let new_val = (old_val >> 1) | carry_in;
        self.set_flag_carry(old_val & 0x01 != 0);
        self.mem_write(addr, new_val);
        self.update_zero_and_neg_flag(new_val);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.reg_a);
    }

    fn tax(&mut self) {
        self.reg_x = self.reg_a;
        self.update_zero_and_neg_flag(self.reg_x);
    }

    fn get_flag_carry(&self) -> bool {
        self.status & 0b0000_0001 != 0
    }

    fn set_flag_carry(&mut self, val: bool) {
        if val {
            self.status |= 0b0000_0001;
        } else {
            self.status &= !0b0000_0001;
        }
    }
    fn update_zero_and_neg_flag(&mut self, res: u8) {
        if res == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }
        if res & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }

    fn update_carry_flag_asl(&mut self, res: u8) {
        if res >> 7 == 1 {
            self.status = self.status | 0b0000_0001;
        } else {
            self.status = self.status & 0b1111_1110;
        }
    }
    fn update_carry_flag_lsr(&mut self, res: u8) {
        if res & 1 == 1 {
            self.status = self.status | 0b0000_0001;
        } else {
            self.status = self.status & 0b1111_1110;
        }
    }

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.pc,
            AddressingMode::ZeroPage => self.mem_read(self.pc) as u16,
            AddressingMode::Absolute => self.mem_read_u16(self.pc),
            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.pc);
                let addr = pos.wrapping_add(self.reg_x) as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.pc);
                let addr = pos.wrapping_add(self.reg_y) as u16;
                addr
            }
            AddressingMode::Absolute_X => {
                let pos = self.mem_read_u16(self.pc);
                let addr = pos.wrapping_add(self.reg_x as u16);
                addr
            }
            AddressingMode::Absolute_Y => {
                let pos = self.mem_read_u16(self.pc);
                let addr = pos.wrapping_add(self.reg_y as u16);
                addr
            }
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.pc);
                let ptr: u8 = (base as u8).wrapping_add(self.reg_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.pc);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.reg_y as u16);
                deref
            }
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }
    pub fn run(&mut self) {
        loop {
            let opcode = self.mem_read(self.pc);
            self.pc += 1;
            match opcode {
                // LDA
                0xa9 => {
                    self.lda(&AddressingMode::Immediate);
                    self.pc += 1;
                }
                0xa5 => {
                    self.lda(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }
                0xb5 => {
                    self.lda(&AddressingMode::ZeroPage_X);
                    self.pc += 1;
                }
                0xad => {
                    self.lda(&AddressingMode::Absolute);
                    self.pc += 2;
                }
                0xbd => {
                    self.lda(&AddressingMode::Absolute_X);
                    self.pc += 2;
                }
                0xb9 => {
                    self.lda(&AddressingMode::Absolute_Y);
                    self.pc += 2;
                }
                0xa1 => {
                    self.lda(&AddressingMode::Indirect_X);
                    self.pc += 1;
                }
                0xb1 => {
                    self.lda(&AddressingMode::Indirect_Y);
                    self.pc += 1;
                }

                // STA
                0x85 => {
                    self.sta(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }
                0x95 => {
                    self.sta(&AddressingMode::ZeroPage_X);
                    self.pc += 1;
                }
                0x8D => {
                    self.sta(&AddressingMode::Absolute);
                    self.pc += 2;
                }
                0x9D => {
                    self.sta(&AddressingMode::Absolute_X);
                    self.pc += 2;
                }
                0x99 => {
                    self.sta(&AddressingMode::Absolute_Y);
                    self.pc += 2;
                }
                0x81 => {
                    self.sta(&AddressingMode::Indirect_X);
                    self.pc += 1;
                }
                0x91 => {
                    self.sta(&AddressingMode::Indirect_Y);
                    self.pc += 1;
                }

                // AND
                0x29 => {
                    self.and(&AddressingMode::Immediate);
                    self.pc += 1;
                }
                0x25 => {
                    self.and(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }
                0x35 => {
                    self.and(&AddressingMode::ZeroPage_X);
                    self.pc += 1;
                }
                0x2d => {
                    self.and(&AddressingMode::Absolute);
                    self.pc += 2;
                }
                0x3d => {
                    self.and(&AddressingMode::Absolute_X);
                    self.pc += 2;
                }
                0x39 => {
                    self.and(&AddressingMode::Absolute_Y);
                    self.pc += 2;
                }
                0x21 => {
                    self.and(&AddressingMode::Indirect_X);
                    self.pc += 1;
                }
                0x31 => {
                    self.and(&AddressingMode::Indirect_Y);
                    self.pc += 1;
                }

                0xaa => self.tax(),
                0xe8 => self.inx(),

                0x00 => {
                    return;
                }
                _ => todo!("implement this"),
            }
        }
    }
    pub fn interpret(&mut self, program: Vec<u8>) {
        // todo!("implement this");
        self.pc = 0;
        loop {
            let opcode = program[self.pc as usize];
            self.pc += 1;
            match opcode {
                0xa9 => {
                    let param = program[self.pc as usize];
                    self.pc += 1;
                    // self.lda(param);
                }
                0xaa => self.tax(),
                0xe8 => self.inx(),

                0x00 => {
                    return;
                }
                _ => todo!("implement this"),
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
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.reg_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }
    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);
        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);
        assert_eq!(cpu.reg_a, 0x55);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.reg_a = 10;
        cpu.interpret(vec![0xaa, 0x00]);

        assert_eq!(cpu.reg_x, 10)
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.reg_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.reg_x = 0xff;
        cpu.interpret(vec![0xe8, 0xe8, 0x00]);
        assert_eq!(cpu.reg_x, 1)
    }
}
