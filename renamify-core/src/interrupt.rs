use std::sync::atomic::{AtomicBool, Ordering};

/// Tracks whether we are currently prompting the user for confirmation.
static CONFIRMATION_PROMPT_ACTIVE: AtomicBool = AtomicBool::new(false);

/// RAII helper that marks the confirmation prompt as active while it is in scope.
pub struct ConfirmationPromptGuard;

impl ConfirmationPromptGuard {
    /// Activate the confirmation prompt state until the guard is dropped.
    pub fn activate() -> Self {
        CONFIRMATION_PROMPT_ACTIVE.store(true, Ordering::SeqCst);
        Self
    }
}

impl Drop for ConfirmationPromptGuard {
    fn drop(&mut self) {
        CONFIRMATION_PROMPT_ACTIVE.store(false, Ordering::SeqCst);
    }
}

/// Returns true when the confirmation prompt is currently waiting for input.
pub fn confirmation_prompt_active() -> bool {
    CONFIRMATION_PROMPT_ACTIVE.load(Ordering::SeqCst)
}
