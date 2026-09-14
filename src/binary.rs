


//? only .text or other sections as well?
pub struct BinaryFile{
    pub instructions: Vec<Instruction>,
    pub start_addr: usize,
    pub end_addr: usize,
}


pub struct Instruction {
    pub raw: u128, // Raw bytes of the instruction
    pub len: usize, // Length of instruction in bits
    pub addr: usize // Address of the instruction in the raw binary
}



pub struct Opcode {
    pub opcode: u64, //stored in the topmost bits
    pub opcode_len: usize,
}

// ?
pub trait InstructionModel {

}



impl BinaryFile {


    pub fn new_fixed_width(raw: Vec<u8>, start_addr: usize, end_addr: usize, instruction_width: usize) -> Self {


        // ! Not sure at all if this works
        let instructions: Vec<Instruction> = raw.chunks(instruction_width).enumerate().map(|(i, chunk)| Instruction {
            raw: {
                let mut b: u128 = 0;
                for (i, byte) in chunk.iter().enumerate() {
                    b |= (*byte as u128) << (120 - 8 * i);
                }
                b
            },
            len: instruction_width * 8,
            addr: start_addr + i * instruction_width,
        }).collect();

        Self {
            instructions,
            start_addr,
            end_addr,
        }

    }
}