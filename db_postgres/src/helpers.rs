// Some data base helpers

use anyhow::{Context, Result};
use std::env;
use url::Url;

/// escaping wild cards in like query strings
pub fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// get postgres url
pub fn url_without_db() -> Result<Url> {
    let without_db = env::var("POSTGRES_URL").context("POSTGRES_URL must be set. Hint: did you run dotenv()?")?;
    let without_db = Url::parse(&without_db)?;
    Ok(without_db)
}

/// get database url with custom database name
pub fn url_custom_db(database: impl Into<String>) -> Result<Url> {
    let without_db = url_without_db()?;
    let url = without_db.join(&database.into())?;
    Ok(url)

}

/// get database url
pub fn url_db() -> Result<Url> {
    let database = env::var("DATABASE_NAME").context("DATABASE_NAME must be set. Hint: did you run dotenv()?")?;
    url_custom_db(database)
}
