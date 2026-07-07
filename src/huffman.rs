#![allow(unused_variables)]

use std::collections::HashMap;
use std::env;
use std::io::{self, IsTerminal};

pub struct App;

pub struct File;

pub struct Key {
    counts: HashMap<u8, i32>
}

impl App {
    pub fn run() {
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
            File::encode(&mut io::stdin(), &mut io::stdout());
        } else {
            File::decode(io::stdin(), io::stdout());
        }
    }
}

impl File {
    pub fn encode(in_stream: &mut impl std::io::Read, out_stream: &mut impl std::io::Write) {
        let key = Key::from_stream(in_stream);
    }

    pub fn decode(in_stream: impl std::io::Read, out_stream: impl std::io::Write) {

    }
}

impl Key {
    /// Builds an instance of a Key from raw input.
    pub fn create(in_stream: &mut impl std::io::Read) -> Key {
        let mut key = Key {
            counts: HashMap::new()
        };

        key.generate_counts(in_stream);

        return key;
    }

    /// Load a previously persisted Key from bytes.
    pub fn load(in_stream: &mut impl std::io::Read) -> Key {

    }

    fn generate_counts(&mut self, in_stream: &mut impl std::io::Read) {
        let mut buf = [0u8; 1024];
        let mut bytes_read;

        bytes_read = in_stream.read(&mut buf).unwrap();

        #[cfg(debug_assertions)]
        println!("bytes_read: {bytes_read}");

        while bytes_read > 0 {
            for chr in buf {
                self.counts.entry(chr)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }

            bytes_read = in_stream.read(&mut buf).unwrap();

            #[cfg(debug_assertions)]
            println!("bytes_read: {bytes_read}");
        }

        #[cfg(debug_assertions)]
        dbg!(&self.counts);

    }
}
