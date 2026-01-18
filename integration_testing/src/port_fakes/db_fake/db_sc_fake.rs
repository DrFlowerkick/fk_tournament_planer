//! Fake implementation of DbpSportConfig for testing

use super::FakeDatabasePort;
use app_core::{DbError, DbResult, DbpSportConfig, SportConfig, utils::id_version::IdVersion};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
impl DbpSportConfig for FakeDatabasePort {
    async fn get_sport_config(&self, id: Uuid) -> DbResult<Option<SportConfig>> {
        let mut guard = self.fail_next_get_sc.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other("injected get failure".into()));
        }
        Ok(self.sport_configs.lock().unwrap().get(&id).cloned())
    }

    async fn save_sport_config(&self, config: &SportConfig) -> DbResult<SportConfig> {
        let mut guard = self.fail_next_save_sc.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other("injected save failure".into()));
        }

        let mut guard = self.sport_configs.lock().unwrap();
        let mut new = config.clone();
        if let Some(id) = config.get_id() {
            if let Some(existing) = guard.get(&id) {
                let version = existing.get_version().unwrap() + 1;
                new.set_id_version(IdVersion::new(id, Some(version)));
            } else {
                new.set_id_version(IdVersion::new(id, Some(0)));
            }
        } else {
            new.set_id_version(IdVersion::new(Uuid::new_v4(), Some(0)));
        }
        guard.insert(new.get_id().unwrap(), new.clone());
        Ok(new)
    }

    async fn list_sport_configs(
        &self,
        sport_id: Uuid,
        name_filter: Option<&str>,
        limit: Option<usize>,
    ) -> DbResult<Vec<SportConfig>> {
        let mut guard = self.fail_next_list_sc.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other("injected list failure".into()));
        }

        let filter = name_filter.map(|s| s.to_lowercase());
        let mut rows: Vec<_> = self
            .sport_configs
            .lock()
            .unwrap()
            .values()
            .filter(|sc| sc.get_sport_id() == sport_id)
            .filter(|sc| {
                if let Some(ref f) = filter {
                    sc.get_name().to_lowercase().contains(f)
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        rows.sort_by(|a, b| match a.get_name().cmp(&b.get_name()) {
            std::cmp::Ordering::Equal => a.get_id().cmp(&b.get_id()),
            cmp => cmp,
        });

        if let Some(l) = limit {
            rows.truncate(l);
        }
        Ok(rows)
    }
}
