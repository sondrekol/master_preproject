use crate::options::SearchParams;
use crate::binary::{BinaryFile, Opcode, Instruction};
use crate::utils::{is_op, strip_opcode};

// ! Result Object?



// ? Alt det som har med call validity og gjøre er vell egentlig branch/jump validity
type CallAddressResolver = fn(opcode: &Opcode, binary: &BinaryFile, instruction: &Instruction) -> usize;

const IMM_ABS_VALID_ADDR: CallAddressResolver = |opcode: &Opcode, binary: &BinaryFile, instruction: &Instruction| {
    0
};

const MIPS_VALID_ADDR: CallAddressResolver = |opcode: &Opcode, binary: &BinaryFile, instruction: &Instruction| {
    let operand_bytes = strip_opcode(opcode.opcode_len, instruction); // ! need to makes this changeable

    let addr = operand_bytes >> (128 - instruction.len);

    
    (addr * 4) as usize
};

//? Where is the sign bit?
const IMM_REL_VALID_ADDR: CallAddressResolver = |opcode: &Opcode, binary: &BinaryFile, instruction: &Instruction| {
    0
};

const REG_ABS_VALID_ADDR: CallAddressResolver = |opcode: &Opcode, binary: &BinaryFile, instruction: &Instruction| {
    0
};

const REG_REL_VALID_ADDR: CallAddressResolver = |opcode: &Opcode, binary: &BinaryFile, instruction: &Instruction| {
    0
};

const CALL_TABLE_VALID_ADDR: CallAddressResolver = |opcode: &Opcode, binary: &BinaryFile, instruction: &Instruction| {
    0
};



fn check_call_validity(opcode: &Opcode, binary: &BinaryFile, resolve_address: &CallAddressResolver) -> f32 {
    let mut occurences:u32 = 0;
    let mut valid_adressing:u32 = 0;

    //There is got to be some smarter way than a linear search
    for instruction in &binary.instructions {
        if is_op(opcode, instruction) {
            occurences += 1;
            let address = resolve_address(opcode, binary, &instruction);

            // ! DEBUG STUFF
            if (opcode.opcode == 0x0c00000000000000){
                println!("address: {:016x}", address);
                println!("instruction raw: {:016x}", instruction.raw);
                println!("instruction addr: {:016x}", instruction.addr);
            }
            // ! DEBUG STUFF

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
        for call_len in self.params.opcode_minlen..self.params.opcode_maxlen+1 {
            for ret_len in self.params.opcode_minlen..self.params.opcode_maxlen+1 {
                for call_opcode in 0..(1 << call_len) {
                    //? might be possible with an early break here if call does not point to in program memory

                    let call_score = check_call_validity(
                        &Opcode::new(call_opcode as u64, call_len),
                        &self.binary, 
                        &MIPS_VALID_ADDR
                    );

                    if self.res_call_opcode.is_none() {
                        self.res_call_opcode = Some(Opcode::new(call_opcode as u64, call_len))
                    } else {
                        if call_score >= best {
                            best = call_score;
                            self.res_call_opcode = Some(Opcode::new(call_opcode as u64, call_len));
                            println!("best call opcode: {:08x} with call score {}", call_opcode as u64, call_score);
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