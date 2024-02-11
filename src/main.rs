use std::{path::PathBuf, time::Duration, fs::{File, self, OpenOptions},io::Write, env::var};
use clap::Parser;
use chrono::{Local, DateTime};

const WORK_TO_REST_RATIO: u64 = 4;

#[derive(Parser)]
#[group(required = false, multiple = false)]
struct Cli {
    message: Option<String>,
    #[arg(long)]
    info: bool,
}

fn get_files_route() -> String{
    let config_home = var("XDG_CONFIG_HOME")
        .or_else(|_| var("HOME").map(|home|format!("{}/.config", home)));
    let base_path = match config_home {
        Ok(path) => path,
        Err(_) =>  "/tmp".to_string(),
    };

    base_path + "/flowmodoro/"
}

struct Lockfile {
    file: PathBuf,
}

impl Lockfile {
    pub fn exists(&self) -> bool {
        self.file.exists()
    }

    pub fn create(&self) -> std::io::Result<()> {
        if let Some(path) = self.file.parent() {fs::create_dir_all(path)?}
        File::create(self.file.as_os_str())?;
        Ok(())
    }

    pub fn clear(&self) -> std::io::Result<()> {
        fs::remove_file(self.file.as_os_str())?;
        Ok(())
    }

    pub fn lock_duration(&self) -> std::io::Result<Duration> {
        let metadata = self.file.metadata()?;
        let creation_time = metadata.created()?;
        let mut duration:Duration = Duration::new(0,0);
        match creation_time.elapsed() {
            Ok(dur) => duration = dur,
            Err(e) => println!("Error en duration {}", e)
        }
        Ok(duration)
    }
}

fn start_flow(lockfile: Lockfile) {
    if lockfile.exists() {
        println!("Already in an flowmodoro");
    } else {
        match lockfile.create() {
            Ok(_) => println!("Starting a flowmodoro"),
            Err(e) => println!("Error starting flowmodoro {}", e)
        }
    }
}

fn info_flow(lockfile: &Lockfile) {
    if lockfile.exists() {
        let lockduration = lockfile.lock_duration();
        match lockduration {
            Ok(i) => println!("The flowmodoro lasted {} mins.\nYour rest should be {} mins.", i.as_secs()/60, i.as_secs()/(60*WORK_TO_REST_RATIO)),
            Err(e) => println!("Can not check flowmodoro duration => {}", e)
        }
    } else {
        println!("Flowmodoro not started");
    }
}

fn end_flow(lockfile: Lockfile) {
    if lockfile.exists() {
        info_flow(&lockfile);
        match lockfile.clear() {
            Ok(_) => (),
            Err(e) => println!("Unable to clear lock: {}", e)
        }
    }
}

fn write_log_str(message: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(get_files_route()+"flowmodoro.log")
        .unwrap();

    if let Err(e) = writeln!(file, "{message}") {
        eprintln!("Couldn't write to file: {}", e);
    }
}
fn write_log(date:DateTime<Local>, status:&str, message: &str) {
    let formated_date = date.format("%Y/%m/%d-%H:%M:%S");
    write_log_str(&format!("{formated_date}, {status}, \"{message}\""));
}

fn create_log() {
    let logfile = Lockfile {
        file: PathBuf::from(get_files_route()+"flowmodoro.log"),
    };
    if !logfile.exists() {
        let _ = logfile.create();
        write_log_str("date,status,message");
    }
}

fn main() {
    let cli = Cli::parse();

    let lockfile = Lockfile {
        file: PathBuf::from(get_files_route()+"flowmodoro.lock"),
    };
    
    create_log();
    let date = Local::now();
    let message = cli.message.unwrap_or("".to_string());
    
    if cli.info {
        info_flow(&lockfile);
    } else if lockfile.exists() {
        end_flow(lockfile);
        write_log(date, "END", &message);
    } else {
        start_flow(lockfile);
        write_log(date, "START", &message);
    }
}
