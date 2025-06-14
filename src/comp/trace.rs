// / Generates a trace string for the current CPU state, disassembling the
// use crate::comp::cpu::AddressingMode;
use crate::comp::cpu::Mem;
use crate::comp::cpu::CPU;
/// instruction at the program counter and showing register values.
/// This is an invaluable tool for debugging an emulator.
pub fn trace(cpu: &CPU) -> String {
    let begin = cpu.pc;
    let code = cpu.mem_read(begin);

    // The tuple is: (Mnemonic, Operand String, Instruction Length)
    let (mnemonic, operand_desc, instr_len): (&str, String, u8) = match code {
        // Official Opcodes
        0x00 => ("BRK", "".to_string(), 1), // Implied
        0x01 => ("ORA", format!("(${:02x},X)", cpu.mem_read(begin + 1)), 2), // Indirect,X
        0x05 => ("ORA", format!("${:02x}", cpu.mem_read(begin + 1)), 2), // Zero Page
        0x06 => ("ASL", format!("${:02x}", cpu.mem_read(begin + 1)), 2), // Zero Page
        0x08 => ("PHP", "".to_string(), 1), // Implied
        0x09 => ("ORA", format!("#${:02x}", cpu.mem_read(begin + 1)), 2), // Immediate
        0x0a => ("ASL", "A".to_string(), 1), // Accumulator
        0x0d => ("ORA", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute
        0x0e => ("ASL", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute

        0x10 => {
            // BPL Relative
            let offset = cpu.mem_read(begin + 1) as i8;
            let target = begin.wrapping_add(2).wrapping_add(offset as u16);
            ("BPL", format!("${:04x}", target), 2)
        }
        0x11 => ("ORA", format!("(${:02x}),Y", cpu.mem_read(begin + 1)), 2), // Indirect,Y
        0x15 => ("ORA", format!("${:02x},X", cpu.mem_read(begin + 1)), 2),   // Zero Page,X
        0x16 => ("ASL", format!("${:02x},X", cpu.mem_read(begin + 1)), 2),   // Zero Page,X
        0x18 => ("CLC", "".to_string(), 1),                                  // Implied
        0x19 => ("ORA", format!("${:04x},Y", cpu.mem_read_u16(begin + 1)), 3), // Absolute,Y
        0x1d => ("ORA", format!("${:04x},X", cpu.mem_read_u16(begin + 1)), 3), // Absolute,X
        0x1e => ("ASL", format!("${:04x},X", cpu.mem_read_u16(begin + 1)), 3), // Absolute,X

        0x20 => ("JSR", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute
        0x21 => ("AND", format!("(${:02x},X)", cpu.mem_read(begin + 1)), 2), // Indirect,X
        0x24 => ("BIT", format!("${:02x}", cpu.mem_read(begin + 1)), 2),     // Zero Page
        0x25 => ("AND", format!("${:02x}", cpu.mem_read(begin + 1)), 2),     // Zero Page
        0x26 => ("ROL", format!("${:02x}", cpu.mem_read(begin + 1)), 2),     // Zero Page
        0x28 => ("PLP", "".to_string(), 1),                                  // Implied
        0x29 => ("AND", format!("#${:02x}", cpu.mem_read(begin + 1)), 2),    // Immediate
        0x2a => ("ROL", "A".to_string(), 1),                                 // Accumulator
        0x2c => ("BIT", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute
        0x2d => ("AND", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute
        0x2e => ("ROL", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute

        0x30 => {
            // BMI Relative
            let offset = cpu.mem_read(begin + 1) as i8;
            let target = begin.wrapping_add(2).wrapping_add(offset as u16);
            ("BMI", format!("${:04x}", target), 2)
        }
        0x31 => ("AND", format!("(${:02x}),Y", cpu.mem_read(begin + 1)), 2), // Indirect,Y
        0x35 => ("AND", format!("${:02x},X", cpu.mem_read(begin + 1)), 2),   // Zero Page,X
        0x36 => ("ROL", format!("${:02x},X", cpu.mem_read(begin + 1)), 2),   // Zero Page,X
        0x38 => ("SEC", "".to_string(), 1),                                  // Implied
        0x39 => ("AND", format!("${:04x},Y", cpu.mem_read_u16(begin + 1)), 3), // Absolute,Y
        0x3d => ("AND", format!("${:04x},X", cpu.mem_read_u16(begin + 1)), 3), // Absolute,X
        0x3e => ("ROL", format!("${:04x},X", cpu.mem_read_u16(begin + 1)), 3), // Absolute,X

        0x40 => ("RTI", "".to_string(), 1), // Implied
        0x41 => ("EOR", format!("(${:02x},X)", cpu.mem_read(begin + 1)), 2), // Indirect,X
        0x45 => ("EOR", format!("${:02x}", cpu.mem_read(begin + 1)), 2), // Zero Page
        0x46 => ("LSR", format!("${:02x}", cpu.mem_read(begin + 1)), 2), // Zero Page
        0x48 => ("PHA", "".to_string(), 1), // Implied
        0x49 => ("EOR", format!("#${:02x}", cpu.mem_read(begin + 1)), 2), // Immediate
        0x4a => ("LSR", "A".to_string(), 1), // Accumulator
        0x4c => ("JMP", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute
        0x4d => ("EOR", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute
        0x4e => ("LSR", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute

        0x50 => {
            // BVC Relative
            let offset = cpu.mem_read(begin + 1) as i8;
            let target = begin.wrapping_add(2).wrapping_add(offset as u16);
            ("BVC", format!("${:04x}", target), 2)
        }
        0x51 => ("EOR", format!("(${:02x}),Y", cpu.mem_read(begin + 1)), 2), // Indirect,Y
        0x55 => ("EOR", format!("${:02x},X", cpu.mem_read(begin + 1)), 2),   // Zero Page,X
        0x56 => ("LSR", format!("${:02x},X", cpu.mem_read(begin + 1)), 2),   // Zero Page,X
        0x58 => ("CLI", "".to_string(), 1),                                  // Implied
        0x59 => ("EOR", format!("${:04x},Y", cpu.mem_read_u16(begin + 1)), 3), // Absolute,Y
        0x5d => ("EOR", format!("${:04x},X", cpu.mem_read_u16(begin + 1)), 3), // Absolute,X
        0x5e => ("LSR", format!("${:04x},X", cpu.mem_read_u16(begin + 1)), 3), // Absolute,X

        0x60 => ("RTS", "".to_string(), 1), // Implied
        0x61 => ("ADC", format!("(${:02x},X)", cpu.mem_read(begin + 1)), 2), // Indirect,X
        0x65 => ("ADC", format!("${:02x}", cpu.mem_read(begin + 1)), 2), // Zero Page
        0x66 => ("ROR", format!("${:02x}", cpu.mem_read(begin + 1)), 2), // Zero Page
        0x68 => ("PLA", "".to_string(), 1), // Implied
        0x69 => ("ADC", format!("#${:02x}", cpu.mem_read(begin + 1)), 2), // Immediate
        0x6a => ("ROR", "A".to_string(), 1), // Accumulator
        0x6c => ("JMP", format!("(${:04x})", cpu.mem_read_u16(begin + 1)), 3), // Indirect
        0x6d => ("ADC", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute
        0x6e => ("ROR", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute

        0x70 => {
            // BVS Relative
            let offset = cpu.mem_read(begin + 1) as i8;
            let target = begin.wrapping_add(2).wrapping_add(offset as u16);
            ("BVS", format!("${:04x}", target), 2)
        }
        0x71 => ("ADC", format!("(${:02x}),Y", cpu.mem_read(begin + 1)), 2), // Indirect,Y
        0x75 => ("ADC", format!("${:02x},X", cpu.mem_read(begin + 1)), 2),   // Zero Page,X
        0x76 => ("ROR", format!("${:02x},X", cpu.mem_read(begin + 1)), 2),   // Zero Page,X
        0x78 => ("SEI", "".to_string(), 1),                                  // Implied
        0x79 => ("ADC", format!("${:04x},Y", cpu.mem_read_u16(begin + 1)), 3), // Absolute,Y
        0x7d => ("ADC", format!("${:04x},X", cpu.mem_read_u16(begin + 1)), 3), // Absolute,X
        0x7e => ("ROR", format!("${:04x},X", cpu.mem_read_u16(begin + 1)), 3), // Absolute,X

        0x81 => ("STA", format!("(${:02x},X)", cpu.mem_read(begin + 1)), 2), // Indirect,X
        0x84 => ("STY", format!("${:02x}", cpu.mem_read(begin + 1)), 2),     // Zero Page
        0x85 => ("STA", format!("${:02x}", cpu.mem_read(begin + 1)), 2),     // Zero Page
        0x86 => ("STX", format!("${:02x}", cpu.mem_read(begin + 1)), 2),     // Zero Page
        0x88 => ("DEY", "".to_string(), 1),                                  // Implied
        0x8a => ("TXA", "".to_string(), 1),                                  // Implied
        0x8c => ("STY", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute
        0x8d => ("STA", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute
        0x8e => ("STX", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute

        0x90 => {
            // BCC Relative
            let offset = cpu.mem_read(begin + 1) as i8;
            let target = begin.wrapping_add(2).wrapping_add(offset as u16);
            ("BCC", format!("${:04x}", target), 2)
        }
        0x91 => ("STA", format!("(${:02x}),Y", cpu.mem_read(begin + 1)), 2), // Indirect,Y
        0x94 => ("STY", format!("${:02x},X", cpu.mem_read(begin + 1)), 2),   // Zero Page,X
        0x95 => ("STA", format!("${:02x},X", cpu.mem_read(begin + 1)), 2),   // Zero Page,X
        0x96 => ("STX", format!("${:02x},Y", cpu.mem_read(begin + 1)), 2),   // Zero Page,Y
        0x98 => ("TYA", "".to_string(), 1),                                  // Implied
        0x99 => ("STA", format!("${:04x},Y", cpu.mem_read_u16(begin + 1)), 3), // Absolute,Y
        0x9a => ("TXS", "".to_string(), 1),                                  // Implied
        0x9d => ("STA", format!("${:04x},X", cpu.mem_read_u16(begin + 1)), 3), // Absolute,X

        0xa0 => ("LDY", format!("#${:02x}", cpu.mem_read(begin + 1)), 2), // Immediate
        0xa1 => ("LDA", format!("(${:02x},X)", cpu.mem_read(begin + 1)), 2), // Indirect,X
        0xa2 => ("LDX", format!("#${:02x}", cpu.mem_read(begin + 1)), 2), // Immediate
        0xa4 => ("LDY", format!("${:02x}", cpu.mem_read(begin + 1)), 2),  // Zero Page
        0xa5 => ("LDA", format!("${:02x}", cpu.mem_read(begin + 1)), 2),  // Zero Page
        0xa6 => ("LDX", format!("${:02x}", cpu.mem_read(begin + 1)), 2),  // Zero Page
        0xa8 => ("TAY", "".to_string(), 1),                               // Implied
        0xa9 => ("LDA", format!("#${:02x}", cpu.mem_read(begin + 1)), 2), // Immediate
        0xaa => ("TAX", "".to_string(), 1),                               // Implied
        0xac => ("LDY", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute
        0xad => ("LDA", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute
        0xae => ("LDX", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute

        0xb0 => {
            // BCS Relative
            let offset = cpu.mem_read(begin + 1) as i8;
            let target = begin.wrapping_add(2).wrapping_add(offset as u16);
            ("BCS", format!("${:04x}", target), 2)
        }
        0xb1 => ("LDA", format!("(${:02x}),Y", cpu.mem_read(begin + 1)), 2), // Indirect,Y
        0xb4 => ("LDY", format!("${:02x},X", cpu.mem_read(begin + 1)), 2),   // Zero Page,X
        0xb5 => ("LDA", format!("${:02x},X", cpu.mem_read(begin + 1)), 2),   // Zero Page,X
        0xb6 => ("LDX", format!("${:02x},Y", cpu.mem_read(begin + 1)), 2),   // Zero Page,Y
        0xb8 => ("CLV", "".to_string(), 1),                                  // Implied
        0xb9 => ("LDA", format!("${:04x},Y", cpu.mem_read_u16(begin + 1)), 3), // Absolute,Y
        0xba => ("TSX", "".to_string(), 1),                                  // Implied
        0xbc => ("LDY", format!("${:04x},X", cpu.mem_read_u16(begin + 1)), 3), // Absolute,X
        0xbd => ("LDA", format!("${:04x},X", cpu.mem_read_u16(begin + 1)), 3), // Absolute,X
        0xbe => ("LDX", format!("${:04x},Y", cpu.mem_read_u16(begin + 1)), 3), // Absolute,Y

        0xc0 => ("CPY", format!("#${:02x}", cpu.mem_read(begin + 1)), 2), // Immediate
        0xc1 => ("CMP", format!("(${:02x},X)", cpu.mem_read(begin + 1)), 2), // Indirect,X
        0xc4 => ("CPY", format!("${:02x}", cpu.mem_read(begin + 1)), 2),  // Zero Page
        0xc5 => ("CMP", format!("${:02x}", cpu.mem_read(begin + 1)), 2),  // Zero Page
        0xc6 => ("DEC", format!("${:02x}", cpu.mem_read(begin + 1)), 2),  // Zero Page
        0xc8 => ("INY", "".to_string(), 1),                               // Implied
        0xc9 => ("CMP", format!("#${:02x}", cpu.mem_read(begin + 1)), 2), // Immediate
        0xca => ("DEX", "".to_string(), 1),                               // Implied
        0xcc => ("CPY", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute
        0xcd => ("CMP", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute
        0xce => ("DEC", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute

        0xd0 => {
            // BNE Relative
            let offset = cpu.mem_read(begin + 1) as i8;
            let target = begin.wrapping_add(2).wrapping_add(offset as u16);
            ("BNE", format!("${:04x}", target), 2)
        }
        0xd1 => ("CMP", format!("(${:02x}),Y", cpu.mem_read(begin + 1)), 2), // Indirect,Y
        0xd5 => ("CMP", format!("${:02x},X", cpu.mem_read(begin + 1)), 2),   // Zero Page,X
        0xd6 => ("DEC", format!("${:02x},X", cpu.mem_read(begin + 1)), 2),   // Zero Page,X
        0xd8 => ("CLD", "".to_string(), 1),                                  // Implied
        0xd9 => ("CMP", format!("${:04x},Y", cpu.mem_read_u16(begin + 1)), 3), // Absolute,Y
        0xdd => ("CMP", format!("${:04x},X", cpu.mem_read_u16(begin + 1)), 3), // Absolute,X
        0xde => ("DEC", format!("${:04x},X", cpu.mem_read_u16(begin + 1)), 3), // Absolute,X

        0xe0 => ("CPX", format!("#${:02x}", cpu.mem_read(begin + 1)), 2), // Immediate
        0xe1 => ("SBC", format!("(${:02x},X)", cpu.mem_read(begin + 1)), 2), // Indirect,X
        0xe4 => ("CPX", format!("${:02x}", cpu.mem_read(begin + 1)), 2),  // Zero Page
        0xe5 => ("SBC", format!("${:02x}", cpu.mem_read(begin + 1)), 2),  // Zero Page
        0xe6 => ("INC", format!("${:02x}", cpu.mem_read(begin + 1)), 2),  // Zero Page
        0xe8 => ("INX", "".to_string(), 1),                               // Implied
        0xe9 => ("SBC", format!("#${:02x}", cpu.mem_read(begin + 1)), 2), // Immediate
        0xea => ("NOP", "".to_string(), 1),                               // Implied
        0xec => ("CPX", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute
        0xed => ("SBC", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute
        0xee => ("INC", format!("${:04x}", cpu.mem_read_u16(begin + 1)), 3), // Absolute

        0xf0 => {
            // BEQ Relative
            let offset = cpu.mem_read(begin + 1) as i8;
            let target = begin.wrapping_add(2).wrapping_add(offset as u16);
            ("BEQ", format!("${:04x}", target), 2)
        }
        0xf1 => ("SBC", format!("(${:02x}),Y", cpu.mem_read(begin + 1)), 2), // Indirect,Y
        0xf5 => ("SBC", format!("${:02x},X", cpu.mem_read(begin + 1)), 2),   // Zero Page,X
        0xf6 => ("INC", format!("${:02x},X", cpu.mem_read(begin + 1)), 2),   // Zero Page,X
        0xf8 => ("SED", "".to_string(), 1),                                  // Implied
        0xf9 => ("SBC", format!("${:04x},Y", cpu.mem_read_u16(begin + 1)), 3), // Absolute,Y
        0xfd => ("SBC", format!("${:04x},X", cpu.mem_read_u16(begin + 1)), 3), // Absolute,X
        0xfe => ("INC", format!("${:04x},X", cpu.mem_read_u16(begin + 1)), 3), // Absolute,X

        // Catch-all for unofficial/illegal opcodes
        _ => ("???", "".to_string(), 1),
    };

    let mut hex_dump = vec![];
    for i in 0..instr_len {
        hex_dump.push(cpu.mem_read(begin + i as u16));
    }
    let hex_str = hex_dump
        .iter()
        .map(|z| format!("{:02x}", z))
        .collect::<Vec<String>>()
        .join(" ");

    let asm_str = format!(
        "{:04x}  {:8} {: >4} {}",
        begin, hex_str, mnemonic, operand_desc
    )
    .trim()
    .to_string();

    format!(
        "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}",
        asm_str, cpu.reg_a, cpu.reg_x, cpu.reg_y, cpu.status, cpu.stk_ptr,
    )
    .to_ascii_uppercase()
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::comp::bus::Bus;
//     use crate::comp::rom::test::test_rom;
//
//     #[test]
//     fn test_format_trace() {
//         let mut bus = Bus::new(test_rom(vec![]));
//         bus.mem_write(100, 0xa2);
//         bus.mem_write(101, 0x01);
//         bus.mem_write(102, 0xca);
//         bus.mem_write(103, 0x88);
//         bus.mem_write(104, 0x00);
//
//         let mut cpu = CPU::new(bus);
//         cpu.pc = 0x64;
//         cpu.reg_a = 1;
//         cpu.reg_x = 2;
//         cpu.reg_y = 3;
//         let mut result: Vec<String> = vec![];
//         cpu.run_with_callback(|cpu| {
//             result.push(trace(cpu));
//         });
//         assert_eq!(
//             "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
//             result[0]
//         );
//         assert_eq!(
//             "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
//             result[1]
//         );
//         assert_eq!(
//             "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
//             result[2]
//         );
//     }
//
//     #[test]
//     fn test_format_mem_access() {
//         let mut bus = Bus::new(test_rom(vec![]));
//         // ORA ($33), Y
//         bus.mem_write(100, 0x11);
//         bus.mem_write(101, 0x33);
//
//         //data
//         bus.mem_write(0x33, 00);
//         bus.mem_write(0x34, 04);
//
//         //target cell
//         bus.mem_write(0x400, 0xAA);
//
//         let mut cpu = CPU::new(bus);
//         cpu.pc = 0x64;
//         cpu.reg_y = 0;
//         let mut result: Vec<String> = vec![];
//         cpu.run_with_callback(|cpu| {
//             result.push(trace(cpu));
//         });
//         assert_eq!(
//             "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
//             result[0]
//         );
//     }
//
//     #[test]
//     fn test_format_trace2() {
//         let mut bus = Bus::new(test_rom(vec![]));
//         bus.mem_write(100, 0xa2);
//         bus.mem_write(101, 0x01);
//         bus.mem_write(102, 0xca);
//         bus.mem_write(103, 0x88);
//         bus.mem_write(104, 0x00);
//
//         let mut cpu = CPU::new(bus);
//         cpu.pc = 0x64;
//         cpu.reg_a = 1;
//         cpu.reg_x = 2;
//         cpu.reg_y = 3;
//         let mut result: Vec<String> = vec![];
//         cpu.run_with_callback(|cpu| {
//             result.push(trace(cpu));
//         });
//         assert_eq!(
//             "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
//             result[0]
//         );
//         assert_eq!(
//             "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
//             result[1]
//         );
//         assert_eq!(
//             "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
//             result[2]
//         );
//     }
//
//     #[test]
//     fn test_format_mem_access2() {
//         let mut bus = Bus::new(test_rom(vec![]));
//         // ORA ($33), Y
//         bus.mem_write(100, 0x11);
//         bus.mem_write(101, 0x33);
//
//         //data
//         bus.mem_write(0x33, 00);
//         bus.mem_write(0x34, 04);
//
//         //target cell
//         bus.mem_write(0x400, 0xAA);
//
//         let mut cpu = CPU::new(bus);
//         cpu.pc = 0x64;
//         cpu.reg_y = 0;
//         let mut result: Vec<String> = vec![];
//         cpu.run_with_callback(|cpu| {
//             result.push(trace(cpu));
//         });
//         assert_eq!(
//             "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
//             result[0]
//         );
//     }
//
//     #[test]
//     fn test_format_trace3() {
//         let mut bus = Bus::new(test_rom(vec![]));
//         bus.mem_write(100, 0xa2);
//         bus.mem_write(101, 0x01);
//         bus.mem_write(102, 0xca);
//         bus.mem_write(103, 0x88);
//         bus.mem_write(104, 0x00);
//
//         let mut cpu = CPU::new(bus);
//         cpu.pc = 0x64;
//         cpu.reg_a = 1;
//         cpu.reg_x = 2;
//         cpu.reg_y = 3;
//         let mut result: Vec<String> = vec![];
//         cpu.run_with_callback(|cpu| {
//             result.push(trace(cpu));
//         });
//         assert_eq!(
//             "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
//             result[0]
//         );
//         assert_eq!(
//             "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
//             result[1]
//         );
//         assert_eq!(
//             "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
//             result[2]
//         );
//     }
//
//     #[test]
//     fn test_format_mem_access3() {
//         let mut bus = Bus::new(test_rom(vec![]));
//         // ORA ($33), Y
//         bus.mem_write(100, 0x11);
//         bus.mem_write(101, 0x33);
//
//         //data
//         bus.mem_write(0x33, 00);
//         bus.mem_write(0x34, 04);
//
//         //target cell
//         bus.mem_write(0x400, 0xAA);
//
//         let mut cpu = CPU::new(bus);
//         cpu.pc = 0x64;
//         cpu.reg_y = 0;
//         let mut result: Vec<String> = vec![];
//         cpu.run_with_callback(|cpu| {
//             result.push(trace(cpu));
//         });
//         assert_eq!(
//             "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
//             result[0]
//         );
//     }
//
//
//
// }
