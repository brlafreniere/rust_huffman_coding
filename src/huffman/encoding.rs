use std::io::{Write, BufReader, Read};

use super::code::Key;
use super::util::BitBuffer;

pub struct BufferedEncoder;

impl BufferedEncoder
{
    pub fn run<R: Read, W: Write>(input: &mut R, output: &mut W, key: &Key) {
        Self::write_key_segment(output, key);
        Self::write_data_segment(input, output, key);
    }

    fn write_data_segment<R: Read, W: Write>(input: R, output: &mut W, key: &Key) {
        let mut bit_buffer = BitBuffer::new(output);

        let input_reader = BufReader::new(input);

        for byte in input_reader.bytes() {
            let encoded_bits = key.encode(byte.unwrap());
            for bit in encoded_bits {
                bit_buffer.push(bit);
            }
        }

        bit_buffer.dump();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn test_write_key_segment() {
        let mut input = VecDeque::new();
        let mut output = VecDeque::new();

        input.append(&mut VecDeque::from([b'd'; 100]));
        input.append(&mut VecDeque::from([b'k'; 50]));
        input.append(&mut VecDeque::from([b'm'; 10]));

        let key = Key::build(&mut input);

        BufferedEncoder::write_key_segment(&mut output, &key);

        // There should be 3 leaf nodes, and 2 stem nodes.
        //
        // * 1 leaf node has a byte value, and no left/right, thus 1 leaf node has 2 bytes.
        // * 1 stem node has both left and right indices, but no byte value, which is 5 bytes.
        //
        // 3 * 2 = 6 bytes.
        // 2 * 5 = 10 bytes.
        // total = 16
        // 
        //
        
        assert_eq!(output.len(), 18);
        assert_eq!(output[0], 0b0000_0000);
        assert_eq!(output[1], 0b0001_0000);
    }
}
