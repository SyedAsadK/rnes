use crate::comp::bus::Bus;
use bitflags::bitflags;

bitflags! {
    /// Represents the 6502 Processor Status (P) register.
    // #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CpuFlags: u8 {
        const CARRY             = 0b0000_0001; // C
        const ZERO              = 0b0000_0010; // Z
        const INTERRUPT_DISABLE = 0b0000_0100; // I
        const DECIMAL_MODE      = 0b0000_1000; // D (Not used on NES)
        const BREAK             = 0b0001_0000; // B
        const BREAK2            = 0b00100000;
        const UNUSED            = 0b0010_0000; // _ (Always 1)
        const OVERFLOW          = 0b0100_0000; // V
        const NEGATIVE          = 0b1000_0000; // N
    }
}

mod interrupt {
    #[derive(PartialEq, Eq)]
    pub enum InterruptType {
        NMI,
        BRK,
    }

    #[derive(PartialEq, Eq)]
    pub(super) struct Interrupt {
        pub(super) itype: InterruptType,
        pub(super) vector_addr: u16,
        pub(super) b_flag_mask: u8,
        pub(super) cpu_cycles: u8,
    }

    pub(super) const NMI: Interrupt = Interrupt {
        itype: InterruptType::NMI,
        vector_addr: 0xfffA,
        b_flag_mask: 0b00100000,
        cpu_cycles: 2,
    };

    pub(super) const BRK: Interrupt = Interrupt {
        itype: InterruptType::BRK,
        vector_addr: 0xfffe,
        b_flag_mask: 0b00110000,
        cpu_cycles: 1,
    };
}
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
pub static OPCODE_CYCLES: [u8; 256] = [
    7, 6, 2, 2, 2, 3, 5, 2, 3, 2, 2, 2, 4, 4, 6, 2, // 0x00
    2, 5, 2, 2, 2, 4, 6, 2, 4, 4, 2, 2, 4, 6, 7, 2, // 0x10
    6, 6, 2, 2, 3, 3, 5, 2, 4, 2, 2, 2, 4, 4, 6, 2, // 0x20
    2, 5, 2, 2, 2, 4, 6, 2, 4, 4, 2, 2, 4, 6, 7, 2, // 0x30
    6, 6, 2, 2, 2, 3, 5, 2, 3, 2, 2, 2, 3, 4, 6, 2, // 0x40
    2, 5, 2, 2, 2, 4, 6, 2, 4, 4, 2, 2, 4, 6, 7, 2, // 0x50
    6, 6, 2, 2, 2, 3, 5, 2, 4, 2, 2, 2, 5, 4, 6, 2, // 0x60
    2, 5, 2, 2, 2, 4, 6, 2, 4, 4, 2, 2, 4, 6, 7, 2, // 0x70
    2, 6, 2, 2, 3, 3, 5, 2, 2, 2, 2, 2, 4, 4, 6, 2, // 0x80
    2, 5, 2, 2, 4, 4, 6, 2, 3, 5, 2, 2, 4, 6, 7, 2, // 0x90
    2, 6, 2, 2, 3, 3, 5, 2, 4, 2, 2, 2, 4, 4, 6, 2, // 0xA0
    2, 5, 2, 2, 2, 4, 6, 2, 4, 4, 2, 2, 4, 6, 7, 2, // 0xB0
    2, 6, 2, 2, 3, 3, 5, 2, 2, 2, 2, 2, 4, 4, 6, 2, // 0xC0
    2, 5, 2, 2, 4, 4, 6, 2, 3, 5, 2, 2, 5, 6, 7, 2, // 0xD0
    2, 6, 2, 2, 3, 3, 5, 2, 4, 2, 2, 2, 4, 4, 6, 2, // 0xE0
    2, 5, 2, 2, 2, 4, 6, 2, 4, 4, 2, 2, 4, 6, 7, 2, // 0xF0
];
pub struct CPU<'a> {
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub status: CpuFlags,
    pub pc: u16,
    pub stk_ptr: u8,
    pub bus: Bus<'a>,
    pub cycles: usize,
}
const STK: u16 = 0x0100;
const STK_RESET: u8 = 0xfd;
pub trait Mem {
    fn mem_read(&mut self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
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

impl Mem for CPU<'_> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data);
    }
    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        self.bus.mem_read_u16(pos)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        self.bus.mem_write_u16(pos, data)
    }
}
fn page_cross(addr1: u16, addr2: u16) -> bool {
    addr1 & 0xFF00 != addr2 & 0xFF00
}

impl<'a> CPU<'a> {
    //constructor i.e. associated function
    pub fn new<'b>(bus: Bus<'b>) -> CPU<'b> {
        CPU {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            pc: 0,
            cycles: 0,
            status: CpuFlags::from_bits_truncate(0b100100),
            stk_ptr: STK_RESET,
            bus,
        }
    }

    fn stk_push(&mut self, data: u8) {
        self.mem_write((STK as u16) + self.stk_ptr as u16, data);
        self.stk_ptr = self.stk_ptr.wrapping_sub(1);
    }
    fn stk_push_u16(&mut self, data: u16) {
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
        // self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        // self.mem_write_u16(0xFFFC, 0x8000);
        for i in 0..(program.len() as u16) {
            self.mem_write(0x0000 + i, program[i as usize]);
        }
        self.mem_write_u16(0xFFFC, 0x0000);
    }

    pub fn reset(&mut self) {
        self.reg_x = 0;
        self.reg_a = 0;
        self.status = CpuFlags::from_bits_truncate(0b0010_0000);
        self.stk_ptr = STK_RESET;
        self.cycles = 0;
        self.pc = self.mem_read_u16(0xFFFC);
    }
    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let val = self.mem_read(addr);
        let and = self.reg_a & val;
        self.status.set(CpuFlags::ZERO, and == 0);
        self.status.set(CpuFlags::NEGATIVE, val & 0b1000_0000 != 0);
        self.status.set(CpuFlags::OVERFLOW, val & 0b0100_0000 != 0);
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let val = self.mem_read(addr);
        self.reg_a = val;
        self.update_zero_and_neg_flag(self.reg_a);
        if page_cross {
            self.bus.tick(1);
        }
    }
    fn ldx(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let val = self.mem_read(addr);
        self.reg_x = val;
        self.update_zero_and_neg_flag(self.reg_x);
        if page_cross {
            self.bus.tick(1);
        }
    }
    fn ldy(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let val = self.mem_read(addr);
        self.reg_y = val;
        self.update_zero_and_neg_flag(self.reg_y);
        if page_cross {
            self.bus.tick(1);
        }
    }

    fn sei(&mut self) {
        self.status.insert(CpuFlags::INTERRUPT_DISABLE);
    }

    fn sed(&mut self) {
        self.status.insert(CpuFlags::DECIMAL_MODE);
    }

    fn cld(&mut self) {
        self.status.remove(CpuFlags::DECIMAL_MODE);
    }

    // fn _brk(&mut self) {
    //     self.stk_push_u16(self.pc + 1);
    //     let mut flag = self.status.clone();
    //     flag.insert(CpuFlags::BREAK);
    //     flag.insert(CpuFlags::UNUSED);
    //
    //     self.stk_push(flag.bits());
    //     self.status.insert(CpuFlags::INTERRUPT_DISABLE);
    //     self.pc = self.mem_read_u16(0xFFFE);
    // }

    // fn _interrupt_irq(&mut self) {
    //     self.stk_push_u16(self.pc);
    //     let mut flag = self.status.clone();
    //     flag.remove(CpuFlags::BREAK);
    //     flag.insert(CpuFlags::UNUSED);
    //
    //     self.stk_push(flag.bits());
    //     self.status.insert(CpuFlags::INTERRUPT_DISABLE);
    //     self.pc = self.mem_read_u16(0xFFFE);
    // }

    fn rti(&mut self) {
        let pulled_status = self.stk_pop();
        self.status = CpuFlags::from_bits_truncate(pulled_status);
        self.status.remove(CpuFlags::BREAK);
        self.status.insert(CpuFlags::UNUSED);
        self.pc = self.stk_pop_u16();
    }

    fn and(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.reg_a = self.reg_a & data;
        self.update_zero_and_neg_flag(self.reg_a);
        if page_cross {
            self.bus.tick(1);
        }
    }

    fn asl(&mut self, mode: &AddressingMode) -> u8 {
        let (addr, _) = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        self.update_carry_flag_asl(data);
        data = data << 1;
        self.mem_write(addr, data);
        self.update_zero_and_neg_flag(data);
        data
    }
    fn asl_acc(&mut self) -> u8 {
        let mut data = self.reg_a;
        self.update_carry_flag_asl(data);
        data = data << 1;
        self.reg_a = data;
        self.update_zero_and_neg_flag(data);
        data
    }
    fn lsr(&mut self, mode: &AddressingMode) -> u8 {
        let (addr, _) = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        self.update_carry_flag_lsr(data);
        data = data >> 1;
        self.mem_write(addr, data);
        self.update_zero_and_neg_flag(data);
        data
    }

    fn lsr_acc(&mut self) -> u8 {
        let mut data = self.reg_a;
        self.update_carry_flag_lsr(data);
        data = data >> 1;
        self.reg_a = data;
        self.update_zero_and_neg_flag(data);
        data
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        let carry = self.status.contains(CpuFlags::CARRY) as u8;
        let val = data ^ 0xFF;
        let sum = self.reg_a as u16 + val as u16 + carry as u16;

        self.status.set(CpuFlags::CARRY, sum > 0xFF);
        let res = sum as u8;
        self.status.set(
            CpuFlags::OVERFLOW,
            ((self.reg_a ^ res) & (val ^ res) & 0x80) != 0,
        );
        self.reg_a = res;
        self.update_zero_and_neg_flag(self.reg_a);

        if page_cross {
            self.bus.tick(1);
        }
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        let carry = self.status.contains(CpuFlags::CARRY) as u8;
        let sum = self.reg_a as u16 + data as u16 + carry as u16;

        self.status.set(CpuFlags::CARRY, sum > 0xFF);
        let res = sum as u8;
        self.status.set(
            CpuFlags::OVERFLOW,
            ((self.reg_a ^ res) & (data ^ res) & 0x80) != 0,
        );
        self.reg_a = res;
        self.update_zero_and_neg_flag(self.reg_a);
        if page_cross {
            self.bus.tick(1);
        }
    }

    // fn branch(&mut self, condition: bool) {
    //     if condition {
    //         let jump: i8 = self.mem_read(self.pc) as i8;
    //         let jump_addr = self.pc.wrapping_add(1).wrapping_add(jump as u16);
    //         self.pc = jump_addr;
    //     }
    // }
    fn compare(&mut self, mode: &AddressingMode, cmp_with: u8) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let data = self.mem_read(addr);

        self.status.set(CpuFlags::CARRY, data <= cmp_with);
        self.update_zero_and_neg_flag(cmp_with.wrapping_sub(data));

        if page_cross {
            self.bus.tick(1);
        }
    }

    fn pla(&mut self) {
        let data = self.stk_pop();
        self.reg_a = data;
        self.update_zero_and_neg_flag(self.reg_a);
    }
    fn pha(&mut self) {
        self.stk_push(self.reg_a);
    }
    fn plp(&mut self) {
        let pull_status = self.stk_pop();
        self.status = CpuFlags::from_bits_truncate(
            (pull_status & !CpuFlags::BREAK.bits()) | CpuFlags::UNUSED.bits(),
        );
    }

    fn php(&mut self) {
        self.stk_push(
            self.status.bits() | CpuFlags::BREAK.bits() | CpuFlags::BREAK2.bits(), // | CpuFlags::UNUSED.bits(),
        );
    }

    fn inc(&mut self, mode: &AddressingMode) -> u8 {
        let (addr, _) = self.get_operand_address(mode);
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
        let (addr, _) = self.get_operand_address(mode);
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
        let (addr, page_cross) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.reg_a = data ^ self.reg_a;
        self.update_zero_and_neg_flag(self.reg_a);

        if page_cross {
            self.bus.tick(1);
        }
    }
    fn ora(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.reg_a = data | self.reg_a;
        self.update_zero_and_neg_flag(self.reg_a);

        if page_cross {
            self.bus.tick(1);
        }
    }

    fn rol_acc(&mut self) {
        let old_val = self.reg_a;
        let carry_in = self.status.contains(CpuFlags::CARRY) as u8;
        self.status.set(CpuFlags::CARRY, old_val & 0x80 != 0);
        let new_val = (old_val << 1) | carry_in;
        self.reg_a = new_val;
        self.update_zero_and_neg_flag(self.reg_a);
    }
    fn rol(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let old_val = self.mem_read(addr);
        let carry_in = self.status.contains(CpuFlags::CARRY) as u8;
        self.status.set(CpuFlags::CARRY, old_val & 0x80 != 0);
        let new_val = (old_val << 1) | carry_in;
        self.mem_write(addr, new_val);
        self.update_zero_and_neg_flag(new_val);
    }
    fn ror_acc(&mut self) {
        let carry_in = if self.status.contains(CpuFlags::CARRY) {
            0x80
        } else {
            0
        };
        let old_val = self.reg_a;
        let new_val = (old_val >> 1) | carry_in;
        self.status.set(CpuFlags::CARRY, old_val & 0x01 != 0);
        self.reg_a = new_val;
        self.update_zero_and_neg_flag(self.reg_a);
    }
    fn ror(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let carry_in = if self.status.contains(CpuFlags::CARRY) {
            0x80
        } else {
            0
        };
        let old_val = self.mem_read(addr);
        let new_val = (old_val >> 1) | carry_in;
        self.status.set(CpuFlags::CARRY, old_val & 0x01 != 0);
        self.mem_write(addr, new_val);
        self.update_zero_and_neg_flag(new_val);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, self.reg_a);
    }
    fn stx(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, self.reg_x);
    }
    fn sty(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, self.reg_y);
    }

    fn tax(&mut self) {
        self.reg_x = self.reg_a;
        self.update_zero_and_neg_flag(self.reg_x);
    }

    fn txa(&mut self) {
        self.reg_a = self.reg_x;
        self.update_zero_and_neg_flag(self.reg_a);
    }

    fn tay(&mut self) {
        self.reg_y = self.reg_a;
        self.update_zero_and_neg_flag(self.reg_y);
    }

    fn tya(&mut self) {
        self.reg_a = self.reg_y;
        self.update_zero_and_neg_flag(self.reg_a);
    }

    fn txs(&mut self) {
        self.stk_ptr = self.reg_x;
    }
    fn tsx(&mut self) {
        self.reg_x = self.stk_ptr;
    }

    fn update_zero_and_neg_flag(&mut self, res: u8) {
        self.status.set(CpuFlags::ZERO, res == 0);
        self.status.set(CpuFlags::NEGATIVE, res & 0b1000_0000 != 0);
    }

    fn update_carry_flag_asl(&mut self, res: u8) {
        self.status.set(CpuFlags::CARRY, res >> 7 == 1);
    }
    fn update_carry_flag_lsr(&mut self, res: u8) {
        self.status.set(CpuFlags::CARRY, res & 1 == 1);
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> (u16, bool) {
        match mode {
            AddressingMode::Immediate => (self.pc, false),
            AddressingMode::ZeroPage => (self.mem_read(self.pc) as u16, false),
            AddressingMode::Absolute => (self.mem_read_u16(self.pc) as u16, false),
            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.pc);
                let addr = pos.wrapping_add(self.reg_x) as u16;
                (addr, false)
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.pc);
                let addr = pos.wrapping_add(self.reg_y) as u16;
                (addr, false)
            }
            AddressingMode::Absolute_X => {
                let pos = self.mem_read_u16(self.pc);
                let addr = pos.wrapping_add(self.reg_x as u16);
                (addr, page_cross(pos, addr))
            }
            AddressingMode::Absolute_Y => {
                let pos = self.mem_read_u16(self.pc);
                let addr = pos.wrapping_add(self.reg_y as u16);
                (addr, page_cross(pos, addr))
            }
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.pc);
                let ptr: u8 = (base as u8).wrapping_add(self.reg_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                ((hi as u16) << 8 | (lo as u16), false)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.pc);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.reg_y as u16);
                (deref, page_cross(deref, deref_base))
            }
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    fn _interrupt_nmi(&mut self) {
        self.stk_push_u16(self.pc);
        let mut flag = self.status.clone();
        flag.remove(CpuFlags::BREAK);
        flag.insert(CpuFlags::UNUSED);

        self.stk_push(flag.bits);
        self.status.insert(CpuFlags::INTERRUPT_DISABLE);

        self.bus.tick(2);
        self.pc = self.mem_read_u16(0xfffA);
    }
    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        loop {
            if let Some(_nmi) = self.bus.poll_nmi_status() {
                self.interrupt(interrupt::NMI);
            }

            callback(self);
            let opcode = self.mem_read(self.pc);
            self.pc += 1;
            let pc_state = self.pc;
            // if self.pc == 0x8008 {
            //     self.pc += 1;
            // } else {
            //     self.pc += 1;
            // }
            let cycles = OPCODE_CYCLES[opcode as usize];
            // self.bus.tick(cycles);
            // self.cycles += cycles as usize;
            match opcode {
                0x00 => {
                    return;
                }
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

                //LDX
                0xa2 => {
                    self.ldx(&AddressingMode::Immediate);
                    self.pc += 1;
                }
                0xa6 => {
                    self.ldx(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }
                0xb6 => {
                    self.ldx(&AddressingMode::ZeroPage_Y);
                    self.pc += 1;
                }
                0xae => {
                    self.ldx(&AddressingMode::Absolute);
                    self.pc += 2;
                }
                0xbe => {
                    self.ldx(&AddressingMode::Absolute_Y);
                    self.pc += 2;
                }

                // LDY
                0xa0 => {
                    self.ldy(&AddressingMode::Immediate);
                    self.pc += 1;
                }
                0xa4 => {
                    self.ldy(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }
                0xb4 => {
                    self.ldy(&AddressingMode::ZeroPage_X);
                    self.pc += 1;
                }
                0xac => {
                    self.ldy(&AddressingMode::Absolute);
                    self.pc += 2;
                }
                0xbc => {
                    self.ldy(&AddressingMode::Absolute_X);
                    self.pc += 2;
                }

                /*LSR - Logical Shift Right
                value = value >> 1, or visually: 0 -> [76543210] -> C
                LSR shifts all of the bits of a memory value or the accumulator one position to the right,
                moving the value of each bit into the next bit. 0 is shifted into bit 7, and bit 0 is shifted into the carry flag.
                This is equivalent to dividing an unsigned value by 2 and rounding down, with the remainder in carry. */
                0x4a => {
                    self.lsr_acc();
                }
                0x46 => {
                    self.lsr(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }
                0x56 => {
                    self.lsr(&AddressingMode::ZeroPage_X);
                    self.pc += 1;
                }
                0x4e => {
                    self.lsr(&AddressingMode::Absolute);
                    self.pc += 2;
                }
                0x5e => {
                    self.lsr(&AddressingMode::Absolute_X);
                    self.pc += 2;
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

                //STX
                0x86 => {
                    self.stx(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }
                0x96 => {
                    self.stx(&AddressingMode::ZeroPage_Y);
                    self.pc += 1;
                }
                0x8E => {
                    self.stx(&AddressingMode::Absolute);
                    self.pc += 2;
                }

                //STY
                0x84 => {
                    self.sty(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }
                0x94 => {
                    self.sty(&AddressingMode::ZeroPage_X);
                    self.pc += 1;
                }
                0x8C => {
                    self.sty(&AddressingMode::Absolute);
                    self.pc += 2;
                }

                //ADC
                0x69 => {
                    self.adc(&AddressingMode::Immediate);
                    self.pc += 1;
                }

                0x65 => {
                    self.adc(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }
                0x75 => {
                    self.adc(&AddressingMode::ZeroPage_X);
                    self.pc += 1;
                }
                0x6d => {
                    self.adc(&AddressingMode::Absolute);
                    self.pc += 2;
                }
                0x7d => {
                    self.adc(&AddressingMode::Absolute_X);
                    self.pc += 2;
                }
                0x79 => {
                    self.adc(&AddressingMode::Absolute_Y);
                    self.pc += 2;
                }
                0x61 => {
                    self.adc(&AddressingMode::Indirect_X);
                    self.pc += 1;
                }
                0x71 => {
                    self.adc(&AddressingMode::Indirect_Y);
                    self.pc += 1;
                }
                /*SBC - Subtract with Carry
                A = A - memory - ~C, or equivalently: A = A + ~memory + C
                SBC subtracts a memory value and the bitwise NOT of carry from the accumulator.
                It does this by adding the bitwise NOT of the memory value using ADC.*/
                0xe9 => {
                    self.sbc(&AddressingMode::Immediate);
                    self.pc += 1;
                }

                0xe5 => {
                    self.sbc(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }
                0xf5 => {
                    self.sbc(&AddressingMode::ZeroPage_X);
                    self.pc += 1;
                }
                0xed => {
                    self.sbc(&AddressingMode::Absolute);
                    self.pc += 2;
                }
                0xfd => {
                    self.sbc(&AddressingMode::Absolute_X);
                    self.pc += 2;
                }
                0xf9 => {
                    self.sbc(&AddressingMode::Absolute_Y);
                    self.pc += 2;
                }
                0xe1 => {
                    self.sbc(&AddressingMode::Indirect_X);
                    self.pc += 1;
                }
                0xf1 => {
                    self.sbc(&AddressingMode::Indirect_Y);
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

                //ASL
                0x0a => {
                    self.asl_acc();
                }
                0x06 => {
                    self.asl(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }
                0x16 => {
                    self.asl(&AddressingMode::ZeroPage_X);
                    self.pc += 1;
                }
                0x0E => {
                    self.asl(&AddressingMode::Absolute);
                    self.pc += 2;
                }
                0x1E => {
                    self.asl(&AddressingMode::Absolute_X);
                    self.pc += 2;
                }

                /*ROR - Rotate Right
                value = value >> 1 through C, or visually: C -> [76543210] -> C
                ROR shifts a memory value or the accumulator to the right, moving the value of each
                bit into the next bit and treating the carry flag as though it is both above bit 7 and below bit 0.
                Specifically, the value in carry is shifted into bit 7, and bit 0 is shifted into carry.
                Rotating right 9 times simply returns the value and carry back to their original state. */
                0x6a => {
                    self.ror_acc();
                }
                0x66 => {
                    self.ror(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }
                0x76 => {
                    self.ror(&AddressingMode::ZeroPage_X);
                    self.pc += 1;
                }
                0x6E => {
                    self.ror(&AddressingMode::Absolute);
                    self.pc += 2;
                }
                0x7E => {
                    self.ror(&AddressingMode::Absolute_X);
                    self.pc += 2;
                }

                /*ROL - Rotate Left
                value = value << 1 through C, or visually: C <- [76543210] <- C
                ROL shifts a memory value or the accumulator to the left, moving the value of each bit into the next bit and treating
                the carry flag as though it is both above bit 7 and below bit 0.
                Specifically, the value in carry is shifted into bit 0, and bit 7 is shifted into carry.
                Rotating left 9 times simply returns the value and carry back to their original state. */
                0x2a => {
                    self.rol_acc();
                }
                0x26 => {
                    self.rol(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }
                0x36 => {
                    self.rol(&AddressingMode::ZeroPage_X);
                    self.pc += 1;
                }
                0x2E => {
                    self.rol(&AddressingMode::Absolute);
                    self.pc += 2;
                }
                0x3E => {
                    self.rol(&AddressingMode::Absolute_X);
                    self.pc += 2;
                }

                //BCC
                0x90 => {
                    let offset = self.mem_read(self.pc) as i8;
                    self.pc += 1;
                    if !self.status.contains(CpuFlags::CARRY) {
                        self.pc = self.pc.wrapping_add(offset as u16);
                    }
                }

                //BCS
                0xB0 => {
                    let offset = self.mem_read(self.pc) as i8;
                    self.pc += 1;
                    if self.status.contains(CpuFlags::CARRY) {
                        self.pc = self.pc.wrapping_add(offset as u16);
                    }
                }

                //BEQ
                0xF0 => {
                    let offset = self.mem_read(self.pc) as i8;
                    self.pc += 1;
                    if self.status.contains(CpuFlags::ZERO) {
                        self.pc = self.pc.wrapping_add(offset as u16);
                    }
                }

                //BMI
                0x30 => {
                    let offset = self.mem_read(self.pc) as i8;
                    self.pc += 1;
                    if self.status.contains(CpuFlags::NEGATIVE) {
                        self.pc = self.pc.wrapping_add(offset as u16);
                    }
                }
                //BNE
                0xD0 => {
                    let offset = self.mem_read(self.pc) as i8;
                    self.pc += 1;
                    if !self.status.contains(CpuFlags::ZERO) {
                        self.pc = self.pc.wrapping_add(offset as u16);
                    }
                }

                //BPL
                0x10 => {
                    let offset = self.mem_read(self.pc) as i8;
                    self.pc += 1;
                    if !self.status.contains(CpuFlags::NEGATIVE) {
                        self.pc = self.pc.wrapping_add(offset as u16);
                    }
                }

                //BVC
                0x50 => {
                    let offset = self.mem_read(self.pc) as i8;
                    self.pc += 1;
                    if !self.status.contains(CpuFlags::OVERFLOW) {
                        self.pc = self.pc.wrapping_add(offset as u16);
                    }
                }
                //BVS
                0x70 => {
                    let offset = self.mem_read(self.pc) as i8;
                    self.pc += 1;
                    if self.status.contains(CpuFlags::OVERFLOW) {
                        self.pc = self.pc.wrapping_add(offset as u16);
                    }
                }

                //CLC
                0x18 => {
                    self.status.remove(CpuFlags::CARRY);
                }
                //SEC
                0x38 => {
                    self.status.insert(CpuFlags::CARRY);
                }

                //CLI
                0x58 => {
                    self.status.remove(CpuFlags::INTERRUPT_DISABLE);
                }

                //CLV
                0xB8 => {
                    self.status.remove(CpuFlags::OVERFLOW);
                }

                //BIT
                0x24 => {
                    self.bit(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }
                0x2C => {
                    self.bit(&AddressingMode::Absolute);
                    self.pc += 2;
                }

                //CompareA
                0xc9 => {
                    self.compare(&AddressingMode::Immediate, self.reg_a);
                    self.pc += 1;
                }
                0xc5 => {
                    self.compare(&AddressingMode::ZeroPage, self.reg_a);
                    self.pc += 1;
                }
                0xd5 => {
                    self.compare(&AddressingMode::ZeroPage_X, self.reg_a);
                    self.pc += 1;
                }
                0xcd => {
                    self.compare(&AddressingMode::Absolute, self.reg_a);
                    self.pc += 2;
                }
                0xdd => {
                    self.compare(&AddressingMode::Absolute_X, self.reg_a);
                    self.pc += 2;
                }
                0xd9 => {
                    self.compare(&AddressingMode::Absolute_Y, self.reg_a);
                    self.pc += 2;
                }
                0xc1 => {
                    self.compare(&AddressingMode::Indirect_X, self.reg_a);
                    self.pc += 1;
                }
                0xd1 => {
                    self.compare(&AddressingMode::Indirect_Y, self.reg_a);
                    self.pc += 1;
                }

                //CompareX
                0xe0 => {
                    self.compare(&AddressingMode::Immediate, self.reg_x);
                    self.pc += 1;
                }
                0xe4 => {
                    self.compare(&AddressingMode::ZeroPage, self.reg_x);
                    self.pc += 1;
                }
                0xec => {
                    self.compare(&AddressingMode::Absolute, self.reg_x);
                    self.pc += 2;
                }

                //CompareY
                0xc0 => {
                    self.compare(&AddressingMode::Immediate, self.reg_y);
                    self.pc += 1;
                }
                0xc4 => {
                    self.compare(&AddressingMode::ZeroPage, self.reg_y);
                    self.pc += 1;
                }
                0xcc => {
                    self.compare(&AddressingMode::Absolute, self.reg_y);
                    self.pc += 2;
                }

                //DEC
                0xc6 => {
                    self.dec(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }

                0xd6 => {
                    self.dec(&AddressingMode::ZeroPage_X);
                    self.pc += 1;
                }
                0xce => {
                    self.dec(&AddressingMode::Absolute);
                    self.pc += 2;
                }
                0xde => {
                    self.dec(&AddressingMode::Absolute_X);
                    self.pc += 2;
                }

                //DECX
                0xCA => {
                    self.dex();
                }

                //DECY
                0x88 => {
                    self.dey();
                }

                //EOR
                0x49 => {
                    self.eor(&AddressingMode::Immediate);
                    self.pc += 1;
                }
                0x45 => {
                    self.eor(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }
                0x55 => {
                    self.eor(&AddressingMode::ZeroPage_X);
                    self.pc += 1;
                }
                0x4d => {
                    self.eor(&AddressingMode::Absolute);
                    self.pc += 2;
                }
                0x5d => {
                    self.eor(&AddressingMode::Absolute_X);
                    self.pc += 2;
                }
                0x59 => {
                    self.eor(&AddressingMode::Absolute_Y);
                    self.pc += 2;
                }
                0x41 => {
                    self.eor(&AddressingMode::Indirect_X);
                    self.pc += 1;
                }
                0x51 => {
                    self.eor(&AddressingMode::Indirect_Y);
                    self.pc += 1;
                }

                //INC
                0xe6 => {
                    self.inc(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }

                0xf6 => {
                    self.inc(&AddressingMode::ZeroPage_X);
                    self.pc += 1;
                }
                0xee => {
                    self.inc(&AddressingMode::Absolute);
                    self.pc += 2;
                }
                0xfe => {
                    self.inc(&AddressingMode::Absolute_X);
                    self.pc += 2;
                }

                //INCX
                0xe8 => {
                    self.incx();
                }

                //INCY
                0xc8 => {
                    self.incy();
                }

                /*JMP - Jump
                PC = memory
                JMP sets the program counter to a new value, allowing code to execute from a new location.
                */
                //Absolute
                0x4c => {
                    let mem_addr = self.mem_read_u16(self.pc);
                    self.pc = mem_addr;
                }
                //Indirect
                0x6c => {
                    let mem_addr = self.mem_read_u16(self.pc);

                    let indirect_ref = if mem_addr & 0x00ff == 0x00ff {
                        // this checks if we are at end of the page i.e.  0x30ff & 0x00ff => 0x00ff
                        let lo = self.mem_read(mem_addr);
                        let hi = self.mem_read(mem_addr & 0xff00);
                        (hi as u16) << 8 | (lo as u16)
                    } else {
                        self.mem_read_u16(mem_addr)
                    };
                    self.pc = indirect_ref;
                }

                /*JSR - Jump to Subroutine
                push PC + 2 to stack
                PC = memory
                JSR pushes the current program counter to the stack and then sets the program counter to a new value.
                This allows code to call a function and return with RTS back to the instruction after the JSR. */
                0x20 => {
                    self.stk_push_u16(self.pc + 2 - 1);
                    let target = self.mem_read_u16(self.pc);
                    self.pc = target
                }

                /* RTS - Return from Subroutine
                pull PC from stack
                PC = PC + 1
                RTS pulls an address from the stack into the program counter and then increments the program counter.
                It is normally used at the end of a function to return to the instruction after the JSR that called the function.
                However, RTS is also sometimes used to implement jump tables (see Jump table and RTS Trick).
                */
                0x60 => {
                    self.pc = self.stk_pop_u16() + 1;
                }

                //ORA
                0x09 => {
                    self.ora(&AddressingMode::Immediate);
                    self.pc += 1;
                }
                0x05 => {
                    self.ora(&AddressingMode::ZeroPage);
                    self.pc += 1;
                }
                0x15 => {
                    self.ora(&AddressingMode::ZeroPage_X);
                    self.pc += 1;
                }
                0x0d => {
                    self.ora(&AddressingMode::Absolute);
                    self.pc += 2;
                }
                0x1d => {
                    self.ora(&AddressingMode::Absolute_X);
                    self.pc += 2;
                }
                0x19 => {
                    self.ora(&AddressingMode::Absolute_Y);
                    self.pc += 2;
                }
                0x01 => {
                    self.ora(&AddressingMode::Indirect_X);
                    self.pc += 1;
                }
                0x11 => {
                    self.ora(&AddressingMode::Indirect_Y);
                    self.pc += 1;
                }

                //TAX
                0xaa => {
                    self.tax();
                }
                //TXA
                0x8a => {
                    self.txa();
                }
                //TAY
                0xa8 => {
                    self.tay();
                }
                //TYA
                0x98 => {
                    self.tya();
                }

                //PHA
                0x48 => {
                    self.pha();
                }
                //PLA
                0x68 => {
                    self.pla();
                }
                //PLP
                0x28 => {
                    self.plp();
                }
                // PHP
                0x08 => {
                    self.php();
                }

                //TXS
                0x9a => {
                    self.txs();
                }
                //TSX
                0xba => {
                    self.tsx();
                }

                // SEI
                0x78 => {
                    self.sei();
                }

                // SED
                0xF8 => {
                    self.sed();
                }

                // CLD
                0xD8 => {
                    self.cld();
                }

                // NOP
                0xEA => {}
                //RTI
                0x40 => {
                    self.rti();
                }

                _ => todo!("implement this"),
            }
            self.bus.tick(cycles);
            if pc_state == self.pc {
                let len = match opcode {
                    0xaa | 0xe8 | 0x00 | 0xd8 | 0x58 | 0xb8 | 0x18 | 0x38 | 0x78 | 0xf8 | 0x48
                    | 0x68 | 0x08 | 0x28 | 0x4a | 0x0a | 0x2a | 0x6a | 0xc8 | 0xca | 0x88
                    | 0xea | 0xa8 | 0xba | 0x8a | 0x9a | 0x98 | 0x1a | 0x3a | 0x5a | 0x7a
                    | 0xda | 0xfa => 1,
                    _ => 2,
                };
                self.pc += (len - 1) as u16;
            }
        }
    }

    fn interrupt(&mut self, interrupt: interrupt::Interrupt) {
        self.stk_push_u16(self.pc);
        let mut flag = self.status.clone();
        flag.set(CpuFlags::BREAK, interrupt.b_flag_mask & 0b010000 == 1);
        flag.set(CpuFlags::BREAK2, interrupt.b_flag_mask & 0b100000 == 1);

        self.stk_push(flag.bits);
        self.status.insert(CpuFlags::INTERRUPT_DISABLE);

        self.bus.tick(interrupt.cpu_cycles);
        self.pc = self.mem_read_u16(interrupt.vector_addr);
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }
    // #[cfg(test)]
    // mod test {
    //     use super::*;
    //
    //     #[test]
    //     fn test_0xa9_lda_immediate_load_data() {
    //         let mut cpu = CPU::new();
    //         cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
    //         assert_eq!(cpu.reg_a, 0x05);
    //         assert!(cpu.status & 0b0000_0010 == 0b00);
    //         assert!(cpu.status & 0b1000_0000 == 0);
    //     }
    //
    //     #[test]
    //     fn test_0xa9_lda_zero_flag() {
    //         let mut cpu = CPU::new();
    //         cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
    //         assert!(cpu.status & 0b0000_0010 == 0b10);
    //     }
    //     #[test]
    //     fn test_lda_from_memory() {
    //         let mut cpu = CPU::new();
    //         cpu.mem_write(0x10, 0x55);
    //         cpu.load_and_run(vec![0xa5, 0x10, 0x00]);
    //         assert_eq!(cpu.reg_a, 0x55);
    //     }
    //
    //     #[test]
    //     fn test_0xaa_tax_move_a_to_x() {
    //         let mut cpu = CPU::new();
    //         cpu.load_and_run(vec![0xa9, 10, 0xaa, 0x00]);
    //         assert_eq!(cpu.reg_x, 10);
    //     }
    //
    //     #[test]
    //     fn test_5_ops_working_together() {
    //         let mut cpu = CPU::new();
    //         cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
    //         // println!("{} ,{}",cpu.reg_x,cpu.reg_a);
    //         assert_eq!(cpu.reg_x, 0xc1)
    //     }
    //
    //     #[test]
    //     fn test_inx_overflow() {
    //         let mut cpu = CPU::new();
    //         cpu.reg_x = 0xff;
    //         cpu.load_and_run(vec![0xe8, 0x00]);
    //         println!("{}", cpu.reg_x);
    //         assert_eq!(cpu.reg_x, 1)
    //     }
    //
    //     #[test]
    //     fn test_sta_absolute_x() {
    //         let mut cpu = CPU::new();
    //         // LDX #$0F
    //         // LDA #$DD
    //         // STA $0200,X ; Store at $0200 + $0F = $020F
    //         cpu.load_and_run(vec![0xa2, 0x0f, 0xa9, 0xdd, 0x9d, 0x00, 0x02, 0x00]);
    //         assert_eq!(cpu.mem_read(0x020f), 0xdd);
    //     }
    //
    //     #[test]
    //     fn test_lda_indirect_y() {
    //         let mut cpu = CPU::new();
    //         // Set up the indirect address pointer at zero-page location $F0.
    //         // It will point to $0300.
    //         cpu.mem_write_u16(0xf0, 0x0300);
    //         // The value to be loaded is at $0300 + Y.
    //         cpu.mem_write(0x0305, 0xAB);
    //
    //         // LDY #$05
    //         // LDA ($F0),Y ; Load from address pointed to by $F0, offset by Y.
    //         let program = vec![0xa0, 0x05, 0xb1, 0xf0, 0x00];
    //         cpu.load_and_run(program);
    //
    //         assert_eq!(cpu.reg_a, 0xab);
    //     }
    //     #[test]
    //     fn test_bne_branch_taken() {
    //         let mut cpu = CPU::new();
    //         // A simple program to check if BNE skips the next instruction.
    //         // LDX #$01 ; X is not zero, so Zero flag is clear.
    //         // BNE +2   ; Branch should be taken, skipping the INX.
    //         // INX      ; This should be skipped.
    //         // BRK
    //         cpu.load_and_run(vec![0xa2, 0x01, 0xd0, 0x01, 0xe8, 0x00]);
    //         assert_eq!(cpu.reg_x, 1, "X should be 1, as INX was skipped");
    //     }
    //
    //     #[test]
    //     fn test_bne_branch_not_taken() {
    //         let mut cpu = CPU::new();
    //         // DEX will make X=0, setting the Zero flag.
    //         // BNE should not be taken.
    //         // LDX #$01
    //         // DEX      ; X becomes 0, Z flag set
    //         // BNE +2   ; Branch not taken
    //         // INX      ; This should execute
    //         // BRK
    //         cpu.load_and_run(vec![0xa2, 0x01, 0xca, 0xd0, 0x01, 0xe8, 0x00]);
    //         assert_eq!(cpu.reg_x, 1, "X should be 1, as INX was executed");
    //     }
    //
    //     #[test]
    //     fn test_compare_and_branch_loop() {
    //         let mut cpu = CPU::new();
    //         // A simple countdown loop.
    //         // LDX #$03
    //         // LOOP:
    //         // DEX
    //         // CPX #$00
    //         // BNE LOOP
    //         // BRK
    //         // Program in hex: A2 03 CA E0 00 D0 FB 00
    //         // FB is the 2's complement of -5 bytes.
    //         let program = vec![0xa2, 0x03, 0xca, 0xe0, 0x00, 0xd0, 0xfb, 0x00];
    //         cpu.load_and_run(program);
    //         assert_eq!(cpu.reg_x, 0, "Loop should terminate when X is 0");
    //     }
    //
    //     #[test]
    //     fn test_compare_flags() {
    //         let mut cpu = CPU::new();
    //         // LDA #$10, CMP #$20 (A < M)
    //         // Carry should be clear, Zero clear, Negative set.
    //         cpu.load_and_run(vec![0xa9, 0x10, 0xc9, 0x20, 0x00]);
    //         assert!(!cpu.get_flag_carry(), "A < M, Carry should be clear");
    //         assert!(
    //             cpu.status & 0b0000_0010 == 0,
    //             "A != M, Zero should be clear"
    //         );
    //         assert!(
    //             cpu.status & 0b1000_0000 != 0,
    //             "Result is negative, Negative flag should be set"
    //         );
    //     }
    //     #[test]
    //     fn test_asl_accumulator() {
    //         let mut cpu = CPU::new();
    //         // LDA #$C1 (11000001)
    //         // ASL A
    //         // Result in A should be 10000010 (0x82).
    //         // Carry should be set (from original bit 7).
    //         // Negative should be set (from new bit 7).
    //         cpu.load_and_run(vec![0xa9, 0xc1, 0x0a, 0x00]);
    //         assert_eq!(cpu.reg_a, 0x82);
    //         assert!(cpu.get_flag_carry(), "Carry flag should be set");
    //         assert!(cpu.status & 0b1000_0000 != 0, "Negative flag should be set");
    //     }
    //
    //     #[test]
    //     fn test_lsr_memory_and_flags() {
    //         let mut cpu = CPU::new();
    //         cpu.mem_write(0x20, 0x01);
    //         cpu.load_and_run(vec![0x46, 0x20, 0x00]);
    //
    //         assert_eq!(cpu.mem_read(0x20), 0x00);
    //         assert!(cpu.get_flag_carry(), "Carry flag should be set");
    //         assert!(cpu.status & 0b0000_0010 != 0, "Zero flag should be set");
    //     }
    //
    //     #[test]
    //     fn test_rol_accumulator_with_carry() {
    //         let mut cpu = CPU::new();
    //         cpu.load_and_run(vec![0x38, 0xa9, 0x7f, 0x2a, 0x00]);
    //
    //         assert_eq!(cpu.reg_a, 0xff, "Accumulator should be 255");
    //         assert!(!cpu.get_flag_carry(), "Carry flag should be cleared");
    //         assert!(cpu.status & 0b1000_0000 != 0, "Negative flag should be set");
    //     }
    //
    //     #[test]
    //     fn test_ror_memory() {
    //         let mut cpu = CPU::new();
    //         cpu.mem_write(0x33, 0x02); // 00000010
    //                                    // SEC (set carry)
    //                                    // ROR $33
    //                                    // Result at $33: 10000001 (0x81).
    //                                    // Carry is cleared (from old bit 0).
    //         cpu.load_and_run(vec![0x38, 0x6e, 0x33, 0x00]);
    //         assert_eq!(cpu.mem_read(0x33), 0x81);
    //         assert!(!cpu.get_flag_carry(), "Carry flag should be cleared");
    //     }
    //     #[test]
    //     fn test_and_immediate() {
    //         let mut cpu = CPU::new();
    //         // LDA #$CF (11001111)
    //         // AND #$3A (00111010)
    //         // Result should be 00001010 = 0x0A
    //         cpu.load_and_run(vec![0xa9, 0xcf, 0x29, 0x3a, 0x00]);
    //         assert_eq!(cpu.reg_a, 0x0a);
    //         assert!(cpu.status & 0b0000_0010 == 0, "Zero flag should not be set");
    //     }
    //
    //     #[test]
    //     fn test_eor_and_zero_flag() {
    //         let mut cpu = CPU::new();
    //         // LDA #$55
    //         // EOR #$55
    //         // Result should be 0, setting the Zero flag.
    //         cpu.load_and_run(vec![0xa9, 0x55, 0x49, 0x55, 0x00]);
    //         assert_eq!(cpu.reg_a, 0x00);
    //         assert!(cpu.status & 0b0000_0010 != 0, "Zero flag should be set");
    //     }
    //
    //     #[test]
    //     fn test_bit_instruction() {
    //         let mut cpu = CPU::new();
    //         cpu.mem_write(0x20, 0b1100_0000); // Value to test against
    //                                           // LDA #$01
    //                                           // BIT $20
    //                                           // Tests accumulator (0x01) against memory (0xC0).
    //                                           // A & M = 0, so Zero flag (Z) is set.
    //                                           // Bit 7 of M is 1, so Negative flag (N) is set.
    //                                           // Bit 6 of M is 1, so Overflow flag (V) is set.
    //         cpu.load_and_run(vec![0xa9, 0x01, 0x24, 0x20, 0x00]);
    //         assert!(cpu.status & 0b0000_0010 != 0, "Zero flag should be set");
    //         assert!(
    //             cpu.status & 0b1000_0000 != 0,
    //             "Negative flag should be set from mem bit 7"
    //         );
    //         assert!(
    //             cpu.status & 0b0100_0000 != 0,
    //             "Overflow flag should be set from mem bit 6"
    //         );
    //     }
    //     #[test]
    //     fn test_adc_simple_add_with_overflow() {
    //         let mut cpu = CPU::new();
    //         // Load accumulator with 127 (0x7F)
    //         // Add 1
    //         // Result should be 128 (0x80), setting Overflow and Negative flags.
    //         cpu.load_and_run(vec![0xa9, 0x7f, 0x69, 0x01, 0x00]);
    //         assert_eq!(cpu.reg_a, 0x80);
    //         assert!(cpu.status & 0b0100_0000 != 0, "Overflow flag should be set");
    //         assert!(cpu.status & 0b1000_0000 != 0, "Negative flag should be set");
    //     }
    //
    //     #[test]
    //     fn test_sbc_with_borrow() {
    //         let mut cpu = CPU::new();
    //         // CLC (clears carry, enabling borrow for SBC)
    //         // LDA #$05
    //         // SBC #$03
    //         // Result should be 5 - 3 - 1 = 1
    //         cpu.load_and_run(vec![0x18, 0xa9, 0x05, 0xe9, 0x03, 0x00]);
    //         assert_eq!(cpu.reg_a, 1);
    //     }
    //
    //     #[test]
    //     fn test_sbc_no_borrow() {
    //         let mut cpu = CPU::new();
    //         // SEC (sets carry, disabling borrow for SBC)
    //         // LDA #$05
    //         // SBC #$03
    //         // Result should be 5 - 3 = 2
    //         cpu.load_and_run(vec![0x38, 0xa9, 0x05, 0xe9, 0x03, 0x00]);
    //         assert_eq!(cpu.reg_a, 2);
    //     }
    //     #[test]
    //     fn test_jsr_and_rts() {
    //         let mut cpu = CPU::new();
    //         // The program consists of a main routine and a subroutine.
    //         // Main routine:
    //         // 0x8000: LDA #$05   ; Load 5 into accumulator
    //         // 0x8002: JSR $800A  ; Jump to subroutine at address 0x800A
    //         // 0x8005: LDA #$0F   ; After returning, load 15 into accumulator
    //         // 0x8007: BRK        ; Halt
    //         // Subroutine:
    //         // 0x800A: LDA #$AA   ; Load 170 into accumulator
    //         // 0x800C: STA $0200  ; Store accumulator's value at memory location 0x0200
    //         // 0x800F: RTS        ; Return from subroutine
    //         let program = vec![
    //             0xa9, 0x05, // LDA #$05
    //             0x20, 0x0a, 0x80, // JSR $800A
    //             0xa9, 0x0f, // LDA #$0F
    //             0x00, // BRK
    //             0x00, 0x00, // Padding to align subroutine
    //             0xa9, 0xaa, // Subroutine starts here (0x800A)
    //             0x8d, 0x00, 0x02, // STA $0200
    //             0x60, // RTS
    //         ];
    //         cpu.load_and_run(program);
    //
    //         // 1. Verify the subroutine executed correctly by checking the memory it modified.
    //         assert_eq!(
    //             cpu.mem_read(0x0200),
    //             0xaa,
    //             "Subroutine should store 0xAA in memory."
    //         );
    //
    //         // 2. Verify that the CPU returned to the correct location and executed the next instruction.
    //         assert_eq!(
    //             cpu.reg_a, 0x0f,
    //             "Accumulator should be 0x0F after returning."
    //         );
    //
    //         // 3. Verify that the stack pointer is back to its original state after push/pop.
    //         assert_eq!(
    //             cpu.stk_ptr, STK_RESET,
    //             "Stack pointer should be reset after JSR/RTS cycle."
    //         );
    //     }
    // }}
    //
    //
}
