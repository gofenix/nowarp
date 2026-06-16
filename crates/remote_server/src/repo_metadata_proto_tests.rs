use std::sync::Arc;

use repo_metadata::entry::{DirectoryEntry, Entry};
use repo_metadata::file_tree_update::RepoMetadataUpdate;
use repo_metadata::FileTreeEntry;
use repo_metadata::{StandingQueryContent, StandingQueryResultsDelta};
use warp_util::standardized_path::StandardizedPath;

use super::{
    file_tree_children_to_proto_entries, proto_snapshot_to_update, proto_to_repo_metadata_update,
};
use crate::proto;

fn path(path: &str) -> StandardizedPath {
    StandardizedPath::try_new(path).unwrap()
}

fn standing_delta() -> StandingQueryResultsDelta {
    StandingQueryResultsDelta {
        upserted_project_skills: vec![StandingQueryContent::file(path(
            "/repo/.agents/skills/review/SKILL.md",
        ))],
        removed_project_skills: vec![StandingQueryContent::directory(path(
            "/repo/.claude/skills",
        ))],
        upserted_project_rules: vec![StandingQueryContent::file(path("/repo/WARP.md"))],
        removed_project_rules: vec![StandingQueryContent::file(path(
            "/repo/packages/api/AGENTS.md",
        ))],
    }
}

#[test]
fn incremental_update_round_trip_preserves_standing_results_delta() {
    let update = RepoMetadataUpdate {
        repo_path: path("/repo"),
        remove_entries: Vec::new(),
        update_entries: Vec::new(),
        standing_results_delta: standing_delta(),
    };

    let proto_update = proto::RepoMetadataUpdatePush::from(&update);
    let round_trip = proto_to_repo_metadata_update(&proto_update).unwrap();

    assert_eq!(
        round_trip.standing_results_delta,
        update.standing_results_delta
    );
}

#[test]
fn snapshot_conversion_seeds_standing_results() {
    let delta = standing_delta();
    let snapshot = proto::RepoMetadataSnapshot {
        repo_path: "/repo".to_string(),
        entries: Vec::new(),
        standing_results: Some((&delta).into()),
        sync_complete: true,
    };

    let update = proto_snapshot_to_update(&snapshot).unwrap();

    assert_eq!(update.standing_results_delta, delta);
}

#[test]
fn empty_directory_children_response_still_marks_parent_as_loaded() {
    let root = Arc::new(path("/repo"));
    let entry = FileTreeEntry::from(Entry::Directory(DirectoryEntry {
        path: (*root).clone(),
        children: Vec::new(),
        ignored: false,
        loaded: true,
    }));

    let entries = file_tree_children_to_proto_entries(&entry, &root);

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].parent_path_to_replace, "/repo");
    assert!(entries[0].subtree_metadata.is_empty());
}
