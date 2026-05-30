use std::path::PathBuf;

use anecdoct_utils_absolute_path::AbsolutePathBuf;

/// Runtime paths needed by exec-server child processes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecServerRuntimePaths {
    /// Stable path to the Anecdoct executable used to launch hidden helper modes.
    pub anecdoct_self_exe: AbsolutePathBuf,
    /// Path to the Linux sandbox helper alias used when the platform sandbox
    /// needs to re-enter Anecdoct by argv0.
    pub anecdoct_linux_sandbox_exe: Option<AbsolutePathBuf>,
}

impl ExecServerRuntimePaths {
    pub fn from_optional_paths(
        anecdoct_self_exe: Option<PathBuf>,
        anecdoct_linux_sandbox_exe: Option<PathBuf>,
    ) -> std::io::Result<Self> {
        let anecdoct_self_exe = anecdoct_self_exe.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Anecdoct executable path is not configured",
            )
        })?;
        Self::new(anecdoct_self_exe, anecdoct_linux_sandbox_exe)
    }

    pub fn new(
        anecdoct_self_exe: PathBuf,
        anecdoct_linux_sandbox_exe: Option<PathBuf>,
    ) -> std::io::Result<Self> {
        Ok(Self {
            anecdoct_self_exe: absolute_path(anecdoct_self_exe)?,
            anecdoct_linux_sandbox_exe: anecdoct_linux_sandbox_exe
                .map(absolute_path)
                .transpose()?,
        })
    }
}

fn absolute_path(path: PathBuf) -> std::io::Result<AbsolutePathBuf> {
    AbsolutePathBuf::from_absolute_path(path.as_path())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))
}
