use crate::binary::{BinaryFile, Instruction, Opcode};


//checks if instruction starts with the given opcode
// ? currently assumes that the first bits are the opcode and that all later bits are the operands.
pub fn is_op(opcode:&Opcode, instruction:&Instruction) -> bool {
    let mut instruction_prefix: u128 = instruction.raw;
    instruction_prefix &= u128::MAX << (128 - opcode.opcode_len);

    return instruction_prefix == (opcode.opcode as u128) << 64;
}


// ? strips opcode from the start of the instruction
// ? can be sufficient as a way to isolate the operand, however in many cases (spesifically many VLIW cases) it may not be good enough
pub fn strip_opcode(opcodelen: usize, instruction:&Instruction) -> u128 {
    instruction.raw & (u128::MAX >> opcodelen)
}



pub fn operand_imm(opcode: &Opcode, instruction: &Instruction) -> u128 {
    let operand_bytes = strip_opcode(opcode.opcode_len, instruction);
    operand_bytes >> (128 - instruction.len)
}


pub fn operand_reg(opcode: &Opcode, instruction: &Instruction, binary: &BinaryFile) -> u128 {
    //TODO: 
    0
}