


pub struct ISAattr{
    pub call_opcode_len: Option<usize>,
    pub ret_opcode_len: Option<usize>,
}

//? Should be different for both call and ret opcodes
pub struct SearchParams{
    pub opcode_minlen: usize,
    pub opcode_maxlen: usize,
}
