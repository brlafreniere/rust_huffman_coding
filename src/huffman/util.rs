pub struct BitBuffer {
    buffer: Vec<bool>
}

impl BitBuffer {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn push(&mut self, bit: bool) {
        self.buffer.push(bit);
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
        let buffer = BitBuffer::new();
    }
}
