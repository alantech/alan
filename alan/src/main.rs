use clap::{Parser, Subcommand};
use program::Program;

mod parse;
mod program;

#[derive(Parser, Debug)]
#[command(author, version, about, propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    commands: Option<Commands>,

    #[arg(value_name = "LN_FILE", help = ".ln source file to interpret")]
    file: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Compile .ln file(s) to an executable")]
    Compile {
        #[arg(
            value_name = "LN_FILE",
            help = ".ln source file to compile. ./index.ln if not specified",
            default_value = "./index.ln"
        )]
        file: String,
    },
    #[command(about = "Install dependencies for your Alan project")]
    Install {
        #[arg(
            value_name = "DEP_FILE",
            help = "The .ln install script to run and install the necessary dependencies into /dependences. ./.dependencies.ln if not specified",
            default_value = "./.dependencies.ln"
        )]
        file: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    println!("{:?}", args);
    if let Some(file) = args.file {
        let program = Program::new(file)?;
        println!("{:?}", program);
    }
    Ok(())
}
