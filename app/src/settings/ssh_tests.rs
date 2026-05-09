use settings::Setting;

use super::EnableSshWrapper;

#[test]
fn legacy_ssh_wrapper_is_disabled_by_default() {
    assert!(!EnableSshWrapper::default_value());
}
