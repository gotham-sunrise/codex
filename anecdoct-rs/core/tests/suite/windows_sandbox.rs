use anecdoct_core::exec::ExecCapturePolicy;
use anecdoct_core::exec::ExecParams;
use anecdoct_core::exec::process_exec_tool_call;
use anecdoct_core::sandboxing::SandboxPermissions;
use anecdoct_protocol::config_types::WindowsSandboxLevel;
use anecdoct_protocol::exec_output::ExecToolCallOutput;
use anecdoct_protocol::models::PermissionProfile;
use anecdoct_protocol::permissions::FileSystemAccessMode;
use anecdoct_protocol::permissions::FileSystemPath;
use anecdoct_protocol::permissions::FileSystemSandboxEntry;
use anecdoct_protocol::permissions::FileSystemSandboxPolicy;
use anecdoct_protocol::permissions::FileSystemSpecialPath;
use anecdoct_protocol::permissions::NetworkSandboxPolicy;
use anyhow::Context;
use core_test_support::PathExt;
use pretty_assertions::assert_eq;
use serial_test::serial;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;
use tempfile::TempDir;

struct EnvVarGuard {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &std::ffi::OsStr) -> Self {
        let original = std::env::var_os(key);
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, original }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.original {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }
}

fn stage_windows_sandbox_helpers() -> anyhow::Result<()> {
    let test_exe = std::env::current_exe().context("resolve current Windows test executable")?;
    let test_exe_dir = test_exe
        .parent()
        .context("Windows test executable should have a parent directory")?;
    let resources_dir = test_exe_dir.join("anecdoct-resources");
    std::fs::create_dir_all(&resources_dir)?;
    for helper_name in ["anecdoct-windows-sandbox-setup", "anecdoct-command-runner"] {
        let helper = anecdoct_utils_cargo_bin::cargo_bin(helper_name)?;
        let file_name = Path::new(helper_name).with_extension("exe");
        std::fs::copy(helper, resources_dir.join(file_name))?;
    }
    Ok(())
}

#[tokio::test]
#[serial(anecdoct_home)]
async fn windows_restricted_token_rejects_exact_and_glob_deny_read_policy() -> anyhow::Result<()> {
    let temp_home = TempDir::new()?;
    let _anecdoct_home_guard = EnvVarGuard::set("ANECDOCT_HOME", temp_home.path().as_os_str());
    let workspace = TempDir::new()?;
    let cwd = dunce::canonicalize(workspace.path())?.abs();
    let secret = cwd.join("secret.env");
    let future_secret = cwd.join("future.env");
    let public = cwd.join("public.txt");
    std::fs::write(&secret, "glob secret\n")?;
    std::fs::write(&public, "public ok\n")?;

    let file_system_sandbox_policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
            },
            access: FileSystemAccessMode::Write,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::GlobPattern {
                pattern: "**/*.env".to_string(),
            },
            access: FileSystemAccessMode::None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: future_secret,
            },
            access: FileSystemAccessMode::None,
        },
    ]);
    let permission_profile = PermissionProfile::from_runtime_permissions(
        &file_system_sandbox_policy,
        NetworkSandboxPolicy::Restricted,
    );

    let err = process_exec_tool_call(
        ExecParams {
            command: vec![
                "cmd.exe".to_string(),
                "/D".to_string(),
                "/C".to_string(),
                "type secret.env >NUL 2>NUL & echo exact secret 1>future.env 2>NUL & type future.env 2>NUL & type public.txt & exit /B 0"
                    .to_string(),
            ],
            cwd: cwd.clone(),
            expiration: 10_000.into(),
            capture_policy: ExecCapturePolicy::ShellTool,
            env: HashMap::new(),
            network: None,
            sandbox_permissions: SandboxPermissions::UseDefault,
            windows_sandbox_level: WindowsSandboxLevel::RestrictedToken,
            windows_sandbox_private_desktop: false,
            justification: None,
            arg0: None,
        },
        &permission_profile,
        &cwd,
        &None,
        /*use_legacy_landlock*/ false,
        /*stdout_stream*/ None,
    )
    .await
    .expect_err("restricted-token sandbox should reject deny-read restrictions");

    assert_eq!(
        err.to_string(),
        "unsupported operation: windows unelevated restricted-token sandbox cannot enforce deny-read restrictions directly; refusing to run unsandboxed"
    );
    Ok(())
}

#[tokio::test]
#[serial(anecdoct_home)]
async fn windows_elevated_enforces_exact_and_glob_deny_read_policy() -> anyhow::Result<()> {
    let temp_home = TempDir::new()?;
    let _anecdoct_home_guard = EnvVarGuard::set("ANECDOCT_HOME", temp_home.path().as_os_str());
    stage_windows_sandbox_helpers()?;
    let workspace = TempDir::new()?;
    let cwd = dunce::canonicalize(workspace.path())?.abs();
    let glob_secret = cwd.join("secret.env");
    let exact_secret = cwd.join("exact-secret.txt");
    let public = cwd.join("public.txt");
    std::fs::write(&glob_secret, "glob secret\n")?;
    std::fs::write(&exact_secret, "exact secret\n")?;
    std::fs::write(&public, "public ok\n")?;

    let file_system_sandbox_policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
            },
            access: FileSystemAccessMode::Write,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::GlobPattern {
                pattern: "**/*.env".to_string(),
            },
            access: FileSystemAccessMode::None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Path { path: exact_secret },
            access: FileSystemAccessMode::None,
        },
    ]);
    let permission_profile = PermissionProfile::from_runtime_permissions(
        &file_system_sandbox_policy,
        NetworkSandboxPolicy::Restricted,
    );

    let ExecToolCallOutput {
        exit_code,
        stdout,
        stderr,
        ..
    } = process_exec_tool_call(
        ExecParams {
            command: vec![
                "cmd.exe".to_string(),
                "/D".to_string(),
                "/C".to_string(),
                "(type secret.env 1>NUL 2>NUL && echo GLOB-READ || echo GLOB-DENIED) & (type exact-secret.txt 1>NUL 2>NUL && echo EXACT-READ || echo EXACT-DENIED) & type public.txt".to_string(),
            ],
            cwd: cwd.clone(),
            expiration: 10_000.into(),
            capture_policy: ExecCapturePolicy::ShellTool,
            env: HashMap::new(),
            network: None,
            sandbox_permissions: SandboxPermissions::UseDefault,
            windows_sandbox_level: WindowsSandboxLevel::Elevated,
            windows_sandbox_private_desktop: false,
            justification: None,
            arg0: None,
        },
        &permission_profile,
        &cwd,
        &None,
        /*use_legacy_landlock*/ false,
        /*stdout_stream*/ None,
    )
    .await?;

    assert_eq!(exit_code, 0, "sandboxed command should complete");
    assert!(
        stdout.text.contains("GLOB-DENIED"),
        "glob deny-read should block the secret: {stdout:?}"
    );
    assert!(
        !stdout.text.contains("GLOB-READ"),
        "glob deny-read should not allow the secret: {stdout:?}"
    );
    assert!(
        stdout.text.contains("EXACT-DENIED"),
        "exact deny-read should block the secret: {stdout:?}"
    );
    assert!(
        !stdout.text.contains("EXACT-READ"),
        "exact deny-read should not allow the secret: {stdout:?}"
    );
    assert!(
        stdout.text.contains("public ok"),
        "allowed reads should still work: {stdout:?}"
    );
    assert_eq!(stderr.text, "");
    Ok(())
}
