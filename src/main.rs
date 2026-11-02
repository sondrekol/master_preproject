mod options;
use crate::options::SearchParams;
mod binary;
use crate::binary::{BinaryFile, Endianness};
mod search;
use crate::search::Search;
mod utils;

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
    
    /// Call opcode length(s) to search: one length ("6") or an inclusive range ("2-12" or "2..12").
    /// Every length in the range is searched in this one run.
    #[arg(short = 'l', long, value_parser = parse_len_range)]
    call_opcode_len: LenRange,
    
    #[arg(short, long)]
    ret_opcode_len: usize,
    
    #[arg(short, long)]
    offset: String,

    /// Byte order(s) to try when reading instruction slots: big, little or both
    #[arg(short, long, default_value = "both")]
    endian: String,
}

/// Inclusive range of opcode lengths (in bits)
#[derive(Clone, Copy, Debug)]
struct LenRange {
    min: usize,
    max: usize,
}

fn parse_len_range(s: &str) -> Result<LenRange, String> {
    let s = s.trim();
    let (min, max) = match s.split_once("..").or_else(|| s.split_once('-')) {
        Some((a, b)) => (a.trim(), b.trim().trim_start_matches('=')),
        None => (s, s),
    };
    let min: usize = min.parse().map_err(|_| format!("invalid opcode length '{min}'"))?;
    let max: usize = max.parse().map_err(|_| format!("invalid opcode length '{max}'"))?;
    if min == 0 || min > max {
        return Err(format!("opcode length range must satisfy 1 <= min <= max, got {min}-{max}"));
    }
    if max > 64 {
        return Err(format!("opcode length can be at most 64, got {max}"));
    }
    Ok(LenRange { min, max })
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
        opcode_minlen: args.call_opcode_len.min,
        opcode_maxlen: args.call_opcode_len.max,
    };

    // TODO: variable-width instruction sets. Until then, skip execution when no fixed width is given.
    let Some(fixed_width) = args.fixed_width else {
        println!("No --fixed-width given: variable-width search is not implemented yet, skipping.");
        return;
    };

    if args.call_opcode_len.max > fixed_width {
        eprintln!(
            "error: max call opcode length ({}) is larger than the instruction width ({})",
            args.call_opcode_len.max, fixed_width
        );
        std::process::exit(2);
    }

    let endians: Vec<Endianness> = match args.endian.to_lowercase().as_str() {
        "big" | "be" => vec![Endianness::Big],
        "little" | "le" => vec![Endianness::Little],
        "both" => vec![Endianness::Big, Endianness::Little],
        other => {
            eprintln!("error: --endian must be big, little or both (got '{other}')");
            std::process::exit(2);
        }
    };
    // one view of the code section per byte order
    let binaries: Vec<(Endianness, BinaryFile)> = endians.iter().map(|&e| {
        (e, BinaryFile::new_fixed_width(textsection.clone(), start_address, start_address+code_size, fixed_width, offset, e))
    }).collect();

    //TODO: algorithm
    let mut search = Search::new(search_params, binaries);
    search.start();

    match search.best {
        Some(best) => println!("BEST: {}", best),
        None => println!("BEST: none (empty search space)"),
    }

}
