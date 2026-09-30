use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::activity_core::activity_arbiter::SharedActivityArbiter;
use crate::activity_core::activity_publisher::ActivityPublisher;
use crate::activity_core::activity_source::ActivitySource;

/// Commands arrive on Tauri's threads, so the registry is shared behind a mutex.
pub type SharedActivitySourceRegistry = Mutex<ActivitySourceRegistry>;

/// Every running activity source, by kind, so a user action can reach the source it
/// belongs to.
#[derive(Default)]
pub struct ActivitySourceRegistry {
    activity_sources_by_kind: HashMap<&'static str, Box<dyn ActivitySource>>,
}

impl ActivitySourceRegistry {
    /// Starts the source with its own publisher and keeps it for routing actions.
    pub fn start_and_register(
        &mut self,
        mut activity_source: Box<dyn ActivitySource>,
        shared_activity_arbiter: &SharedActivityArbiter,
    ) -> Result<(), String> {
        let activity_kind = activity_source.activity_kind();
        if self.activity_sources_by_kind.contains_key(activity_kind) {
            return Err(format!("two activity sources use the kind \"{activity_kind}\""));
        }
        activity_source.start_publishing(ActivityPublisher::new(activity_kind, Arc::clone(shared_activity_arbiter)))?;
        self.activity_sources_by_kind.insert(activity_kind, activity_source);
        Ok(())
    }

    pub fn perform_activity_action(&self, activity_kind: &str, activity_action: &str) -> Result<(), String> {
        self.activity_sources_by_kind
            .get(activity_kind)
            .ok_or_else(|| format!("no activity source has the kind \"{activity_kind}\""))?
            .perform_activity_action(activity_action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity_core::activity_arbiter::ActivityArbiter;

    struct RecordingActivitySource {
        performed_actions: Arc<Mutex<Vec<String>>>,
    }

    impl ActivitySource for RecordingActivitySource {
        fn activity_kind(&self) -> &'static str {
            "recording"
        }
        fn start_publishing(&mut self, _activity_publisher: ActivityPublisher) -> Result<(), String> {
            Ok(())
        }
        fn perform_activity_action(&self, activity_action: &str) -> Result<(), String> {
            self.performed_actions.lock().unwrap().push(activity_action.to_string());
            Ok(())
        }
    }

    fn registry_with_recording_source() -> (ActivitySourceRegistry, Arc<Mutex<Vec<String>>>) {
        let performed_actions = Arc::new(Mutex::new(Vec::new()));
        let shared_activity_arbiter = Arc::new(Mutex::new(ActivityArbiter::new(Box::new(|_| {}))));
        let mut activity_source_registry = ActivitySourceRegistry::default();
        activity_source_registry
            .start_and_register(
                Box::new(RecordingActivitySource { performed_actions: Arc::clone(&performed_actions) }),
                &shared_activity_arbiter,
            )
            .unwrap();
        (activity_source_registry, performed_actions)
    }

    #[test]
    fn routes_an_action_to_the_source_of_that_kind() {
        let (activity_source_registry, performed_actions) = registry_with_recording_source();
        activity_source_registry.perform_activity_action("recording", "do-something").unwrap();
        assert_eq!(*performed_actions.lock().unwrap(), vec!["do-something".to_string()]);
    }

    #[test]
    fn rejects_actions_for_unknown_kinds_and_duplicate_kinds() {
        let (mut activity_source_registry, performed_actions) = registry_with_recording_source();
        assert!(activity_source_registry.perform_activity_action("unknown", "do-something").is_err());
        let shared_activity_arbiter = Arc::new(Mutex::new(ActivityArbiter::new(Box::new(|_| {}))));
        let duplicate_source = Box::new(RecordingActivitySource { performed_actions });
        assert!(activity_source_registry.start_and_register(duplicate_source, &shared_activity_arbiter).is_err());
    }
}
