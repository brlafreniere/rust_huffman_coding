#![allow(unused_variables)]

pub mod code;
pub mod util;
pub mod io;

use std::env;
use std::io::IsTerminal;

use io::Operation;

pub struct App;

impl App {
    pub fn run() {
        Self::validate_input();

        let args = Self::read_program_arguments();
        let mode = Self::validate_arguments(&args);

        Self::invoke_action(mode);
    }

    fn validate_input() {
        if std::io::stdin().is_terminal() {
            println!("Error: No piped input. You must provide some input via standard input.");
            std::process::exit(1);
        }
    }

    fn read_program_arguments() -> Vec<String> { env::args().collect() }

    fn validate_arguments(args: &Vec<String>) -> &str {
        let encode_selected = args.iter().any(|a| a == "--encode");
        let decode_selected = args.iter().any(|a| a == "--decode");

        if !encode_selected && !decode_selected {
            println!("Must specify either --encode or --decode, you didn't specify either one!");
            std::process::exit(1);
        }

        if encode_selected && decode_selected {
            println!("You must specify only one of either --encode or --decode, not both!");
            std::process::exit(1);
        }

        if encode_selected {
            return "encode";
        } else {
            return "decode";
        }
    }

    fn invoke_action(mode: &str) {
        let input = std::io::stdin();
        let output = std::io::stdout();

        let mut operation = Operation::new(input, output);

        if mode == "encode" {
            operation.encode();
        } else {
            operation.decode();
        }
    }
}
