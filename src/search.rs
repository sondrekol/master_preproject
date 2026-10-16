use crate::options::SearchParams;
use crate::binary::{BinaryFile, Opcode, Instruction};
use crate::utils::{is_op, strip_opcode};


// ? Alt det som har med call validity og gjøre er vell egentlig branch/jump validity
type CallAddressResolver = fn(opcode: &Opcode, instruction: &Instruction) -> usize;


const MIPS_ADDR_RESOLVE: CallAddressResolver = |opcode: &Opcode, instruction: &Instruction| {
    let operand_bytes = strip_opcode(opcode.opcode_len, instruction); // ! need to makes this changeable

    let addr = operand_bytes >> (128 - instruction.len);

    
    (addr * 4) as usize
};

const PC_REL_ADDRESS_RESOLVE: CallAddressResolver = |opcode: &Opcode, instruction: &Instruction| {
    let operand_bytes = strip_opcode(opcode.opcode_len, instruction); // ! need to makes this changeable

    let addr = operand_bytes >> (128 - instruction.len);

    // ? byte/word/chunk adressing
    // ? sign bit?
    addr as usize + instruction.addr
};

const ABS_IMM_ADDR_RESOLVE: CallAddressResolver = |opcode: &Opcode, instruction: &Instruction| {

    let operand_bytes = strip_opcode(opcode.opcode_len, instruction); // ! need to makes this changeable

    let addr = operand_bytes >> (128 - instruction.len);

    // ? byte/word/chunk adressing
    addr as usize
};



const REG_ABS_ADDR_RESOLVE: CallAddressResolver = |opcode: &Opcode, instruction: &Instruction| {
    0
};

const REG_DISP_ADDR_RESOLVE: CallAddressResolver = |opcode: &Opcode, instruction: &Instruction| {
    0
};

const REG_REL_ADDR_RESOLVE: CallAddressResolver = |opcode: &Opcode, instruction: &Instruction| {
    0
};



// ! Likely not feasible, additionally, very few ISAs actually use memory addressing
const MEM_ADDR_RESOLVE: CallAddressResolver = |opcode: &Opcode, instruction: &Instruction| {
    0
};

const ADDR_RESOLVERS: [CallAddressResolver; 3] = [
    MIPS_ADDR_RESOLVE,
    PC_REL_ADDRESS_RESOLVE,
    ABS_IMM_ADDR_RESOLVE
];


fn check_call_validity(opcode: &Opcode, binary: &BinaryFile, resolve_address: &CallAddressResolver) -> f32 {
    let mut occurences:u32 = 0;
    let mut valid_adressing:u32 = 0;

    //There is got to be some smarter way than a linear search
    for instruction in &binary.instructions {
        if is_op(opcode, instruction) {
            occurences += 1;
            let address = resolve_address(opcode, &instruction);

            if address <= binary.end_addr && address >= binary.start_addr {
                valid_adressing += 1;
            }
        }
    }
    return if occurences > 0 {
        valid_adressing as f32 / occurences as f32
    } else {
        0.0
    };
}




pub struct ParamIterator {

}


// TODO: Search space iterator
// TODO: iterate over all possible parameter combinations: Call opcode(and length), ret opcode(and length), jump target formation method, and potentially endianess
// TODO: Locking of certain parameters
impl Iterator for ParamIterator {
    //TODO
    type Item = Opcode;

    fn next(&mut self) -> Option<Self::Item> {
        // Implement the logic for iterating over parameters here
        None
    }
}





pub struct Search {
    params: SearchParams,
    binary: BinaryFile,
    pub res_call_opcode: Option<Opcode>,
    pub res_ret_opcode: Option<Opcode>,

}


impl Search {
    pub fn new(params: SearchParams, binary: BinaryFile) -> Self {
        Search { params, binary, res_call_opcode: None, res_ret_opcode: None }
    }

    pub fn start(&mut self){
        let mut best:f32 = 0.0;

        // ! should have different min/max length for ret and call
        // ? ret "opcode" should be a single constant value for the whole instruction, however, its more efficient to search for prefixes fully determine the instruction, instead of going trough the entire return
        // ? perhaps its even possible to pregenerate all possible ret opcodes, which are prefixes in which all remaining bits are fixed
        for call_len in self.params.opcode_minlen..self.params.opcode_maxlen+1 {
            for ret_len in self.params.opcode_minlen..self.params.opcode_maxlen+1 {
                for call_opcode in 0..(1 << call_len) {
                    //? might be possible with an early break here if call does not point to in program memory

                    let call_score = check_call_validity(
                        &Opcode::new(call_opcode as u64, call_len),
                        &self.binary, 
                        &MIPS_ADDR_RESOLVE
                    );

                    if self.res_call_opcode.is_none() {
                        self.res_call_opcode = Some(Opcode::new(call_opcode as u64, call_len))
                    } else {
                        if call_score >= best {
                            best = call_score;
                            self.res_call_opcode = Some(Opcode::new(call_opcode as u64, call_len));
                            println!("New best call opcode found: {:016x} with score: {}", call_opcode, call_score);
                        }
                    }

                    for ret_opcode in 0..(1 << ret_len) {
                        //check validity
                        //check occurences?
                    }
                }
            }
        }
    }   
}   