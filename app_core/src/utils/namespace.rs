//! Namespace utilities

/// Provides a UUID namespace for the FK Tournament Planer project.
use uuid::Uuid;

pub fn project_namespace() -> Uuid {
    let namespace = Uuid::NAMESPACE_OID;
    let project_name = "fk_tournament_planer";
    let project_namespace = Uuid::new_v5(&namespace, project_name.as_bytes());
    Uuid::from_bytes(*project_namespace.as_bytes())
}
