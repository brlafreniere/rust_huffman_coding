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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_buffer_output_1() {
        // let buffer = BitBuffer::new();
    }
}
