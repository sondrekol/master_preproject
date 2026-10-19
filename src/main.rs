mod options;
use crate::options::SearchParams;
mod binary;
use crate::binary::{BinaryFile, Instruction};
mod search;
use crate::search::Search;
mod utils;

use std::env;
use clap::Parser;

#[derive(Parser)]
#[command(name = "Algorithm")]
struct Command {
    #[arg(short, long)]
    path: String,
    
    #[arg(short, long)]
    fixed_width: Option<usize>,
    
    #[arg(short, long)]
    start_address: String,
    
    #[arg(short, long)]
    code_size: String,
    
    #[arg(short, long)]
    call_opcode_len: usize,
    
    #[arg(short, long)]
    ret_opcode_len: usize,
    
    #[arg(short, long)]
    offset: String,
}

//Three main groups of arguments:
//1. the raw binary
//2. options for search
//3. known features/properties of the ISA/binary
fn main() {
    let args = Command::parse();

    let start_address = usize::from_str_radix(&args.start_address.trim_start_matches("0x"), 16)
        .unwrap_or_else(|_| usize::from_str_radix(&args.start_address, 10).unwrap());
    let code_size = usize::from_str_radix(&args.code_size.trim_start_matches("0x"), 16)
        .unwrap_or_else(|_| usize::from_str_radix(&args.code_size, 10).unwrap());
    let offset = usize::from_str_radix(&args.offset.trim_start_matches("0x"), 16)
        .unwrap_or_else(|_| usize::from_str_radix(&args.offset, 10).unwrap());



    //TODO: load raw binary
    let bytes = std::fs::read(&args.path).unwrap();

    let textsection = bytes[start_address..start_address+code_size].to_vec();

    //TODO: load parameters
    let search_params: SearchParams = SearchParams{
        opcode_minlen: args.call_opcode_len,
        opcode_maxlen: args.call_opcode_len,
    };

    // TODO: variable-width instruction sets. Until then, skip execution when no fixed width is given.
    let Some(fixed_width) = args.fixed_width else {
        println!("No --fixed-width given: variable-width search is not implemented yet, skipping.");
        return;
    };

    let binary_file: BinaryFile = BinaryFile::new_fixed_width(textsection, start_address, start_address+code_size, fixed_width, offset);

    //TODO: algorithm
    let mut search = Search::new(search_params, binary_file);
    search.start();

    match search.best {
        Some(best) => println!(
            "BEST: {} score={} occurrences={} valid_targets={}",
            best.point, best.call_score, best.occurrences, best.valid_targets
        ),
        None => println!("BEST: none (empty search space)"),
    }

}
