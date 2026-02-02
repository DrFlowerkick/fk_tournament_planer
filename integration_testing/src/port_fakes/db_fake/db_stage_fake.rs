//! Fakes for DbpStage port

use super::FakeDatabasePort;
use app_core::{
    DbError, DbResult, DbpStage, Stage,
    utils::{id_version::IdVersion, traits::ObjectIdVersion},
};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
impl DbpStage for FakeDatabasePort {
    async fn get_stage_by_id(&self, stage_id: Uuid) -> DbResult<Option<Stage>> {
        let mut guard = self.fail_next_get_stage.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other("injected get failure".into()));
        }
        Ok(self.stages.lock().unwrap().get(&stage_id).cloned())
    }

    async fn get_stage_by_number(&self, t_id: Uuid, num: u32) -> DbResult<Option<Stage>> {
        let mut guard = self.fail_next_get_stage.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other("injected get failure".into()));
        }

        let stages = self.stages.lock().unwrap();
        let found = stages
            .values()
            .find(|s| s.get_tournament_id() == t_id && s.get_number() == num)
            .cloned();

        Ok(found)
    }

    async fn save_stage(&self, stage: &Stage) -> DbResult<Stage> {
        let mut guard = self.fail_next_save_stage.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other("injected save failure".into()));
        }

        let mut guard = self.stages.lock().unwrap();
        let mut new = stage.clone();

        match stage.get_id_version() {
            IdVersion::Existing(inner) => {
                if let Some(existing) = guard.get(&inner.get_id()) {
                    // Check Optimistic Locking
                    let existing_v = existing.get_version().unwrap_or(0);
                    let update_v = inner.get_version();

                    if existing_v != update_v {
                        return Err(DbError::OptimisticLockConflict);
                    }

                    // Increment version
                    new.set_id_version(IdVersion::new(inner.get_id(), Some(existing_v + 1)));
                } else {
                    return Err(DbError::NotFound);
                }
            }
            IdVersion::NewWithId(id) => {
                if guard.contains_key(&id) {
                    // Simulation: Inserting with specific ID that already exists usually fails
                    // unless we treat it as an upset/seed. For simplicity in tests:
                    return Err(DbError::Other(format!(
                        "Stage with ID {} already exists",
                        id
                    )));
                }
                new.set_id_version(IdVersion::new(id, Some(0)));
            }
        }

        guard.insert(new.get_id(), new.clone());
        Ok(new)
    }

    async fn list_stages_of_tournament(&self, t_id: Uuid) -> DbResult<Vec<Stage>> {
        let mut guard = self.fail_next_list_stage.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other("injected list failure".into()));
        }

        let mut rows: Vec<_> = self
            .stages
            .lock()
            .unwrap()
            .values()
            .filter(|s| s.get_tournament_id() == t_id)
            .cloned()
            .collect();

        // Simulate DB order by number ASC
        rows.sort_by_key(|s| s.get_number());

        Ok(rows)
    }
}
