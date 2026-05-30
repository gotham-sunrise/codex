use std::sync::Arc;
use std::sync::Weak;

use anecdoct_core::NewThread;
use anecdoct_core::StartThreadOptions;
use anecdoct_core::ThreadManager;
use anecdoct_core::config::Config;
use anecdoct_extension_api::AgentSpawnFuture;
use anecdoct_extension_api::AgentSpawner;
use anecdoct_extension_api::ExtensionRegistry;
use anecdoct_extension_api::ExtensionRegistryBuilder;
use anecdoct_protocol::ThreadId;
use anecdoct_protocol::error::AnecdoctErr;

pub(crate) fn thread_extensions<S>(guardian_agent_spawner: S) -> Arc<ExtensionRegistry<Config>>
where
    S: AgentSpawner<StartThreadOptions, Spawned = NewThread, Error = AnecdoctErr> + 'static,
{
    let mut builder = ExtensionRegistryBuilder::<Config>::new();
    anecdoct_guardian::install(&mut builder, guardian_agent_spawner);
    anecdoct_memories_extension::install(&mut builder);
    Arc::new(builder.build())
}

pub(crate) fn guardian_agent_spawner(
    thread_manager: Weak<ThreadManager>,
) -> impl AgentSpawner<StartThreadOptions, Spawned = NewThread, Error = AnecdoctErr> {
    move |forked_from_thread_id: ThreadId,
          options: StartThreadOptions|
          -> AgentSpawnFuture<'static, NewThread, AnecdoctErr> {
        let thread_manager = thread_manager.clone();
        Box::pin(async move {
            let thread_manager = thread_manager.upgrade().ok_or_else(|| {
                AnecdoctErr::UnsupportedOperation("thread manager dropped".to_string())
            })?;
            thread_manager
                .spawn_subagent(forked_from_thread_id, options)
                .await
        })
    }
}
