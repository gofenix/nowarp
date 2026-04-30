/// Returns true for this terminal-focused build, where automatic first-run
/// onboarding and product-promo surfaces should stay hidden.
pub(crate) fn suppress_automatic_onboarding() -> bool {
    true
}

pub(crate) fn suppress_automatic_launch_modals() -> bool {
    suppress_automatic_onboarding()
}

pub(crate) fn suppress_automatic_new_feature_prompts() -> bool {
    suppress_automatic_onboarding()
}
