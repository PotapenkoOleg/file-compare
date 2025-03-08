use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "File compare - fast compare lines in two files ignoring relative order"
)]
pub struct Args {
    #[arg(long, short)]
    pub first: String,
    #[arg(long, short)]
    pub second: String,
    #[arg(long, short, default_value = "false")]
    pub ignore_case: bool,
    #[arg(long, short, default_value = "false")]
    pub render_html: bool,
}
