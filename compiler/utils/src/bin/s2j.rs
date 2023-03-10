use notify::{DebouncedEvent, RecursiveMode, Watcher};
use slang_core::parser::parser_constructs::{ParserStatement, ParserAssemble};
use std::fs;
use std::time::Duration;
use std::{path::PathBuf, sync::mpsc::channel};

use clap::{value_parser, Arg, ArgAction, Command};

#[derive(Debug)]
struct CliOptions {
    pub input: PathBuf,
    pub output: PathBuf,
    pub watch: bool,
}

fn get_args() -> CliOptions {
    let matches = Command::new("s2j")
        .version("0.1.0")
        .author("Zach Wade <zach@nuvo.finance>")
        .arg(
            Arg::new("input")
                .index(1)
                .required(true)
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .required(false)
                .num_args(1)
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("watch")
                .short('w')
                .long("watch")
                .required(false)
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let input = matches.get_one::<PathBuf>("input").unwrap();

    let output_default_name = input
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .split('.')
        .next()
        .unwrap()
        .to_owned()
        + ".json";
    let output_default = input.with_file_name(output_default_name);

    let output = matches
        .get_one::<PathBuf>("output")
        .unwrap_or(&output_default);
    let watch = matches.get_one::<bool>("watch").unwrap_or(&false);

    CliOptions {
        input: input.clone(),
        output: output.clone(),
        watch: watch.clone(),
    }
}

fn compile_one(input: &PathBuf, output: &PathBuf) {
    let input_content = fs::read_to_string(input).unwrap();
    match ParserStatement::parse(input_content.as_str()) {
        Ok(parsed) => {
            let as_file = parsed.assemble();
            let as_json = serde_json::to_string_pretty(&as_file).unwrap();
            fs::write(output, as_json).unwrap();
        }
        Err(e) => {
            println!("Error parsing {:?}: {:?}", input, e);
        }
    }
}

fn compile_on_change(input: &PathBuf, output: &PathBuf) {
    let (tx, rx) = channel();
    let mut watcher = notify::watcher(tx, Duration::from_millis(200)).unwrap();
    watcher.watch(input, RecursiveMode::NonRecursive).unwrap();

    loop {
        match rx.recv() {
            Ok(DebouncedEvent::Write(_)) => {
                println!("File {:?} changed. Recompiling...", input);
                compile_one(input, output);
            }
            Ok(_) => {}
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn main() {
    let args = get_args();
    compile_one(&args.input, &args.output);

    if args.watch {
        println!("Watching {:?} for changes...", args.input);
        compile_on_change(&args.input, &args.output);
    } else {
        println!(
            "Successfully compiled {:?} to {:?}",
            args.input, args.output
        );
    }
}
