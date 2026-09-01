use std::io::{Write, BufReader, Read};
use std::collections::VecDeque;

use super::code::Key;
use super::util::BitBuffer;

pub struct BufferedEncoder<W: Write> {
    input: std::fs::File,
    output: W,
    key: Key
}

impl<W: Write> BufferedEncoder<W> {
    pub fn new(input: std::fs::File, output: W, key: Key) -> Self {
        Self { input: input, output: output, key: key }
    }

    pub fn run(&mut self) {
        self.write_key_segment();
        self.write_data_segment();
    }

    fn write_data_segment(&mut self) {
        let mut bit_buffer = BitBuffer::new(&mut self.output);

        let input_reader = BufReader::new(&self.input);

        for byte in input_reader.bytes() {
            let encoded_bits = self.key.encode(byte.unwrap());
            for bit in encoded_bits {
                bit_buffer.push(bit);
            }
        }

        bit_buffer.dump();
    }

    fn write_key_segment(&mut self) {
        let key_bytes = self.key.serialize();

        let key_length = key_bytes.len() as u16;

        let high_byte: u8 = (key_length >> 8) as u8;
        let low_byte: u8 = (key_length & 0xff) as u8;

        let count_segment = [high_byte, low_byte];

        self.output.write(&count_segment)
            .expect("Failed to write key count segment.");

        self.output.write(&key_bytes[..])
            .expect("Failed to write key segment.");
    }
}
