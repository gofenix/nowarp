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

/// When true, the passive prompt suggestion custom endpoint bypasses the
/// Warp login check and the active-AI master toggle, so the custom endpoint
/// works for users who have not signed in to a Warp account. The 4 fields
/// (`custom_endpoint_enabled`, `base_url`, `model`, `api_key`) still have to
/// be configured.
pub(crate) fn bypass_auth_for_custom_endpoint() -> bool {
    true
}
