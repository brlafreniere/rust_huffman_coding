pub struct File;

use super::encoding::BufferedEncoder;
use super::code::Key;

use std::io::{Read, Write, Seek, copy};
use uuid::Uuid;

impl File {
    pub fn encode<R: Read, W: Write>(input: &mut R, output: &mut W) {
        let tempfile_path = Self::copy_to_tempfile(input);
        let mut tempfile = Self::open_tempfile(&tempfile_path);

        let key = Key::build(&mut tempfile);
        tempfile.rewind().expect("Unable to rewind the temporary file that holds program input.");

        BufferedEncoder::run(&mut tempfile, output, key);
    }

    pub fn decode(in_stream: impl std::io::Read, out_stream: impl std::io::Write) {

    }

    fn open_tempfile(path: &String) -> std::fs::File {
        match std::fs::File::open(path) {
            Err(err) => { panic!("Unable to open tempfile for reading: {path}"); },
            Ok(result) => result
        }
    }

    fn copy_to_tempfile<R: Read>(input: &mut R) -> String {
        let id = Uuid::new_v4();
        let path = format!("/tmp/{id}");

        let mut temp_file = std::fs::File::create(&path)
            .expect("Failed to create temporary file.");

        copy(input, &mut temp_file)
            .expect("Failed to save input to temporary file.");

        return path;
    }
}

