use std::fs;
use std::collections::HashMap;
use std::collections::BinaryHeap;
use std::i32;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Node<K, V> {
  pub value: V,
  pub key: Option<K>,
  pub left: Option<Box<Node<K, V>>>,
  pub right: Option<Box<Node<K, V>>>
}

pub struct Encoder {
  file_contents: String,
  frequencies: HashMap<char, i32>,
  queue: BinaryHeap<Node<char, i32>>,
  coding_key_root: Option<Node<char, i32>>,
  key_segment_bytes: Vec<u8>
}

impl Encoder {
  pub fn new(file_path: &String) -> Encoder {
    let file_contents = fs::read_to_string(file_path)
      .expect("Unable to open file.");
    let frequencies: HashMap<char, i32> = HashMap::new();
    let queue: BinaryHeap<Node<char, i32>> = BinaryHeap::new();

    return Encoder {
      file_contents: file_contents,
      frequencies: frequencies,
      queue: queue,
      coding_key_root: None,
      key_segment_bytes: Vec::new()
    };
  }

  pub fn encode(&mut self) {
    self.count_characters();
    self.init_queue();
    self.build_coding_key();
    self.build_key_segment_bytes();
  }

  fn count_characters(&mut self) {
    for c in self.file_contents.chars() {
      let insert_value = match self.frequencies.get(&c) {
        Some(count) => count + 1,
        None => 1
      };
      self.frequencies.insert(c, insert_value);
    }
  }

  // Takes the counts from 'frequencies' and creates Node objects, and loads
  // them into the priority queue by weight.
  fn init_queue(&mut self) {
    for (char_val, count) in self.frequencies.iter() {
      let node = Node::<char, i32> {
        key: Some(*char_val),
        value: *count,
        left: None,
        right: None
      };
      self.queue.push(node);
    }
  }

  fn build_coding_key(&mut self) {
    while self.queue.len() > 1 {
      let left = self.queue.pop();
      let right = self.queue.pop();

      let parent_weight = left.as_ref().unwrap().value + right.as_ref().unwrap().value;

      let parent = Node::<char, i32> {
        key: None,
        value: parent_weight,
        left: left.map(Box::new),
        right: right.map(Box::new)
      };

      self.queue.push(parent);
    }

    self.coding_key_root = self.queue.pop();
  }

  fn build_key_segment_bytes(&mut self) {

  }
}

struct Decoder {
  file_contents: String
}

impl Decoder {

}