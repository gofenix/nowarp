use warp_core::user_preferences::GetUserPreferences as _;
use warpui::{App, SingletonEntity};

use super::{
    has_completed_local_onboarding, should_complete_post_auth_onboarding_without_showing,
    should_open_post_auth_onboarding, unauthenticated_startup_target, RootView,
    UnauthenticatedStartupTarget, HAS_COMPLETED_ONBOARDING_KEY,
};
use crate::auth::auth_manager::AuthManager;
use crate::auth::AuthStateProvider;
use crate::features::FeatureFlag;
use crate::server::server_api::ServerApiProvider;

fn initialize_app(app: &mut App) {
    app.update(crate::settings::init_and_register_user_preferences);
    app.add_singleton_model(|_ctx| ServerApiProvider::new_for_test());
    app.add_singleton_model(|_| AuthStateProvider::new_for_test());
    app.add_singleton_model(AuthManager::new_for_test);
}

#[test]
fn test_unauthenticated_startup_defaults_to_terminal() {
    let _force_login = FeatureFlag::ForceLogin.override_enabled(false);
    let _open_warp_new_settings_modes =
        FeatureFlag::OpenWarpNewSettingsModes.override_enabled(false);
    let _agent_onboarding = FeatureFlag::AgentOnboarding.override_enabled(false);

    assert_eq!(
        unauthenticated_startup_target(false),
        UnauthenticatedStartupTarget::Terminal
    );
}

#[test]
fn test_unauthenticated_startup_force_login_still_shows_auth() {
    let _force_login = FeatureFlag::ForceLogin.override_enabled(true);
    let _open_warp_new_settings_modes =
        FeatureFlag::OpenWarpNewSettingsModes.override_enabled(false);
    let _agent_onboarding = FeatureFlag::AgentOnboarding.override_enabled(false);

    assert_eq!(
        unauthenticated_startup_target(false),
        UnauthenticatedStartupTarget::Auth
    );
}

#[test]
fn test_unauthenticated_startup_suppresses_pre_login_onboarding() {
    let _force_login = FeatureFlag::ForceLogin.override_enabled(false);
    let _open_warp_new_settings_modes =
        FeatureFlag::OpenWarpNewSettingsModes.override_enabled(true);
    let _agent_onboarding = FeatureFlag::AgentOnboarding.override_enabled(true);

    assert_eq!(
        unauthenticated_startup_target(false),
        UnauthenticatedStartupTarget::Terminal
    );
}

#[test]
fn test_post_auth_onboarding_is_completed_without_opening_slides() {
    let _agent_onboarding = FeatureFlag::AgentOnboarding.override_enabled(true);

    assert!(should_complete_post_auth_onboarding_without_showing(
        false, /* is_onboarded */
        false, /* is_anonymous */
    ));
    assert!(!should_open_post_auth_onboarding(
        false, /* is_onboarded */
        false, /* is_anonymous */
        false, /* has_completed_local_onboarding */
    ));
}

fn set_local_onboarding_completed(app: &mut App, completed: bool) {
    app.update(|ctx| {
        ctx.private_user_preferences()
            .write_value(
                HAS_COMPLETED_ONBOARDING_KEY,
                serde_json::to_string(&completed).unwrap(),
            )
            .unwrap();
    });
}

/// Regression test for the bug fixed by introducing
/// `RootView::sync_local_onboarding_to_server`: when a user completed onboarding
/// pre-login and later authenticated via a non-login-slide entrypoint (i.e. while
/// already in `Terminal` state), the server-side `is_onboarded` flag was never
/// flipped. The helper runs unconditionally on `AuthComplete` and must flip the
/// flag when all preconditions hold.
#[test]
fn test_sync_flips_server_is_onboarded_when_local_onboarding_completed() {
    App::test((), |mut app| async move {
        initialize_app(&mut app);

        // Seed the "has_completed_local_onboarding" preference and make the user
        // appear not yet onboarded on the server. The default test user is
        // non-anonymous, so the guards in the helper won't short-circuit.
        set_local_onboarding_completed(&mut app, true);
        app.update(|ctx| {
            AuthStateProvider::as_ref(ctx).get().set_is_onboarded(false);
            assert!(has_completed_local_onboarding(ctx));
            assert_eq!(
                AuthStateProvider::as_ref(ctx).get().is_onboarded(),
                Some(false)
            );
        });

        app.update(|ctx| {
            let auth_state = AuthStateProvider::as_ref(ctx).get().clone();
            RootView::sync_local_onboarding_to_server(&auth_state, ctx);
        });

        app.read(|ctx| {
            assert_eq!(
                AuthStateProvider::as_ref(ctx).get().is_onboarded(),
                Some(true),
                "sync should have invoked AuthManager::set_user_onboarded"
            );
        });
    });
}

/// If the user hasn't completed local onboarding, the helper must leave the
/// server-side flag untouched — onboarding hasn't actually happened yet.
#[test]
fn test_sync_noop_when_local_onboarding_not_completed() {
    App::test((), |mut app| async move {
        initialize_app(&mut app);

        // Do not set HAS_COMPLETED_ONBOARDING_KEY; it defaults to false.
        app.update(|ctx| {
            AuthStateProvider::as_ref(ctx).get().set_is_onboarded(false);
        });

        app.update(|ctx| {
            let auth_state = AuthStateProvider::as_ref(ctx).get().clone();
            RootView::sync_local_onboarding_to_server(&auth_state, ctx);
        });

        app.read(|ctx| {
            assert_eq!(
                AuthStateProvider::as_ref(ctx).get().is_onboarded(),
                Some(false),
                "sync should not have changed is_onboarded when local onboarding is incomplete"
            );
        });
    });
}

/// The server-side flag should also be left untouched when it is already set,
/// even if local onboarding is complete — avoids redundant server calls.
#[test]
fn test_sync_noop_when_already_onboarded_on_server() {
    App::test((), |mut app| async move {
        initialize_app(&mut app);

        set_local_onboarding_completed(&mut app, true);
        app.update(|ctx| {
            // User::test() defaults to is_onboarded = true; assert that and
            // leave it in place.
            assert_eq!(
                AuthStateProvider::as_ref(ctx).get().is_onboarded(),
                Some(true)
            );
        });

        app.update(|ctx| {
            let auth_state = AuthStateProvider::as_ref(ctx).get().clone();
            RootView::sync_local_onboarding_to_server(&auth_state, ctx);
        });

        app.read(|ctx| {
            assert_eq!(
                AuthStateProvider::as_ref(ctx).get().is_onboarded(),
                Some(true)
            );
        });
    });
}
