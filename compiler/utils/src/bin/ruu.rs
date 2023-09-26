use std::env;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

use clap::{value_parser, Arg, ArgAction, Command};
use notify::{DebouncedEvent, RecursiveMode, Watcher};
use ruulang_core::{config::config::RuuLangConfig, workspace::workspace::Workspace};

#[derive(Debug)]
struct CliOptions {
    pub config_path: String,
    pub watch: bool,
    pub no_check: bool,
    pub no_emit: bool,
}

fn get_args() -> CliOptions {
    let matches = Command::new("ruu")
        .version("0.1.0")
        .author("Zach Wade <zach@dttw.tech>")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
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
        .arg(
            Arg::new("no-check")
                .long("no-check")
                .required(false)
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-emit")
                .long("no-emit")
                .required(false)
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let config_default = "ruu.toml".to_string();
    let config_path = matches
        .get_one::<String>("config")
        .unwrap_or(&config_default);
    let watch = matches.get_one::<bool>("watch").unwrap_or(&false);
    let no_check = matches.get_one::<bool>("no-check").unwrap_or(&false);
    let no_emit = matches.get_one::<bool>("no-emit").unwrap_or(&false);

    CliOptions {
        config_path: config_path.clone(),
        watch: watch.clone(),
        no_check: no_check.clone(),
        no_emit: no_emit.clone(),
    }
}

async fn compile_all(workspace: &mut Workspace, options: &CliOptions) {
    workspace.reload().await.unwrap();

    if !options.no_check {
        workspace.typecheck().await.unwrap();
    }

    if !options.no_emit {
        workspace.compile_all().await.unwrap();
        println!("Finished compiling!");
    }
}

async fn compile_on_change(workspace: &mut Workspace, options: &CliOptions) {
    let (tx, rx) = channel();
    let mut watcher = notify::watcher(tx, Duration::from_millis(200)).unwrap();

    let dir_to_watch = workspace
        .config
        .workspace
        .root
        .clone()
        .unwrap_or_else(|| workspace.working_dir.clone());
    watcher
        .watch(dir_to_watch, RecursiveMode::Recursive)
        .unwrap();

    loop {
        match rx.recv() {
            Ok(
                DebouncedEvent::Write(p)
                | DebouncedEvent::Create(p)
                | DebouncedEvent::Remove(p)
                | DebouncedEvent::Rename(p, _),
            ) => {
                if workspace.file_is_ruulang_source(&p).await {
                    println!("Change detected, recompiling...");
                    compile_all(workspace, options).await;
                }
            }
            Ok(_) => {}
            Err(e) => {
                println!("Error while watching for changes:\n{:?}", e);
                return;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args = get_args();

    let working_dir = env::current_dir().unwrap();
    let config = RuuLangConfig::load(Path::new(&args.config_path), &working_dir)
        .await
        .unwrap();

    let mut workspace = Workspace::new(config, working_dir);

    compile_all(&mut workspace, &args).await;

    if args.watch {
        compile_on_change(&mut workspace, &args).await;
    }
}
