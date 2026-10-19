use crate::options::SearchParams;
use crate::binary::{BinaryFile, Opcode, Instruction};
use std::fmt;
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

    let mut addr = operand_bytes >> (128 - instruction.len);

    let addr_len = instruction.len - opcode.opcode_len;

    if addr & (1 << (addr_len - 1)) != 0 {
        addr ^= 1 << (addr_len - 1);
        return instruction.addr - addr as usize;
    }
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

/// A named way of turning a call instruction into a target address.
pub struct AddressResolver {
    pub name: &'static str,
    pub resolve: CallAddressResolver,
}

// Every address interpretation the search tries. Add new resolvers here and the
// ParamIterator picks them up automatically.
pub static ADDR_RESOLVERS: [AddressResolver; 3] = [
    AddressResolver { name: "mips_abs_word", resolve: MIPS_ADDR_RESOLVE },
    AddressResolver { name: "pc_relative",   resolve: PC_REL_ADDRESS_RESOLVE },
    AddressResolver { name: "abs_immediate", resolve: ABS_IMM_ADDR_RESOLVE },
];


/// Outcome of checking one (opcode, resolver) pair against the binary.
#[derive(Clone, Copy, Debug)]
pub struct CallValidity {
    /// Number of instructions that start with the opcode
    pub occurrences: u32,
    /// How many of those resolve to an address inside the code section
    pub valid_targets: u32,
}

impl CallValidity {
    pub fn score(&self) -> f32 {
        if self.occurrences > 0 {
            self.valid_targets as f32 / self.occurrences as f32
        } else {
            0.0
        }
    }
}

fn check_call_validity(opcode: &Opcode, binary: &BinaryFile, resolve_address: &CallAddressResolver) -> CallValidity {
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
    CallValidity { occurrences: occurences, valid_targets: valid_adressing }
}



/// One coordinate in the parameter space the search runs through.
// TODO: ret opcode, endianess
#[derive(Clone, Copy)]
pub struct SearchPoint {
    pub call_opcode: Opcode,
    pub addr_resolver: &'static AddressResolver,
}

impl fmt::Display for SearchPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.call_opcode.opcode_len;
        let value = self.call_opcode.opcode >> (64 - len);
        write!(
            f,
            "call_opcode={:0width$b} (len={}, value=0x{:x}, raw={:016x}) addr_resolver={}",
            value, len, value, self.call_opcode.opcode, self.addr_resolver.name,
            width = len
        )
    }
}

/// A search point together with its score.
#[derive(Clone, Copy)]
pub struct SearchResult {
    pub point: SearchPoint,
    pub call_score: f32,
    /// Number of instructions in the binary that start with the call opcode
    pub occurrences: u32,
    /// How many of those occurrences resolved to an address inside the code section
    pub valid_targets: u32,
}


// TODO: Locking of certain parameters
/// Iterates over every combination of
///   call opcode length (opcode_minlen..=opcode_maxlen)
///   x call opcode value (0..2^len)
///   x address resolver (ADDR_RESOLVERS).
/// The resolver changes fastest, then the opcode value, then the length.
pub struct ParamIterator {
    max_len: usize,
    call_len: usize,
    call_opcode: u64,
    resolver_idx: usize,
}

impl ParamIterator {
    pub fn new(params: &SearchParams) -> Self {
        ParamIterator {
            max_len: params.opcode_maxlen,
            call_len: params.opcode_minlen,
            call_opcode: 0,
            resolver_idx: 0,
        }
    }
}

impl Iterator for ParamIterator {
    type Item = SearchPoint;

    fn next(&mut self) -> Option<Self::Item> {
        // Opcode::new shifts the opcode into a u64, so the length must be 1..=64
        if self.call_len == 0 || self.call_len > self.max_len || self.call_len > 64 {
            return None;
        }

        let point = SearchPoint {
            call_opcode: Opcode::new(self.call_opcode, self.call_len),
            addr_resolver: &ADDR_RESOLVERS[self.resolver_idx],
        };

        // Advance to the next coordinate
        self.resolver_idx += 1;
        if self.resolver_idx == ADDR_RESOLVERS.len() {
            self.resolver_idx = 0;
            self.call_opcode += 1;
            // Number of opcodes of this length, written so that len=64 cannot overflow
            let count_exhausted = self.call_len == 64 && self.call_opcode == 0
                || self.call_len < 64 && self.call_opcode >= (1u64 << self.call_len);
            if count_exhausted {
                self.call_opcode = 0;
                self.call_len += 1;
            }
        }

        Some(point)
    }
}





pub struct Search {
    params: SearchParams,
    binary: BinaryFile,
    pub best: Option<SearchResult>,
}


impl Search {
    pub fn new(params: SearchParams, binary: BinaryFile) -> Self {
        Search { params, binary, best: None }
    }

    pub fn start(&mut self){
        // ? ret "opcode" should be a single constant value for the whole instruction, however, its more efficient to search for prefixes fully determine the instruction, instead of going trough the entire return
        // ? perhaps its even possible to pregenerate all possible ret opcodes, which are prefixes in which all remaining bits are fixed
        for point in ParamIterator::new(&self.params) {
            //? might be possible with an early break here if call does not point to in program memory
            let validity = check_call_validity(
                &point.call_opcode,
                &self.binary,
                &point.addr_resolver.resolve,
            );

            let call_score = validity.score();

            let is_better = match &self.best {
                None => true,
                Some(best) => call_score >= best.call_score,
            };

            if is_better {
                self.best = Some(SearchResult {
                    point,
                    call_score,
                    occurrences: validity.occurrences,
                    valid_targets: validity.valid_targets,
                });
                println!(
                    "New best: {} score={} occurrences={} valid_targets={}",
                    point, call_score, validity.occurrences, validity.valid_targets
                );
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn param_iterator_covers_space_once() {
        let params = SearchParams { opcode_minlen: 2, opcode_maxlen: 4 };
        let points: Vec<SearchPoint> = ParamIterator::new(&params).collect();

        // (2^2 + 2^3 + 2^4) opcodes x number of resolvers
        assert_eq!(points.len(), (4 + 8 + 16) * ADDR_RESOLVERS.len());

        let unique: HashSet<(u64, usize, &str)> = points
            .iter()
            .map(|p| (p.call_opcode.opcode, p.call_opcode.opcode_len, p.addr_resolver.name))
            .collect();
        assert_eq!(unique.len(), points.len());
    }
}
