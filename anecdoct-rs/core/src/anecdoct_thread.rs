use crate::agent::AgentStatus;
use crate::config::ConstraintResult;
use crate::goals::ExternalGoalSet;
use crate::goals::GoalRuntimeEvent;
use crate::session::Anecdoct;
use crate::session::SessionSettingsUpdate;
use crate::session::SteerInputError;
use anecdoct_features::Feature;
use anecdoct_otel::SessionTelemetry;
use anecdoct_protocol::config_types::ApprovalsReviewer;
use anecdoct_protocol::config_types::CollaborationMode;
use anecdoct_protocol::config_types::Personality;
use anecdoct_protocol::config_types::ReasoningSummary;
use anecdoct_protocol::config_types::WindowsSandboxLevel;
use anecdoct_protocol::error::AnecdoctErr;
use anecdoct_protocol::error::Result as AnecdoctResult;
use anecdoct_protocol::lambogenius_models::ReasoningEffort;
use anecdoct_protocol::mcp::CallToolResult;
use anecdoct_protocol::models::ActivePermissionProfile;
use anecdoct_protocol::models::ContentItem;
use anecdoct_protocol::models::PermissionProfile;
use anecdoct_protocol::models::ResponseInputItem;
use anecdoct_protocol::models::ResponseItem;
use anecdoct_protocol::protocol::AskForApproval;
use anecdoct_protocol::protocol::Event;
use anecdoct_protocol::protocol::Op;
use anecdoct_protocol::protocol::SandboxPolicy;
use anecdoct_protocol::protocol::SessionConfiguredEvent;
use anecdoct_protocol::protocol::SessionSource;
use anecdoct_protocol::protocol::Submission;
use anecdoct_protocol::protocol::ThreadMemoryMode;
use anecdoct_protocol::protocol::ThreadSource;
use anecdoct_protocol::protocol::TokenUsageInfo;
use anecdoct_protocol::protocol::TurnEnvironmentSelection;
use anecdoct_protocol::protocol::W3cTraceContext;
use anecdoct_protocol::user_input::UserInput;
use anecdoct_thread_store::StoredThread;
use anecdoct_thread_store::StoredThreadHistory;
use anecdoct_thread_store::ThreadMetadataPatch;
use anecdoct_thread_store::ThreadStoreError;
use anecdoct_thread_store::ThreadStoreResult;
use anecdoct_utils_absolute_path::AbsolutePathBuf;
use rmcp::model::ReadResourceRequestParams;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::watch;

use anecdoct_rollout::state_db::StateDbHandle;

#[derive(Clone, Debug)]
pub struct ThreadConfigSnapshot {
    pub model: String,
    pub model_provider_id: String,
    pub service_tier: Option<String>,
    pub approval_policy: AskForApproval,
    pub approvals_reviewer: ApprovalsReviewer,
    pub permission_profile: PermissionProfile,
    pub active_permission_profile: Option<ActivePermissionProfile>,
    pub cwd: AbsolutePathBuf,
    pub workspace_roots: Vec<AbsolutePathBuf>,
    pub profile_workspace_roots: Vec<AbsolutePathBuf>,
    pub ephemeral: bool,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub personality: Option<Personality>,
    pub session_source: SessionSource,
    pub thread_source: Option<ThreadSource>,
}

impl ThreadConfigSnapshot {
    pub fn sandbox_policy(&self) -> SandboxPolicy {
        let file_system_sandbox_policy = self.permission_profile.file_system_sandbox_policy();
        anecdoct_sandboxing::compatibility_sandbox_policy_for_permission_profile(
            &self.permission_profile,
            &file_system_sandbox_policy,
            self.permission_profile.network_sandbox_policy(),
            self.cwd.as_path(),
        )
    }
}

/// Turn context overrides that app-server validates before starting a turn.
#[derive(Clone, Default)]
pub struct AnecdoctThreadTurnContextOverrides {
    pub cwd: Option<PathBuf>,
    pub workspace_roots: Option<Vec<AbsolutePathBuf>>,
    pub profile_workspace_roots: Option<Vec<AbsolutePathBuf>>,
    pub approval_policy: Option<AskForApproval>,
    pub approvals_reviewer: Option<ApprovalsReviewer>,
    pub sandbox_policy: Option<SandboxPolicy>,
    pub permission_profile: Option<PermissionProfile>,
    pub active_permission_profile: Option<ActivePermissionProfile>,
    pub windows_sandbox_level: Option<WindowsSandboxLevel>,
    pub model: Option<String>,
    pub effort: Option<Option<ReasoningEffort>>,
    pub summary: Option<ReasoningSummary>,
    pub service_tier: Option<Option<String>>,
    pub collaboration_mode: Option<CollaborationMode>,
    pub personality: Option<Personality>,
}

pub struct AnecdoctThread {
    pub(crate) anecdoct: Anecdoct,
    pub(crate) session_source: SessionSource,
    session_configured: SessionConfiguredEvent,
    rollout_path: Option<PathBuf>,
    out_of_band_elicitation_count: Mutex<u64>,
}

/// Conduit for the bidirectional stream of messages that compose a thread
/// (formerly called a conversation) in Anecdoct.
impl AnecdoctThread {
    pub(crate) fn new(
        anecdoct: Anecdoct,
        session_configured: SessionConfiguredEvent,
        rollout_path: Option<PathBuf>,
        session_source: SessionSource,
    ) -> Self {
        Self {
            anecdoct,
            session_source,
            session_configured,
            rollout_path,
            out_of_band_elicitation_count: Mutex::new(0),
        }
    }

    pub async fn submit(&self, op: Op) -> AnecdoctResult<String> {
        self.anecdoct.submit(op).await
    }

    /// Returns the session telemetry handle for thread-scoped production instrumentation.
    pub fn session_telemetry(&self) -> SessionTelemetry {
        self.anecdoct.session.services.session_telemetry.clone()
    }

    pub async fn shutdown_and_wait(&self) -> AnecdoctResult<()> {
        self.anecdoct.shutdown_and_wait().await
    }

    /// Wait until the underlying session loop has terminated.
    pub async fn wait_until_terminated(&self) {
        self.anecdoct.session_loop_termination.clone().await;
    }

    pub(crate) fn emit_thread_resume_lifecycle(&self) {
        for contributor in self
            .anecdoct
            .session
            .services
            .extensions
            .thread_lifecycle_contributors()
        {
            contributor.on_thread_resume(anecdoct_extension_api::ThreadResumeInput {
                session_store: &self.anecdoct.session.services.session_extension_data,
                thread_store: &self.anecdoct.session.services.thread_extension_data,
            });
        }
    }

    pub async fn apply_goal_resume_runtime_effects(&self) -> anyhow::Result<()> {
        self.anecdoct
            .session
            .goal_runtime_apply(GoalRuntimeEvent::ThreadResumed)
            .await
    }

    pub async fn continue_active_goal_if_idle(&self) -> anyhow::Result<()> {
        self.anecdoct
            .session
            .goal_runtime_apply(GoalRuntimeEvent::MaybeContinueIfIdle)
            .await
    }

    pub async fn prepare_external_goal_mutation(&self) {
        if let Err(err) = self
            .anecdoct
            .session
            .goal_runtime_apply(GoalRuntimeEvent::ExternalMutationStarting)
            .await
        {
            tracing::warn!("failed to prepare external goal mutation: {err}");
        }
    }

    pub async fn apply_external_goal_set(&self, external_set: ExternalGoalSet) {
        if let Err(err) = self
            .anecdoct
            .session
            .goal_runtime_apply(GoalRuntimeEvent::ExternalSet { external_set })
            .await
        {
            tracing::warn!("failed to apply external goal status runtime effects: {err}");
        }
    }

    pub async fn apply_external_goal_clear(&self) {
        if let Err(err) = self
            .anecdoct
            .session
            .goal_runtime_apply(GoalRuntimeEvent::ExternalClear)
            .await
        {
            tracing::warn!("failed to apply external goal clear runtime effects: {err}");
        }
    }

    #[doc(hidden)]
    pub async fn ensure_rollout_materialized(&self) {
        self.anecdoct.session.ensure_rollout_materialized().await;
    }

    #[doc(hidden)]
    pub async fn flush_rollout(&self) -> std::io::Result<()> {
        self.anecdoct.session.flush_rollout().await
    }

    pub async fn submit_with_trace(
        &self,
        op: Op,
        trace: Option<W3cTraceContext>,
    ) -> AnecdoctResult<String> {
        self.anecdoct.submit_with_trace(op, trace).await
    }

    /// Persist whether this thread is eligible for future memory generation.
    pub async fn set_thread_memory_mode(&self, mode: ThreadMemoryMode) -> anyhow::Result<()> {
        self.anecdoct.set_thread_memory_mode(mode).await
    }

    pub async fn steer_input(
        &self,
        input: Vec<UserInput>,
        expected_turn_id: Option<&str>,
        responsesapi_client_metadata: Option<HashMap<String, String>>,
    ) -> Result<String, SteerInputError> {
        self.anecdoct
            .steer_input(input, expected_turn_id, responsesapi_client_metadata)
            .await
    }

    pub async fn set_app_server_client_info(
        &self,
        app_server_client_name: Option<String>,
        app_server_client_version: Option<String>,
        mcp_elicitations_auto_deny: bool,
    ) -> ConstraintResult<()> {
        self.anecdoct
            .set_app_server_client_info(
                app_server_client_name,
                app_server_client_version,
                mcp_elicitations_auto_deny,
            )
            .await
    }

    /// Validate persistent turn context overrides without committing them.
    pub async fn validate_turn_context_overrides(
        &self,
        overrides: AnecdoctThreadTurnContextOverrides,
    ) -> ConstraintResult<()> {
        let AnecdoctThreadTurnContextOverrides {
            cwd,
            workspace_roots,
            profile_workspace_roots,
            approval_policy,
            approvals_reviewer,
            sandbox_policy,
            permission_profile,
            active_permission_profile,
            windows_sandbox_level,
            model,
            effort,
            summary,
            service_tier,
            collaboration_mode,
            personality,
        } = overrides;
        let collaboration_mode = if let Some(collaboration_mode) = collaboration_mode {
            collaboration_mode
        } else {
            self.anecdoct
                .session
                .collaboration_mode()
                .await
                .with_updates(model, effort, /*developer_instructions*/ None)
        };

        let updates = SessionSettingsUpdate {
            cwd,
            workspace_roots,
            profile_workspace_roots,
            approval_policy,
            approvals_reviewer,
            sandbox_policy,
            permission_profile,
            active_permission_profile,
            windows_sandbox_level,
            collaboration_mode: Some(collaboration_mode),
            reasoning_summary: summary,
            service_tier,
            personality,
            ..Default::default()
        };
        self.anecdoct.session.validate_settings(&updates).await
    }

    /// Use sparingly: this is intended to be removed soon.
    pub async fn submit_with_id(&self, sub: Submission) -> AnecdoctResult<()> {
        self.anecdoct.submit_with_id(sub).await
    }

    pub async fn next_event(&self) -> AnecdoctResult<Event> {
        self.anecdoct.next_event().await
    }

    pub async fn agent_status(&self) -> AgentStatus {
        self.anecdoct.agent_status().await
    }

    pub(crate) fn subscribe_status(&self) -> watch::Receiver<AgentStatus> {
        self.anecdoct.agent_status.clone()
    }

    /// Returns the complete token usage snapshot currently cached for this thread.
    ///
    /// This accessor is intentionally narrower than direct session access: it lets
    /// app-server lifecycle paths replay restored usage after resume or fork without
    /// exposing broader session mutation authority. A caller that only reads
    /// `total_token_usage` would drop last-turn usage and make the v2
    /// `thread/tokenUsage/updated` payload incomplete.
    pub async fn token_usage_info(&self) -> Option<TokenUsageInfo> {
        self.anecdoct.session.token_usage_info().await
    }

    /// Records a user-role session-prefix message without creating a new user turn boundary.
    pub(crate) async fn inject_user_message_without_turn(&self, message: String) {
        let message = ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText { text: message }],
            phase: None,
        };
        let pending_item = match pending_message_input_item(&message) {
            Ok(pending_item) => pending_item,
            Err(err) => {
                debug_assert!(false, "session-prefix message append should succeed: {err}");
                return;
            }
        };
        if self
            .anecdoct
            .session
            .inject_response_items(vec![pending_item])
            .await
            .is_err()
        {
            let turn_context = self.anecdoct.session.new_default_turn().await;
            self.anecdoct
                .session
                .record_conversation_items(turn_context.as_ref(), &[message])
                .await;
        }
    }

    /// Append a prebuilt message to the thread history without treating it as a user turn.
    ///
    /// If the thread already has an active turn, the message is queued as pending input for that
    /// turn. Otherwise it is queued at session scope and a regular turn is started so the agent
    /// can consume that pending input through the normal turn pipeline.
    #[cfg(test)]
    pub(crate) async fn append_message(&self, message: ResponseItem) -> AnecdoctResult<String> {
        let submission_id = uuid::Uuid::new_v4().to_string();
        let pending_item = pending_message_input_item(&message)?;
        if let Err(items) = self
            .anecdoct
            .session
            .inject_response_items(vec![pending_item])
            .await
        {
            self.anecdoct
                .session
                .queue_response_items_for_next_turn(items)
                .await;
            self.anecdoct
                .session
                .maybe_start_turn_for_pending_work()
                .await;
        }

        Ok(submission_id)
    }

    /// Append raw Responses API items to the thread's model-visible history.
    pub async fn inject_response_items(&self, items: Vec<ResponseItem>) -> AnecdoctResult<()> {
        if items.is_empty() {
            return Err(AnecdoctErr::InvalidRequest(
                "items must not be empty".to_string(),
            ));
        }

        let turn_context = self.anecdoct.session.new_default_turn().await;
        if self
            .anecdoct
            .session
            .reference_context_item()
            .await
            .is_none()
        {
            self.anecdoct
                .session
                .record_context_updates_and_set_reference_context_item(turn_context.as_ref())
                .await;
        }
        self.anecdoct
            .session
            .record_conversation_items(turn_context.as_ref(), &items)
            .await;
        self.anecdoct.session.flush_rollout().await?;
        Ok(())
    }

    pub fn rollout_path(&self) -> Option<PathBuf> {
        self.rollout_path.clone()
    }

    pub fn session_configured(&self) -> SessionConfiguredEvent {
        self.session_configured.clone()
    }

    pub(crate) fn is_running(&self) -> bool {
        !self.anecdoct.tx_sub.is_closed()
    }

    pub async fn guardian_trunk_rollout_path(&self) -> Option<PathBuf> {
        self.anecdoct
            .session
            .guardian_review_session
            .trunk_rollout_path()
            .await
    }

    pub async fn load_history(
        &self,
        include_archived: bool,
    ) -> ThreadStoreResult<StoredThreadHistory> {
        let live_thread = self
            .anecdoct
            .session
            .live_thread_for_persistence("load history")
            .map_err(|err| ThreadStoreError::Internal {
                message: err.to_string(),
            })?;
        live_thread.load_history(include_archived).await
    }

    pub async fn read_thread(
        &self,
        include_archived: bool,
        include_history: bool,
    ) -> ThreadStoreResult<StoredThread> {
        let live_thread = self
            .anecdoct
            .session
            .live_thread_for_persistence("read thread")
            .map_err(|err| ThreadStoreError::Internal {
                message: err.to_string(),
            })?;
        live_thread
            .read_thread(include_archived, include_history)
            .await
    }

    pub async fn update_thread_metadata(
        &self,
        patch: ThreadMetadataPatch,
        include_archived: bool,
    ) -> ThreadStoreResult<StoredThread> {
        let live_thread = self
            .anecdoct
            .session
            .live_thread_for_persistence("update thread metadata")
            .map_err(|err| ThreadStoreError::Internal {
                message: err.to_string(),
            })?;
        live_thread.update_metadata(patch, include_archived).await
    }

    pub fn state_db(&self) -> Option<StateDbHandle> {
        self.anecdoct.state_db()
    }

    pub async fn config_snapshot(&self) -> ThreadConfigSnapshot {
        self.anecdoct.thread_config_snapshot().await
    }

    pub async fn config(&self) -> Arc<crate::config::Config> {
        self.anecdoct.session.get_config().await
    }

    /// Refresh the thread's layer-backed user config state from a caller-supplied
    /// config snapshot. Thread-scoped layers and session-static settings remain
    /// unchanged.
    pub async fn refresh_runtime_config(&self, next_config: crate::config::Config) {
        self.anecdoct
            .session
            .refresh_runtime_config(next_config)
            .await;
    }

    pub async fn environment_selections(&self) -> Vec<TurnEnvironmentSelection> {
        self.anecdoct.thread_environment_selections().await
    }

    pub async fn read_mcp_resource(
        &self,
        server: &str,
        uri: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let result = self
            .anecdoct
            .session
            .read_resource(
                server,
                ReadResourceRequestParams {
                    meta: None,
                    uri: uri.to_string(),
                },
            )
            .await?;

        Ok(serde_json::to_value(result)?)
    }

    pub async fn call_mcp_tool(
        &self,
        server: &str,
        tool: &str,
        arguments: Option<serde_json::Value>,
        meta: Option<serde_json::Value>,
    ) -> anyhow::Result<CallToolResult> {
        self.anecdoct
            .session
            .call_tool(server, tool, arguments, meta)
            .await
    }

    pub fn enabled(&self, feature: Feature) -> bool {
        self.anecdoct.enabled(feature)
    }

    pub async fn increment_out_of_band_elicitation_count(&self) -> AnecdoctResult<u64> {
        let mut guard = self.out_of_band_elicitation_count.lock().await;
        let was_zero = *guard == 0;
        *guard = guard.checked_add(1).ok_or_else(|| {
            AnecdoctErr::Fatal("out-of-band elicitation count overflowed".to_string())
        })?;

        if was_zero {
            self.anecdoct
                .session
                .set_out_of_band_elicitation_pause_state(/*paused*/ true);
        }

        Ok(*guard)
    }

    pub async fn decrement_out_of_band_elicitation_count(&self) -> AnecdoctResult<u64> {
        let mut guard = self.out_of_band_elicitation_count.lock().await;
        if *guard == 0 {
            return Err(AnecdoctErr::InvalidRequest(
                "out-of-band elicitation count is already zero".to_string(),
            ));
        }

        *guard -= 1;
        let now_zero = *guard == 0;
        if now_zero {
            self.anecdoct
                .session
                .set_out_of_band_elicitation_pause_state(/*paused*/ false);
        }

        Ok(*guard)
    }
}

fn pending_message_input_item(message: &ResponseItem) -> AnecdoctResult<ResponseInputItem> {
    match message {
        ResponseItem::Message {
            role,
            content,
            phase,
            ..
        } => Ok(ResponseInputItem::Message {
            role: role.clone(),
            content: content.clone(),
            phase: phase.clone(),
        }),
        _ => Err(AnecdoctErr::InvalidRequest(
            "append_message only supports ResponseItem::Message".to_string(),
        )),
    }
}
