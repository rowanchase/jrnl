use chrono::Datelike;
use chrono::Local;
use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use config::Config;
use std::env::home_dir;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::{fs::OpenOptions, process::Command};

fn git_commit(_settings: &Option<Config>) {
    let root = "/home/rowan/journal";
    let _stage_all = Command::new("git")
        .args(["-C", root, "add", "."])
        .spawn()
        .expect("failed to stage changes")
        .wait();

    let t = Local::now();

    let _commit = Command::new("git")
        .args(["-C", root, "commit", "-m", &format!("{}", t)])
        .spawn()
        .expect("failed to stage changes")
        .wait();

    let _push = Command::new("git")
        .args(["-C", root, "push", "--force"])
        .spawn()
        .expect("failed to push changes")
        .wait();
}

fn get_header(ns: Vec<String>) -> String {
    let h = ns.join(".");
    format!("# {}", h)
}

fn open(_settings: &Option<Config>, ns: Vec<String>, header: Option<String>) {
    // TODO: Root should come from config
    let root = home_dir().unwrap().join("journal");
    let (last, rest) = ns.split_last().expect("ns must be at least two deep");
    let dir_path = rest.iter().fold(root.clone(), |acc, e| acc.join(e));
    let file_path = dir_path.join(last);

    let _ = fs::create_dir_all(dir_path);

    // Ensure file
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file_path);

    let ns_header = get_header(ns.clone());

    let _output = match file {
        Ok(mut f) => {
            let h = header.or(Some(ns_header)).expect("No header found");
            f.write_all(h.as_bytes()).expect("Couldn't write");
        }
        Err(_err) => {
            // Already exists, do nothing
        }
    };

    let _ = Command::new("nvim")
        .args([
            "-c",
            &format!("cd {}", root.to_str().unwrap()),
            &file_path.to_str().unwrap(),
        ])
        .args(["-c", ":set spell"])
        .spawn()
        .expect("failed to open today's note")
        .wait();
}

fn open_date(settings: &Option<Config>, year: i32, month: u32, day: u32) {
    let date = NaiveDate::from_ymd_opt(year, month, day)
        .expect("Not a valid date")
        .format("%A %e %B %Y");
    let header = format!("# {}", date);
    open(
        settings,
        vec![year.to_string(), month.to_string(), day.to_string()],
        Some(header),
    );
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
    /// Open notes for date
    Date {
        /// Open notes for date
        year: i32,
        month: u32,
        day: u32,
    },
    NS {
        ns: Vec<String>,
    },
}
fn main() {
    let settings = Config::builder()
        .add_source(config::File::with_name(".settings.toml"))
        .build()
        .ok();
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Date { year, month, day }) => open_date(&settings, *year, *month, *day),
        Some(Commands::NS { ns }) => open(&settings, ns.clone(), None),
        None => open_today(&settings),
    }

    git_commit(&settings);
}
