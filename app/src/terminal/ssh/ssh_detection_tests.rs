use super::{evaluate_warpify_ssh_host, SshInteractiveSessionDetected};
use crate::terminal::warpify::settings::WarpifySettings;
use warp_util::path::ShellFamily;
use warpui::{App, SingletonEntity};

#[test]
fn ssh_warpification_is_feature_disabled_by_default() {
    App::test((), |app| async move {
        app.add_singleton_model(WarpifySettings::new_with_defaults);

        app.read(|ctx| {
            let warpify_settings = WarpifySettings::as_ref(ctx);
            let result = evaluate_warpify_ssh_host(
                "ssh workspace",
                Some("workspace"),
                ShellFamily::Posix,
                warpify_settings,
            );

            assert!(matches!(
                result,
                SshInteractiveSessionDetected::FeatureDisabled
            ));
        });
    });
}
