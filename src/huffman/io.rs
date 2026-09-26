use super::code::{Key};
use super::util::{BitBuffer, Byte};

use std::io::{Read, Write, Seek, copy, BufReader};
use std::fs::File;
use uuid::Uuid;

pub struct Operation<R: Read, W: Write> {
    input: R,
    output: W,
    tempfile: File
}

impl<W: Write, R: Read> Operation<R, W> {
    pub fn new(mut input: R, output: W) -> Operation<R, W> {
        let tempfile = Self::copy_to_tempfile(&mut input);
        Operation { input, output, tempfile }
    }

    pub fn encode(&mut self) {
        let mut encoder = BufferedEncoder::new(&mut self.tempfile, &mut self.output);
        encoder.run();
    }

    pub fn decode(&mut self) {
        let decoder = BufferedDecoder::new(&mut self.input, &mut self.output);
        decoder.run();
    }

    fn copy_to_tempfile(input: &mut R) -> std::fs::File {
        let id = Uuid::new_v4();
        let path = format!("/tmp/{id}");

        let mut temp_file = std::fs::File::create(&path).unwrap();

        copy(input, &mut temp_file).unwrap();

        return std::fs::File::open(path).unwrap();
    }
}

pub struct BufferedEncoder<R: Read, W: Write> {
    key: Key,
    input: R,
    output: W
}

impl<R: Read + Seek, W: Write> BufferedEncoder<R, W> {
    pub fn new(mut input: R, output: W) -> BufferedEncoder<R, W> {
        let key = Key::build(&mut input);

        input.rewind()
            .expect("Failed to rewind input");

        BufferedEncoder { key, input, output }
    }

    pub fn run(&mut self) {
        self.write_key_segment();
        self.write_data_segment();
    }

    fn write_data_segment(&mut self) {
        let mut bit_buffer = BitBuffer::new(&mut self.output);

        let input_reader = BufReader::new(&mut self.input);

        for byte in input_reader.bytes() {
            let encoded_bits = self.key.encode(byte.unwrap());

            for bit in encoded_bits.unwrap() {
                bit_buffer.push(bit);
            }
        }

        bit_buffer.dump();
    }

    fn write_key_segment(&mut self) {
        let key_bytes = self.key.serialize();

        let key_length = key_bytes.len() as u16;

        Byte::write_u16(&mut self.output, key_length);

        self.output.write(&key_bytes[..])
            .expect("Failed to write key segment.");
    }
}

pub struct BufferedDecoder<R: Read, W: Write> {
    key: Key,
    input: R,
    output: W
}

impl<R: Read, W: Write> BufferedDecoder<R, W> {
    pub fn new(mut input: R, output: W) -> BufferedDecoder<R, W> {
        let key = Key::deserialize(&mut input);
        BufferedDecoder { key, input, output }
    }

    pub fn run(&self) {
        self.read_data_segment();
    }

    fn read_data_segment(&self) {

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_write_key_segment() {
        let mut input = Vec::new();
        let mut output = Vec::new();

        input.append(&mut Vec::from([b'd'; 100]));
        input.append(&mut Vec::from([b'k'; 50]));
        input.append(&mut Vec::from([b'm'; 10]));

        let mut cursor = Cursor::new(input);
        let mut encoder = BufferedEncoder::new(&mut cursor, &mut output);
        encoder.write_key_segment();

        // There should be 3 leaf nodes, and 2 stem nodes.
        //
        // * 1 leaf node has a byte value, and no left/right, thus 1 leaf node has 2 bytes.
        // * 1 stem node has both left and right indices, but no byte value, which is 5 bytes.
        //
        // 3 * 2 = 6 bytes.
        // 2 * 5 = 10 bytes.
        // total = 16
        // 
        // 2 bytes to indicate key length
        // 16 byte key
        // = 18 bytes written out
        assert_eq!(output.len(), 18);

        // First byte = 16, representing the 16 bytes used to serialize the key
        assert_eq!(output[0], 0b0000_0000);
        assert_eq!(output[1], 0b0001_0000);

        // The correct serialization of nodes is already tested elsewhere.
    }

    #[test]
    fn test_write_data_segment() {
        
    }
}

