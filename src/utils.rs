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

pub fn operand_reg(opcode: &Opcode, instruction: &Instruction, binary: &BinaryFile, window: usize) -> ([u128; 5], usize) { // ? might need to return multiple values to test?
    let instructions = &binary.instructions;
    let Some(instr_index) = binary.addr_to_index(instruction.addr) else {return ([0; 5], 0)}; // ! better error handling

    for i in 0..window {
        let move_candidate = &instructions[instr_index - i];
        //TODO: Figure out operand for the move candidate, and return either as a imm abs or imm rel
    }

    ([0; 5], 0)
}