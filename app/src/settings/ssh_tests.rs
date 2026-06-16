use settings::Setting;
use warpui::{App, SingletonEntity};

use super::{EnableSshWrapper, SshSettings};
use crate::test_util::settings::initialize_settings_for_tests;

#[test]
fn legacy_ssh_wrapper_is_enabled_by_default() {
    assert!(EnableSshWrapper::default_value());
}

#[test]
fn legacy_ssh_wrapper_can_be_explicitly_disabled() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);

        SshSettings::handle(&app).update(&mut app, |settings, ctx| {
            settings
                .enable_ssh_wrapper
                .set_value(false, ctx)
                .expect("setting should update");
        });

        SshSettings::handle(&app).read(&app, |settings, _ctx| {
            assert!(!*settings.enable_ssh_wrapper.value());
        });
    });
}
