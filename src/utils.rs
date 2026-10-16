use crate::binary::{Opcode, Instruction};
use std::cmp::min;


//checks if instruction starts with the given opcode
// ? currently assumes that the first bits are the opcode and that all later bits are the operands.
pub fn is_op(opcode:&Opcode, instruction:&Instruction) -> bool {
    let mut instruction_prefix: u128 = instruction.raw;
    instruction_prefix &= u128::MAX << (128 - opcode.opcode_len);

    return instruction_prefix == (opcode.opcode as u128) << 64;
}


// ? strips opcode from the start of the instruction
// ? can be sufficient as a way to isolate the operand, however in many cases (spesifically many VLIW cases) it may not be good enough
pub fn strip_opcode(mut opcodelen: usize, instruction:&Instruction) -> u128 {
    instruction.raw & (u128::MAX >> opcodelen)
}

// ! needs to determine if we should left align or right align the extracted subinstruction
// ! also needs better name than subinstruction
pub fn get_subinstruction(instruction:&Instruction, start_bit: usize, end_bit: usize) -> u128 {
    let mask: u128 = (u128::MAX >> start_bit) & (u128::MAX << (128 - end_bit));
    (instruction.raw & mask) >> (128 - end_bit)
}




// ! rename
pub type blabla = fn(opcode: &Opcode, instruction: &Instruction) -> i32;



pub const test1:blabla = |opcode: &Opcode, instruction: &Instruction| -> i32 {
    let operands = strip_opcode(opcode.opcode_len, instruction);
    // TODO: signbit?
    // TODO: endianess?
    // TODO: shift back

    operands as i32
};


//assumes the first bit in the operand part is the sign bit
pub const direct_signed:blabla = |opcode: &Opcode, instruction: &Instruction| -> i32 {
    let mut operand = strip_opcode(opcode.opcode_len, instruction);

    let sign_bit = 1 << (128 - opcode.opcode_len - 1);
    let mut signed_operand:i32;

    if operand & sign_bit != 0 {
        operand ^= sign_bit;
        signed_operand = -(operand as i32);
    }
    signed_operand = operand as i32;
    //TODO: handle sign


    signed_operand
};


pub const direct_unsigned:blabla = |opcode: &Opcode, instruction: &Instruction| -> i32 {
    strip_opcode(opcode.opcode_len, instruction) as i32
};


//TODO: endianess?
//TODO: other idk?