use std::fs::{read_to_string, write};
use std::io::Write;
use std::path::Path;
use std::process;

use clap::{Parser, Subcommand};

use alan_compiler::fmt::fmt;
use alan_compiler::parse::get_ast;

pub mod compile;

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
        #[arg(
            short,
            long,
            help = "Test via Javascript & Node.js, not natively",
            default_value_t = false
        )]
        js: bool,
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
    #[command(about = "Format .ln source files")]
    Fmt {
        #[arg(
            value_name = "FILE",
            help = ".ln source file(s) or director(ies) to format"
        )]
        files: Vec<String>,
        #[arg(
            long,
            help = "Check formatting without writing; exit 1 if any file differs",
            default_value_t = false
        )]
        check: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    if args.file.is_some() {
        println!("TODO: Interpreter mode someday");
        Ok(())
    } else {
        match &args.commands {
            Some(Commands::Bundle { file }) => Ok(compile::bundle(file.to_string())?),
            Some(Commands::Compile { file }) => Ok(compile::compile(file.to_string())?),
            Some(Commands::Test { file, js }) => Ok(compile::test(file.to_string(), *js)?),
            Some(Commands::ToRs { file }) => Ok(compile::to_rs(file.to_string())?),
            Some(Commands::ToJs { file }) => Ok(compile::to_js(file.to_string())?),
            Some(Commands::Fmt { files, check }) => fmt_command(files, *check),
            _ => Err("Command not yet supported".into()),
        }
    }
}

fn fmt_command(files: &[String], check: bool) -> Result<(), Box<dyn std::error::Error>> {
    if files.is_empty() {
        eprintln!("error: no file or directory specified");
        eprintln!("Usage: alan fmt [FILE|DIR]...");
        eprintln!("       alan fmt --check [FILE|DIR]...");
        process::exit(1);
    }

    let mut ln_files: Vec<String> = Vec::new();
    for arg in files {
        let path = Path::new(arg);
        if path.is_dir() {
            collect_ln_files(path, &mut ln_files)?;
        } else if path.is_file() {
            ln_files.push(arg.clone());
        } else {
            eprintln!("File not found: {}", arg);
            process::exit(1);
        }
    }

    let mut had_diff = false;

    for file_path in &ln_files {
        let path = Path::new(file_path);
        let src = match read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading {}: {}", file_path, e);
                had_diff = true;
                continue;
            }
        };
        let ast = match get_ast(&src) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Parse error in {}: {}", file_path, e);
                had_diff = true;
                continue;
            }
        };
        let formatted = fmt(&ast);

        if check {
            if src != formatted {
                let diff = diff_format(&src, &formatted, file_path);
                print!("{}", diff);
                had_diff = true;
            }
        } else if src != formatted {
            write(path, &formatted)?;
            println!("Formatted {}", file_path);
        }
    }

    if had_diff {
        std::io::stdout().flush().ok();
        process::exit(1);
    }
    Ok(())
}

fn collect_ln_files(dir: &Path, out: &mut Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_ln_files(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "ln") {
            out.push(path.to_string_lossy().to_string());
        }
    }
    Ok(())
}

fn diff_format(src: &str, formatted: &str, file_path: &str) -> String {
    const CONTEXT: usize = 3;
    const RED: &str = "\x1b[31m";
    const GREEN: &str = "\x1b[32m";
    const CYAN: &str = "\x1b[36m";
    const BOLD: &str = "\x1b[1m";
    const RESET: &str = "\x1b[0m";

    let mut out = String::new();
    out.push_str(&format!("--- {}\n+++ {}\n", file_path, file_path));

    let src_lines: Vec<&str> = src.lines().collect();
    let fmt_lines: Vec<&str> = formatted.lines().collect();

    let ops = compute_diff(&src_lines, &fmt_lines);
    let groups = group_ops(&ops);

    // Build an index mapping from group index to old/new line counts before that group
    let mut src_before: Vec<usize> = vec![0];
    let mut fmt_before: Vec<usize> = vec![0];
    for group in &groups {
        match group {
            DiffGroup::Equal(lines) => {
                src_before.push(src_before.last().unwrap() + lines.len());
                fmt_before.push(fmt_before.last().unwrap() + lines.len());
            }
            DiffGroup::Remove(lines) => {
                src_before.push(src_before.last().unwrap() + lines.len());
                fmt_before.push(*fmt_before.last().unwrap());
            }
            DiffGroup::Insert(lines) => {
                src_before.push(*src_before.last().unwrap());
                fmt_before.push(fmt_before.last().unwrap() + lines.len());
            }
        }
    }

    // Partition groups into hunks: each hunk starts with a change, preceded by ≤CONTEXT
    // equal lines, and ends with ≤CONTEXT equal lines after the last change.
    let mut hunk_ranges: Vec<(usize, usize, usize)> = Vec::new(); // (start_gi, end_gi_exclusive, skip_leading)
    let mut hunk_start: Option<usize> = None;
    for (gi, group) in groups.iter().enumerate() {
        match group {
            DiffGroup::Equal(lines) => {
                if hunk_start.is_some()
                    && lines.len() > 2 * CONTEXT {
                        // End current hunk with CONTEXT trailing lines
                        let start = hunk_start.unwrap();
                        if start < gi {
                            hunk_ranges.push((start, gi, 0));
                        }
                        hunk_start = None;
                    }
            }
            _ => {
                if hunk_start.is_none() {
                    hunk_start = Some(gi);
                }
            }
        }
    }
    if let Some(start) = hunk_start {
        if start < groups.len() {
            hunk_ranges.push((start, groups.len(), 0));
        }
    }

    for (hunk_start_gi, hunk_end_gi, _skip_leading) in &hunk_ranges {
        // Determine the range of groups in this hunk, including context
        let first_change_gi = *hunk_start_gi;
        let last_change_gi = *hunk_end_gi - 1;

        // Find the first group to include: back up through equal groups to get CONTEXT lines
        let mut include_start = first_change_gi;
        if include_start > 0 {
            if let DiffGroup::Equal(prev_lines) = &groups[include_start - 1] {
                let skip = prev_lines.len().saturating_sub(CONTEXT);
                if prev_lines.len() > skip {
                    include_start -= 1;
                }
            }
        }

        // Find the last group to include: go forward through equal groups to get CONTEXT lines
        let mut include_end = last_change_gi + 1;
        if include_end < groups.len() {
            if let DiffGroup::Equal(next_lines) = &groups[include_end] {
                let take = next_lines.len().min(CONTEXT);
                if take > 0 {
                    include_end += 1;
                }
            }
        }

        // Compute hunk header
        let src_start = src_before[include_start] + 1;
        let fmt_start = fmt_before[include_start] + 1;
        let src_count = src_before[include_end].saturating_sub(src_before[include_start]);
        let fmt_count = fmt_before[include_end].saturating_sub(fmt_before[include_start]);
        out.push_str(&format!(
            "{}{}@@ -{},{} +{},{} @@{}\n",
            BOLD, CYAN, src_start, src_count, fmt_start, fmt_count, RESET
        ));

        // Emit the groups in this hunk
        let groups_in_hunk: Vec<(usize, &DiffGroup)> = groups
            .iter()
            .enumerate()
            .skip(include_start)
            .take(include_end - include_start)
            .collect();

        // If the first included group is Equal and starts before first_change_gi,
        // we need to take only the last CONTEXT lines of it
        for (gi, group) in &groups_in_hunk {
            let is_leading_context = *gi == include_start
                && *gi < first_change_gi
                && matches!(group, DiffGroup::Equal(lines) if lines.len() > CONTEXT);
            let is_trailing_context = *gi == include_end - 1
                && *gi > last_change_gi
                && matches!(group, DiffGroup::Equal(lines) if lines.len() > CONTEXT);

            match group {
                DiffGroup::Equal(lines) => {
                    let to_show = if is_leading_context {
                        &lines[lines.len() - CONTEXT..]
                    } else if is_trailing_context {
                        &lines[..CONTEXT.min(lines.len())]
                    } else {
                        &lines[..]
                    };
                    for line in to_show {
                        out.push_str(&format!("  {}\n", line));
                    }
                }
                DiffGroup::Remove(lines) => {
                    for line in lines {
                        out.push_str(&format!("{}- {}{}\n", RED, line, RESET));
                    }
                }
                DiffGroup::Insert(lines) => {
                    for line in lines {
                        out.push_str(&format!("{}+ {}{}\n", GREEN, line, RESET));
                    }
                }
            }
        }
    }

    out
}

#[derive(Debug)]
enum DiffOp<'a> {
    Equal(&'a str),
    Remove(&'a str),
    Insert(&'a str),
}

#[derive(Debug)]
enum DiffGroup<'a> {
    Equal(Vec<&'a str>),
    Remove(Vec<&'a str>),
    Insert(Vec<&'a str>),
}

fn compute_diff<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<DiffOp<'a>> {
    let m = old.len();
    let n = new.len();

    let mut dp = vec![vec![0u16; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            if old[i - 1] == new[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut result = Vec::new();
    let mut i = m;
    let mut j = n;
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old[i - 1] == new[j - 1] {
            result.push(DiffOp::Equal(old[i - 1]));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            result.push(DiffOp::Insert(new[j - 1]));
            j -= 1;
        } else {
            result.push(DiffOp::Remove(old[i - 1]));
            i -= 1;
        }
    }

    result.reverse();
    result
}

fn group_ops<'a>(ops: &[DiffOp<'a>]) -> Vec<DiffGroup<'a>> {
    let mut groups = Vec::new();
    let mut i = 0;
    while i < ops.len() {
        match &ops[i] {
            DiffOp::Equal(_) => {
                let mut lines = Vec::new();
                while i < ops.len() && matches!(&ops[i], DiffOp::Equal(_)) {
                    if let DiffOp::Equal(l) = ops[i] {
                        lines.push(l);
                    }
                    i += 1;
                }
                groups.push(DiffGroup::Equal(lines));
            }
            DiffOp::Remove(_) => {
                let mut removed = Vec::new();
                while i < ops.len() && matches!(&ops[i], DiffOp::Remove(_)) {
                    if let DiffOp::Remove(l) = ops[i] {
                        removed.push(l);
                    }
                    i += 1;
                }
                let mut inserted = Vec::new();
                while i < ops.len() && matches!(&ops[i], DiffOp::Insert(_)) {
                    if let DiffOp::Insert(l) = ops[i] {
                        inserted.push(l);
                    }
                    i += 1;
                }
                if !removed.is_empty() {
                    groups.push(DiffGroup::Remove(removed));
                }
                if !inserted.is_empty() {
                    groups.push(DiffGroup::Insert(inserted));
                }
            }
            DiffOp::Insert(_) => {
                let mut inserted = Vec::new();
                while i < ops.len() && matches!(&ops[i], DiffOp::Insert(_)) {
                    if let DiffOp::Insert(l) = ops[i] {
                        inserted.push(l);
                    }
                    i += 1;
                }
                groups.push(DiffGroup::Insert(inserted));
            }
        }
    }
    groups
}
