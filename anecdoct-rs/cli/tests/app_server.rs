use std::path::Path;

use anyhow::Result;
use predicates::str::contains;
use tempfile::TempDir;

fn anecdoct_command(anecdoct_home: &Path) -> Result<assert_cmd::Command> {
    let mut cmd = assert_cmd::Command::new(anecdoct_utils_cargo_bin::cargo_bin("anecdoct")?);
    cmd.env("ANECDOCT_HOME", anecdoct_home);
    Ok(cmd)
}

#[test]
fn strict_config_rejects_unknown_config_fields_for_app_server() -> Result<()> {
    let anecdoct_home = TempDir::new()?;
    std::fs::write(
        anecdoct_home.path().join("config.toml"),
        r#"
foo = "bar"
"#,
    )?;

    let mut cmd = anecdoct_command(anecdoct_home.path())?;
    cmd.args(["app-server", "--strict-config", "--listen", "off"])
        .assert()
        .failure()
        .stderr(contains("unknown configuration field"));

    Ok(())
}
