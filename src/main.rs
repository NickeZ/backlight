use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io;
use std::string::String;
use notify_rust::Notification;

use structopt::StructOpt;

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

#[derive(StructOpt, Debug)]
enum Action {
    #[structopt(name = "get")]
    Get,
    #[structopt(name = "set")]
    Set {
        set: f32,
    },
    #[structopt(name = "inc")]
    Inc {
        inc: f32,
    },
    #[structopt(name = "dec")]
    Dec {
        dec: f32,
    },
}

#[derive(StructOpt, Debug)]
#[structopt(name = "backlight")]
struct Opt {
    #[structopt(short="q", long="quiet")]
    quiet: bool,
    #[structopt(subcommand)]
    action: Action,
}

fn main() {
    let max = get_number("max_brightness").unwrap();
    let actual = get_number("actual_brightness").unwrap();

    let opt = Opt::from_args();

    let mut new = actual;

    match opt.action {
        Action::Get => {
            println!("{:.0}% ({})", 100.*actual/max, actual);
        },
        Action::Set {set} => {
            new = clamp(max/100.0*set, 0., max);
        },
        Action::Inc {inc} => {
            let step = max/100.0*inc;
            new = clamp(actual+step, 0., max);
        },
        Action::Dec {dec} => {
            let step = max/100.0*dec;
            new = clamp(actual-step, 0., max);
        }
    }

    if (new - actual).abs() < 0.001 {
        println!("{:.0}% ({})", 100.*new/max, new);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open("/sys/class/backlight/intel_backlight/brightness").unwrap();
        write!(file, "{}", new as i32).unwrap();
        Notification::new().summary(&format!("Brightness at {}%", 100.*new/max)).show().unwrap();
    }
}
