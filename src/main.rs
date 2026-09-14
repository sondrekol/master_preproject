mod options;
use crate::options::SearchParams;
mod binary;
use crate::binary::{BinaryFile, Instruction};
mod search;
use crate::search::Search;
mod utils;

use std::env;

//Three main groups of arguments:
//1. the raw binary
//2. options for search
//3. known features/properties of the ISA/binary
fn main() {


    //TODO: load raw binary
    
    //TODO: load parameters
    let search_params: SearchParams = SearchParams{
        opcode_minlen: 4,
        opcode_maxlen: 10,
    };

    //TODO: init BinaryFile
    let binary_file: BinaryFile = BinaryFile {
        instructions: Vec::new(),
        start_addr: 0,
        end_addr: 0,
    };

    //TODO: algorithm
    let search = Search::new(search_params, binary_file);
    search.start();
}
