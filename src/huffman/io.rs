use super::code::{Key, ByteSearch};
use super::util::{BitBuffer, Byte};

use std::io::{Read, Write, Seek, copy, BufReader};
use std::fs::File;
use uuid::Uuid;

const BUFFER_SIZE: usize = 1024;

pub struct Operation<R: Read, W: Write> {
    input: R,
    output: W,
    tempfile: File
}

impl<W: Write, R: Read> Operation<R, W> {
    pub fn new(mut input: R, output: W) -> Operation<R, W> {
        let tempfile = Self::save_input_to_tempfile(&mut input);
        Operation { input, output, tempfile }
    }

    pub fn encode(&mut self) {
        let mut encoder = BufferedEncoder::new(&mut self.tempfile, &mut self.output);
        encoder.run();
    }

    pub fn decode(&mut self) {
        let mut decoder = BufferedDecoder::new(&mut self.tempfile, &mut self.output);
        decoder.run();
    }

    fn save_input_to_tempfile(input: &mut R) -> std::fs::File {
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
        let key = Key::build_from(&mut input);

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
            let encoded_bits = self.key.encode_byte(byte.unwrap());

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
    output: W,
}

impl<R: Read, W: Write> BufferedDecoder<R, W> {
    pub fn new(mut input: R, output: W) -> BufferedDecoder<R, W> {
        let key = Key::deserialize_from(&mut input);

        BufferedDecoder { key, input, output }
    }

    pub fn run(&mut self) {
        let chunk = self.get_chunk();

        while chunk != None {
            let bits = self.bytes_to_bits(chunk.unwrap());
            let leftover = self.process_bits(bits);
        }
    }

    fn process_bits(&self, bits: [bool; BUFFER_SIZE]) -> Vec<bool> {
        let search = ByteSearch::new(&self.key);
        let byte_written = false;

        for bit in bits {
            let byte_written = self.add_bit_to_search(bit, search);
        }
    }

    fn add_bit_to_search(&self, bit: bool, search: ByteSearch) -> bool {
        let result = search.find_byte_at_next_bit(bit);

        match result {
            Ok(Some(byte)) => {
                self.output.write(&[byte]);
                search = ByteSearch::new(&self.key);

                true
            },
            Ok(None) => { false },
            Err(msg) => { }
        }
    }

    fn get_chunk(&mut self) -> Option<[u8; BUFFER_SIZE]> {
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        
        let bytes_read = self.input.read(&mut buffer)
            .expect("Error while reading data segment");

        match bytes_read {
            0 => None,
            x => Some(buffer)
        }
    }

    fn bytes_to_bits(&self, chunk: [u8; BUFFER_SIZE]) -> [bool; BUFFER_SIZE] {
        let mut bits: [bool; BUFFER_SIZE] = [false; BUFFER_SIZE];
        let mut index = 0;

        for byte in chunk {
            for bit in Byte::get_bits(byte) {
                bits[index] = bit;
                index += 1;
            }
        }

        return None;
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

