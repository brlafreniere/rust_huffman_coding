#![allow(unused_variables)]

use std::collections::{HashMap, BinaryHeap};
use std::cmp::Reverse;

use std::env;
use std::io::{self, IsTerminal, copy, Read, Write};

use uuid::Uuid;

pub struct App;

struct File;

struct Key {
    nodes: Vec<Node>,
    root_idx: Option<usize>
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug)]
struct Node {
    weight: u32,
    value: Option<u8>,
    index: usize,
    parent_idx: Option<usize>,
    left_idx: Option<usize>,
    right_idx: Option<usize>
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

impl Node {
    pub fn new(weight: u32, value: Option<u8>, index: usize) -> Node {
        Node {
            weight: weight,
            value: value,
            index: index,
            parent_idx: None,
            left_idx: None,
            right_idx: None,
        }
    }
}

impl File {
    pub fn encode<R: Read, W: Write>(input: &mut R, output: &mut W) {
        let id = Uuid::new_v4();

        let mut temp_file = std::fs::File::create(format!("/tmp/{id}"))
            .expect("Failed to create temporary file.");

        copy(input, &mut temp_file)
            .expect("Failed to save input to temporary file.");

        let key = Key::build_from_stream(temp_file);
    }

    pub fn decode(in_stream: impl std::io::Read, out_stream: impl std::io::Write) {

    }
}

impl Key {
    fn new() -> Key {
        Key {
            nodes: Vec::new(),
            root_idx: None
        }
    }

    pub fn build_from_stream(in_stream: impl std::io::Read) -> Key {
        let mut key = Key::new();

        let counts = Key::count_frequencies(in_stream);
        let leaf_nodes = Key::create_leaf_nodes(counts);
        let next_index = leaf_nodes.len();
        let queue = Key::queue_nodes(leaf_nodes);
        let (root_idx, nodes) = Key::assemble_tree(queue, next_index);

        key.root_idx = Some(root_idx);
        key.nodes = nodes;

        return key;
    }

    fn create_leaf_nodes(counts: HashMap<u8, u32>) -> Vec<Node> {
        let mut nodes = Vec::new();

        for (i, (byte, freq)) in counts.into_iter().enumerate() {
            let node = Node::new(freq, Some(byte), i);
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

    fn queue_nodes(nodes: Vec<Node>) -> BinaryHeap<Reverse<Node>> {
        let mut queue: BinaryHeap<Reverse<Node>> = BinaryHeap::new();

        for node in nodes {
            queue.push(Reverse(node));
        }

        return queue;
    }

    fn assemble_tree(mut queue: BinaryHeap<Reverse<Node>>, mut next_index: usize) -> (usize, Vec<Node>) {
        // The thinking here is to have a place to put the new nodes to live.
        let mut output_nodes = Vec::new();

        let first_parent = next_index;

        // This loop will take the two next nodes, and create a new parent for these two nodes, and
        // link them by index.
        while queue.len() > 1 {
            let mut left = queue.pop().unwrap().0;
            let mut right = queue.pop().unwrap().0;

            let weight = left.weight + right.weight;

            let mut parent = Node::new(weight, None, next_index);

            parent.left_idx = Some(left.index);
            parent.right_idx = Some(right.index);

            left.parent_idx = Some(parent.index);
            right.parent_idx = Some(parent.index);

            output_nodes.push(left);
            output_nodes.push(right);

            queue.push(Reverse(parent));

            next_index += 1;
        }

        // last node is root
        let root = queue.pop().unwrap().0;
        let root_index = root.index;

        return (root_index, output_nodes);
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
                    (b'b', 10),
                    (b'c', 20)
                ]);

                let output_nodes = Key::create_leaf_nodes(input);

                let n1 = output_nodes.iter().find(|node| node.value == Some(b'a')).unwrap();
                assert_eq!(n1.weight, 5);

                let n2 = output_nodes.iter().find(|node| node.value == Some(b'b')).unwrap();
                assert_eq!(n2.weight, 10);

                let n3 = output_nodes.iter().find(|node| node.value == Some(b'c')).unwrap();
                assert_eq!(n3.weight, 20);
            }
        }

        mod queue_nodes { use super::*;
            #[test]
            fn test_returns_expected_output() {
                let nodes = Vec::from([
                    Node::new(5, Some(b'a'), 0),
                    Node::new(10, Some(b'b'), 1)
                ]);

                let mut queue = Key::queue_nodes(nodes);

                let n1 = queue.pop().unwrap().0;
                assert_eq!(n1.weight, 5);
                assert_eq!(n1.value, Some(b'a'));

                let n1 = queue.pop().unwrap().0;
                assert_eq!(n1.weight, 10);
                assert_eq!(n1.value, Some(b'b'));
            }
        }
    }
}
