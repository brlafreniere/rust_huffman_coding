use std::collections::{HashMap, BinaryHeap, VecDeque};
use std::cmp::Reverse;
use std::io::{Read, Write};

use super::util::{Byte, BitBuffer};

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct Node {
    byte: Option<u8>,
    left: Option<u16>,
    right: Option<u16>
}
impl Node {
    const LEFT_PRESENCE: u8  = 0b0000_0100;
    const VALUE_PRESENCE: u8 = 0b0000_0010;
    const RIGHT_PRESENCE: u8 = 0b0000_0001;

    // There will only be two types of nodes: 
    // 1. Leaf nodes which have a byte value, and no children.
    // 2. Parent nodes which have no byte value, and just connect child nodes.
    // 
    // So we only need to test those two cases.
    pub fn serialize(&self) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::new();

        let mut presence: u8 = 0;
        let mut l1: u8 = 0;
        let mut l2: u8 = 0;
        let mut val: u8 = 0;
        let mut r1: u8 = 0;
        let mut r2: u8 = 0;

        if self.left.is_some() {
            presence |= Self::LEFT_PRESENCE;

            l1 = (self.left.unwrap() >> 8) as u8;
            l2 = (self.left.unwrap() & 0xff) as u8;
        }

        if self.byte.is_some() {
            presence |= Self::VALUE_PRESENCE;

            val = self.byte.unwrap();
        }

        if self.right.is_some() {
            presence |= Self::RIGHT_PRESENCE;

            r1 = (self.right.unwrap() >> 8) as u8;
            r2 = (self.right.unwrap() & 0xff) as u8;
        }

        output.push(presence);

        if self.left.is_some() { output.push(l1); output.push(l2); }
        if self.byte.is_some() { output.push(val); }
        if self.right.is_some() { output.push(r1); output.push(r2); }

        return output;
    }

    pub fn deserialize(presence: u8, input: &mut impl Read) -> Node {
        let left = {
            if presence & Self::LEFT_PRESENCE > 0 { Some(Byte::get_u16(input)) }
            else { None }
        };

        let byte = {
            if presence & Self::VALUE_PRESENCE > 0 { Byte::get_u8(input) }
            else { None }
        };

        let right = {
            if presence & Self::RIGHT_PRESENCE > 0 { Some(Byte::get_u16(input)) }
            else { None }
        };

        Node { left, byte, right }
    }
}


#[derive(Debug)]
pub struct Key {
    nodes: Vec<Node>,
    root: u16
}
impl Key {
    pub fn build_from(input: impl Read) -> Key {
        KeyBuilder::new(input)
    }

    pub fn deserialize_from(mut input: impl Read) -> Key {
        KeyDeserializer::run(&mut input)
    }

    pub fn encode_byte(&self, byte: u8) -> Option<Vec<bool>> {
        ByteToBitsSearch::new(self).find_bit_seq_for(byte)
    }

    pub fn decode(&self, bits: Vec<bool>) -> Option<u8> {
        let mut current_node = self.root_node();

        for bit in bits {
            if bit == false {
                if current_node.left.is_some() {
                    current_node = &self.nodes[current_node.left.unwrap() as usize];
                } else {
                    return None;
                }
            } else if bit == true {
                if current_node.right.is_some() {
                    current_node = &self.nodes[current_node.right.unwrap() as usize];
                } else {
                    return None;
                }
            }
        }

        if current_node.byte.is_some() {
            return Some(current_node.byte.unwrap());
        } else {
            return None;
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut output = Vec::new();

        for node in &self.nodes {
            let mut node_bytes = node.serialize();
            output.append(&mut node_bytes);
        }

        return output;
    }

    pub fn root_node(&self) -> &Node {
        & self.nodes[usize::from(self.root)]
    }
}

struct KeyDeserializer;
impl KeyDeserializer {
    pub fn run(input: &mut impl Read) -> Key {
        let mut nodes: Vec<Node> = Vec::new();

        let key_length = Byte::get_u16(input);

        let mut key_data = input.take(key_length as u64);

        while let Some(node) = Self::get_next_node(&mut key_data) {
            nodes.push(node);
        }

        let root = (nodes.len() - 1) as u16;
        let key = Key { nodes: nodes, root: root };

        return key;
    }

    fn get_next_node(input: &mut impl Read) -> Option<Node> {
        let presence = Byte::get_u8(input);

        if presence.is_none() { return None }

        let presence = presence.unwrap();

        let node = Node::deserialize(presence, input);

        return Some(node);
    }
}

struct BitsToByteSearch<'a> {
    key: &'a Key,
    current_node_index: u16,
}

impl<'a> BitsToByteSearch<'a> {
    pub fn new(key: &'a Key) -> BitsToByteSearch<'a> {
        Self { key, current_node_index: key.root }
    }

    // Add 1 bit at a time.
    // If a byte is found => Ok(Some(u8))
    // If no byte found => Ok(None) // this signals to keep adding bits
    //
    // If you have walked off the tree => Err("bro")
    //   If this is the very last byte of input, then this is fine and expected
    //   Otherwise, you have some data corruption
    fn add_bit(&mut self, bit: bool) -> Result<Option<u8>, String> {
        let current_node = &self.key.nodes[self.current_node_index as usize];

        let next_node_index = match bit {
            false => current_node.left,
            true => current_node.right
        };

        let next_node = match next_node_index {
            Some(i) => &self.key.nodes[next_node_index.unwrap() as usize],
            // This means we have fallen off the tree... which, if we're toward the end of the data
            // segment, and we are on our last input byte, then this is expected and fine.
            // If we haven't reached the last byte... then there is a serious problem, possibly data
            // corruption.
            None => return Err(String::from("You have fallen off the tree bro"))
        };

        self.current_node_index = next_node_index.unwrap();

        if next_node.byte.is_some() {
            // This result says: we have found the byte, here it is!
            return Ok(Some(next_node.byte.unwrap()));
        } else {
            // This result says: we didn't find the byte you're looking for, but we haven't fallen
            // off the tree yet.
            return Ok(None);
        }
    }
}

type QueueEntry = (u16, Vec<bool>);
pub struct ByteToBitsSearch<'a> {
    key: &'a Key,
    queue: VecDeque<QueueEntry>
}

impl<'a> ByteToBitsSearch<'a> {
    pub fn new(key: &'a Key) -> ByteToBitsSearch<'a> {
        let mut query = Self { key, queue: VecDeque::new() };

        query.init_queue();

        return query;
    }

    fn find_bit_seq_for(&mut self, byte: u8) -> Option<Vec<bool>> {
        while self.queue.len() > 0 {
            let (index, bit_seq) = self.queue.pop_front().unwrap();
            let node = & self.key.nodes[usize::from(index)];

            if node.byte.is_some() && node.byte.unwrap() == byte {
                return Some(bit_seq)
            }

            if node.left.is_some() {
                let mut next_bit_seq = bit_seq.clone();
                next_bit_seq.push(false);

                let entry = (node.left.unwrap(), next_bit_seq);
                self.queue.push_back(entry);
            }

            if node.right.is_some() {
                let mut next_bit_seq = bit_seq.clone();
                next_bit_seq.push(true);

                let entry = (node.right.unwrap(), next_bit_seq);
                self.queue.push_back(entry);
            }
        }

        return None;
    }

    fn init_queue(&mut self) {
        let root_node: &Node = self.key.root_node();

        if root_node.left.is_some() {
            let left_entry = (
                root_node.left.unwrap(),
                Vec::from([false])
            );

            self.queue.push_back(left_entry);
        }

        if root_node.right.is_some() {
            let right_entry = (
                root_node.right.unwrap(),
                Vec::from([true])
            );

            self.queue.push_back(right_entry);
        }
    }
}

struct KeyBuilder;
impl KeyBuilder {
    fn new(in_stream: impl std::io::Read) -> Key {
        let counts = KeyBuilder::count_frequencies(in_stream);
        let leaf_nodes = KeyBuilder::create_leaf_nodes(counts);

        if leaf_nodes.len() == 0 {
            panic!("Cannot construct a key from empty input.");
        }

        let queue = KeyBuilder::build_queue(leaf_nodes);
        let (root_index, nodes) = KeyBuilder::process_queue(queue);
        return Key { nodes: nodes, root: root_index };
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

    fn create_leaf_nodes(counts: HashMap<u8, u32>) -> Vec<(u32, Node)> {
        let mut leaf_nodes = Vec::new();

        for (byte, freq) in counts.into_iter() {
            let leaf_node = Node {
                byte: Some(byte),
                left: None,
                right: None,
            };

            leaf_nodes.push((freq, leaf_node));
        }

        return leaf_nodes;
    }

    // Queue the nodes using a min-heap
    fn build_queue(nodes: Vec<(u32, Node)>) -> BinaryHeap<Reverse<(u32, Node)>> {
        let mut queue: BinaryHeap<Reverse<(u32, Node)>> = BinaryHeap::new();

        for node in nodes {
            queue.push(Reverse(node));
        }

        return queue;
    }

    fn process_queue(mut queue: BinaryHeap<Reverse<(u32, Node)>>) -> (u16, Vec<Node>) {
        let mut output_nodes = Vec::new();

        // 1. Pop two nodes
        // 2. Create a parent for the two nodes
        // 3. Set parent's indices
        // 4. Build final Node for two popped nodes
        // 5. Place final Nodes in output vector
        while queue.len() > 1 {
            let (left_weight, left) = queue.pop().unwrap().0;
            let (right_weight, right) = queue.pop().unwrap().0;

            let left_index = u16::try_from(output_nodes.len())
                .expect("Could not convert usize to u16; usize is possibly too large.");
            let right_index = u16::try_from(output_nodes.len() + 1)
                .expect("Could not convert usize to u16; usize is possibly too large.");

            let mut parent = Node {
                byte: None,
                left: Some(left_index),
                right: Some(right_index)
            };

            parent.left = Some(left_index);
            parent.right = Some(right_index);

            output_nodes.push(left);
            output_nodes.push(right);

            queue.push(Reverse((left_weight + right_weight, parent)));
        }

        // last node is root
        let (root_weight, root) = queue.pop().unwrap().0;

        let root_index = u16::try_from(output_nodes.len())
            .expect("Could not convert usize to u16; usize is possibly too large.");

        output_nodes.push(root);

        return (root_index, output_nodes);
    }
}

#[cfg(test)]
mod tests { use super::*;
    mod key { use super::*;
        mod encode_byte { use super::*;
            #[test]
            fn test_encode_byte_sample_input_1() {
                let input_str = "aaabbc";
                let mut input = VecDeque::new();

                for chr in input_str.bytes() { input.push_back(chr) }

                let key = Key::build_from(&mut input);

                // Since 'a' is the most common character, we should expect 'a' to encode to just
                // a '1' bit.

                let bits = key.encode_byte(b'a').unwrap();

                assert_eq!(bits, Vec::from([true]));

                let bits = key.encode_byte(b'b').unwrap();

                assert_eq!(bits, Vec::from([false, true]));

                let bits = key.encode_byte(b'c').unwrap();

                assert_eq!(bits, Vec::from([false, false]));
            }
        }
    }

    mod node { use super::*;
        mod serialize { use super::*;
            #[test]
            fn test_leaf_node() {
                let node = Node { byte: Some(b'a'), left: None, right: None };
                let bytes = node.serialize();

                assert_eq!(bytes[0], Node::VALUE_PRESENCE);
                assert_eq!(bytes[1], b'a');

                assert_eq!(bytes.len(), 2);
            }

            #[test]
            fn test_stem_node() {
                let node = Node { byte: None, left: Some(500), right: Some(400) };
                let bytes = node.serialize();

                assert_eq!(bytes[0], Node::LEFT_PRESENCE | Node::RIGHT_PRESENCE);
                // 500 = b1: 00000001, b2: 11110100
                assert_eq!(bytes[1], 0b0000_0001);
                assert_eq!(bytes[2], 0b1111_0100);
                // 400 = b1: 00000001, b2: 1001_0000
                assert_eq!(bytes[3], 0b0000_0001);
                assert_eq!(bytes[4], 0b1001_0000);

                assert_eq!(bytes.len(), 5);
            }
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

                let (weight, node) = output_nodes.iter().find(|tuple| tuple.1.byte == Some(b'a')).unwrap();
                assert_eq!(*weight, 5);

                let (weight, node) = output_nodes.iter().find(|tuple| tuple.1.byte == Some(b'b')).unwrap();
                assert_eq!(*weight, 10);

                let (weight, node) = output_nodes.iter().find(|tuple| tuple.1.byte == Some(b'c')).unwrap();
                assert_eq!(*weight, 20);
            }
        }

        mod build_queue { use super::*;
            #[test]
            fn test_returns_expected_output() {
                let nodes = Vec::from([
                    (5, Node { byte: Some(b'a'), left: None, right: None }),
                    (10, Node { byte: Some(b'b'), left: None, right: None }),
                ]);

                let mut queue = KeyBuilder::build_queue(nodes);

                let (weight, node) = queue.pop().unwrap().0;
                assert_eq!(weight, 5);
                assert_eq!(node.byte, Some(b'a'));

                let (weight, node) = queue.pop().unwrap().0;
                assert_eq!(weight, 10);
                assert_eq!(node.byte, Some(b'b'));
            }
        }

        mod process_queue { use super::*;
            #[test]
            fn test_returns_expected_output() {
                let queue: BinaryHeap<Reverse<(u32, Node)>> = BinaryHeap::from([
                    Reverse( (3, Node { byte: Some(b'a'), left: None, right: None }) ),
                    Reverse( (2, Node { byte: Some(b'b'), left: None, right: None }) ),
                    Reverse( (1, Node { byte: Some(b'c'), left: None, right: None }) ),
                ]);

                let (root_index, nodes) = KeyBuilder::process_queue(queue);

                let current = & nodes[usize::try_from(root_index).ok().unwrap()];
                let left = & nodes[usize::try_from(current.left.unwrap()).ok().unwrap()];
                let right = & nodes[usize::try_from(current.right.unwrap()).ok().unwrap()];

                assert_eq!(right.byte, Some(b'a'));
                assert_eq!(left.byte, None);

                // now we move to the left node
                
                let current = left;
                let left = & nodes[usize::try_from(current.left.unwrap()).ok().unwrap()];
                let right = & nodes[usize::try_from(current.right.unwrap()).ok().unwrap()];

                assert_eq!(left.byte, Some(b'c'));
                assert_eq!(right.byte, Some(b'b'));
            }
        }
    }

    mod key_deserializer { use super::*;
        #[test]
        fn test_run_with_one_node() {
            let mut input: VecDeque<u8> = VecDeque::new();

            let node_a = Node { byte: Some(b'a'), left: None, right: None };
            let node_b = Node { byte: Some(b'b'), left: None, right: None };
            let node_p = Node { byte: None, left: Some(0), right: Some(1) };

            let mut node_bytes: VecDeque<u8> = VecDeque::new();

            for byte in node_a.serialize() { node_bytes.push_back(byte) }
            for byte in node_b.serialize() { node_bytes.push_back(byte) }
            for byte in node_p.serialize() { node_bytes.push_back(byte) }

            // The first part of our simulated input will be a u16 representing the length of the
            // key in bytes
            Byte::write_u16(&mut input, node_bytes.len() as u16);

            for byte in node_bytes { input.push_back(byte); }

            let key = KeyDeserializer::run(&mut input);

            let node_a = &key.nodes[0];
            let node_b = &key.nodes[1];
            let node_p = &key.nodes[2];

            assert_eq!(node_a.byte, Some(b'a'));
            assert_eq!(node_b.byte, Some(b'b'));

            assert_eq!(node_p.left, Some(0));
            assert_eq!(node_p.right, Some(1));

            assert_eq!(key.root, 2);
        }
    }

    mod bits_to_byte_search { use super::*;
        #[test]
        fn test_add_bit() {
            let mut input = VecDeque::new();

            let mut a = VecDeque::from([b'a'; 3]);
            let mut b = VecDeque::from([b'b'; 2]);
            let mut c = VecDeque::from([b'c'; 1]);

            input.append(&mut a);
            input.append(&mut b);
            input.append(&mut c);

            let key = Key::build_from(input);

            let mut search = BitsToByteSearch::new(&key);
            let result = search.add_bit(true);
            assert_eq!(result, Ok(Some(b'a')));

            let mut search = BitsToByteSearch::new(&key);

            let result = search.add_bit(false);
            assert_eq!(result, Ok(None));

            let result = search.add_bit(false);
            assert_eq!(result, Ok(Some(b'c')));

            let mut search = BitsToByteSearch::new(&key);

            let result = search.add_bit(false);
            assert_eq!(result, Ok(None));

            let result = search.add_bit(true);
            assert_eq!(result, Ok(Some(b'b')));
        }
    }
}
