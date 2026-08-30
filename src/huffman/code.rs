use std::collections::{HashMap, BinaryHeap};
use std::cmp::Reverse;

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug)]
struct Node {
    byte: Option<u8>,
    left: Option<u16>,
    right: Option<u16>
}

#[derive(Debug)]
pub struct Key {
    nodes: Vec<Node>,
    root: u16
}

struct KeyBuilder;

impl Node {
    const LEFT_PRESENCE: u8  = 0b0000_0100;
    const VALUE_PRESENCE: u8 = 0b0000_0010;
    const RIGHT_PRESENCE: u8 = 0b0000_0001;

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
}

impl Key {
    pub fn build(in_file: &mut std::fs::File) -> Key {
        KeyBuilder::new(in_file)
    }

    pub fn encode(&self, byte: u8) -> Vec<bool> {
        Vec::from([true])
    }

    fn root(&self) -> &Node {
        & self.nodes[usize::from(self.root)]
    }
}

impl KeyBuilder {
    fn new(in_stream: impl std::io::Read) -> Key {
        let counts = KeyBuilder::count_frequencies(in_stream);
        let leaf_nodes = KeyBuilder::create_leaf_nodes(counts);
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
    mod node { use super::*;
        mod serialize { use super::*;
            #[test]
            fn test_node_with_only_byte() {
                let node = Node { byte: Some(b'a'), left: None, right: None };
                let bytes = node.serialize();

                assert_eq!(bytes[0], Node::VALUE_PRESENCE);
                assert_eq!(bytes[1], b'a');

                assert_eq!(bytes.len(), 2);
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
}
