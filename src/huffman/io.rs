pub struct Operation;
pub struct BufferedEncoder;
pub struct BufferedDecoder;

use super::code::{Key};
use super::util::{BitBuffer, Byte};

use std::io::{Read, Write, Seek, copy, BufReader};
use uuid::Uuid;

impl Operation {
    pub fn encode<R: Read, W: Write>(input: &mut R, output: &mut W) {
        let mut tempfile = Self::copy_to_tempfile(input);

        let key = Key::build(&mut tempfile);
        tempfile.rewind().expect("Unable to rewind the temporary file that holds program input.");

        BufferedEncoder::run(&mut tempfile, output, &key);
    }

    pub fn decode<R: Read, W: Write>(input: &mut R, output: &mut W) {
        let mut tempfile = Self::copy_to_tempfile(input);
        BufferedDecoder::run(&mut tempfile, output);
    }

    fn copy_to_tempfile<R: Read>(input: &mut R) -> std::fs::File {
        let id = Uuid::new_v4();
        let path = format!("/tmp/{id}");

        let mut temp_file = std::fs::File::create(&path).unwrap();

        copy(input, &mut temp_file).unwrap();

        return std::fs::File::open(path).unwrap();
    }
}

impl BufferedEncoder {
    pub fn run<R: Read, W: Write>(input: &mut R, output: &mut W, key: &Key) {
        Self::write_key_segment(output, key);
        Self::write_data_segment(input, output, key);
    }

    fn write_data_segment<R: Read, W: Write>(input: R, output: &mut W, key: &Key) {
        let mut bit_buffer = BitBuffer::new(output);

        let input_reader = BufReader::new(input);

        for byte in input_reader.bytes() {
            let encoded_bits = key.encode(byte.unwrap());

            for bit in encoded_bits.unwrap() {
                bit_buffer.push(bit);
            }
        }

        bit_buffer.dump();
    }

    fn write_key_segment<W: Write>(output: &mut W, key: &Key) {
        let key_bytes = key.serialize();

        let key_length = key_bytes.len() as u16;

        Byte::write_u16(output, key_length);

        output.write(&key_bytes[..])
            .expect("Failed to write key segment.");
    }
}

impl BufferedDecoder {
    pub fn run<W: Write>(input: &mut std::fs::File, output: &mut W) {
        let key = Self::load_key(input, output);
    }

    fn load_key<W: Write>(input: &mut std::fs::File, output: &mut W) -> Key {
        Key::deserialize(input)
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

