use chrono::Datelike;
use chrono::Local;
use clap::{Parser, Subcommand};
use config::Config;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::{fs::OpenOptions, process::Command};

fn open_today(_settings: Option<Config>) {
    let date = Local::now();
    let year = date.year();
    let month = date.month();
    let day = date.day();
    // TODO: Root should come from config
    let root = "/home/rowanchase/ro-notes";
    let dir_path = format!("{}/{}/{}", root, year, month);
    let file_path = format!("{}/{}.md", dir_path, day);

    let _ = fs::create_dir_all(dir_path);

    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file_path);

    // Ensure file
    let _output = match file {
        Ok(mut f) => {
            let date_header = format!("# {}", date.format("%A %e %B %Y"));
            f.write_all(date_header.as_bytes()).expect("Couldn't write");
        }
        Err(_err) => {
            // Already exists, do nothing
        }
    };

    let _ = Command::new("nvim")
        .args(["-c", &format!("cd {}", root), &file_path])
        .spawn()
        .expect("failed to open today's note")
        .wait();
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
}
fn main() {
    let settings = Config::builder()
        .add_source(config::File::with_name(".settings.toml"))
        .build()
        .ok();
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Test { list }) => {
            if *list {
                println!("Printing testing lists...");
            } else {
                println!("Not printing testing lists...");
            }
        }
        None => {open_today(settings)}
    }
}
