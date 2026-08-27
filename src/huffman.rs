#![allow(unused_variables)]

use std::collections::{HashMap, BinaryHeap, VecDeque};
use std::cmp::Reverse;

use std::env;
use std::io::{self, IsTerminal, copy, Read, Write, Seek};

use uuid::Uuid;

pub struct App;

struct File;
struct BufferedEncoder;

#[derive(Debug)]
struct Key {
    nodes: Vec<Node>,
    root_idx: Option<usize>
}

struct KeyBuilder;

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug)]
struct Node {
    weight: u32,
    value: Option<u8>,
    index: Option<usize>,
    parent_idx: Option<usize>,
    left_idx: Option<usize>,
    right_idx: Option<usize>
}

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
        if io::stdin().is_terminal() {
            println!("Error: No piped input. You must provide some input via standard input.");
            std::process::exit(1);
        }
    }

    fn invoke_action(mode: &str) {
        if mode == "encode" {
            File::encode(&mut io::stdin(), &mut io::stdout());
        } else {
            File::decode(io::stdin(), io::stdout());
        }
    }
}

impl Node {
    pub fn new(weight: u32, value: Option<u8>) -> Node {
        Node {
            weight: weight,
            value: value,
            index: None,
            parent_idx: None,
            left_idx: None,
            right_idx: None,
        }
    }
}

impl File {
    pub fn encode<R: Read, W: Write>(input: &mut R, output: &mut W) {
        let tempfile_path = Self::copy_to_tempfile(input);
        let mut tempfile = Self::open_tempfile(&tempfile_path);

        let key = Key::build(&mut tempfile);
        tempfile.rewind().expect("Unable to rewind the temporary file that holds program input.");

        BufferedEncoder::run(&mut tempfile, output, key);
    }

    pub fn decode(in_stream: impl std::io::Read, out_stream: impl std::io::Write) {

    }

    fn open_tempfile(path: &String) -> std::fs::File {
        match std::fs::File::open(path) {
            Err(err) => { panic!("Unable to open tempfile for reading: {path}"); },
            Ok(result) => result
        }
    }

    fn copy_to_tempfile<R: Read>(input: &mut R) -> String {
        let id = Uuid::new_v4();
        let path = format!("/tmp/{id}");

        let mut temp_file = std::fs::File::create(&path)
            .expect("Failed to create temporary file.");

        copy(input, &mut temp_file)
            .expect("Failed to save input to temporary file.");

        return path;
    }
}

impl BufferedEncoder {
    const BUFFER_SIZE: usize = 1024;

    pub fn run<W: Write>(input: &mut std::fs::File, output: &mut W, key: Key) {
        let input_buffer: [u8; Self::BUFFER_SIZE] = [0; Self::BUFFER_SIZE];
        let mut bit_buffer: VecDeque<bool> = VecDeque::new();

        for byte in input_buffer {
            let result_bits = key.encode(byte);
            for bit in result_bits {
                if bit_buffer.len() < Self::BUFFER_SIZE {
                    bit_buffer.push_back(bit);
                } else {
                    // dump the buffer
                }
            }
        }
    }
}

impl KeyBuilder {
    fn new(in_stream: impl std::io::Read) -> Key {
        let mut key = Key::new();

        let counts = KeyBuilder::count_frequencies(in_stream);
        let leaf_nodes = KeyBuilder::create_leaf_nodes(counts);
        let queue = KeyBuilder::queue_nodes(leaf_nodes);
        let (root_idx, nodes) = KeyBuilder::assemble_tree(queue);

        key.root_idx = root_idx;
        key.nodes = nodes;

        return key;
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

    fn create_leaf_nodes(counts: HashMap<u8, u32>) -> Vec<Node> {
        let mut nodes = Vec::new();

        for (byte, freq) in counts.into_iter() {
            let node = Node::new(freq, Some(byte));
            nodes.push(node);
        }

        return nodes;
    }

    // Queue the nodes using a min-heap
    fn queue_nodes(nodes: Vec<Node>) -> BinaryHeap<Reverse<Node>> {
        let mut queue: BinaryHeap<Reverse<Node>> = BinaryHeap::new();

        for node in nodes {
            queue.push(Reverse(node));
        }

        return queue;
    }

    fn assemble_tree(mut queue: BinaryHeap<Reverse<Node>>) -> (Option<usize>, Vec<Node>) {
        // The thinking here is to have a place to put the new nodes to live.
        let mut output_nodes = Vec::new();

        // This loop will take the two next nodes, and create a new parent for these two nodes, and
        // link them by index.
        while queue.len() > 1 {
            let mut left = queue.pop().unwrap().0;
            let mut right = queue.pop().unwrap().0;

            left.index = Some(output_nodes.len());
            right.index = Some(output_nodes.len() + 1);

            let weight = left.weight + right.weight;

            let mut parent = Node::new(weight, None);

            parent.left_idx = left.index;
            parent.right_idx = right.index;

            left.parent_idx = parent.index;
            right.parent_idx = parent.index;

            // We need to make sure that the leaf nodes `index` attribute matches where they
            // actually end up in the vector. If a leaf node has index=5, but it's actually sitting
            // in index=7, that's a problem.
            output_nodes.push(left);
            output_nodes.push(right);

            queue.push(Reverse(parent));
        }

        // last node is root
        let mut root = queue.pop().unwrap().0;
        root.index = Some(output_nodes.len());
        let root_index = root.index;
        output_nodes.push(root);

        return (root_index, output_nodes);
    }
}

impl Key {
    fn new() -> Key {
        Key {
            nodes: Vec::new(),
            root_idx: None
        }
    }

    pub fn build(in_file: &mut std::fs::File) -> Key {
        KeyBuilder::new(in_file)
    }

    pub fn encode(&self, byte: u8) -> Vec<bool> {
        Vec::from([true])
    }

    fn root(&self) -> &Node {
        & self.nodes[self.root_idx.unwrap()]
    }
}

#[cfg(test)]
mod tests { use super::*;
    mod key { use super::*;
        #[test]
        fn test_root_returns_correct_node() {

        }
    }

    mod key_builder { use super::*;
        mod count_frequencies { use super::*;
            #[test]
            fn test_produces_expected_counts() {
                let input = [b'a', b'a', b'a', b'a', b'b', b'c'];
                let counts = KeyBuilder::count_frequencies(&input[..]);

                assert_eq!(counts.get(&b'a'), Some(&4));
                assert_eq!(counts.get(&b'b'), Some(&1));
                assert_eq!(counts.get(&b'c'), Some(&1));
            }

            #[test]
            fn test_does_not_count_null_bytes() {
                let input = [b'a', b'a', b'a', b'a', b'b', b'c'];
                let counts = KeyBuilder::count_frequencies(&input[..]);

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

                let output_nodes = KeyBuilder::create_leaf_nodes(input);

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
                    Node::new(5, Some(b'a')),
                    Node::new(10, Some(b'b'))
                ]);

                let mut queue = KeyBuilder::queue_nodes(nodes);

                let n1 = queue.pop().unwrap().0;
                assert_eq!(n1.weight, 5);
                assert_eq!(n1.value, Some(b'a'));

                let n1 = queue.pop().unwrap().0;
                assert_eq!(n1.weight, 10);
                assert_eq!(n1.value, Some(b'b'));
            }
        }

        mod assemble_tree { use super::*;
            #[test]
            fn test_returns_expected_output() {
                let queue: BinaryHeap<Reverse<Node>> = BinaryHeap::from([
                    Reverse(Node::new(3, Some(b'a'))),
                    Reverse(Node::new(2, Some(b'b'))),
                    Reverse(Node::new(1, Some(b'c'))),
                ]);

                let (root, nodes) = KeyBuilder::assemble_tree(queue);

                let current = & nodes[root.unwrap()];
                let left = & nodes[current.left_idx.unwrap()];
                let right = & nodes[current.right_idx.unwrap()];

                assert_eq!(current.weight, 6);
                assert_eq!(right.value.unwrap(), b'a');
                assert_eq!(left.value, None);

                // now we move to the left node
                
                let current = left;
                let left = & nodes[current.left_idx.unwrap()];
                let right = & nodes[current.right_idx.unwrap()];

                assert_eq!(current.weight, 3);
                assert_eq!(left.value.unwrap(), b'c');
                assert_eq!(right.value.unwrap(), b'b');
            }
        }
    }
}
