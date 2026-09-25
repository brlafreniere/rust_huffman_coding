use std::io::{Write, Read};

pub struct BitBuffer<'a, W: Write> {
    max: u32,
    buffer: Vec<bool>,
    output: &'a mut W
}

pub struct Byte {}

impl Byte {
    pub fn get_u16(input: &mut impl Read) -> u16 {
        let mut bytes: [u8; 2] = [0; 2];

        input.read_exact(&mut bytes[..]).expect("fail");

        ((bytes[0] as u16) << 8) | bytes[1] as u16
    }

    pub fn get_u8(input: &mut impl Read) -> Option<u8> {
        let mut buffer = Vec::new();

        match input.take(1).read_to_end(&mut buffer) {
            Ok(0) => { return None },
            Ok(bytes_read) => { return Some(buffer[0]) },
            Err(e) => { panic!("Error: {}", e) }
        }
    }

    pub fn write_u16(output: &mut impl Write, data: u16) {
        let high_byte: u8 = (data >> 8) as u8;
        let low_byte: u8 = (data & 0xff) as u8;

        let byte_pair = [high_byte, low_byte];

        output.write(&byte_pair)
            .expect("Failed to write byte pair.");
    }
}

impl<'a, W: Write> BitBuffer<'a, W> {
    const DEFAULT_MAX: u32 = 1024;

    pub fn new(output: &'a mut W) -> Self {
        Self { max: Self::DEFAULT_MAX, buffer: Vec::new(), output: output }
    }

    pub fn push(&mut self, bit: bool) {
        if self.buffer.len() as u32 == self.max {
            self.dump();
        }

        self.buffer.push(bit);
    }

    pub fn dump(&mut self) {
        let mut bytes = Vec::new();

        while self.buffer.len() > 0 {
            if self.buffer.len() < 8 {
                // If there is only 5 bits left, then we need to pad with 8 - 5 = 3 zeroes;
                let zeroes = 8 - self.buffer.len();
                for i in 0..zeroes { self.buffer.push(false); }
            }

            let bit_slice = self.buffer.drain(0..8);
            let mut byte: u8 = 0;

            for bit in bit_slice {
                byte = byte << 1;
                if bit { byte += 1; }
            }

            bytes.push(byte);
        }

        self.output.write(&bytes[..])
            .expect("Could not dump bit buffer to output");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    mod byte { use super::*;
        #[test]
        fn test_write_u16() {
            let mut output: VecDeque<u8> = VecDeque::new();

            // the value 10_598 in binary is:
            // 0b0010_1001 0b0110_0110
            let value = 10_598;
            let expected_high_byte = 0b0010_1001;
            let expected_low_byte = 0b0110_0110;

            Byte::write_u16(&mut output, value);

            assert_eq!(output.pop_front().unwrap(), expected_high_byte);
            assert_eq!(output.pop_front().unwrap(), expected_low_byte);
        }

        #[test]
        fn test_get_u16() {
            let mut input: VecDeque<u8> = VecDeque::new();

            // the value 10_598 in binary is:
            // 0b0010_1001 0b0110_0110
            let value = 10_598;
            let high_byte = 0b0010_1001;
            let low_byte = 0b0110_0110;

            input.push_back(high_byte);
            input.push_back(low_byte);

            let result = Byte::get_u16(&mut input);

            assert_eq!(result, value);
        }
    }

    mod bit_buffer { use super::*;
        #[test]
        fn test_bit_buffer_output_1() {
            let input: Vec<bool> = Vec::from([true, true, false, false, true, true, false, true]);
            let mut output: VecDeque<u8> = VecDeque::new();

            let mut buffer = BitBuffer::new(&mut output);

            for bool in input {
                buffer.push(bool);
            }

            buffer.dump();

            let output_byte = output.pop_front().unwrap();

            assert_eq!(output_byte, 0b1100_1101);
        }

        #[test]
        fn test_bit_buffer_output_2() {
            let input: Vec<bool> = Vec::from([true, true, false, false, true, true, false, true, true]);
            let mut output: VecDeque<u8> = VecDeque::new();

            let mut buffer = BitBuffer::new(&mut output);

            for bool in input {
                buffer.push(bool);
            }

            buffer.dump();

            let byte_1 = output.pop_front().unwrap();
            let byte_2 = output.pop_front().unwrap();

            assert_eq!(byte_1, 0b1100_1101);
            assert_eq!(byte_2, 0b1000_0000);
        }
    }
}
