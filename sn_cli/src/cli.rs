use clap::Parser;
use clap::Subcommand;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Run {
        #[arg(short, long)]
        verbose: bool,
    },
    #[command(subcommand)]
    Model(ModelCommands),
}

#[derive(Subcommand, Debug)]
pub enum ModelCommands {
    List {
        #[arg(short, long)]
        name: Option<String>,
    },
    PS {
        #[arg(short, long)]
        name: Option<String>,
    },
    Run {
        #[arg(short, long)]
        name: Option<String>,
        #[arg()]
        model: Option<String>,
    },
    Stop {
        #[arg(short, long)]
        name: Option<String>,
        #[arg()]
        model: Option<String>,
    },
}
