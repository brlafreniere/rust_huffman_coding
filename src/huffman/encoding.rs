use std::io::Write;
use std::collections::VecDeque;

use super::code::Key;

pub struct BufferedEncoder;

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
