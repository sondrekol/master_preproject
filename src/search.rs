use crate::options::SearchParams;
use crate::binary::{BinaryFile, Endianness, Opcode, Instruction};
use std::fmt;
use crate::utils::{is_op, strip_opcode, operand_imm};


// ? Alt det som har med call validity og gjøre er vell egentlig branch/jump validity
/// Turns a matched instruction into a target address. Signed (i128) so that
/// negative offsets and addresses below the code section cannot wrap around.
type CallAddressResolver = fn(opcode: &Opcode, instruction: &Instruction) -> i128;

/// Probability that the target lands inside the code section if the operand bits
/// were random (uniform over all 2^k values). Used as the chance baseline.
type ChanceModel = fn(opcode: &Opcode, instruction: &Instruction, binary: &BinaryFile) -> f64;


/// The k = instruction.len - opcode_len operand bits, as an unsigned number.

/// Number of operand bits.
fn operand_len(opcode: &Opcode, instruction: &Instruction) -> usize {
    instruction.len - opcode.opcode_len
}

/// The operand read as a k-bit two's complement number.
fn signed_operand(opcode: &Opcode, instruction: &Instruction) -> i128 {
    let k = operand_len(opcode, instruction);
    let a = operand_imm(opcode, instruction) as i128;
    if k == 0 {
        0
    } else if (a >> (k - 1)) & 1 == 1 {
        a - (1i128 << k)
    } else {
        a
    }
}

fn pc_relative(opcode: &Opcode, instruction: &Instruction, scale: i128) -> i128 {
    // ? relative to the instruction itself; some ISAs use PC+4 / PC+8, which only moves
    // ? the target by a few bytes and hardly changes whether it lands in the section
    instruction.addr as i128 + scale * signed_operand(opcode, instruction)
}

const PC_REL_ADDRESS_RESOLVE: CallAddressResolver = |opcode: &Opcode, instruction: &Instruction| pc_relative(opcode, instruction, 1);
const PC_REL_X2_ADDRESS_RESOLVE: CallAddressResolver = |opcode: &Opcode, instruction: &Instruction| pc_relative(opcode, instruction, 2);
const PC_REL_X4_ADDRESS_RESOLVE: CallAddressResolver = |opcode: &Opcode, instruction: &Instruction| pc_relative(opcode, instruction, 4);

const MIPS_ADDR_RESOLVE: CallAddressResolver = |opcode: &Opcode, instruction: &Instruction| {
    operand_imm(opcode, instruction) as i128 * 4
};

const ABS_IMM_ADDR_RESOLVE: CallAddressResolver = |opcode: &Opcode, instruction: &Instruction| {
    // ? byte/word/chunk adressing
    operand_imm(opcode, instruction) as i128
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


// ---- chance models -------------------------------------------------------

/// Count of integers x in [lo, hi].
fn count_between(lo: i128, hi: i128) -> i128 {
    if hi < lo { 0 } else { hi - lo + 1 }
}

fn floor_div(a: i128, b: i128) -> i128 {
    let q = a / b;
    if (a % b != 0) && ((a < 0) != (b < 0)) { q - 1 } else { q }
}

fn ceil_div(a: i128, b: i128) -> i128 {
    -floor_div(-a, b)
}

/// Random signed k-bit offset o (scaled): P(start <= pc + scale*o <= end).
fn pc_relative_chance(opcode: &Opcode, instruction: &Instruction, binary: &BinaryFile, scale: i128) -> f64 {
    let k = operand_len(opcode, instruction);
    let pc = instruction.addr as i128;
    let (start, end) = (binary.start_addr as i128, binary.end_addr as i128);
    if k == 0 {
        return if pc >= start && pc <= end { 1.0 } else { 0.0 };
    }
    let (omin, omax) = (-(1i128 << (k - 1)), (1i128 << (k - 1)) - 1);
    let lo = ceil_div(start - pc, scale).max(omin);
    let hi = floor_div(end - pc, scale).min(omax);
    count_between(lo, hi) as f64 / (1u128 << k) as f64
}

/// Random unsigned k-bit operand a: P(start <= scale*a <= end).
fn absolute_chance(opcode: &Opcode, instruction: &Instruction, binary: &BinaryFile, scale: i128) -> f64 {
    let k = operand_len(opcode, instruction);
    let (start, end) = (binary.start_addr as i128, binary.end_addr as i128);
    let lo = ceil_div(start, scale).max(0);
    let hi = floor_div(end, scale).min((1i128 << k) - 1);
    count_between(lo, hi) as f64 / (1u128 << k) as f64
}

const PC_REL_CHANCE: ChanceModel = |o: &Opcode, i: &Instruction, b: &BinaryFile| pc_relative_chance(o, i, b, 1);
const PC_REL_X2_CHANCE: ChanceModel = |o: &Opcode, i: &Instruction, b: &BinaryFile| pc_relative_chance(o, i, b, 2);
const PC_REL_X4_CHANCE: ChanceModel = |o: &Opcode, i: &Instruction, b: &BinaryFile| pc_relative_chance(o, i, b, 4);
const MIPS_CHANCE: ChanceModel = |o: &Opcode, i: &Instruction, b: &BinaryFile| absolute_chance(o, i, b, 4);
const ABS_IMM_CHANCE: ChanceModel = |o: &Opcode, i: &Instruction, b: &BinaryFile| absolute_chance(o, i, b, 1);


/// A named way of turning a call instruction into a target address, together with
/// how often a random operand would land in the code section anyway.
pub struct AddressResolver {
    pub name: &'static str,
    pub resolve: CallAddressResolver,
    pub chance: ChanceModel,
}

// Every address interpretation the search tries. Add new resolvers here and the
// ParamIterator picks them up automatically.
pub static ADDR_RESOLVERS: [AddressResolver; 5] = [
    AddressResolver { name: "mips_abs_word",  resolve: MIPS_ADDR_RESOLVE,         chance: MIPS_CHANCE },
    AddressResolver { name: "pc_relative",    resolve: PC_REL_ADDRESS_RESOLVE,    chance: PC_REL_CHANCE },
    AddressResolver { name: "pc_relative_x2", resolve: PC_REL_X2_ADDRESS_RESOLVE, chance: PC_REL_X2_CHANCE },
    AddressResolver { name: "pc_relative_x4", resolve: PC_REL_X4_ADDRESS_RESOLVE, chance: PC_REL_X4_CHANCE },
    AddressResolver { name: "abs_immediate",  resolve: ABS_IMM_ADDR_RESOLVE,      chance: ABS_IMM_CHANCE },
];

/// Byte orders the search tries.
pub static ENDIANNESSES: [Endianness; 2] = [Endianness::Big, Endianness::Little];


/// Outcome of checking one (opcode, resolver) pair against the binary.
#[derive(Clone, Copy, Debug)]
pub struct CallValidity {
    /// Number of instructions that start with the opcode
    pub occurrences: u32,
    /// How many of those resolve to an address inside the code section
    pub valid_targets: u32,
    /// How many would land inside the code section if the operand bits were random
    pub expected_valid: f64,
    /// Number of different operand values among the occurrences
    pub distinct_operands: u32,
}

impl CallValidity {
    /// Valid targets beyond what chance gives, weighted by how varied the operands are.
    ///
    /// * valid_targets - expected_valid: a high hit rate only counts when random operand
    ///   bits would not hit the code section just as often (short-range fields, or an
    ///   opcode so long that the operand is tiny, score ~0).
    /// * distinct_operands / occurrences: an address field takes many values; a field
    ///   that is (nearly) constant, like the operand of a nop, does not look like one.
    pub fn score(&self) -> f64 {
        if self.occurrences == 0 {
            return 0.0;
        }
        (self.valid_targets as f64 - self.expected_valid)
            * (self.distinct_operands as f64 / self.occurrences as f64)
    }
}

fn check_call_validity(opcode: &Opcode, binary: &BinaryFile, resolver: &AddressResolver) -> CallValidity {
    let mut occurences: u32 = 0;
    let mut valid_adressing: u32 = 0;
    let mut expected: f64 = 0.0;
    let mut operands: Vec<u128> = Vec::new();
    let (start, end) = (binary.start_addr as i128, binary.end_addr as i128);

    // ! There is got to be some smarter way than a linear search
    for instruction in &binary.instructions {
        if is_op(opcode, instruction) {
            occurences += 1;
            operands.push(operand_imm(opcode, instruction));
            let address = (resolver.resolve)(opcode, instruction);
            if address <= end && address >= start {
                valid_adressing += 1;
            }
            expected += (resolver.chance)(opcode, instruction, binary);
        }
    }
    operands.sort_unstable();
    operands.dedup();
    CallValidity {
        occurrences: occurences,
        valid_targets: valid_adressing,
        expected_valid: expected,
        distinct_operands: operands.len() as u32,
    }
}



/// One coordinate in the parameter space the search runs through.
// TODO: ret opcode
#[derive(Clone, Copy)]
pub struct SearchPoint {
    pub call_opcode: Opcode,
    pub addr_resolver: &'static AddressResolver,
    pub endian: Endianness,
}

impl fmt::Display for SearchPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.call_opcode.opcode_len;
        let value = self.call_opcode.opcode >> (64 - len);
        return write!(
            f,
            "call_opcode={:0width$b} (len={}, value=0x{:x}, raw={:016x}) addr_resolver={} endian={}",
            value, len, value, self.call_opcode.opcode, self.addr_resolver.name, self.endian,
            width = len
        );
    }
}

/// A search point together with its score.
#[derive(Clone, Copy)]
pub struct SearchResult {
    pub point: SearchPoint,
    pub call_score: f64,
    pub validity: CallValidity,
}

impl fmt::Display for SearchResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = &self.validity;
        return write!(
            f,
            "{} score={:.3} occurrences={} valid_targets={} expected_valid={:.1} distinct_operands={}",
            self.point, self.call_score, v.occurrences, v.valid_targets, v.expected_valid, v.distinct_operands
        );
    }
}


// TODO: Locking of certain parameters
/// Iterates over every combination of
///   endianness (ENDIANNESSES)
///   x call opcode length (opcode_minlen..=opcode_maxlen)
///   x call opcode value (0..2^len)
///   x address resolver (ADDR_RESOLVERS).
/// The resolver changes fastest, then the opcode value, then the length, then the byte order.
pub struct ParamIterator {
    min_len: usize,
    max_len: usize,
    endian_idx: usize,
    call_len: usize,
    call_opcode: u64,
    resolver_idx: usize,
}

impl ParamIterator {
    pub fn new(params: &SearchParams) -> Self {
        ParamIterator {
            min_len: params.opcode_minlen,
            max_len: params.opcode_maxlen,
            endian_idx: 0,
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
        if self.min_len == 0 || self.max_len > 64 || self.min_len > self.max_len
            || self.endian_idx >= ENDIANNESSES.len() {
            return None;
        }

        let point = SearchPoint {
            call_opcode: Opcode::new(self.call_opcode, self.call_len),
            addr_resolver: &ADDR_RESOLVERS[self.resolver_idx],
            endian: ENDIANNESSES[self.endian_idx],
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
                if self.call_len > self.max_len {
                    self.call_len = self.min_len;
                    self.endian_idx += 1;
                }
            }
        }

        Some(point)
    }
}




/// One view of the code section per byte order.
pub struct Search {
    params: SearchParams,
    binaries: Vec<(Endianness, BinaryFile)>,
    pub best: Option<SearchResult>,
}


impl Search {
    pub fn new(params: SearchParams, binaries: Vec<(Endianness, BinaryFile)>) -> Self {
        Search { params, binaries, best: None }
    }

    fn binary(&self, endian: Endianness) -> Option<&BinaryFile> {
        self.binaries.iter().find(|(e, _)| *e == endian).map(|(_, b)| b)
    }

    pub fn start(&mut self){
        // ? ret "opcode" should be a single constant value for the whole instruction, however, its more efficient to search for prefixes fully determine the instruction, instead of going trough the entire return
        // ? perhaps its even possible to pregenerate all possible ret opcodes, which are prefixes in which all remaining bits are fixed
        for point in ParamIterator::new(&self.params) {
            // byte orders that were not loaded (see --endian) are skipped
            let Some(binary) = self.binary(point.endian) else { continue };
            //? might be possible with an early break here if call does not point to in program memory
            let validity = check_call_validity(&point.call_opcode, binary, point.addr_resolver);
            let call_score = validity.score();

            // strictly better score wins; on a tie, more valid targets wins (iteration
            // order no longer decides)
            let is_better = match &self.best {
                None => true,
                Some(best) => call_score > best.call_score
                    || (call_score == best.call_score && validity.valid_targets > best.validity.valid_targets),
            };

            if is_better {
                let result = SearchResult { point, call_score, validity };
                println!("New best: {}", result);
                self.best = Some(result);
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

        // (2^2 + 2^3 + 2^4) opcodes x number of resolvers x byte orders
        assert_eq!(points.len(), (4 + 8 + 16) * ADDR_RESOLVERS.len() * ENDIANNESSES.len());

        let unique: HashSet<(u64, usize, &str, bool)> = points
            .iter()
            .map(|p| (p.call_opcode.opcode, p.call_opcode.opcode_len, p.addr_resolver.name,
                      p.endian == Endianness::Big))
            .collect();
        assert_eq!(unique.len(), points.len());
    }

    fn insn(raw_value: u128, len: usize, addr: usize) -> Instruction {
        Instruction { raw: raw_value << (128 - len), len, addr }
    }

    #[test]
    fn pc_relative_is_twos_complement() {
        // 6-bit opcode, 26-bit operand = -1 -> target = pc - 4 with scale 4
        let op = Opcode::new(0b010010, 6);
        let i = insn((0b010010u128 << 26) | 0x3FF_FFFF, 32, 0x1000);
        assert_eq!(PC_REL_X4_ADDRESS_RESOLVE(&op, &i), 0x1000 - 4);
        // -4 as a 26-bit field
        let i = insn((0b010010u128 << 26) | (0x400_0000 - 4), 32, 0x1000);
        assert_eq!(PC_REL_ADDRESS_RESOLVE(&op, &i), 0x1000 - 4);
        // positive offset
        let i = insn((0b010010u128 << 26) | 0x10, 32, 0x1000);
        assert_eq!(PC_REL_ADDRESS_RESOLVE(&op, &i), 0x1010);
    }

    #[test]
    fn chance_model_matches_counting() {
        // code section 0x1000..=0x1100, 4-bit operand (12-bit opcode in a 16-bit slot)
        let b = BinaryFile { instructions: vec![], start_addr: 0x1000, end_addr: 0x1100,
                             fixed_width: Some(16), offset: 0 };
        let op = Opcode::new(0, 12);
        let i = insn(0, 16, 0x1000);
        // offsets -8..=7 from 0x1000: 0..=7 land inside -> 8/16
        assert!((PC_REL_CHANCE(&op, &i, &b) - 0.5).abs() < 1e-12);
        // absolute 0..=15 never reaches 0x1000
        assert_eq!(ABS_IMM_CHANCE(&op, &i, &b), 0.0);
    }
}
