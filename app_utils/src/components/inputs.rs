//! Input components for the app

use crate::{enum_utils::SelectableOption, hooks::is_field_valid::is_object_field_valid};
use app_core::utils::validation::{FieldResult, ValidationResult};
use displaydoc::Display;
use leptos::prelude::*;
use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

// ToDo: deprecated, replace with ValidatedTextInput
#[component]
pub fn ValidatedTextInput(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<String>,
    #[prop(into)] validation_error: Signal<FieldResult<()>>,
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
                <span class="label-text">{label}</span>
            </label>
            <input
                type="text"
                class="input input-bordered w-full"
                class:input-error=move || validation_error.get().is_err()
                name=name.clone()
                // Auto-generate test-id
                data-testid=format!("input-{}", name)
                aria-invalid=move || validation_error.get().is_err().to_string()
                prop:value=value
                placeholder=move || {
                    if is_new.get() { placeholder_text.clone() } else { String::new() }
                }
                on:input:target=move |ev| value.set(ev.target().value())
                on:blur=move |_| {
                    if let Some(cb) = on_blur {
                        cb.run(());
                    }
                }
            />
            <Show when=move || validation_error.get().is_err()>
                <label class="label">
                    <span class="label-text-alt text-error w-full text-left block whitespace-normal">
                        {move || { validation_error.get().err().map(|e| e.to_string()) }}
                    </span>
                </label>
            </Show>
        </div>
    }
}

// ToDo: replace all instances of ValidatedTextInput and TextInput with this more advanced component
#[component]
pub fn TextInputWithValidation<T>(
    /// Label text for the input
    #[prop(into)]
    label: String,
    /// Name attribute for the input (also used for test-id)
    #[prop(into)]
    name: String,
    /// Reactive read-access to Option<T>.
    /// Using Signal<Option<T>> allows passing ReadSignal, Memo, or derived closures.
    #[prop(into)]
    value: Signal<Option<T>>,
    /// Callback to push changes to source of value
    /// Using Callback<T> allows passing closures and Callbacks.
    #[prop(into)]
    set_value: Callback<T>,
    /// Reactive read-access to validation results
    /// Using Signal<ValidationResult<()>> allows passing ReadSignal, Memo, or derived closures.
    validation_result: Signal<ValidationResult<()>>,
    /// Object ID for field error lookup
    #[prop(into)]
    object_id: Signal<Option<Uuid>>,
    /// Field name for field error lookup
    #[prop(into)]
    field: String,
    /// Whether the field is optional (affects label and placeholder)
    #[prop(into, default = false)]
    optional: bool,
) -> impl IntoView
where
    T: FromStr + Display + Clone + Send + Sync + 'static,
{
    // Local buffer: Some(string) while typing, None when synced with Core
    let (draft, set_draft) = signal(None::<String>);

    // type parse error
    let (parse_err, set_parse_err) = signal(None::<String>);

    // Error state from validation of all objects
    let error = Signal::derive(move || {
        if let Some(e) = parse_err.get() {
            return Some(e);
        } else {
            is_object_field_valid(validation_result, object_id, &field)
                .err()
                .map(|e| e.to_string())
        }
    });

    // Derived: What to actually show in the <input>
    let display_value = move || match draft.get() {
        Some(d) => d,
        None => value.get().map(|v| v.to_string()).unwrap_or_default(),
    };

    // Derived: Error visibility logic
    // We hide errors while the user is actively typing (proactive reset)
    let show_error = move || draft.get().is_none() && error.get().is_some();

    // Auto-generate label and placeholder text based on label and optionality
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
                <span class="label-text">{label}</span>
            </label>
            <input
                type="text"
                class="input input-bordered w-full"
                aria-invalid=move || show_error().to_string()
                prop:value=display_value
                name=name.clone()
                // Auto-generate test-id
                data-testid=format!("input-{}", name)
                placeholder=placeholder_text
                // USER TYPING: Update draft to take control from the core
                on:input:target=move |ev| {
                    set_draft.set(Some(ev.target().value()));
                }
                // USER FINISHED: Release control and attempt to commit to core
                on:change:target=move |ev| {
                    let new_val = ev.target().value();
                    match new_val.parse::<T>() {
                        Ok(val) => {
                            set_value.run(val);
                            set_parse_err.set(None);
                            set_draft.set(None);
                        }
                        Err(_) => {
                            set_parse_err.set(Some(format!("Invalid input format, parse failed.")));
                        }
                    }
                }
                // USER LEAVES FIELD: Reset draft to sync with core
                on:blur=move |_| {
                    set_parse_err.set(None);
                    set_draft.set(None);
                }
            />
            // Display error only when not typing and an error exists
            <Show when=show_error>
                <label class="label">
                    <span class="label-text-alt text-error w-full text-left block whitespace-normal">
                        {move || error.get()}
                    </span>
                </label>
            </Show>
        </div>
    }
}

// ToDo: deprecated, replace with ValidatedTextInput
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

// ToDo: deprecated, replace with EnumSelectWithValidation
#[component]
pub fn ValidatedSelect(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<String>,
    #[prop(into)] validation_error: Signal<FieldResult<()>>,
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
                class:select-error=move || validation_error.get().is_err()
                name=name.clone()
                // Auto-generate test-id
                data-testid=format!("select-{}", name)
                aria-invalid=move || validation_error.get().is_err().to_string()
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
            <Show when=move || validation_error.get().is_err()>
                <label class="label">
                    <span class="label-text-alt text-error w-full text-left block whitespace-normal">
                        {move || { validation_error.get().err().map(|e| e.to_string()) }}
                    </span>
                </label>
            </Show>
        </div>
    }
}

// ToDo: replace all instances of ValidatedTextInput and TextInput with this more advanced component
#[component]
pub fn EnumSelectWithValidation<E>(
    /// Label text for the input
    #[prop(into)]
    label: String,
    /// Name attribute for the input (also used for test-id)
    #[prop(into)]
    name: String,
    /// Reactive read-access to Option<T>.
    /// Using Signal<Option<E>> allows passing ReadSignal, Memo, or derived closures.
    #[prop(into)]
    value: Signal<Option<E>>,
    /// Callback to push changes to source of value
    /// Using Callback<E> allows passing closures and Callbacks.
    #[prop(into)]
    on_write: Callback<E>,
    /// Reactive read-access to validation results
    /// Using Signal<ValidationResult<()>> allows passing ReadSignal, Memo, or derived closures.
    validation_result: Signal<ValidationResult<()>>,
    /// Object ID for field error lookup
    #[prop(into)]
    object_id: Signal<Option<Uuid>>,
    /// Field name for field error lookup
    #[prop(into)]
    field: String,
    /// Whether the field is optional (affects label and placeholder)
    #[prop(into, default = false)]
    optional: bool,
) -> impl IntoView
where
    E: SelectableOption,
{
    // Local state: true while user is interacting with the select
    let (is_selecting, set_is_selecting) = signal(false);

    // Error state from validation of all objects
    let error = Signal::derive(move || {
        is_object_field_valid(validation_result, object_id, &field)
            .err()
            .map(|e| e.to_string())
    });

    // Derived: Error visibility logic
    // We hide errors while the user is actively selecting (proactive reset)
    let show_error = move || !is_selecting.get() && error.get().is_some();

    // Auto-generate label and placeholder text based on label and optionality
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
                <span class="label-text">{label}</span>
            </label>
            <select
                class="select select-bordered w-full"
                aria-invalid=move || show_error().to_string()
                prop:value=move || value.get().as_ref().map(|v| v.value()).unwrap_or_default()
                name=name.clone()
                // Auto-generate test-id
                data-testid=format!("select-{}", name)
                // USER STARTED: Mark as selecting to hide errors
                on:focus=move |_| {
                    set_is_selecting.set(true);
                }
                // USER FINISHED: Release control and attempt to commit to core
                on:blur=move |_| {
                    set_is_selecting.set(false);
                }
                // USER FINISHED: Release control and attempt to commit to core
                on:change:target=move |ev| {
                    let new_val = ev.target().value();
                    if let Some(selected_variant) = E::options()
                        .into_iter()
                        .find(|o| o.value() == new_val)
                    {
                        on_write.run(selected_variant);
                    }
                    set_is_selecting.set(false);
                }
            >
                <option disabled selected value="">
                    {placeholder_text}
                </option>
                {E::options()
                    .into_iter()
                    .map(|opt| {
                        let val = opt.value();
                        let text = opt.label();
                        // We need to check equality for the "selected" attribute.
                        // Since E implements PartialEq, we can compare the variant structure directly
                        // OR compare the value strings if variants with different data are considered "same selection"
                        view! {
                            <option
                                value=val.clone()
                                selected=move || {
                                    value.get().map(|v| v.value()).unwrap_or_default() == val
                                }
                            >
                                {text}
                            </option>
                        }
                    })
                    .collect_view()}
            </select>
            // Display error only when not typing and an error exists
            <Show when=show_error>
                <label class="label">
                    <span class="label-text-alt text-error w-full text-left block whitespace-normal">
                        {move || error.get()}
                    </span>
                </label>
            </Show>
        </div>
    }
}

// ToDo: deprecated, replace with EnumSelectWithValidation
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

// ToDo: deprecated, replace with NumberInputWithValidation
#[component]
pub fn ValidatedNumberInput<T>(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<T>,
    #[prop(into)] validation_error: Signal<FieldResult<()>>,
    #[prop(into, optional)] step: String, // "1" for int, "0.1" for float
    #[prop(into, optional)] min: String,
    // Optional on blur callback, e.g. for update of json config
    #[prop(into, optional)] on_blur: Option<Callback<()>>,
) -> impl IntoView
where
    T: FromStr + Display + Default + Copy + Send + Sync + 'static,
    <T as FromStr>::Err: std::fmt::Debug,
{
    // Default step to "1" if not provided
    let step_val = if step.is_empty() {
        "1".to_string()
    } else {
        step
    };
    // Default min to "0" if not provided
    let min = if min.is_empty() { "0".to_string() } else { min };

    let (parse_err, set_parse_err) = signal(None::<String>);

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
                class:input-error=move || validation_error.get().is_err()
                name=name.clone()
                data-testid=format!("input-{}", name)
                aria-invalid=move || validation_error.get().is_err().to_string()
                // We bind the value via prop:value which expects a string/number
                prop:value=move || value.get().to_string()
                on:input=move |ev| {
                    let val_str = event_target_value(&ev);
                    match val_str.parse::<T>() {
                        Ok(val) => {
                            value.set(val);
                            set_parse_err.set(None);
                        }
                        Err(err) => {
                            set_parse_err.set(Some(format!("Invalid number format: {:?}", err)));
                        }
                    }
                }
                on:blur=move |_| {
                    if let Some(cb) = on_blur {
                        cb.run(());
                    }
                }
            />
            <Show when=move || validation_error.get().is_err()>
                <label class="label">
                    <span class="label-text-alt text-error w-full text-left block whitespace-normal">
                        {move || { validation_error.get().err().map(|e| e.to_string()) }}
                    </span>
                </label>
            </Show>
            <Show when=move || parse_err.get().is_some()>
                <label class="label">
                    <span class="label-text-alt text-error w-full text-left block whitespace-normal">
                        {move || { parse_err.get().clone().unwrap_or_default() }}
                    </span>
                </label>
            </Show>
        </div>
    }
}

// ToDo: replace all instances of ValidatedNumberInput and ValidatedOptionNumberInput with this more advanced component
#[component]
pub fn NumberInputWithValidation<T>(
    /// Label text for the input
    #[prop(into)]
    label: String,
    /// Name attribute for the input (also used for test-id)
    #[prop(into)]
    name: String,
    /// Reactive read-access to Option<T>.
    /// Using Signal<Option<T>> allows passing ReadSignal, Memo, or derived closures.
    #[prop(into)]
    value: Signal<Option<T>>,
    /// Callback to push changes to source of value
    /// Using Callback<T> allows passing closures and Callbacks.
    #[prop(into)]
    on_write: Callback<T>,
    /// Optional step attribute for the number input
    #[prop(into, optional)]
    step: String,
    /// Optional min attribute for the number input
    #[prop(into, optional)]
    min: String,
    /// Reactive read-access to validation results
    /// Using Signal<ValidationResult<()>> allows passing ReadSignal, Memo, or derived closures.
    validation_result: Signal<ValidationResult<()>>,
    /// Object ID for field error lookup
    #[prop(into)]
    object_id: Signal<Option<Uuid>>,
    /// Field name for field error lookup
    #[prop(into)]
    field: String,
    /// Whether the field is optional (affects label and placeholder)
    #[prop(into, default = false)]
    optional: bool,
) -> impl IntoView
where
    T: FromStr + Display + Clone + Send + Sync + 'static,
    T::Err: std::fmt::Display,
{
    // Local buffer: Some(string) while typing, None when synced with Core
    let (draft, set_draft) = signal(None::<String>);

    // type parse error
    let (parse_err, set_parse_err) = signal(None::<String>);

    // Error state from validation of all objects
    let error = Signal::derive(move || {
        if let Some(e) = parse_err.get() {
            return Some(e);
        } else {
            is_object_field_valid(validation_result, object_id, &field)
                .err()
                .map(|e| e.to_string())
        }
    });

    // Derived: What to actually show in the <input>
    let display_value = move || match draft.get() {
        Some(d) => d,
        None => value.get().map(|v| v.to_string()).unwrap_or_default(),
    };

    // Derived: Error visibility logic
    // We hide errors while the user is actively typing (proactive reset)
    let show_error = move || draft.get().is_none() && error.get().is_some();

    // Auto-generate label and placeholder text based on label and optionality
    let (label, placeholder_text) = if optional {
        (
            format!("{} (optional)", label.clone()),
            format!("Enter {} (optional)...", label.to_lowercase()),
        )
    } else {
        (label.clone(), format!("Enter {}...", label.to_lowercase()))
    };
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
                <span class="label-text">{label}</span>
            </label>
            <input
                type="number"
                step=step_val
                min=min
                class="input input-bordered w-full"
                aria-invalid=move || show_error().to_string()
                prop:value=display_value
                name=name.clone()
                // Auto-generate test-id
                data-testid=format!("input-{}", name)
                placeholder=placeholder_text
                // USER TYPING: Update draft to take control from the core
                on:input:target=move |ev| {
                    set_draft.set(Some(ev.target().value()));
                }
                // USER FINISHED: Release control and attempt to commit to core
                on:change:target=move |ev| {
                    let new_val = ev.target().value();
                    match new_val.parse::<T>() {
                        Ok(val) => {
                            on_write.run(val);
                            set_parse_err.set(None);
                            set_draft.set(None);
                        }
                        Err(err) => {
                            set_parse_err.set(Some(format!("{err}")));
                        }
                    }
                }
                // USER LEAVES FIELD: Reset draft to sync with core
                on:blur=move |_| {
                    set_parse_err.set(None);
                    set_draft.set(None);
                }
            />
            // Display error only when not typing and an error exists
            <Show when=show_error>
                <label class="label">
                    <span class="label-text-alt text-error w-full text-left block whitespace-normal">
                        {move || error.get()}
                    </span>
                </label>
            </Show>
        </div>
    }
}

// ToDo: deprecated, replace with NumberInputWithValidation
#[component]
pub fn ValidatedOptionNumberInput<T>(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<Option<T>>,
    #[prop(into)] validation_error: Signal<FieldResult<()>>,
    #[prop(into, optional)] step: String,
    #[prop(into, optional)] min: String,
    // Optional on blur callback, e.g. for update of json config
    #[prop(into, optional)] on_blur: Option<Callback<()>>,
) -> impl IntoView
where
    T: FromStr + Display + Copy + Send + Sync + 'static,
    <T as FromStr>::Err: std::fmt::Debug,
{
    // Default step to "1" if not provided
    let step_val = if step.is_empty() {
        "1".to_string()
    } else {
        step
    };
    // Default min to "0" if not provided
    let min = if min.is_empty() { "0".to_string() } else { min };

    let (parse_err, set_parse_err) = signal(None::<String>);

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
                class:input-error=move || validation_error.get().is_err()
                name=name.clone()
                data-testid=format!("input-{}", name)
                aria-invalid=move || validation_error.get().is_err().to_string()
                prop:value=move || {
                    match value.get() {
                        Some(v) => v.to_string(),
                        None => String::new(),
                    }
                }
                on:input=move |ev| {
                    let val_str = event_target_value(&ev);
                    if val_str.is_empty() {
                        value.set(None);
                    } else {
                        match val_str.parse::<T>() {
                            Ok(val) => {
                                value.set(Some(val));
                                set_parse_err.set(None);
                            }
                            Err(err) => {
                                set_parse_err
                                    .set(Some(format!("Invalid number format: {:?}", err)));
                            }
                        }
                    }
                }
                on:blur=move |_| {
                    if let Some(cb) = on_blur {
                        cb.run(());
                    }
                }
            />
            <Show when=move || validation_error.get().is_err()>
                <label class="label">
                    <span class="label-text-alt text-error w-full text-left block whitespace-normal">
                        {move || { validation_error.get().err().map(|e| e.to_string()) }}
                    </span>
                </label>
            </Show>
            <Show when=move || parse_err.get().is_some()>
                <label class="label">
                    <span class="label-text-alt text-error w-full text-left block whitespace-normal">
                        {move || { parse_err.get().clone().unwrap_or_default() }}
                    </span>
                </label>
            </Show>
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

// ToDo: deprecated, replace with DurationInputWithValidation
#[component]
pub fn ValidatedDurationInput(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    value: RwSignal<Duration>,
    #[prop(into)] unit: DurationInputUnit,
    #[prop(into)] validation_error: Signal<FieldResult<()>>,
    // Optional on blur callback, e.g. for update of json config
    #[prop(into, optional)] on_blur: Option<Callback<()>>,
) -> impl IntoView {
    let (parse_err, set_parse_err) = signal(None::<String>);

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
                class:input-error=move || validation_error.get().is_err()
                name=name.clone()
                data-testid=format!("input-{}", name)
                aria-invalid=move || validation_error.get().is_err().to_string()
                // Convert Duration to unit for display
                prop:value=move || {
                    match unit {
                        DurationInputUnit::Seconds => value.get().as_secs().to_string(),
                        DurationInputUnit::Minutes => (value.get().as_secs() / 60).to_string(),
                        DurationInputUnit::Hours => (value.get().as_secs() / 3600).to_string(),
                    }
                }
                on:input=move |ev| {
                    let val_str = event_target_value(&ev);
                    match val_str.parse::<u64>() {
                        Ok(input) => {
                            match unit {
                                DurationInputUnit::Seconds => value.set(Duration::from_secs(input)),
                                DurationInputUnit::Minutes => {
                                    value.set(Duration::from_secs(input * 60))
                                }
                                DurationInputUnit::Hours => {
                                    value.set(Duration::from_secs(input * 3600))
                                }
                            }
                            set_parse_err.set(None);
                        }
                        Err(err) => {
                            set_parse_err.set(Some(format!("Invalid number format: {:?}", err)));
                        }
                    }
                }
                on:blur=move |_| {
                    if let Some(cb) = on_blur {
                        cb.run(());
                    }
                }
            />
            <Show when=move || validation_error.get().is_err()>
                <label class="label">
                    <span class="label-text-alt text-error w-full text-left block whitespace-normal">
                        {move || { validation_error.get().err().map(|e| e.to_string()) }}
                    </span>
                </label>
            </Show>
            <Show when=move || parse_err.get().is_some()>
                <label class="label">
                    <span class="label-text-alt text-error w-full text-left block whitespace-normal">
                        {move || { parse_err.get().clone().unwrap_or_default() }}
                    </span>
                </label>
            </Show>
        </div>
    }
}

// ToDo: replace all instances of ValidatedDurationInput and TextInput with this more advanced component
#[component]
pub fn DurationInputWithValidation(
    /// Label text for the input
    #[prop(into)]
    label: String,
    /// Name attribute for the input (also used for test-id)
    #[prop(into)]
    name: String,
    /// Reactive read-access to Option<T>.
    /// Using Signal<Option<T>> allows passing ReadSignal, Memo, or derived closures.
    #[prop(into)]
    value: Signal<Option<Duration>>,
    /// Callback to push changes to source of value
    /// Using Callback<T> allows passing closures and Callbacks.
    #[prop(into)]
    on_write: Callback<Duration>,
    /// Duration unit for input and display
    #[prop(into)]
    unit: DurationInputUnit,
    /// Reactive read-access to validation results
    /// Using Signal<ValidationResult<()>> allows passing ReadSignal, Memo, or derived closures.
    validation_result: Signal<ValidationResult<()>>,
    /// Object ID for field error lookup
    #[prop(into)]
    object_id: Signal<Option<Uuid>>,
    /// Field name for field error lookup
    #[prop(into)]
    field: String,
    /// Whether the field is optional (affects label and placeholder)
    #[prop(into, default = false)]
    optional: bool,
) -> impl IntoView
where
{
    // Local buffer: Some(string) while typing, None when synced with Core
    let (draft, set_draft) = signal(None::<String>);

    // type parse error
    let (parse_err, set_parse_err) = signal(None::<String>);

    // Error state from validation of all objects
    let error = Signal::derive(move || {
        if let Some(e) = parse_err.get() {
            return Some(e);
        } else {
            is_object_field_valid(validation_result, object_id, &field)
                .err()
                .map(|e| e.to_string())
        }
    });

    // Derived: What to actually show in the <input>
    let display_value = move || match draft.get() {
        Some(d) => d,
        None => value
            .get()
            .map(|v| match unit {
                DurationInputUnit::Seconds => v.as_secs().to_string(),
                DurationInputUnit::Minutes => (v.as_secs() / 60).to_string(),
                DurationInputUnit::Hours => (v.as_secs() / 3600).to_string(),
            })
            .unwrap_or_default(),
    };

    // Derived: Error visibility logic
    // We hide errors while the user is actively typing (proactive reset)
    let show_error = move || draft.get().is_none() && error.get().is_some();

    // Auto-generate label and placeholder text based on label and optionality
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
                <span class="label-text">{label}</span>
            </label>
            <input
                type="number"
                step=1
                min=0
                class="input input-bordered w-full"
                aria-invalid=move || show_error().to_string()
                prop:value=display_value
                name=name.clone()
                // Auto-generate test-id
                data-testid=format!("input-{}", name)
                placeholder=placeholder_text
                // USER TYPING: Update draft to take control from the core
                on:input:target=move |ev| {
                    set_draft.set(Some(ev.target().value()));
                }
                // USER FINISHED: Release control and attempt to commit to core
                on:change:target=move |ev| {
                    let new_val = ev.target().value();
                    match new_val.parse::<u64>() {
                        Ok(val) => {
                            let new_duration = match unit {
                                DurationInputUnit::Seconds => Duration::from_secs(val),
                                DurationInputUnit::Minutes => Duration::from_secs(val * 60),
                                DurationInputUnit::Hours => Duration::from_secs(val * 3600),
                            };
                            on_write.run(new_duration);
                            set_parse_err.set(None);
                            set_draft.set(None);
                        }
                        Err(err) => {
                            set_parse_err.set(Some(format!("{err}")));
                        }
                    }
                }
                // USER LEAVES FIELD: Reset draft to sync with core
                on:blur=move |_| {
                    set_parse_err.set(None);
                    set_draft.set(None);
                }
            />
            // Display error only when not typing and an error exists
            <Show when=show_error>
                <label class="label">
                    <span class="label-text-alt text-error w-full text-left block whitespace-normal">
                        {move || error.get()}
                    </span>
                </label>
            </Show>
        </div>
    }
}
