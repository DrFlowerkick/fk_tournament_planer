//! Input components for the app

use leptos::prelude::*;
use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;

#[component]
pub fn ValidatedTextInput(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<String>,
    #[prop(into)] error_message: Signal<Option<String>>,
    // Context signals to handle placeholder logic internally
    #[prop(into)] is_loading: Signal<bool>,
    #[prop(into)] is_new: Signal<bool>,
    // Optional normalization callback
    #[prop(into, optional)] on_blur: Option<Callback<()>>,
) -> impl IntoView {
    // Auto-generate placeholder text based on label
    let placeholder_text = format!("Enter {}...", label.to_lowercase());

    view! {
        <div class="form-control w-full">
            <label class="label">
                <span class="label-text">{label.clone()}</span>
            </label>
            <input
                type="text"
                class="input input-bordered w-full"
                class:input-error=move || error_message.get().is_some()
                name=name.clone()
                // Auto-generate test-id
                data-testid=format!("input-{}", name)
                aria-invalid=move || error_message.get().is_some().to_string()
                prop:value=value
                placeholder=move || {
                    if is_loading.get() {
                        "Loading...".to_string()
                    } else if is_new.get() {
                        placeholder_text.clone()
                    } else {
                        String::new()
                    }
                }
                on:input=move |ev| value.set(event_target_value(&ev))
                on:blur=move |_| {
                    if let Some(cb) = on_blur {
                        cb.run(());
                    }
                }
            />
            {move || {
                error_message
                    .get()
                    .map(|msg| {
                        view! {
                            <label class="label">
                                <span class="label-text-alt text-error">{msg}</span>
                            </label>
                        }
                    })
            }}
        </div>
    }
}

#[component]
pub fn TextInput(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<String>,
    #[prop(into)] optional: bool,
    // Context signals to handle placeholder logic internally
    #[prop(into)] is_loading: Signal<bool>,
    #[prop(into)] is_new: Signal<bool>,
    // Optional normalization callback
    #[prop(into, optional)] on_blur: Option<Callback<()>>,
) -> impl IntoView {
    // Auto-generate placeholder text based on label
    let (label, placeholder_text) = if optional {
        (
            format!("{} (optional)", label.clone()),
            format!("Enter {} (optional)...", label.to_lowercase()),
        )
    } else {
        (label.clone(), format!("Enter {}...", label.to_lowercase()))
    };

    view! {
        <div class="form-control w-full">
            <label class="label">
                <span class="label-text">{label.clone()}</span>
            </label>
            <input
                type="text"
                class="input input-bordered w-full"
                name=name.clone()
                // Auto-generate test-id
                data-testid=format!("input-{}", name)
                prop:value=value
                placeholder=move || {
                    if is_loading.get() {
                        "Loading...".to_string()
                    } else if is_new.get() {
                        placeholder_text.clone()
                    } else {
                        String::new()
                    }
                }
                on:input=move |ev| value.set(event_target_value(&ev))
                on:blur=move |_| {
                    if let Some(cb) = on_blur {
                        cb.run(());
                    }
                }
            />
        </div>
    }
}

#[component]
pub fn ValidatedSelect(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<String>,
    #[prop(into)] error_message: Signal<Option<String>>,
    /// List of (value, label) tuples for the options
    #[prop(into)]
    options: Vec<(String, String)>,
    /// Optional custom placeholder text (default: "Select [label]...")
    #[prop(into, optional)]
    placeholder: Option<String>,
) -> impl IntoView {
    // Generate default placeholder if none provided
    let placeholder_text =
        placeholder.unwrap_or_else(|| format!("Select {}...", label.to_lowercase()));

    view! {
        <div class="form-control w-full">
            <label class="label">
                <span class="label-text">{label.clone()}</span>
            </label>
            <select
                class="select select-bordered w-full"
                class:select-error=move || error_message.get().is_some()
                name=name.clone()
                // Auto-generate test-id
                data-testid=format!("input-{}", name)
                aria-invalid=move || error_message.get().is_some().to_string()
                on:change=move |ev| value.set(event_target_value(&ev))
                prop:value=value
            >
                <option disabled selected value="">
                    {placeholder_text}
                </option>
                {options
                    .into_iter()
                    .map(|(val, text)| {
                        view! {
                            <option value=val.clone() selected=move || value.get() == val>
                                {text}
                            </option>
                        }
                    })
                    .collect_view()}
            </select>
            {move || {
                error_message
                    .get()
                    .map(|msg| {
                        view! {
                            <label class="label">
                                <span class="label-text-alt text-error">{msg}</span>
                            </label>
                        }
                    })
            }}
        </div>
    }
}

#[component]
pub fn ValidatedNumberInput<T>(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<T>,
    #[prop(into)] error_message: Signal<Option<String>>,
    #[prop(into)] is_loading: Signal<bool>,
    #[prop(into)] is_new: Signal<bool>,
    #[prop(into, optional)] step: String, // "1" for int, "0.1" for float
    #[prop(into, optional)] min: String,
    // Optional normalization callback
    #[prop(into, optional)] on_blur: Option<Callback<()>>,
) -> impl IntoView
where
    T: FromStr + Display + Default + Copy + Send + Sync + 'static,
    <T as FromStr>::Err: std::fmt::Debug,
{
    let placeholder_text = format!("Enter {}...", label.to_lowercase());
    // Default step to "1" if not provided
    let step_val = if step.is_empty() {
        "1".to_string()
    } else {
        step
    };
    // Default min to "0" if not provided
    let min = if min.is_empty() { "0".to_string() } else { min };

    view! {
        <div class="form-control w-full">
            <label class="label">
                <span class="label-text">{label.clone()}</span>
            </label>
            <input
                type="number"
                step=step_val
                min=min
                class="input input-bordered w-full"
                class:input-error=move || error_message.get().is_some()
                name=name.clone()
                data-testid=format!("input-{}", name)
                aria-invalid=move || error_message.get().is_some().to_string()
                // We bind the value via prop:value which expects a string/number
                prop:value=move || value.get().to_string()
                placeholder=move || {
                    if is_loading.get() {
                        "Loading...".to_string()
                    } else if is_new.get() {
                        placeholder_text.clone()
                    } else {
                        String::new()
                    }
                }
                on:input=move |ev| {
                    let val_str = event_target_value(&ev);
                    if let Ok(val) = val_str.parse::<T>() {
                        value.set(val);
                    }
                }
                on:blur=move |_| {
                    if let Some(cb) = on_blur {
                        cb.run(());
                    }
                }
            />
            {move || {
                error_message
                    .get()
                    .map(|msg| {
                        view! {
                            <label class="label">
                                <span class="label-text-alt text-error">{msg}</span>
                            </label>
                        }
                    })
            }}
        </div>
    }
}

#[component]
pub fn ValidatedOptionNumberInput<T>(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<Option<T>>,
    #[prop(into)] error_message: Signal<Option<String>>,
    #[prop(into)] is_loading: Signal<bool>,
    #[prop(into)] is_new: Signal<bool>,
    #[prop(into, optional)] step: String,
    #[prop(into, optional)] min: String,
    // Optional normalization callback
    #[prop(into, optional)] on_blur: Option<Callback<()>>,
) -> impl IntoView
where
    T: FromStr + Display + Copy + Send + Sync + 'static,
    <T as FromStr>::Err: std::fmt::Debug,
{
    let placeholder_text = format!("Enter {}...", label.to_lowercase());
    // Default step to "1" if not provided
    let step_val = if step.is_empty() {
        "1".to_string()
    } else {
        step
    };
    // Default min to "0" if not provided
    let min = if min.is_empty() { "0".to_string() } else { min };

    view! {
        <div class="form-control w-full">
            <label class="label">
                <span class="label-text">{format!("{}", label)}</span>
            </label>
            <input
                type="number"
                step=step_val
                min=min
                class="input input-bordered w-full"
                class:input-error=move || error_message.get().is_some()
                name=name.clone()
                data-testid=format!("input-{}", name)
                aria-invalid=move || error_message.get().is_some().to_string()
                prop:value=move || {
                    match value.get() {
                        Some(v) => v.to_string(),
                        None => String::new(),
                    }
                }
                placeholder=move || {
                    if is_loading.get() {
                        "Loading...".to_string()
                    } else if is_new.get() {
                        placeholder_text.clone()
                    } else {
                        String::new()
                    }
                }
                on:input=move |ev| {
                    let val_str = event_target_value(&ev);
                    if val_str.is_empty() {
                        value.set(None);
                    } else if let Ok(val) = val_str.parse::<T>() {
                        value.set(Some(val));
                    }
                }
                on:blur=move |_| {
                    if let Some(cb) = on_blur {
                        cb.run(());
                    }
                }
            />
            {move || {
                error_message
                    .get()
                    .map(|msg| {
                        view! {
                            <label class="label">
                                <span class="label-text-alt text-error">{msg}</span>
                            </label>
                        }
                    })
            }}
        </div>
    }
}

#[component]
pub fn ValidatedDurationMinutesInput(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<Duration>,
    #[prop(into)] error_message: Signal<Option<String>>,
    #[prop(into)] is_loading: Signal<bool>,
    #[prop(into)] is_new: Signal<bool>,
    // Optional normalization callback
    #[prop(into, optional)] on_blur: Option<Callback<()>>,
) -> impl IntoView {
    let placeholder_text = format!("Enter {} (minutes)...", label.to_lowercase());

    view! {
        <div class="form-control w-full">
            <label class="label">
                <span class="label-text">{format!("{} (minutes)", label)}</span>
            </label>
            <input
                type="number"
                min="0"
                step="1"
                class="input input-bordered w-full"
                class:input-error=move || error_message.get().is_some()
                name=name.clone()
                data-testid=format!("input-{}", name)
                aria-invalid=move || error_message.get().is_some().to_string()
                // Convert Duration to Minutes for display
                prop:value=move || (value.get().as_secs() / 60).to_string()
                placeholder=move || {
                    if is_loading.get() {
                        "Loading...".to_string()
                    } else if is_new.get() {
                        placeholder_text.clone()
                    } else {
                        String::new()
                    }
                }
                on:input=move |ev| {
                    let val_str = event_target_value(&ev);
                    if let Ok(minutes) = val_str.parse::<u64>() {
                        value.set(Duration::from_secs(minutes * 60));
                    }
                }
                on:blur=move |_| {
                    if let Some(cb) = on_blur {
                        cb.run(());
                    }
                }
            />
            {move || {
                error_message
                    .get()
                    .map(|msg| {
                        view! {
                            <label class="label">
                                <span class="label-text-alt text-error">{msg}</span>
                            </label>
                        }
                    })
            }}
        </div>
    }
}
