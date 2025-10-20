use std::env;
use std::fs;
use std::process;

mod huffman;

pub struct App;

impl App {
  pub fn invoke(args: Vec<String>) {
    match Self::validate_args(&args) {
      Err(message) => {
        println!("{}", message);
        process::exit(1);
      },
      Ok(_) => ()
    }

    let mode_flag = &args[1];
    let file_path = &args[2];

    if mode_flag == "-e" {
      let mut encoder = huffman::Encoder::new(file_path);
      let _binary_output = encoder.encode();
    }

    if mode_flag == "-d" {
    }
  }

  fn validate_args(args: &Vec<String>) -> Result<bool, &str> {
    let usage = "Usage: huffc [ -d | -e ] <file>";

    if args.len() > 3 {
      return Err("Too many arguments given.");
    }

    if args.len() <= 1 {
      return Err(usage);
    }

    if args[1] == "-h" {
      return Err(usage);
    }

    if args[1] == "-d" || args[1] == "-e" {
      if args.len() == 2 {
        return Err("Must specify a file path with -e or -d option.");
      }

      let file_exists = fs::exists(&args[2])
        .expect("Error occurred while checking if file exists");

      if !file_exists {
        return Err("File does not exist.");
      }
    } else {
      return Err("Non-existent program argument given.");
    }

    return Ok(true);
  }
}

fn main() {
  let args = env::args().collect();
  App::invoke(args);
}
