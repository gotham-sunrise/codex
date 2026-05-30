use std::collections::HashMap;
use std::sync::Arc;

use crate::SkillsManager;
use crate::agent::AgentControl;
use crate::attestation::AttestationProvider;
use crate::client::ModelClient;
use crate::config::StartedNetworkProxy;
use crate::exec_policy::ExecPolicyManager;
use crate::guardian::GuardianRejection;
use crate::guardian::GuardianRejectionCircuitBreaker;
use crate::mcp::McpManager;
use crate::tools::code_mode::CodeModeService;
use crate::tools::network_approval::NetworkApprovalService;
use crate::tools::sandboxing::ApprovalStore;
use crate::unified_exec::UnifiedExecProcessManager;
use anecdoct_analytics::AnalyticsEventsClient;
use anecdoct_core_plugins::PluginsManager;
use anecdoct_exec_server::EnvironmentManager;
use anecdoct_extension_api::ExtensionData;
use anecdoct_extension_api::ExtensionRegistry;
use anecdoct_hooks::Hooks;
use anecdoct_login::AuthManager;
use anecdoct_mcp::McpConnectionManager;
use anecdoct_models_manager::manager::SharedModelsManager;
use anecdoct_otel::SessionTelemetry;
use anecdoct_rollout::state_db::StateDbHandle;
use anecdoct_rollout_trace::ThreadTraceContext;
use anecdoct_thread_store::LiveThread;
use anecdoct_thread_store::ThreadStore;
use arc_swap::ArcSwap;
use std::path::PathBuf;
use tokio::runtime::Handle;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

pub(crate) struct SessionServices {
    pub(crate) mcp_connection_manager: Arc<RwLock<McpConnectionManager>>,
    pub(crate) mcp_startup_cancellation_token: Mutex<CancellationToken>,
    pub(crate) unified_exec_manager: UnifiedExecProcessManager,
    #[cfg_attr(not(unix), allow(dead_code))]
    pub(crate) shell_zsh_path: Option<PathBuf>,
    #[cfg_attr(not(unix), allow(dead_code))]
    pub(crate) main_execve_wrapper_exe: Option<PathBuf>,
    pub(crate) analytics_events_client: AnalyticsEventsClient,
    pub(crate) hooks: ArcSwap<Hooks>,
    pub(crate) rollout_thread_trace: ThreadTraceContext,
    pub(crate) user_shell: Arc<crate::shell::Shell>,
    pub(crate) shell_snapshot_tx: watch::Sender<Option<Arc<crate::shell_snapshot::ShellSnapshot>>>,
    pub(crate) show_raw_agent_reasoning: bool,
    pub(crate) exec_policy: Arc<ExecPolicyManager>,
    pub(crate) auth_manager: Arc<AuthManager>,
    pub(crate) models_manager: SharedModelsManager,
    pub(crate) session_telemetry: SessionTelemetry,
    pub(crate) tool_approvals: Mutex<ApprovalStore>,
    pub(crate) guardian_rejections: Mutex<HashMap<String, GuardianRejection>>,
    pub(crate) guardian_rejection_circuit_breaker: Mutex<GuardianRejectionCircuitBreaker>,
    pub(crate) runtime_handle: Handle,
    pub(crate) skills_manager: Arc<SkillsManager>,
    pub(crate) plugins_manager: Arc<PluginsManager>,
    pub(crate) mcp_manager: Arc<McpManager>,
    pub(crate) extensions: Arc<ExtensionRegistry<crate::config::Config>>,
    pub(crate) session_extension_data: ExtensionData,
    pub(crate) thread_extension_data: ExtensionData,
    pub(crate) agent_control: AgentControl,
    pub(crate) network_proxy: Option<StartedNetworkProxy>,
    pub(crate) network_approval: Arc<NetworkApprovalService>,
    pub(crate) state_db: Option<StateDbHandle>,
    pub(crate) live_thread: Option<LiveThread>,
    pub(crate) thread_store: Arc<dyn ThreadStore>,
    pub(crate) attestation_provider: Option<Arc<dyn AttestationProvider>>,
    /// Session-scoped model client shared across turns.
    pub(crate) model_client: ModelClient,
    pub(crate) code_mode_service: CodeModeService,
    /// Shared process-level environment registry. Sessions carry an `Arc` handle so they can pass
    /// the same manager through child-thread spawn paths without reconstructing it.
    pub(crate) environment_manager: Arc<EnvironmentManager>,
}
