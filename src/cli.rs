use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum Action {
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
pub struct Opt {
    #[structopt(short = "q", long = "quiet")]
    pub quiet: bool,
    #[structopt(long = "time", help = "Change brightness over `time` milliseconds")]
    pub time: Option<u64>,
    #[structopt(subcommand)]
    pub action: Action,
    #[structopt(
        long = "device",
        help = "Specify device (default: file:///sys/class/backlight/intel_backlight)"
    )]
    pub device: Option<String>,
}
