use anecdoct_core::config::Constrained;
use anecdoct_protocol::config_types::CollaborationMode;
use anecdoct_protocol::config_types::ModeKind;
use anecdoct_protocol::config_types::Settings;
use anecdoct_protocol::protocol::AskForApproval;
use anecdoct_protocol::protocol::EventMsg;
use anecdoct_protocol::protocol::Op;
use anyhow::Result;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::test_anecdoct::test_anecdoct;
use core_test_support::wait_for_event;
use tempfile::TempDir;

fn collab_mode_with_instructions(instructions: Option<&str>) -> CollaborationMode {
    CollaborationMode {
        mode: ModeKind::Default,
        settings: Settings {
            model: "gpt-5.4".to_string(),
            reasoning_effort: None,
            developer_instructions: instructions.map(str::to_string),
        },
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn override_turn_context_without_user_turn_does_not_record_permissions_update() -> Result<()>
{
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let mut builder = test_anecdoct().with_config(|config| {
        config.permissions.approval_policy = Constrained::allow_any(AskForApproval::OnRequest);
    });
    let test = builder.build(&server).await?;

    test.anecdoct
        .submit(Op::OverrideTurnContext {
            cwd: None,
            approval_policy: Some(AskForApproval::Never),
            approvals_reviewer: None,
            sandbox_policy: None,
            permission_profile: None,
            windows_sandbox_level: None,
            model: None,
            effort: None,
            summary: None,
            service_tier: None,
            collaboration_mode: None,
            personality: None,
        })
        .await?;

    test.anecdoct.submit(Op::Shutdown).await?;
    wait_for_event(&test.anecdoct, |ev| {
        matches!(ev, EventMsg::ShutdownComplete)
    })
    .await;

    let rollout_path = test.anecdoct.rollout_path().expect("rollout path");
    assert!(
        !rollout_path.exists(),
        "did not expect a rollout before a new user turn"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn override_turn_context_without_user_turn_does_not_record_environment_update() -> Result<()>
{
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let test = test_anecdoct().build(&server).await?;
    let new_cwd = TempDir::new()?;

    test.anecdoct
        .submit(Op::OverrideTurnContext {
            cwd: Some(new_cwd.path().to_path_buf()),
            approval_policy: None,
            approvals_reviewer: None,
            sandbox_policy: None,
            permission_profile: None,
            windows_sandbox_level: None,
            model: None,
            effort: None,
            summary: None,
            service_tier: None,
            collaboration_mode: None,
            personality: None,
        })
        .await?;

    test.anecdoct.submit(Op::Shutdown).await?;
    wait_for_event(&test.anecdoct, |ev| {
        matches!(ev, EventMsg::ShutdownComplete)
    })
    .await;

    let rollout_path = test.anecdoct.rollout_path().expect("rollout path");
    assert!(
        !rollout_path.exists(),
        "did not expect a rollout before a new user turn"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn override_turn_context_without_user_turn_does_not_record_collaboration_update() -> Result<()>
{
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let test = test_anecdoct().build(&server).await?;
    let collab_text = "override collaboration instructions";
    let collaboration_mode = collab_mode_with_instructions(Some(collab_text));

    test.anecdoct
        .submit(Op::OverrideTurnContext {
            cwd: None,
            approval_policy: None,
            approvals_reviewer: None,
            sandbox_policy: None,
            permission_profile: None,
            windows_sandbox_level: None,
            model: None,
            effort: None,
            summary: None,
            service_tier: None,
            collaboration_mode: Some(collaboration_mode),
            personality: None,
        })
        .await?;

    test.anecdoct.submit(Op::Shutdown).await?;
    wait_for_event(&test.anecdoct, |ev| {
        matches!(ev, EventMsg::ShutdownComplete)
    })
    .await;

    let rollout_path = test.anecdoct.rollout_path().expect("rollout path");
    assert!(
        !rollout_path.exists(),
        "did not expect a rollout before a new user turn"
    );

    Ok(())
}
