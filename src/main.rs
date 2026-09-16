mod options;
use crate::options::SearchParams;
mod binary;
use crate::binary::{BinaryFile, Instruction};
mod search;
use crate::search::Search;
mod utils;

use std::{env, print};
use std::fs::File;
use std::io::Read;




// ! TEST PARAMETERS

const filepath:&str = "data/mips32(Bootlin)/lua_O0.elf";
const width:usize = 32;
const start_address:usize = 0x0003a0;
const code_size:usize = 0x0fc7e0;
const call_opcode_len:usize = 6;
const ret_opcode_len:usize = 32;
const offset:usize = 0x400000;




// ! TEST PARAMETERS

//Three main groups of arguments:
//1. the raw binary
//2. options for search
//3. known features/properties of the ISA/binary
fn main() {


    //TODO: load raw binary
    let bytes = std::fs::read(filepath).unwrap();

    let textsection = bytes[start_address..start_address+code_size].to_vec();




    //TODO: load parameters
    let search_params: SearchParams = SearchParams{
        opcode_minlen: call_opcode_len,
        opcode_maxlen: call_opcode_len,
    };

    //TODO: init BinaryFile
    let binary_file: BinaryFile = BinaryFile::new_fixed_width(textsection,start_address, start_address+code_size, width, offset);

    let test = binary_file.instructions[0].raw;

    //TODO: algorithm
    let mut search = Search::new(search_params, binary_file);
    search.start();

    println!("{:016x}", search.res_call_opcode.unwrap().opcode);

}
