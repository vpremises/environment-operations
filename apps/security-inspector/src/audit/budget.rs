//! Share a monotonic execution budget across all configured collector invocations.
use super::config::Tool;
use std::time::{Duration, Instant};

pub(super) struct Budget(Instant);
impl Budget {
    pub(super) fn new(seconds: u64) -> Result<Self, &'static str> {
        if !(1..=3600).contains(&seconds) {
            return Err("collector-budget-invalid");
        }
        Ok(Self(Instant::now() + Duration::from_secs(seconds)))
    }
    pub(super) fn tool(&self, tool: &Tool) -> Result<Tool, &'static str> {
        if !(1..=600).contains(&tool.timeout_seconds) {
            return Err("invalid-timeout");
        }
        let remaining = self.0.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("collector-budget-exhausted");
        }
        let mut tool = tool.clone();
        tool.timeout_seconds = tool.timeout_seconds.min(remaining.as_secs().max(1));
        Ok(tool)
    }
}
