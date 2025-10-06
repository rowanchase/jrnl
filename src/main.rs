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

#[derive(Debug, Clone)]
struct ProfileConfig {
    config: HashMap<String, String>,
}

impl ProfileConfig {
    fn new(config: HashMap<String, String>) -> Self {
        Self { config }
    }

    fn get_root(&self) -> String {
        self.config
            .get("root")
            .expect("Profile has no root")
            .clone()
    }

    fn get_commit(&self) -> bool {
        self.config
            .get("commit")
            .and_then(|s| s.parse().ok())
            .unwrap_or(true)
    }
}

fn get_root_path(profile_config: &ProfileConfig) -> PathBuf {
    let root = profile_config.get_root();
    PathBuf::from_str(&root).expect("Root doesn't exist")
}

fn git_commit(profile_config: &ProfileConfig) {
    let root = &profile_config.get_root();
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

fn should_commit(profile_config: &ProfileConfig, cli: &Cli) -> bool {
    if cli.commit && cli.no_commit {
        panic!("Cannot specify both --commit and --no-commit flags");
    }

    if cli.commit {
        return true;
    }

    if cli.no_commit {
        return false;
    }

    profile_config.get_commit()
}

fn git_push(profile_config: &ProfileConfig, force: bool) {
    let root = &profile_config.get_root();
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

fn open(profile_config: &ProfileConfig, ns: Vec<String>, header: Option<String>) {
    let root = get_root_path(profile_config);
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

fn open_date(profile_config: &ProfileConfig, year: i32, month: u32, day: u32) {
    let date = NaiveDate::from_ymd_opt(year, month, day)
        .expect("Not a valid date")
        .format("%A %e %B %Y");
    let header = format!("# {}", date);
    open(
        profile_config,
        vec![year.to_string(), month.to_string(), day.to_string()],
        Some(header),
    );
}

fn open_today(profile_config: &ProfileConfig) {
    let date = Local::now().date_naive();
    let year = date.year();
    let month = date.month();
    let day = date.day();
    open_date(profile_config, year, month, day)
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    #[arg(short, long)]
    profile: Option<String>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Enable git commit (overrides config)
    #[arg(long)]
    commit: bool,

    /// Disable git commit (overrides config)
    #[arg(long)]
    no_commit: bool,

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

    let cli = Cli::parse();

    let config_file = base_directories
        .find_config_file(Path::new("jrnl/config.toml"))
        .expect("No config found.  Please create %XDG_HOME/jrnl/config.toml");

    let config = Config::builder()
        .add_source(config::File::from(config_file))
        .add_source(config::Environment::with_prefix("JRNL"))
        .build()
        .expect("Couldn't parse jrnl config file");

    let profile: String = cli
        .profile
        .clone()
        .unwrap_or(config.get("profile").unwrap_or(String::from("default")));
    let profile_config_map: HashMap<String, String> =
        config.get(&profile).expect("Profile doesn't exist");
    let profile_config = ProfileConfig::new(profile_config_map);

    match &cli.command {
        Some(Commands::Date { year, month, day }) => {
            open_date(&profile_config, *year, *month, *day)
        }
        Some(Commands::NS { ns }) => open(&profile_config, ns.clone(), None),
        Some(Commands::Push { force }) => git_push(&profile_config, force.unwrap_or(false)),
        None => open_today(&profile_config),
    }

    if should_commit(&profile_config, &cli) {
        git_commit(&profile_config);
    }
}
