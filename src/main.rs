use std::{path::PathBuf, time::Duration, fs::{File, self}};
use clap::Parser;

const LOCK_PATH:&str = "/tmp/flowmodoro.lock";
const WORK_TO_REST_RATIO: u64 = 4;

#[derive(Parser)]
#[group(required = false, multiple = false)]
struct Cli {
    #[arg(short, long)]
    start: bool,
    #[arg(short, long)]
    end: bool,
}

struct Lockfile {
    file: PathBuf,
}

impl Lockfile {
    pub fn exists(&self) -> bool {
        self.file.exists()
    }

    pub fn create(&self) -> std::io::Result<()> {
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

fn end_flow(lockfile: Lockfile) {
    if lockfile.exists() {
        println!("Ending flowmodoro");
        let lockduration = lockfile.lock_duration();
        match lockduration {
            Ok(i) => println!("The flowmodoro lasted {} mins.\nYour rest should be {} mins.", i.as_secs()/60, i.as_secs()/(60*WORK_TO_REST_RATIO)),
            Err(e) => println!("Can not check flowmodoro duration => {}", e)
        }
        match lockfile.clear() {
            Ok(_) => println!("Lock cleared!"),
            Err(e) => println!("Unable to clear lock: {}", e)
        }
    } else {
        println!("Flowmodoro not started");
    }
}

fn main() {
    let cli = Cli::parse();

    let lockfile = Lockfile {
        file: PathBuf::from(LOCK_PATH),
    };
    println!("Welcome to Flowmodoro");

    if cli.start {
        start_flow(lockfile);
    } else if cli.end {
        end_flow(lockfile);
    } else if lockfile.exists() {
        end_flow(lockfile)
    } else {
        start_flow(lockfile)
    }
}
