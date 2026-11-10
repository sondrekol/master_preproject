


use std::fmt;

/// Byte order used to assemble each instruction slot before the opcode is read
/// from its top bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endianness {
    Big,
    Little,
}

impl fmt::Display for Endianness {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{}", match self { Endianness::Big => "big", Endianness::Little => "little" });
    }
}

//? only .text or other sections as well?
pub struct BinaryFile{
    pub instructions: Vec<Instruction>,
    pub start_addr: usize,
    pub end_addr: usize,
    pub fixed_width: Option<usize>,
    pub offset: usize,
}


pub struct Instruction {
    pub raw: u128, // Raw bytes of the instruction
    pub len: usize, // Length of instruction in bits
    pub addr: usize // Address of the instruction in the raw binary
}



#[derive(Clone, Copy, Debug)]
pub struct Opcode {
    pub opcode: u64, //stored in the topmost bits
    pub opcode_len: usize,
}

// ?
pub trait InstructionModel {

}



impl BinaryFile {


    /// Split `raw` into fixed-width slots. Each slot is read in byte order `endian`
    /// and stored left-aligned in `Instruction::raw` (first bit of the value = bit 127).
    pub fn new_fixed_width(raw: Vec<u8>, start_addr: usize, end_addr: usize, instruction_width: usize, offset: usize, endian: Endianness) -> Self {
        let width_bytes = instruction_width / 8;
        let instructions: Vec<Instruction> = raw.chunks(width_bytes).enumerate().map(|(i, chunk)| Instruction {
            raw: {
                // a short final chunk is padded with zero bytes at the end of the slot
                let mut bytes = chunk.to_vec();
                bytes.resize(width_bytes, 0);
                if endian == Endianness::Little {
                    bytes.reverse();
                }
                let mut b: u128 = 0;
                for (i, byte) in bytes.iter().enumerate() {
                    b |= (*byte as u128) << (120 - 8 * i);
                }
                b
            },
            len: instruction_width,
            addr: offset + start_addr + i * instruction_width/8,
        }).collect();

        Self {
            instructions,
            start_addr: start_addr + offset,
            end_addr: end_addr + offset,
            fixed_width: Some(instruction_width),
            offset,
        }

    }

    pub fn addr_to_index(&self, addr: usize) -> Option<usize> {
        if let Some(fixed_width) = self.fixed_width {
            if addr < self.start_addr || addr >= self.end_addr {
                return None;
            }
            Some((addr - self.start_addr) / (fixed_width / 8))
        } else {
            //TODO: Implement binary search
            None
        }
    }
}


impl Opcode {
    pub fn new(opcode: u64, opcode_len: usize) -> Self {
        Self {
            opcode:opcode << (64 - opcode_len),
            opcode_len,
        }
    }
}