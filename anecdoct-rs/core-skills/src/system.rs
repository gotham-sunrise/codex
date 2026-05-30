pub(crate) use anecdoct_skills::install_system_skills;
pub(crate) use anecdoct_skills::system_cache_root_dir;

use anecdoct_utils_absolute_path::AbsolutePathBuf;

pub(crate) fn uninstall_system_skills(anecdoct_home: &AbsolutePathBuf) {
    let _ = std::fs::remove_dir_all(system_cache_root_dir(anecdoct_home));
}
