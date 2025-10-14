use clap::Parser;

/// Linter for cooklang recipes
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Directory containing cooklang recipes
    #[arg(short, long, default_value_t = String::from("."))]
    pub dir: String,
}
