use crate::binary::{Opcode, Instruction};
use std::cmp::min;


//checks if instruction starts with the given opcode
// ? currently assumes that the first bits are the opcode and that all later bits are the operands.
pub fn is_op(opcode:&Opcode, instruction:&Instruction) -> bool {
    // ? rewrite?
    let mut instruction_prefix: u64 = 0;
    for i in 0..min((opcode.opcode_len+7)/8, instruction.bytes.len()){
        instruction_prefix ^= (instruction.bytes[i] as u64) << (56-8*i);
    }
    instruction_prefix &= 0xFFFFFFFFFFFFFFFF << (64 - opcode.opcode_len);

    return instruction_prefix == opcode.opcode;
}


// ? strips opcode from the start of the instruction
// ? can be sufficient as a way to isolate the operand, however in many cases (spesifically many VLIW cases) it may not be good enough
pub fn strip_opcode(mut opcodelen: usize, instruction:&Instruction) -> Vec<u8> {
    let mut res:Vec<u8> = instruction.bytes.clone();
    let mut i:usize = 0;
    while opcodelen >= 8 {
        res[i] = 0;
        opcodelen -= 8;
        i += 1;
    }
    res[i] &= 0xFF >> (8 - opcodelen);

    return res;
}



type OperandExtractor = fn(opcode: &Opcode, instruction: &Instruction) -> Vec<u8>;


const STRIP_OPCODE_EXTRACTOR: OperandExtractor = |opcode: &Opcode, instruction: &Instruction| {
    strip_opcode(opcode.opcode_len, instruction)
};

