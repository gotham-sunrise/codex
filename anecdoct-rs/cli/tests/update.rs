use anyhow::Result;
use predicates::str::contains;
use std::path::Path;
use tempfile::TempDir;

fn anecdoct_command(anecdoct_home: &Path) -> Result<assert_cmd::Command> {
    let mut cmd = assert_cmd::Command::new(anecdoct_utils_cargo_bin::cargo_bin("anecdoct")?);
    cmd.env("ANECDOCT_HOME", anecdoct_home);
    Ok(cmd)
}

#[cfg(debug_assertions)]
#[tokio::test]
async fn update_does_not_start_interactive_prompt() -> Result<()> {
    let anecdoct_home = TempDir::new()?;

    anecdoct_command(anecdoct_home.path())?
        .arg("update")
        .assert()
        .failure()
        .stderr(contains(
            "`anecdoct update` is not available in debug builds",
        ));

    Ok(())
}
