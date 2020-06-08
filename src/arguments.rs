use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Michał Fita")]
pub struct Opts {
    pub directory: String,
    #[clap(short = "o", long = "output", default_value = "photo-sword.json")]
    pub output: String,
}

pub fn digest() -> Opts {
    Opts::parse()
}