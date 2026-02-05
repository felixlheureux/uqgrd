// src/cli.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "uqgrd")]
#[command(about = "UQAM Grades Notifier", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Credentials {
        #[arg(long, short = 's')]
        skip_encryption: bool,
    },
    Grades {
        /// Automatically select the current semester based on today's date
        #[arg(long, short = 'c')]
        current: bool,
    },
    Start,
}
