use std::collections::HashMap;

pub struct

pub struct File;

pub struct Key {
    counts: HashMap<u8, i32>
}

impl File {
    pub fn encode(mut in_stream: impl std::io::Read, out_stream: impl std::io::Write) {
        let mut buf = [0u8; 1024];
        let mut bytes_read: usize = 0;
        let mut key = Key::new();

        match in_stream.read(&mut buf) {
            Ok(val) => bytes_read = val,
            Err(err) => {
                println!("An error occurred: {}", err);
                std::process::exit(1);
            }
        }

        while bytes_read > 0 {
            println!("bytes_read: {}", bytes_read);

            key.add_to_key(&buf);

            bytes_read = in_stream.read(&mut buf).unwrap();
        }
    }

    pub fn decode(in_stream: impl std::io::Read, out_stream: impl std::io::Write) {

    }
}

impl Key {
    pub fn new() -> Key {
        return Key {
            counts: HashMap::new()
        };
    }

    pub fn add_to_key(&mut self, buf: &[u8]) -> () {
        for chr in buf {
            *self.counts.entry(*chr).or_insert(0) += 1;
        }
    }
}
