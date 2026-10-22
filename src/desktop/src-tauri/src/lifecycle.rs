use std::sync::atomic::{AtomicU8, Ordering};
#[derive(Debug, PartialEq, Eq)]
pub enum CloseAction {
    Hide,
    Allow,
}
#[derive(Debug, PartialEq, Eq)]
pub enum ExitAction {
    BeginShutdown,
    Wait,
    Exit,
}
#[derive(Default)]
pub struct Lifecycle {
    phase: AtomicU8,
}
impl Lifecycle {
    pub fn close_action(&self) -> CloseAction {
        if self.phase.load(Ordering::Acquire) == 0 {
            CloseAction::Hide
        } else {
            CloseAction::Allow
        }
    }
    pub fn exit_action(&self) -> ExitAction {
        match self
            .phase
            .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => ExitAction::BeginShutdown,
            Err(1) => ExitAction::Wait,
            Err(_) => ExitAction::Exit,
        }
    }
    pub fn complete_shutdown(&self) {
        self.phase.store(2, Ordering::Release);
    }
}
