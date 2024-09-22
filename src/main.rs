use anyhow::{Ok, Result};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::env;

mod chunk;

fn print_version() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!("loss {VERSION}");
}

fn print_usage() {
    println!("loss - A modern terminal pager and log viewer");
    println!("usage: loss <filename>");
}

fn init_logger() {
    let logfile = "loss.log";
    // let logfile = Utc::now().format("%Y%m%d-%H%M%S").to_string() + "-loss.log";
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(logfile).unwrap())
        .apply()
        .unwrap();
}

fn main() -> Result<()> {
    init_logger();
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        print_usage();
    } else if args[1] == "-v" {
        print_version();
    } else {
        let filename = args[1].as_str();
        println!("{filename}");
        enable_raw_mode().unwrap();
        disable_raw_mode().unwrap();
    }
    Ok(())
}
