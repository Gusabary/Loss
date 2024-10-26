use anyhow::{Ok, Result};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use manager::Manager;
use std::env;

mod bookmark;
mod canvas;
mod chunk;
mod document;
mod event_source;
mod finder;
mod log_timestamp;
mod manager;
mod prompt;
mod render;
mod status_bar;
mod window;

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
        enable_raw_mode().unwrap();

        // todo: catch error and make sure raw mode is disabled when exit
        let mut manager = Manager::new(filename)?;
        manager.run()?;

        disable_raw_mode().unwrap();
    }
    Ok(())
}
