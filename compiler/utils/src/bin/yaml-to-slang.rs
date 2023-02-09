use std::env;
use std::io::Read;

use slang_core::parser::slang_ast::{SlangFile, SlangSerialize};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} [file.yaml]\nUsage: {} --", args[0], args[0]);
        return;
    }

    let contents = if args[1] == "--" {
        let mut contents = String::new();
        std::io::stdin().read_to_string(&mut contents).unwrap();
        contents
    } else {
        std::fs::read_to_string(&args[1]).unwrap()
    };

    let parsed: SlangFile = serde_yaml::from_str(&contents).unwrap();

    let as_slang = parsed.slang_serialize(0);
    print!("{}", as_slang);
}