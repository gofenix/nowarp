use crate::transport::Error;
use futures::channel::oneshot;
use warp_core::SessionId;
use warp_util::standardized_path::StandardizedPath;
use warpui_core::App;

use super::{
    should_run_preinstall_after_binary_check, version_is_compatible, HostRequestError,
    PendingHostRequest, RemoteServerManager, RipgrepSearchParams,
};
use crate::proto::{host_scoped_request, ClientMessage, WriteFile};
use crate::protocol::RequestId;
use crate::HostId;

#[test]
fn release_tags_must_match_when_both_sides_report_versions() {
    assert!(version_is_compatible(Some("v1"), "v1"));
    assert!(!version_is_compatible(Some("v1"), "v2"));
}

#[test]
fn dev_client_without_release_tag_accepts_tagged_or_untagged_server() {
    assert!(version_is_compatible(None, ""));
    assert!(version_is_compatible(None, "v0.2026.05.06.15.42.stable_03"));
}

#[test]
fn release_client_rejects_untagged_server() {
    assert!(!version_is_compatible(Some("v1"), ""));
}

#[test]
fn installed_binary_skips_preinstall_gate() {
    assert!(!should_run_preinstall_after_binary_check(&Ok(true)));
}

#[test]
fn missing_or_unusable_binary_runs_preinstall_gate() {
    assert!(should_run_preinstall_after_binary_check(&Ok(false)));
    assert!(should_run_preinstall_after_binary_check(&Err(
        Error::TimedOut
    )));
}

#[test]
fn abort_host_request_removes_pending_request_and_resolves_caller() {
    App::test((), |mut app| async move {
        let manager = app.add_model(RemoteServerManager::new);
        let host_id = HostId::new("test-host".to_string());
        let request_id = RequestId::new();
        let (result_tx, result_rx) = oneshot::channel();
        let msg = ClientMessage::host_scoped(
            request_id.to_string(),
            host_scoped_request::Message::WriteFile(WriteFile {
                path: "/tmp/test".to_string(),
                content: String::new(),
            }),
        );

        manager.update(&mut app, |manager, _ctx| {
            manager.pending_host_requests.insert(
                request_id.clone(),
                PendingHostRequest {
                    host_id,
                    dispatched_session_id: SessionId::from(1),
                    msg,
                    result_tx,
                    timeout_abort: None,
                },
            );
            manager.abort_host_request(&request_id);
            assert!(!manager.pending_host_requests.contains_key(&request_id));
        });

        assert!(matches!(
            result_rx.await.expect("manager should resolve caller"),
            Err(HostRequestError::Aborted)
        ));
    });
}

#[test]
fn start_ripgrep_search_without_connected_host_resolves_immediately() {
    App::test((), |mut app| async move {
        let manager = app.add_model(RemoteServerManager::new);
        let host_id = HostId::new("missing-host".to_string());
        let pending = manager.update(&mut app, |manager, _ctx| {
            manager.start_ripgrep_search(
                &host_id,
                RipgrepSearchParams {
                    pattern: "needle".to_string(),
                    roots: vec![StandardizedPath::try_new("/repo").unwrap()],
                    ignore_case: false,
                    multiline: false,
                    max_matches: 100,
                },
            )
        });

        assert!(matches!(
            pending.result().await,
            Err(HostRequestError::AllSessionsDisconnected)
        ));
    });
}
