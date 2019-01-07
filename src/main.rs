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

fn get_number(file: &str) -> io::Result<i32> {
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
    #[structopt(name = "get", help = "Get brightness")]
    Get,
    #[structopt(name = "set", help = "Set brightness")]
    Set { set: f32 },
    #[structopt(name = "inc", help = "Increase brightness")]
    Inc { inc: f32 },
    #[structopt(name = "dec", help = "Decrease brightness")]
    Dec { dec: f32 },
}

#[derive(StructOpt, Debug)]
#[structopt(name = "backlight")]
struct Opt {
    #[structopt(short = "q", long = "quiet")]
    quiet: bool,
    #[structopt(long = "time", help = "Change brightness over `time` milliseconds")]
    time: Option<u64>,
    #[structopt(subcommand)]
    action: Action,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    let max = get_number("max_brightness")?;
    let actual = get_number("actual_brightness")?;

    let new = match opt.action {
        Action::Get => {
            println!("{:.0}% ({})", 100. * actual as f32 / max as f32, actual);
            return Ok(());
        }
        Action::Set { set } => clamp((max as f32 / 100.0 * set).round() as i32, 0, max),
        Action::Inc { inc } => {
            let step = max as f32 / 100.0 * inc;
            clamp(actual + step as i32, 0, max)
        }
        Action::Dec { dec } => {
            let step = max as f32 / 100.0 * dec;
            clamp(actual - step as i32, 0, max)
        }
    };

    let filename = get_notification_filename();
    let current_id = match &filename {
        Some(filename) => get_notification_id(&opt, filename),
        None => None,
    };

    let diff = new - actual;

    let (steps, sleep_time) = if let Some(time) = opt.time {
        let mut res = vec![actual; diff.abs() as usize];
        for (i, r) in &mut res.iter_mut().enumerate() {
            if diff > 0 {
                *r += i as i32;
            } else {
                *r -= i as i32;
            }
        }
        let sleep_time = if diff.abs() > 1 {
            Some(std::time::Duration::from_micros(time*1000 / diff.abs() as u64))
        } else {
            None
        };
        (res, sleep_time)
    } else if diff != 0 {
        (vec![new], None)
    } else {
        (Vec::new(), None)
    };

    let init_time = std::time::SystemTime::now();

    for (i, step) in steps.iter().enumerate() {
        if !opt.quiet {
            println!("{:.0}% ({})", 100 * step / max, step);
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open("/sys/class/backlight/intel_backlight/brightness")
            .expect("Could not open file for controlling brightness");
        write!(file, "{}", step).unwrap();
        if !opt.quiet {
            let mut builder = Notification::new();
            builder.summary(&format!("Brightness at {:.0}%", 100 * step / max));
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
