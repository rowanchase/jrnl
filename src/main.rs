use chrono::Datelike;
use chrono::Local;
use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use config::Config;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::{fs::OpenOptions, process::Command};

fn git_commit(_settings: &Option<Config>) {
    let root = "/home/rowanchase/journal";
    let _stage_all = Command::new("git")
        .args(["-C", root, "add", "."])
        .spawn()
        .expect("failed to stage changes")
        .wait();

    let t = Local::now();

    let _commit = Command::new("git")
        .args([
            "-C",
            root,
            "commit",
            "-m",
            &format!("{}", t),
        ])
        .spawn()
        .expect("failed to stage changes")
        .wait();

    let _push = Command::new("git")
        .args(["-C", root, "push", "--force"])
        .spawn()
        .expect("failed to stage changes")
        .wait();
}

fn open_date(_settings: &Option<Config>, year: i32, month: u32, day: u32) {
    // TODO: Root should come from config
    let root = "/home/rowanchase/journal";
    let dir_path = format!("{}/{}/{}", root, year, month);
    let file_path = format!("{}/{}.md", dir_path, day);

    let _ = fs::create_dir_all(dir_path);

    let date = NaiveDate::from_ymd_opt(year, month, day)
        .expect("Not a valid date")
        .format("%A %e %B %Y");

    // Ensure file
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file_path);

    let _output = match file {
        Ok(mut f) => {
            let date_header = format!("# {}", date);
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

fn open_today(settings: &Option<Config>) {
    let date = Local::now().date_naive();
    let year = date.year();
    let month = date.month();
    let day = date.day();
    open_date(settings, year, month, day)
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
    /// Open notes for date
    Date {
        /// Open notes for date
        year: i32,
        month: u32,
        day: u32,
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
        Some(Commands::Date { year, month, day }) => open_date(&settings, *year, *month, *day),
        None => open_today(&settings),
    }

    git_commit(&settings);
}
