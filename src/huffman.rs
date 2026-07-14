#![allow(unused_variables)]

use std::collections::{HashMap, BinaryHeap};
use std::cmp::Reverse;

use std::env;
use std::io::{self, IsTerminal};

use uuid::Uuid;

pub struct App;

struct File;

struct Key {
    nodes: Vec<Node>,
    root: Option<usize>
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug)]
struct Node {
    weight: u32,
    value: Option<u8>,
    parent: Option<usize>,
    left: Option<usize>,
    right: Option<usize>
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

        let file = Self::read_input_to_file(&mut io::stdin());

        if encode_selected {
            File::encode(&mut io::stdin(), &mut io::stdout());
        } else {
            File::decode(io::stdin(), io::stdout());
        }
    }

    fn read_input_to_file(in_stream: &mut impl std::io::Read) -> std::fs::File {
        let id = Uuid::new_v4();
        let file = std::fs::File::create(format!("/tmp/{id}"))
            .expect("Failed to save input to temporary file.");

        return file;
    }
}

impl Node {
    pub fn new(weight: u32, value: Option<u8>) -> Node {
        Node {
            weight: weight,
            value: value,
            parent: None,
            left: None,
            right: None,
        }
    }
}

impl File {
    pub fn encode(in_stream: &mut impl std::io::Read, out_stream: &mut impl std::io::Write) {
        let key = Key::build_from_stream(in_stream);
    }

    pub fn decode(in_stream: impl std::io::Read, out_stream: impl std::io::Write) {

    }
}

impl Key {
    fn new() -> Key {
        Key {
            nodes: Vec::new(),
            root: None
        }
    }

    pub fn build_from_stream(in_stream: impl std::io::Read) {
        let mut key = Key::new();

        let counts = Key::count_frequencies(in_stream);

        key.nodes = Key::create_leaf_nodes(counts);
    }

    fn create_leaf_nodes(counts: HashMap<u8, u32>) -> Vec<Node> {
        let mut nodes = Vec::new();

        for (byte, freq) in counts {
            let node = Node::new(freq, Some(byte));
            nodes.push(node);
        }

        return nodes;
    }

    fn count_frequencies(mut in_stream: impl std::io::Read) -> HashMap<u8, u32> {
        let mut counts: HashMap<u8, u32> = HashMap::new();
        let mut buf = [0u8; 1024];
        let mut bytes_read;

        bytes_read = in_stream.read(&mut buf).unwrap();

        while bytes_read > 0 {
            for byte in buf {
                if byte == 0 { continue; }
                counts.entry(byte)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }

            bytes_read = in_stream.read(&mut buf).unwrap();
        }

        return counts;
    }
}

#[cfg(test)]
mod tests { use super::*;
    mod key { use super::*;
        mod count_frequencies { use super::*;
            #[test]
            fn test_produces_expected_counts() {
                let input = [b'a', b'a', b'a', b'a', b'b', b'c'];
                let counts = Key::count_frequencies(&input[..]);

                assert_eq!(counts.get(&b'a'), Some(&4));
                assert_eq!(counts.get(&b'b'), Some(&1));
                assert_eq!(counts.get(&b'c'), Some(&1));
            }

            #[test]
            fn test_does_not_count_null_bytes() {
                let input = [b'a', b'a', b'a', b'a', b'b', b'c'];
                let counts = Key::count_frequencies(&input[..]);

                assert_eq!(counts.get(&0), None);
            }
        }

        mod create_leaf_nodes { use super::*;
            #[test]
            fn test_returns_expected_output() {
                let input = HashMap::from([
                    (b'a', 5),
                    (b'b', 10)
                ]);

                let mut nodes = Key::create_leaf_nodes(input);

                let node = nodes.pop().unwrap();
                assert_eq!(node.value, Some(b'a'));
                assert_eq!(node.weight, 5);

                let node = nodes.pop().unwrap();
                assert_eq!(node.value, Some(b'b'));
                assert_eq!(node.weight, 10);
            }
        }
    }
}
