//! Input components for the app

use displaydoc::Display;
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
    #[prop(into)] is_new: Signal<bool>,
    // Optional on blur callback, e.g. for normalization
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
                    if is_new.get() { placeholder_text.clone() } else { String::new() }
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
    #[prop(into)] is_new: Signal<bool>,
    // Optional on blur callback, e.g. for normalization
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
                    if is_new.get() { placeholder_text.clone() } else { String::new() }
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
    // Optional callback
    #[prop(into, optional)] on_blur: Option<Callback<()>>,
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
                data-testid=format!("select-{}", name)
                aria-invalid=move || error_message.get().is_some().to_string()
                on:change=move |ev| value.set(event_target_value(&ev))
                on:blur=move |_| {
                    if let Some(cb) = on_blur {
                        cb.run(());
                    }
                }
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

pub trait SelectableOption: Sized + Clone + PartialEq + Send + Sync + 'static {
    /// Returns the unique string representation for the <option value="...">
    fn value(&self) -> String;

    /// Returns the display text for the UI
    fn label(&self) -> String;

    /// Returns all available options for the dropdown.
    /// For variants with data fields, return a default instance.
    fn options() -> Vec<Self>;
}

#[component]
pub fn EnumSelect<E: SelectableOption>(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<E>,
    // Optional on change callback, e.g. for update of json config
    #[prop(into, optional)] on_change: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="form-control w-full">
            <label class="label">
                <span class="label-text">{label.clone()}</span>
            </label>
            <select
                class="select select-bordered w-full"
                name=name.clone()
                // Auto-generate test-id
                data-testid=format!("select-{}", name)
                on:change=move |ev| {
                    let val_str = event_target_value(&ev);
                    if let Some(selected_variant) = E::options()
                        .into_iter()
                        .find(|o| o.value() == val_str)
                    {
                        value.set(selected_variant);
                        if let Some(cb) = on_change {
                            cb.run(());
                        }
                    }
                }
                prop:value=move || value.get().value()
            >
                {E::options()
                    .into_iter()
                    .map(|opt| {
                        let val = opt.value();
                        let text = opt.label();
                        // We need to check equality for the "selected" attribute.
                        // Since E implements PartialEq, we can compare the variant structure directly
                        // OR compare the value strings if variants with different data are considered "same selection"
                        view! {
                            <option value=val.clone() selected=move || value.get().value() == val>
                                {text}
                            </option>
                        }
                    })
                    .collect_view()}
            </select>
        </div>
    }
}

#[component]
pub fn ValidatedNumberInput<T>(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<T>,
    #[prop(into)] error_message: Signal<Option<String>>,
    #[prop(into)] is_new: Signal<bool>,
    #[prop(into, optional)] step: String, // "1" for int, "0.1" for float
    #[prop(into, optional)] min: String,
    // Optional on blur callback, e.g. for update of json config
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
                    if is_new.get() { placeholder_text.clone() } else { String::new() }
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
    #[prop(into)] is_new: Signal<bool>,
    #[prop(into, optional)] step: String,
    #[prop(into, optional)] min: String,
    // Optional on blur callback, e.g. for update of json config
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
                    if is_new.get() { placeholder_text.clone() } else { String::new() }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum DurationInputUnit {
    /// seconds
    Seconds,
    /// minutes
    Minutes,
    /// hours
    Hours,
}

#[component]
pub fn ValidatedDurationInput(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<Duration>,
    #[prop(into)] unit: DurationInputUnit,
    #[prop(into)] error_message: Signal<Option<String>>,
    #[prop(into)] is_new: Signal<bool>,
    // Optional on blur callback, e.g. for update of json config
    #[prop(into, optional)] on_blur: Option<Callback<()>>,
) -> impl IntoView {
    let placeholder_text = format!("Enter {} ({})...", label.to_lowercase(), unit);

    view! {
        <div class="form-control w-full">
            <label class="label">
                <span class="label-text">{format!("{} ({})", label, unit)}</span>
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
                // Convert Duration to unit for display
                prop:value=move || {
                    match unit {
                        DurationInputUnit::Seconds => value.get().as_secs().to_string(),
                        DurationInputUnit::Minutes => (value.get().as_secs() / 60).to_string(),
                        DurationInputUnit::Hours => (value.get().as_secs() / 3600).to_string(),
                    }
                }
                placeholder=move || {
                    if is_new.get() { placeholder_text.clone() } else { String::new() }
                }
                on:input=move |ev| {
                    let val_str = event_target_value(&ev);
                    if let Ok(input) = val_str.parse::<u64>() {
                        match unit {
                            DurationInputUnit::Seconds => value.set(Duration::from_secs(input)),
                            DurationInputUnit::Minutes => value.set(Duration::from_secs(input * 60)),
                            DurationInputUnit::Hours => value.set(Duration::from_secs(input * 3600)),
                        }
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
