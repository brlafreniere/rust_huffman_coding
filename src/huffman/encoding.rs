use std::io::Write;
use std::collections::VecDeque;

use super::code::Key;
use super::util::BitBuffer;

pub struct BufferedEncoder;

impl BufferedEncoder {
    const BUFFER_SIZE: usize = 1024;

    pub fn run<W: Write>(input: &mut std::fs::File, output: &mut W, key: Key) {
        Self::write_key_segment(output, &key);

        let input_buffer: [u8; Self::BUFFER_SIZE] = [0; Self::BUFFER_SIZE];
        let mut bit_buffer = BitBuffer::new(output);

        // for byte in input_buffer {
        //     let result_bits = key.encode(byte);
        //     for bit in result_bits {
        //         if bit_buffer.len() < Self::BUFFER_SIZE {
        //             bit_buffer.push_back(bit);
        //         } else {
        //             // dump the buffer
        //         }
        //     }
        // }
    }

    fn write_key_segment<W: Write>(output: &mut W, key: &Key) {
        let key_bytes = key.serialize();

        let key_length = key_bytes.len() as u16;

        let high_byte: u8 = (key_length >> 8) as u8;
        let low_byte: u8 = (key_length & 0xff) as u8;

        let count_segment = [high_byte, low_byte];

        output.write(&count_segment)
            .expect("Failed to write key count segment.");

        output.write(&key_bytes[..])
            .expect("Failed to write key segment.");
    }
}
