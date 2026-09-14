
use crate::options::SearchParams;
use crate::binary::{BinaryFile, Opcode, Instruction};
use crate::utils::{is_op, strip_opcode};

// ! Result Object?



// ? Alt det som har med call validity og gjøre er vell egentlig branch/jump validity
type CallAddressResolver = fn(opcode: &Opcode, binary: &BinaryFile, instruction: &Instruction) -> usize;



const IMM_ABS_VALID_ADDR: CallAddressResolver = |opcode: &Opcode, binary: &BinaryFile, instruction: &Instruction| {

    let operand_bytes = strip_opcode(opcode.opcode_len, instruction);

    let intrprt: AddressInterpreter = IMM_ABS_ADDRESS_INTERPRETER;
    let addr = intrprt(operand_bytes, binary, instruction);
    
    addr

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



// ? two distinct problems? 1 is interpreting the data to as a value, then the value could be anything from absolute 
type AddressInterpreter = fn(operand: Vec<u8>, binary: &BinaryFile, instruction: &Instruction) -> usize;


const IMM_ABS_ADDRESS_INTERPRETER: AddressInterpreter = |operand: Vec<u8>, binary: &BinaryFile, instruction: &Instruction| {
    let mut addr: usize = 0;
    for (i, byte) in operand.iter().enumerate().rev() {
        addr |= (*byte as usize) << (i * 8);
    }
    addr
};


fn check_call_validity(opcode: &Opcode, binary: &BinaryFile, resolve_address: &CallAddressResolver) -> f32 {
    let mut occurences:u32 = 0;
    let mut valid_adressing:u32 = 0;

    //There is got to be some smarter way than a linear search
    for instruction in &binary.instructions {
        if is_op(opcode, instruction) {
            occurences += 1;
            let address = resolve_address(opcode, binary, &instruction);
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
    res_call_opcode: Option<Opcode>,
    res_ret_opcode: Option<Opcode>,
}


impl Search {
    pub fn new(params: SearchParams, binary: BinaryFile) -> Self {
        Search { params, binary, res_call_opcode: None, res_ret_opcode: None }
    }

    pub fn start(&self){
        for call_len in self.params.opcode_minlen..self.params.opcode_maxlen+1 {
            for ret_len in self.params.opcode_minlen..self.params.opcode_maxlen+1 {
                for call_opcode in 0..(1 << call_len) {
                    //? might be possible with an early break here if call does not point to in program memory

                    check_call_validity(
                        &Opcode{opcode: call_opcode as u64, opcode_len: call_len}, 
                        &self.binary, 
                        &IMM_ABS_VALID_ADDR
                    );

                    for ret_opcode in 0..(1 << ret_len) {
                        //check validity
                        //check occurences?
                    }
                }
            }
        }
    }   
}   