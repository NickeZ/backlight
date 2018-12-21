use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::string::String;

use directories::ProjectDirs;
use notify_rust::Notification;
use structopt::StructOpt;

// Ignore errors when outputing errors
// TODO: should only print if opt.quiet == false
macro_rules! error {
    ($($x:tt)*) => {
        let _ = writeln!(std::io::stderr(), $($x)*);
    }
}

fn get_number(file: &str) -> io::Result<f32> {
    let mut target = String::new();
    get_contents(file, &mut target).unwrap();
    let res = target.trim().parse().unwrap();
    Ok(res)
}

fn get_contents(file: &str, target: &mut String) -> io::Result<()> {
    let mut filename = String::from("/sys/class/backlight/intel_backlight/");
    filename.push_str(file);

    let mut file = File::open(&filename)?;
    file.read_to_string(target)?;
    Ok(())
}

fn clamp<T: PartialOrd>(val: T, min: T, max: T) -> T {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
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

#[derive(StructOpt, Debug)]
enum Action {
    #[structopt(name = "get")]
    Get,
    #[structopt(name = "set")]
    Set { set: f32 },
    #[structopt(name = "inc")]
    Inc { inc: f32 },
    #[structopt(name = "dec")]
    Dec { dec: f32 },
}

#[derive(StructOpt, Debug)]
#[structopt(name = "backlight")]
struct Opt {
    #[structopt(short = "q", long = "quiet")]
    quiet: bool,
    #[structopt(subcommand)]
    action: Action,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    let max = get_number("max_brightness")?;
    let actual = get_number("actual_brightness")?;

    let new = match opt.action {
        Action::Get => {
            println!("{:.0}% ({})", 100. * actual / max, actual);
            return Ok(());
        }
        Action::Set { set } => clamp(max / 100.0 * set, 0., max),
        Action::Inc { inc } => {
            let step = max / 100.0 * inc;
            clamp(actual + step, 0., max)
        }
        Action::Dec { dec } => {
            let step = max / 100.0 * dec;
            clamp(actual - step, 0., max)
        }
    };

    let filename = get_notification_filename();
    let current_id = match &filename {
        Some(filename) => get_notification_id(&opt, filename),
        None => None,
    };

    if (new - actual).abs() > 0.001 {
        if !opt.quiet {
            println!("{:.0}% ({})", 100. * new / max, new);
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open("/sys/class/backlight/intel_backlight/brightness")
            .expect("Could not open file for controlling brightness");
        write!(file, "{}", new as i32).unwrap();
        if !opt.quiet {
            let mut builder = Notification::new();
            builder.summary(&format!("Brightness at {:.0}%", 100. * new / max));
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
    }
    Ok(())
}
