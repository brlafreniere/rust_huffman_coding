use std::env;
use std::io::{self, IsTerminal};

mod huffman;

fn main() {
    let args: Vec<String> = env::args().collect();

    let encode_selected = args.iter().any(|a| a == "--encode");
    let decode_selected = args.iter().any(|a| a == "--decode");

    if !encode_selected && !decode_selected {
        println!("Must specify either --encode or --decode");
        std::process::exit(1);
    }

    if io::stdin().is_terminal() {
        println!("Error: No piped input. You must provide some input via standard input.");
        std::process::exit(1);
    }

    if encode_selected {
        huffman::File::encode(io::stdin(), io::stdout());
    } else {
        huffman::File::decode(io::stdin(), io::stdout());
    }
}
