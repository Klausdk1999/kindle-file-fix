use colored::Colorize;
use kindle_fix_core::FixReport;

pub fn print_report(report: &FixReport, quiet: bool) {
    if quiet {
        return;
    }

    for fix in &report.fixes_applied {
        println!("  {} {}", "[FIXED]".green().bold(), fix.details);
    }

    for warning in &report.warnings {
        println!("  {} {}", "[WARN]".yellow().bold(), warning);
    }

    if report.fixes_applied.is_empty() && report.warnings.is_empty() {
        println!("  {} No issues found", "[OK]".blue().bold());
    }
}
