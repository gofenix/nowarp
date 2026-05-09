use super::version_is_compatible;

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
