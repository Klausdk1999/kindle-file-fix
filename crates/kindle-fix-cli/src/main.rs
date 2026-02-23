mod output;

use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use colored::Colorize;
use dialoguer::Input;

use kindle_fix_core::{process_file, FixOptions};

#[derive(Parser, Debug)]
#[command(
    name = "kindle-file-fix",
    about = "Fix ebook files for Kindle compatibility",
    version
)]
struct Cli {
    /// Input files or directories to process
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Output directory (default: same as input file)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Set language code (skip interactive prompt)
    #[arg(short, long)]
    language: Option<String>,

    /// Keep original filename (no prefix)
    #[arg(long)]
    keep_name: bool,

    /// Show fixes without writing files
    #[arg(long)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Suppress output except errors
    #[arg(short, long)]
    quiet: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::init();
    }

    let files = collect_files(&cli.files);

    if files.is_empty() {
        eprintln!("{}", "No supported files found.".red());
        std::process::exit(1);
    }

    let mut total_fixes = 0;
    let mut processed = 0;
    let mut errors = 0;

    for path in &files {
        let filename = path.file_name().unwrap_or_default().to_string_lossy();

        if !cli.quiet {
            println!("{} {}", "Processing:".bold(), filename);
        }

        let data = match fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("  {} Could not read {}: {}", "[ERROR]".red(), filename, e);
                errors += 1;
                continue;
            }
        };

        let options = FixOptions {
            language: cli.language.clone(),
            keep_name: cli.keep_name,
            dry_run: cli.dry_run,
        };

        match process_file(&data, &filename, &options) {
            Ok(result) => {
                // Handle unsupported language warnings with interactive prompt
                for warning in &result.report.warnings {
                    if warning.contains("not supported by Kindle")
                        && cli.language.is_none()
                        && !cli.quiet
                    {
                        eprintln!("  {} {}", "[WARN]".yellow().bold(), warning);
                        if let Ok(lang) = Input::<String>::new()
                            .with_prompt("  Enter language code (e.g., en, fr, ja)")
                            .default("en".into())
                            .interact_text()
                        {
                            let new_options = FixOptions {
                                language: Some(lang),
                                keep_name: cli.keep_name,
                                dry_run: cli.dry_run,
                            };
                            if let Ok(new_result) = process_file(&data, &filename, &new_options) {
                                output::print_report(&new_result.report, cli.quiet);
                                if !cli.dry_run {
                                    write_output(
                                        path,
                                        &new_result.data,
                                        cli.keep_name,
                                        &cli.output,
                                    );
                                }
                                total_fixes += new_result.report.fixes_applied.len();
                                processed += 1;
                                continue;
                            }
                        }
                    }
                }

                output::print_report(&result.report, cli.quiet);

                if !cli.dry_run && !result.data.is_empty() {
                    write_output(path, &result.data, cli.keep_name, &cli.output);
                }

                total_fixes += result.report.fixes_applied.len();
                processed += 1;
            }
            Err(e) => {
                eprintln!("  {} {}", "[ERROR]".red().bold(), e);
                errors += 1;
            }
        }

        if !cli.quiet {
            println!();
        }
    }

    if !cli.quiet {
        println!(
            "{}",
            format!(
                "Processed {} file(s), {} fix(es) applied, {} error(s).",
                processed, total_fixes, errors
            )
            .bold()
        );
    }

    if errors > 0 {
        std::process::exit(1);
    }
}

fn collect_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let supported_extensions = ["epub", "mobi", "azw3"];
    let mut result = Vec::new();

    for path in paths {
        if path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if let Some(ext) = p.extension() {
                        if supported_extensions
                            .contains(&ext.to_string_lossy().to_lowercase().as_str())
                        {
                            result.push(p);
                        }
                    }
                }
            }
        } else if path.is_file() {
            result.push(path.clone());
        } else {
            eprintln!("{}: {} not found", "Warning".yellow(), path.display());
        }
    }

    result
}

fn write_output(input_path: &Path, data: &[u8], keep_name: bool, output_dir: &Option<PathBuf>) {
    let filename = input_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    let output_filename = if keep_name {
        filename.to_string()
    } else {
        format!("(fixed) {}", filename)
    };

    let output_path = if let Some(dir) = output_dir {
        fs::create_dir_all(dir).ok();
        dir.join(&output_filename)
    } else {
        input_path.with_file_name(&output_filename)
    };

    match fs::write(&output_path, data) {
        Ok(_) => {
            println!(
                "  {} {}",
                "Saved:".green().bold(),
                output_path.display()
            );
        }
        Err(e) => {
            eprintln!(
                "  {} Could not write {}: {}",
                "[ERROR]".red(),
                output_path.display(),
                e
            );
        }
    }
}
