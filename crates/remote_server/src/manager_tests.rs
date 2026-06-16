use crate::transport::Error;

use super::{should_run_preinstall_after_binary_check, version_is_compatible};

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
