use crate::config::Config;
pub use anecdoct_rollout::ARCHIVED_SESSIONS_SUBDIR;
pub use anecdoct_rollout::Cursor;
pub use anecdoct_rollout::EventPersistenceMode;
pub use anecdoct_rollout::INTERACTIVE_SESSION_SOURCES;
pub use anecdoct_rollout::RolloutRecorder;
pub use anecdoct_rollout::RolloutRecorderParams;
pub use anecdoct_rollout::SESSIONS_SUBDIR;
pub use anecdoct_rollout::SessionMeta;
pub use anecdoct_rollout::SortDirection;
pub use anecdoct_rollout::ThreadItem;
pub use anecdoct_rollout::ThreadSortKey;
pub use anecdoct_rollout::ThreadsPage;
pub use anecdoct_rollout::append_thread_name;
pub use anecdoct_rollout::find_archived_thread_path_by_id_str;
#[deprecated(note = "use find_thread_path_by_id_str")]
pub use anecdoct_rollout::find_conversation_path_by_id_str;
pub use anecdoct_rollout::find_thread_meta_by_name_str;
pub use anecdoct_rollout::find_thread_name_by_id;
pub use anecdoct_rollout::find_thread_names_by_ids;
pub use anecdoct_rollout::find_thread_path_by_id_str;
pub use anecdoct_rollout::parse_cursor;
pub use anecdoct_rollout::read_head_for_summary;
pub use anecdoct_rollout::read_session_meta_line;
pub use anecdoct_rollout::rollout_date_parts;

impl anecdoct_rollout::RolloutConfigView for Config {
    fn anecdoct_home(&self) -> &std::path::Path {
        self.anecdoct_home.as_path()
    }

    fn sqlite_home(&self) -> &std::path::Path {
        self.sqlite_home.as_path()
    }

    fn cwd(&self) -> &std::path::Path {
        self.cwd.as_path()
    }

    fn model_provider_id(&self) -> &str {
        self.model_provider_id.as_str()
    }

    fn generate_memories(&self) -> bool {
        self.memories.generate_memories
    }
}

pub(crate) mod list {
    pub use anecdoct_rollout::find_thread_path_by_id_str;
}

#[cfg(test)]
pub(crate) mod recorder {
    pub use anecdoct_rollout::RolloutRecorder;
}

pub(crate) use crate::session_rollout_init_error::map_session_init_error;

pub(crate) mod truncation {
    pub(crate) use crate::thread_rollout_truncation::*;
}
