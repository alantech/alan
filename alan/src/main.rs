use crate::compile::{bundle, compile, test, to_js, to_rs};
use clap::{Parser, Subcommand};

mod compile;
mod lntojs;
mod lntors;
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
    #[command(about = "Compile .ln file(s) to a web bundle")]
    Bundle {
        #[arg(
            value_name = "LN_FILE",
            help = ".ln source file to compile.",
            default_value = "./index.ln"
        )]
        file: String,
    },
    #[command(about = "Compile .ln file(s) to an executable")]
    Compile {
        #[arg(
            value_name = "LN_FILE",
            help = ".ln source file to compile.",
            default_value = "./index.ln"
        )]
        file: String,
    },
    #[command(about = "Compile .ln file(s) to Rust")]
    ToRs {
        #[arg(
            value_name = "LN_FILE",
            help = ".ln source file to transpile to Rust.",
            default_value = "./index.ln"
        )]
        file: String,
    },
    #[command(about = "Compile .ln file(s) to Javascript")]
    ToJs {
        #[arg(
            value_name = "LN_FILE",
            help = ".ln source file to transpile to Javascript.",
            default_value = "./index.ln"
        )]
        file: String,
    },
    #[command(about = "Test a specified .ln file")]
    Test {
        #[arg(
            value_name = "LN_FILE",
            help = ".ln source file to compile in test mode.",
            default_value = "./index.ln"
        )]
        file: String,
    },
    #[command(about = "Install dependencies for your Alan project")]
    Install {
        #[arg(
            value_name = "DEP_FILE",
            help = "The .ln install script to run and install the necessary dependencies into /dependences",
            default_value = "./.dependencies.ln"
        )]
        file: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    if let Some(_) = args.file {
        println!("TODO: Interpreter mode someday");
        Ok(())
    } else {
        match &args.commands {
            Some(Commands::Bundle { file }) => Ok(bundle(file.to_string())?),
            Some(Commands::Compile { file }) => Ok(compile(file.to_string())?),
            Some(Commands::Test { file }) => Ok(test(file.to_string())?),
            Some(Commands::ToRs { file }) => Ok(to_rs(file.to_string())?),
            Some(Commands::ToJs { file }) => Ok(to_js(file.to_string())?),
            _ => Err("Command not yet supported".into()),
        }
    }
}
