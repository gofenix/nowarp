use remote_server::setup::UnsupportedReason;

use crate::terminal::event::RemoteServerSetupState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RemoteSessionExplorerState {
    /// A remote-server-backed SSH session exists or is bootstrapping, so
    /// remote metadata may still arrive and Project Explorer should show
    /// loading until roots are registered.
    RemoteServerAvailable,
    /// The terminal is remote, but there is no remote-server-backed
    /// Project Explorer path for this session.
    RemoteServerUnavailable,
    /// The remote-server preinstall check classified the host as
    /// incompatible with the bundled remote server.
    RemoteServerUnsupported { reason: UnsupportedReason },
}

impl RemoteSessionExplorerState {
    pub(crate) fn from_remote_server_setup_state(
        has_remote_server: bool,
        setup_state: Option<&RemoteServerSetupState>,
    ) -> Self {
        if has_remote_server {
            return Self::RemoteServerAvailable;
        }

        match setup_state {
            Some(RemoteServerSetupState::Unsupported { reason }) => Self::RemoteServerUnsupported {
                reason: reason.clone(),
            },
            _ => Self::RemoteServerUnavailable,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CodingPanelEnablementState {
    Enabled,
    /// An SSH command has been detected at preexec time but the remote
    /// session has not finished bootstrapping yet. The file tree should
    /// show a loading state immediately to avoid flickering the stale
    /// local tree.
    PendingRemoteSession,
    /// An SSH command is running, but this terminal cannot bootstrap a
    /// remote-server-backed session. Do not show the stale local tree.
    PendingRemoteSessionUnavailable,
    /// The active session is on a remote host.
    ///
    /// `explorer_state` is `RemoteServerAvailable` when the session is
    /// registered with `RemoteServerManager` (i.e. Auto SSH Warpification /
    /// mode 1). In that state remote repo metadata may arrive and the file
    /// tree should show loading. Other states render an unavailable message
    /// without falling back to the stale local tree.
    RemoteSession {
        explorer_state: RemoteSessionExplorerState,
    },
    UnsupportedSession,
    Disabled,
}

impl CodingPanelEnablementState {
    pub(crate) fn from_session_env(
        is_enabled: bool,
        is_remote: bool,
        is_unsupported_session: bool,
        remote_session_explorer_state: RemoteSessionExplorerState,
    ) -> Self {
        if is_remote {
            Self::RemoteSession {
                explorer_state: remote_session_explorer_state,
            }
        } else if is_unsupported_session {
            Self::UnsupportedSession
        } else if is_enabled {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }

    pub(crate) fn with_pending_ssh(
        self,
        has_pending_ssh: bool,
        can_bootstrap_pending_ssh: bool,
    ) -> Self {
        if has_pending_ssh && matches!(self, Self::Enabled) {
            if can_bootstrap_pending_ssh {
                Self::PendingRemoteSession
            } else {
                Self::PendingRemoteSessionUnavailable
            }
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use remote_server::setup::{GlibcVersion, UnsupportedReason};

    use crate::terminal::event::RemoteServerSetupState;

    use super::{CodingPanelEnablementState, RemoteSessionExplorerState};

    #[test]
    fn pending_ssh_only_shows_loading_when_bootstrap_can_follow() {
        assert_eq!(
            CodingPanelEnablementState::Enabled.with_pending_ssh(true, true),
            CodingPanelEnablementState::PendingRemoteSession
        );

        assert_eq!(
            CodingPanelEnablementState::Enabled.with_pending_ssh(true, false),
            CodingPanelEnablementState::PendingRemoteSessionUnavailable
        );
    }

    #[test]
    fn pending_ssh_does_not_override_non_enabled_states() {
        let remote = CodingPanelEnablementState::RemoteSession {
            explorer_state: RemoteSessionExplorerState::RemoteServerUnsupported {
                reason: UnsupportedReason::GlibcTooOld {
                    detected: GlibcVersion::new(2, 28),
                    required: GlibcVersion::new(2, 31),
                },
            },
        };
        assert_eq!(remote.clone().with_pending_ssh(true, true), remote);

        assert_eq!(
            CodingPanelEnablementState::Disabled.with_pending_ssh(true, true),
            CodingPanelEnablementState::Disabled
        );
    }

    #[test]
    fn remote_explorer_state_preserves_unsupported_reason() {
        let reason = UnsupportedReason::GlibcTooOld {
            detected: GlibcVersion::new(2, 28),
            required: GlibcVersion::new(2, 31),
        };
        let setup_state = RemoteServerSetupState::Unsupported {
            reason: reason.clone(),
        };

        assert_eq!(
            RemoteSessionExplorerState::from_remote_server_setup_state(false, Some(&setup_state)),
            RemoteSessionExplorerState::RemoteServerUnsupported { reason }
        );

        assert_eq!(
            RemoteSessionExplorerState::from_remote_server_setup_state(true, Some(&setup_state)),
            RemoteSessionExplorerState::RemoteServerAvailable
        );
    }
}
