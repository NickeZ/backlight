use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::string::String;

use directories::ProjectDirs;
use notify_rust::Notification;
use structopt::StructOpt;

mod util;
mod cli;
mod backlight;

use crate::cli::{Action, Opt};
use crate::backlight::{BacklightDevice, Intel};

mod error {
    #[derive(Debug)]
    pub enum Error {
        Io(std::io::Error),
    }

    impl From<std::io::Error> for Error {
        fn from(err: std::io::Error) -> Self {
            Error::Io(err)
        }
    }
}

// TODO: Implement more device types
fn create_backlight(device: &str) -> Box<dyn BacklightDevice> {
    Box::new(Intel::new(Path::new(&device[7..])))
}

fn get_notification_filename() -> Option<PathBuf> {
    ProjectDirs::from("", "", "Backlight").map(|dirs| PathBuf::from(dirs.cache_dir()))
}

fn get_notification_id(opt: &Opt, filename: &PathBuf) -> Option<u32> {
    let mut file = File::open(filename)
        .map_err(|_err| {
            if !opt.quiet {
                error!("Failed to open file");
            }
        })
        .ok()?;

    let mut current_id = String::new();
    file.read_to_string(&mut current_id)
        .map_err(|_err| {
            if !opt.quiet {
                error!("Failed to read file");
            }
        })
        .ok()?;

    current_id
        .parse::<u32>()
        .map_err(|_err| {
            if !opt.quiet {
                error!("Failed to parse u32");
            }
        })
        .ok()
}

fn main() -> Result<(), error::Error> {
    let opt = Opt::from_args();

    let mut backlight = if let Some(device) = &opt.device {
        create_backlight(device)
    } else {
        create_backlight("file:///sys/class/backlight/intel_backlight")
    };

    let (actual, native) = backlight.get()?;

    let new = match opt.action {
        Action::Get => {
            println!("{:.0}% ({})", actual, native);
            return Ok(());
        }
        Action::Set { set } => set,
        Action::Inc { inc } => actual + inc,
        Action::Dec { dec } => actual - dec,
    };

    let filename = get_notification_filename();
    let current_id = match &filename {
        Some(filename) => get_notification_id(&opt, filename),
        None => None,
    };


    let diff = (new - actual).round() as i32;

    let (steps, sleep_time) = if let Some(time) = opt.time {
        // When we do multi-step, round to nearest whole percent
        let mut res = vec![actual.round(); diff.abs() as usize];
        for (i, r) in &mut res.iter_mut().enumerate() {
            let offset = i as f32 + 1.;
            if diff > 0 {
                *r += offset;
            } else {
                *r -= offset;
            }
        }
        println!("DEBUG: {:?}", res);
        let sleep_time = if diff.abs() > 1 {
            Some(std::time::Duration::from_micros(time*1000 / diff.abs() as u64))
        } else {
            None
        };
        (res, sleep_time)
    } else if (new - actual).abs() > 0.01 {
        (vec![new], None)
    } else {
        (Vec::new(), None)
    };

    let init_time = std::time::SystemTime::now();

    for (i, step) in steps.iter().enumerate() {
        let step = *step;
        if !opt.quiet {
            println!("{:.1}%", step);
        }
        backlight.set(step)?;
        if !opt.quiet {
            let mut builder = Notification::new();
            builder.summary(&format!("Brightness at {:.1}%", step));
            builder.appname("backlight");
            if let Some(id) = current_id {
                builder.id(id);
            }
            let nf = builder.show().unwrap();
            if let Some(filename) = &filename {
                if let Ok(mut run_state_file) = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(filename)
                {
                    let _ = write!(run_state_file, "{}", nf.id());
                }
            }
        }
        if let Some(sleep_time) = sleep_time {
            if let Ok(elapsed) = init_time.elapsed() {
                let should_elapsed = (i + 1) as u32 * sleep_time;
                if elapsed < should_elapsed {
                    std::thread::sleep(should_elapsed - elapsed);
                }
            }
        }
    }
    Ok(())
}
