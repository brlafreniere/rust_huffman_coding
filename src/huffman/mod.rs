#![allow(unused_variables)]

pub mod code;
pub mod util;
pub mod encoding;
pub mod io;

use std::env;
use std::io::IsTerminal;

use io::File;

pub struct App;

impl App {
    pub fn run() {
        Self::exit_if_no_piped_input();

        let args = Self::read_cli_args();
        let mode = Self::determine_mode(&args);

        Self::invoke_action(mode);
    }

    fn read_cli_args() -> Vec<String> { env::args().collect() }

    fn determine_mode(args: &Vec<String>) -> &str {
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

    fn exit_if_no_piped_input() {
        if std::io::stdin().is_terminal() {
            println!("Error: No piped input. You must provide some input via standard input.");
            std::process::exit(1);
        }
    }

    fn invoke_action(mode: &str) {
        if mode == "encode" {
            File::encode(std::io::stdin(), std::io::stdout());
        } else {
            File::decode(std::io::stdin(), std::io::stdout());
        }
    }
}
