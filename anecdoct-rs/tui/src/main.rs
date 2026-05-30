use anecdoct_arg0::Arg0DispatchPaths;
use anecdoct_arg0::arg0_dispatch_or_else;
use anecdoct_config::LoaderOverrides;
use anecdoct_tui::AppExitInfo;
use anecdoct_tui::Cli;
use anecdoct_tui::ExitReason;
use anecdoct_tui::run_main;
use anecdoct_utils_cli::CliConfigOverrides;
use anecdoct_utils_cli::resume_command;
use clap::Parser;
use supports_color::Stream;

fn format_exit_messages(exit_info: AppExitInfo, color_enabled: bool) -> Vec<String> {
    let AppExitInfo {
        token_usage,
        thread_id,
        ..
    } = exit_info;

    let mut lines = Vec::new();
    if !token_usage.is_zero() {
        lines.push(token_usage.to_string());
    }

    if let Some(resume_cmd) = resume_command(/*thread_name*/ None, thread_id) {
        let command = if color_enabled {
            format!("\u{1b}[36m{resume_cmd}\u{1b}[39m")
        } else {
            resume_cmd
        };
        lines.push(format!("To continue this session, run {command}"));
    }

    lines
}

#[derive(Parser, Debug)]
struct TopCli {
    #[clap(flatten)]
    config_overrides: CliConfigOverrides,

    #[clap(flatten)]
    inner: Cli,
}

fn main() -> anyhow::Result<()> {
    arg0_dispatch_or_else(|arg0_paths: Arg0DispatchPaths| async move {
        let top_cli = TopCli::parse();
        let mut inner = top_cli.inner;
        inner
            .config_overrides
            .raw_overrides
            .splice(0..0, top_cli.config_overrides.raw_overrides);
        let exit_info = run_main(
            inner,
            arg0_paths,
            LoaderOverrides::default(),
            /*explicit_remote_endpoint*/ None,
        )
        .await?;
        match exit_info.exit_reason {
            ExitReason::Fatal(message) => {
                eprintln!("ERROR: {message}");
                std::process::exit(1);
            }
            ExitReason::UserRequested => {}
        }

        let color_enabled = supports_color::on(Stream::Stdout).is_some();
        for line in format_exit_messages(exit_info, color_enabled) {
            println!("{line}");
        }
        Ok(())
    })
}
