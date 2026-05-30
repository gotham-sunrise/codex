//! Implements the MultiAgentV2 collaboration tool surface.

use crate::agent::AgentStatus;
use crate::agent::agent_resolver::resolve_agent_target;
use crate::function_tool::FunctionCallError;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::handlers::multi_agents_common::*;
use crate::tools::handlers::parse_arguments;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use anecdoct_protocol::AgentPath;
use anecdoct_protocol::lambogenius_models::ReasoningEffort;
use anecdoct_protocol::models::ResponseInputItem;
use anecdoct_protocol::protocol::CollabAgentInteractionBeginEvent;
use anecdoct_protocol::protocol::CollabAgentInteractionEndEvent;
use anecdoct_protocol::protocol::CollabAgentSpawnBeginEvent;
use anecdoct_protocol::protocol::CollabAgentSpawnEndEvent;
use anecdoct_protocol::protocol::CollabCloseBeginEvent;
use anecdoct_protocol::protocol::CollabCloseEndEvent;
use anecdoct_protocol::protocol::CollabWaitingBeginEvent;
use anecdoct_protocol::protocol::CollabWaitingEndEvent;
use anecdoct_protocol::user_input::UserInput;
use anecdoct_tools::ToolName;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as JsonValue;

pub(crate) use close_agent::Handler as CloseAgentHandler;
pub(crate) use followup_task::Handler as FollowupTaskHandler;
pub(crate) use list_agents::Handler as ListAgentsHandler;
pub(crate) use send_message::Handler as SendMessageHandler;
pub(crate) use spawn::Handler as SpawnAgentHandler;
pub(crate) use wait::Handler as WaitAgentHandler;

mod close_agent;
mod followup_task;
mod list_agents;
mod message_tool;
mod send_message;
mod spawn;
pub(crate) mod wait;
