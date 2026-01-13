//! Fakes for DbpTournamentBase port

use super::FakeDatabasePort;
use app_core::{
    DbError, DbResult, DbpTournamentBase, TournamentBase, utils::id_version::IdVersion,
};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
impl DbpTournamentBase for FakeDatabasePort {
    async fn get_tournament_base(&self, id: Uuid) -> DbResult<Option<TournamentBase>> {
        let mut guard = self.fail_next_get_tb.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other("injected get failure".into()));
        }
        Ok(self.tournament_bases.lock().unwrap().get(&id).cloned())
    }
    async fn save_tournament_base(&self, tournament: &TournamentBase) -> DbResult<TournamentBase> {
        let mut guard = self.fail_next_save_tb.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other("injected save failure".into()));
        }

        let mut guard = self.tournament_bases.lock().unwrap();
        let mut new = tournament.clone();
        if let Some(id) = tournament.get_id() {
            if let Some(existing) = guard.get(&id) {
                let version = existing.get_version().unwrap() + 1;
                new.set_id_version(IdVersion::new(id, Some(version)));
            } else {
                // This case can happen if an ID is provided but not found (e.g., update on non-existent row)
                // For simplicity, we treat it as an insert, but a real DB might error.
                new.set_id_version(IdVersion::new(id, Some(0)));
            }
        } else {
            new.set_id_version(IdVersion::new(Uuid::new_v4(), Some(0)));
        }

        guard.insert(new.get_id().unwrap(), new.clone());
        Ok(new)
    }
    async fn list_tournament_bases(
        &self,
        sport_id: Uuid,
        name_filter: Option<&str>,
        limit: Option<usize>,
    ) -> DbResult<Vec<TournamentBase>> {
        let mut guard = self.fail_next_list_tb.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other("injected list failure".into()));
        }

        let filter = name_filter.map(|s| s.to_lowercase());
        let mut rows: Vec<_> = self
            .tournament_bases
            .lock()
            .unwrap()
            .values()
            .filter(|sc| sc.get_sport_id() == sport_id)
            .cloned()
            .collect();

        if let Some(f) = filter {
            rows.retain(|tb| tb.get_name().to_lowercase().contains(&f));
        }

        if let Some(lim) = limit {
            rows.truncate(lim);
        }

        Ok(rows)
    }
}
