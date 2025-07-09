use chrono::Datelike;
use chrono::Local;
use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use config::Config;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs::OpenOptions, process::Command};
use xdg::BaseDirectories;

fn get_root(config: &Config) -> String {
    let profile: String = config.get("profile").expect("No profile found");
    let profile_conf: HashMap<String, String> =
        config.get(&profile).expect("Profile doesn't exist");
    profile_conf
        .get("root")
        .expect("Profile has no root")
        .clone()
}

fn get_root_path(config: &Config) -> PathBuf {
    let root = get_root(config);
    PathBuf::from_str(&root).expect("Root doesn't exist")
}

fn git_commit(config: &Config) {
    let root = &get_root(config);
    let _stage_all = Command::new("git")
        .args(["-C", root, "add", "."])
        .spawn()
        .expect("failed to stage changes")
        .wait();

    let t = Local::now();

    let _commit = Command::new("git")
        .args(["-C", root, "commit", "-m", &format!("jrnl: {}", t)])
        .spawn()
        .expect("failed to commit changes")
        .wait();
}

fn git_push(config: &Config, force: bool) {
    let root = &get_root(config);
    if force {
        let _push = Command::new("git")
            .args(["-C", root, "push", "--force"])
            .spawn()
            .expect("failed to push changes")
            .wait();
    } else {
        let _push = Command::new("git")
            .args(["-C", root, "push"])
            .spawn()
            .expect("failed to push changes")
            .wait();
    }
}

fn get_header(ns: Vec<String>) -> String {
    let h = ns.join(".");
    format!("# {}", h)
}

fn open(config: &Config, ns: Vec<String>, header: Option<String>) {
    let root = get_root_path(config);
    let (last, rest) = ns.split_last().expect("ns must be at least two deep");
    let dir_path = rest.iter().fold(root.clone(), |acc, e| acc.join(e));
    let file_path = dir_path.join(format!("{}.md", last));

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
        .args(["-c", ":set wrap"])
        .spawn()
        .expect("failed to open")
        .wait();
}

fn open_date(config: &Config, year: i32, month: u32, day: u32) {
    let date = NaiveDate::from_ymd_opt(year, month, day)
        .expect("Not a valid date")
        .format("%A %e %B %Y");
    let header = format!("# {}", date);
    open(
        config,
        vec![year.to_string(), month.to_string(), day.to_string()],
        Some(header),
    );
}

fn open_today(config: &Config) {
    let date = Local::now().date_naive();
    let year = date.year();
    let month = date.month();
    let day = date.day();
    open_date(config, year, month, day)
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
    Push {
        force: Option<bool>,
    },
}

fn main() {
    let base_directories = BaseDirectories::new();
    let config_file = base_directories
        .find_config_file(Path::new("jrnl/config.toml"))
        .expect("No config found.  Please create %XDG_HOME/jrnl/config.toml");
    let config = Config::builder()
        .add_source(config::File::from(config_file))
        .add_source(config::Environment::with_prefix("JRNL"))
        .build()
        .expect("Couldn't parse jrnl config file");

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Date { year, month, day }) => open_date(&config, *year, *month, *day),
        Some(Commands::NS { ns }) => open(&config, ns.clone(), None),
        Some(Commands::Push { force }) => git_push(&config, force.unwrap_or(false)),
        None => open_today(&config),
    }

    git_commit(&config);
}
