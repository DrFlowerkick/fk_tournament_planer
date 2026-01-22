//! provides fn call for on cancel actions in error banners

use crate::params::SportParams;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_query};

pub fn use_on_cancel() -> Callback<()> {
    let navigate = use_navigate();
    let sport_query = use_query::<SportParams>();

    let on_cancel = Callback::new(move |()| {
        if let Ok(sport_params) = sport_query.get()
            && let Some(sport_id) = sport_params.sport_id
        {
            navigate(&format!("/?sport_id={}", sport_id), Default::default());
        } else {
            navigate("/", Default::default());
        }
    });
    on_cancel
}
