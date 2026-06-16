use super::{
    EnableSshWarpification, SshExtensionInstallMode, SshExtensionInstallModeSetting,
    WarpifySettings,
};
use crate::test_util::settings::initialize_settings_for_tests;
use settings::Setting;
use warpui::{App, SingletonEntity};

#[test]
fn ssh_warpification_is_disabled_by_default() {
    assert!(!EnableSshWarpification::default_value());
}

#[test]
fn ssh_extension_is_always_installed_by_default() {
    assert_eq!(
        SshExtensionInstallMode::default(),
        SshExtensionInstallMode::AlwaysInstall
    );
    assert_eq!(
        SshExtensionInstallModeSetting::default_value(),
        SshExtensionInstallMode::AlwaysInstall
    );
}

#[test]
fn ssh_extension_install_mode_can_be_explicitly_disabled() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);

        WarpifySettings::handle(&app).update(&mut app, |settings, ctx| {
            settings
                .ssh_extension_install_mode
                .set_value(SshExtensionInstallMode::NeverInstall, ctx)
                .expect("setting should update");
        });

        WarpifySettings::handle(&app).read(&app, |settings, _ctx| {
            assert_eq!(
                *settings.ssh_extension_install_mode.value(),
                SshExtensionInstallMode::NeverInstall
            );
        });
    });
}

#[cfg(windows)]
#[test]
fn test_wsl_subshell_detection_success() {
    [
        "wsl",
        "wsl.exe",
        "wsl -d Ubuntu",
        "wsl --distribution Ubuntu",
        "wsl -u user",
        "wsl --cd /home/user",
        "wsl --system",
        "wsl --shell-type login",
        "wsl -d Ubuntu --cd /home/user -u username",
        "wsl.exe -d Ubuntu --cd /home/user -u username",
    ]
    .iter()
    .for_each(|cmd| {
        assert!(
            WarpifySettings::is_built_in_subshell_match(cmd),
            "{} failed to match",
            *cmd
        )
    });
}

#[cfg(windows)]
#[test]
fn test_wsl_subshell_detection_fail() {
    [
        "wsl --install",
        "wsl --status",
        "wsl --list",
        "wsl --export Ubuntu file.tar",
        "wsl --uninstall",
        "wsl --shutdown",
        "wslfetch",
        "nowsl",
        "wsl --help",
        "wsl --version",
        "wsl --terminate Ubuntu",
        "wsl --unregister Ubuntu",
        "wsl --update",
        "wsl --import-in-place Ubuntu",
        "wsl --default-user root",
        "wsl --mount \\device",
    ]
    .iter()
    .for_each(|cmd| {
        assert!(
            !WarpifySettings::is_built_in_subshell_match(cmd),
            "{} accidentally matched",
            *cmd
        )
    });
}
