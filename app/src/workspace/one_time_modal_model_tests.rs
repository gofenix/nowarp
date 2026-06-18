use futures::FutureExt;
use settings::Setting as _;
use warp_core::features::FeatureFlag;
use warp_util::sync::Condition;
use warpui::{App, SingletonEntity};

use super::OneTimeModalModel;
use crate::settings::AISettings;
use crate::terminal::general_settings::GeneralSettings;
use crate::test_util::settings::initialize_settings_for_tests;
use crate::test_util::terminal::{add_window_with_terminal, initialize_app_for_terminal_view};
use crate::workspace::hoa_onboarding;

fn model_for_test() -> OneTimeModalModel {
    let auto_handoff_sleep_modal_closed = Condition::new();
    auto_handoff_sleep_modal_closed.set();

    OneTimeModalModel {
        is_build_plan_migration_modal_open: false,
        is_oz_launch_modal_open: false,
        is_openwarp_launch_modal_open: false,
        is_orchestration_launch_modal_open: false,
        is_auto_handoff_sleep_modal_open: false,
        auto_handoff_sleep_modal_closed,
        is_hoa_onboarding_open: false,
        target_window_id: None,
    }
}

#[test]
fn wait_until_auto_handoff_sleep_modal_closed_tracks_modal_state() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);

        terminal.update(&mut app, |_, ctx| {
            OneTimeModalModel::handle(ctx).update(ctx, |model, ctx| {
                // Resolves immediately while the modal is closed.
                assert!(model
                    .wait_until_auto_handoff_sleep_modal_closed()
                    .now_or_never()
                    .is_some());

                // The auto-resume path creates its wait future before the
                // modal opens (e.g. while offline during sleep); it must
                // still observe the modal that opens later.
                let pending_probe = model.wait_until_auto_handoff_sleep_modal_closed();
                let resolving_waiter = model.wait_until_auto_handoff_sleep_modal_closed();

                model.set_auto_handoff_sleep_modal_open(true, ctx);

                // Pending while the modal is open, because the future reads
                // live modal state at poll time.
                assert!(pending_probe.now_or_never().is_none());

                model.mark_auto_handoff_sleep_modal_dismissed(ctx);

                // An existing waiter resolves once the modal closes.
                assert!(resolving_waiter.now_or_never().is_some());
            });
        });
    });
}

#[test]
fn test_openwarp_launch_modal_is_marked_checked_without_opening() {
    let _openwarp_launch_modal = FeatureFlag::OpenWarpLaunchModal.override_enabled(true);

    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);
        let model = app.add_model(|_| model_for_test());

        let did_open = model.update(&mut app, |model, ctx| {
            model.check_and_trigger_openwarp_launch_modal(ctx)
        });

        app.read(|ctx| {
            assert!(!did_open);
            assert!(!model.as_ref(ctx).is_openwarp_launch_modal_open());
            assert!(*GeneralSettings::as_ref(ctx)
                .did_check_to_trigger_openwarp_launch_modal
                .value());
        });
    });
}

#[test]
fn test_oz_launch_modal_is_marked_checked_without_opening() {
    let _oz_launch_modal = FeatureFlag::OzLaunchModal.override_enabled(true);

    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);
        let model = app.add_model(|_| model_for_test());

        let did_open = model.update(&mut app, |model, ctx| {
            model.check_and_trigger_oz_launch_modal(ctx)
        });

        app.read(|ctx| {
            assert!(!did_open);
            assert!(!model.as_ref(ctx).is_oz_launch_modal_open());
            assert!(*AISettings::as_ref(ctx).did_check_to_trigger_oz_launch_modal);
        });
    });
}

#[test]
fn test_hoa_onboarding_is_marked_completed_without_opening() {
    let _hoa_onboarding = FeatureFlag::HOAOnboardingFlow.override_enabled(true);
    let _vertical_tabs = FeatureFlag::VerticalTabs.override_enabled(true);
    let _hoa_notifications = FeatureFlag::HOANotifications.override_enabled(true);
    let _tab_configs = FeatureFlag::TabConfigs.override_enabled(true);

    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);
        let model = app.add_model(|_| model_for_test());

        let did_open = model.update(&mut app, |model, ctx| {
            model.check_and_trigger_hoa_onboarding(ctx)
        });

        app.read(|ctx| {
            assert!(!did_open);
            assert!(!model.as_ref(ctx).is_hoa_onboarding_open());
            assert!(hoa_onboarding::has_completed_hoa_onboarding(ctx));
        });
    });
}
