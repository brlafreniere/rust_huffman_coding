## Encoding Process

1. Read file contents
2. Count each character in the file contents, store as HashMap<char, i32>
3. Turn each pair in the HashMap into a Node (single Node tree)
4. Add all of the Nodes to a priority queue, ordered by weight (BTreeMap)
5. Pop the two lowest weighted trees from the queue
6. Form a new tree with the two popped nodes as children of a newly created
   internal node.
  a. The newly formed internal node should have its weight be the sum of its
     children's weights.
  b. Place the newly formed parent node into the queue.
  c. Repeat until there is only 1 tree left in the queue.
5. Build a lookup table of codes from the Huffman Coding Tree.
  a. Follow each path on the tree, recursively, and once a leaf node is reached,
     add its path to a hash table, keyed by character it encodes.
5. Assemble a byte array (Vec<u8>) to be written to the output file first, the
   key segment.

### Building The Coding Key 



### Coding Key Storage

The key must be stored along with the encoded data, otherwise it wouldn't be
possible to decode.

Key data will be in this format:
<8 byte character><8 byte count><8 byte character><8 byte count>

The first byte of the encoded file will be an unsigned integer indicating the
length of the key data. It will be a number representing how many characters are
in the key.

For example, if there are 10 characters, the first byte will represent the
number 10, and there will be 10 <characters> * 2 <bytes per character> bytes of
data to read. Each character needs two bytes in the key. A byte for the
character, and a byte for its frequency.

Then the decoder can build a key from this data.

Key data won't be encoded.

## Writing Bytes to a File

Example: 

```
use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    let mut file = File::create("foo.txt")?;
    file.write_all(b"Hello, world!")?;
    Ok(())
}
```