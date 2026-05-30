#[cfg(not(unix))]
fn main() {
    eprintln!("anecdoct-execve-wrapper is only implemented for UNIX");
    std::process::exit(1);
}

#[cfg(unix)]
pub use anecdoct_shell_escalation::main_execve_wrapper as main;
