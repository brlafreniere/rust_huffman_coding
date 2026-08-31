use std::io::Write;

pub struct BitBuffer<'a, W: Write> {
    max: u32,
    buffer: Vec<bool>,
    output: &'a mut W
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

    fn dump(&mut self) {
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

    pub fn as_bytes(&self) -> Vec<u8> {
        let bytes = Vec::new();
        let mut bits = self.buffer.clone();

        // While there are bits in the buffer...
        while bits.len() >= 0 {
            let mut byte: u8 = 0;

            // Pull a slice of 8 or less
            let slice_len = if bits.len() >= 8 { 8 } else { bits.len() };
            let slice = bits.drain(0..slice_len);

            // For each bit in the slice, add it to our output byte
            for bit in slice {
                byte << 1;
                if bit { byte += 1; }
            }

            // Handle any bit lengths < 8
            // Example: if we have 3 bits left, then slice_len=3
            // And we would need to pad with 8 - slice_len = 5 zero bits
            if slice_len < 8 {
                let pad_count = 8 - slice_len;
                for num in 1..=pad_count {
                    byte << 1;
                }
            }
        }

        return bytes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_buffer_output_1() {
        // let buffer = BitBuffer::new();
    }
}
