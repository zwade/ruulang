use std::env;
use std::io::Read;

use slang_core::parser::parser_constructs::ParserAssemble;
use slang_core::slang::TermParser;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} [file.slang]\nUsage: {} --", args[0], args[0]);
        return;
    }

    let contents = if args[1] == "--" {
        let mut contents = String::new();
        std::io::stdin().read_to_string(&mut contents).unwrap();
        contents
    } else {
        std::fs::read_to_string(&args[1]).unwrap()
    };

    let parsed = TermParser::new().parse(contents.as_str()).unwrap().assemble();

    let as_yaml = serde_yaml::to_string(&parsed).unwrap();
    print!("{}", as_yaml);
}
