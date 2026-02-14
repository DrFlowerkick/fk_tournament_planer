//! Input components for the app

use crate::{enum_utils::SelectableOption, hooks::is_field_valid::is_object_field_valid};
use app_core::utils::validation::ValidationResult;
use displaydoc::Display;
use leptos::{
    ev::{Event, Targeted},
    prelude::*,
    web_sys::HtmlInputElement,
};
use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

/// Determines when the input value is committed to the parent signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputUpdateStrategy {
    /// Commit value on `change` event (blur or enter). This is the default.
    #[default]
    Change,
    /// Commit value on `input` event (every keystroke).
    Input,
}

impl InputUpdateStrategy {
    pub fn commit_input<T>(
        &self,
        ev: Targeted<Event, HtmlInputElement>,
        set_draft: WriteSignal<Option<String>>,
        action: InputCommitAction<T>,
        set_parse_err: WriteSignal<Option<String>>,
    ) where
        T: FromStr + Display + Clone + Send + Sync + 'static,
    {
        match self {
            InputUpdateStrategy::Change => {
                // Only update the draft on input, commit on change
                set_draft.set(Some(ev.target().value()));
            }
            InputUpdateStrategy::Input => {
                // Update and commit on every input event
                self.commit_update(ev, set_draft, action, set_parse_err);
            }
        }
    }

    pub fn commit_change<T>(
        &self,
        ev: Targeted<Event, HtmlInputElement>,
        set_draft: WriteSignal<Option<String>>,
        action: InputCommitAction<T>,
        set_parse_err: WriteSignal<Option<String>>,
    ) where
        T: FromStr + Display + Clone + Send + Sync + 'static,
    {
        match self {
            InputUpdateStrategy::Change => {
                self.commit_update(ev, set_draft, action, set_parse_err);
            }
            InputUpdateStrategy::Input => {
                // No additional action needed on change, since value is already committed on input
            }
        }
    }

    pub fn commit_update<T>(
        &self,
        ev: Targeted<Event, HtmlInputElement>,
        set_draft: WriteSignal<Option<String>>,
        action: InputCommitAction<T>,
        set_parse_err: WriteSignal<Option<String>>,
    ) where
        T: FromStr + Display + Clone + Send + Sync + 'static,
    {
        let new_val = ev.target().value();
        let submit = if new_val.is_empty() {
            set_parse_err.set(None);
            set_draft.set(None);
            action.execute(None)
        } else {
            match new_val.parse::<T>() {
                Ok(val) => {
                    set_parse_err.set(None);
                    set_draft.set(None);
                    action.execute(Some(val))
                }
                Err(_) => {
                    set_parse_err.set(Some(format!("Invalid input format, parse failed.")));
                    false
                }
            }
        };
        if submit {
            // Trigger form submission if requested by the action
            ev.target().form().map(|f| f.request_submit());
        }
    }
    pub fn commit_duration_input(
        &self,
        ev: Targeted<Event, HtmlInputElement>,
        set_draft: WriteSignal<Option<String>>,
        action: InputCommitAction<Duration>,
        set_parse_err: WriteSignal<Option<String>>,
        unit: DurationInputUnit,
    ) {
        match self {
            InputUpdateStrategy::Change => {
                // Only update the draft on input, commit on change
                set_draft.set(Some(ev.target().value()));
            }
            InputUpdateStrategy::Input => {
                // Update and commit on every input event
                self.commit_duration_update(ev, set_draft, action, set_parse_err, unit);
            }
        }
    }

    pub fn commit_duration_change(
        &self,
        ev: Targeted<Event, HtmlInputElement>,
        set_draft: WriteSignal<Option<String>>,
        action: InputCommitAction<Duration>,
        set_parse_err: WriteSignal<Option<String>>,
        unit: DurationInputUnit,
    ) {
        match self {
            InputUpdateStrategy::Change => {
                self.commit_duration_update(ev, set_draft, action, set_parse_err, unit);
            }
            InputUpdateStrategy::Input => {
                // No additional action needed on change, since value is already committed on input
            }
        }
    }

    pub fn commit_duration_update(
        &self,
        ev: Targeted<Event, HtmlInputElement>,
        set_draft: WriteSignal<Option<String>>,
        action: InputCommitAction<Duration>,
        set_parse_err: WriteSignal<Option<String>>,
        unit: DurationInputUnit,
    ) {
        let new_val = ev.target().value();
        let submit = if new_val.is_empty() {
            set_parse_err.set(None);
            set_draft.set(None);
            action.execute(None)
        } else {
            match new_val.parse::<u64>() {
                Ok(val) => {
                    let new_duration = match unit {
                        DurationInputUnit::Seconds => Duration::from_secs(val),
                        DurationInputUnit::Minutes => Duration::from_secs(val * 60),
                        DurationInputUnit::Hours => Duration::from_secs(val * 3600),
                    };
                    set_parse_err.set(None);
                    set_draft.set(None);
                    action.execute(Some(new_duration))
                }
                Err(err) => {
                    set_parse_err.set(Some(format!("{err}")));
                    false
                }
            }
        };
        if submit {
            // Trigger form submission if requested by the action
            ev.target().form().map(|f| f.request_submit());
        }
    }
}

/// Defines what happens when an input value is committed.
/// Enforces compile-time safety to ensure at least one action (callback or submit) is taken.
pub enum InputCommitAction<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Only update the parent signal via callback.
    WriteTo(Callback<Option<T>>),
    /// Only trigger a form submission.
    SubmitForm,
    /// Update the parent signal AND trigger a form submission.
    WriteAndSubmit(Callback<Option<T>>),
}

impl<T> Clone for InputCommitAction<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for InputCommitAction<T> where T: Clone + Send + Sync + 'static {}

impl<T> InputCommitAction<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Executes the configured action. Returns true if a submit is requested.
    pub fn execute(&self, value: Option<T>) -> bool {
        match self {
            InputCommitAction::WriteTo(cb) => {
                cb.run(value);
                false
            }
            InputCommitAction::SubmitForm => true,
            InputCommitAction::WriteAndSubmit(cb) => {
                cb.run(value);
                true
            }
        }
    }
}

#[component]
pub fn TextInput<T>(
    /// Label text for the input
    #[prop(into)]
    label: String,
    /// Name attribute for the input (also used for test-id)
    /// If None, input will not be submitted in forms.
    #[prop(into, optional)]
    name: Option<String>,
    /// Optional data-testid attribute for testing
    #[prop(into, optional)]
    data_testid: Option<String>,
    /// Reactive read-access to Option<T>.
    /// Using Signal<Option<T>> allows passing ReadSignal, Memo, or derived closures.
    #[prop(into)]
    value: Signal<Option<T>>,
    /// Defines the action to take when the value changes.
    action: InputCommitAction<T>,
    /// Strategy for committing values to the parent signal.
    #[prop(into, default = InputUpdateStrategy::default())]
    update_on: InputUpdateStrategy,
    /// Reactive read-access to validation results
    /// Using Signal<ValidationResult<()>> allows passing ReadSignal, Memo, or derived closures.
    #[prop(into, default = Ok(()).into())]
    validation_result: Signal<ValidationResult<()>>,
    /// Object ID for field error lookup
    #[prop(into, default = None.into())]
    object_id: Signal<Option<Uuid>>,
    /// Field name for field error lookup
    #[prop(into, default = String::new())]
    field: String,
    /// Whether the field is optional (affects label and placeholder)
    #[prop(into, default = false)]
    optional: bool,
    /// Placeholder text for the input
    #[prop(into, default = String::new())]
    placeholder: String,
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
    let (label, placeholder_text) = generate_label_placeholder(label, optional, placeholder);

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
                name=name
                data-testid=data_testid
                placeholder=placeholder_text
                on:input:target=move |ev| {
                    update_on.commit_input(ev, set_draft, action, set_parse_err)
                }
                on:change:target=move |ev| {
                    update_on.commit_change(ev, set_draft, action, set_parse_err)
                }
                // USER LEAVES FIELD: Reset draft to sync with core
                on:blur=move |_| {
                    if parse_err.get().is_some() {
                        set_parse_err.set(None);
                        set_draft.set(None);
                    }
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

#[component]
pub fn NumberInput<T>(
    /// Label text for the input
    #[prop(into)]
    label: String,
    /// Name attribute for the input (also used for test-id)
    /// If None, input will not be submitted in forms.
    #[prop(into, optional)]
    name: Option<String>,
    /// Optional data-testid attribute for testing
    #[prop(into, optional)]
    data_testid: Option<String>,
    /// Reactive read-access to Option<T>.
    /// Using Signal<Option<T>> allows passing ReadSignal, Memo, or derived closures.
    #[prop(into)]
    value: Signal<Option<T>>,
    /// Defines the action to take when the value changes.
    action: InputCommitAction<T>,
    /// Strategy for committing values to the parent signal.
    #[prop(into, default = InputUpdateStrategy::default())]
    update_on: InputUpdateStrategy,
    /// Optional step attribute for the number input
    #[prop(into, optional)]
    step: String,
    /// Optional min attribute for the number input
    #[prop(into, optional)]
    min: String,
    /// Reactive read-access to validation results
    /// Using Signal<ValidationResult<()>> allows passing ReadSignal, Memo, or derived closures.
    #[prop(into, default = Ok(()).into())]
    validation_result: Signal<ValidationResult<()>>,
    /// Object ID for field error lookup
    #[prop(into, default = None.into())]
    object_id: Signal<Option<Uuid>>,
    /// Field name for field error lookup
    #[prop(into, default = String::new())]
    field: String,
    /// Whether the field is optional (affects label and placeholder)
    #[prop(into, default = false)]
    optional: bool,
    /// Placeholder text for the input
    #[prop(into, default = String::new())]
    placeholder: String,
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
    let (label, placeholder_text) = generate_label_placeholder(label, optional, placeholder);

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
                name=name
                data-testid=data_testid
                placeholder=placeholder_text
                on:input:target=move |ev| {
                    update_on.commit_input(ev, set_draft, action, set_parse_err)
                }
                on:change:target=move |ev| {
                    update_on.commit_change(ev, set_draft, action, set_parse_err)
                }
                // USER LEAVES FIELD: Reset draft to sync with core
                on:blur=move |_| {
                    if parse_err.get().is_some() {
                        set_parse_err.set(None);
                        set_draft.set(None);
                    }
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
pub fn DurationInput(
    /// Label text for the input
    #[prop(into)]
    label: String,
    /// Name attribute for the input (also used for test-id)
    /// If None, input will not be submitted in forms.
    #[prop(into, optional)]
    name: Option<String>,
    /// Optional data-testid attribute for testing
    #[prop(into, optional)]
    data_testid: Option<String>,
    /// Reactive read-access to Option<T>.
    /// Using Signal<Option<T>> allows passing ReadSignal, Memo, or derived closures.
    #[prop(into)]
    value: Signal<Option<Duration>>,
    /// Defines the action to take when the value changes.
    action: InputCommitAction<Duration>,
    /// Strategy for committing values to the parent signal.
    #[prop(into, default = InputUpdateStrategy::default())]
    update_on: InputUpdateStrategy,
    /// Duration unit for input and display
    #[prop(into)]
    unit: DurationInputUnit,
    /// Reactive read-access to validation results
    /// Using Signal<ValidationResult<()>> allows passing ReadSignal, Memo, or derived closures.
    #[prop(into, default = Ok(()).into())]
    validation_result: Signal<ValidationResult<()>>,
    /// Object ID for field error lookup
    #[prop(into, default = None.into())]
    object_id: Signal<Option<Uuid>>,
    /// Field name for field error lookup
    #[prop(into, default = String::new())]
    field: String,
    /// Whether the field is optional (affects label and placeholder)
    #[prop(into, default = false)]
    optional: bool,
    /// Placeholder text for the input
    #[prop(into, default = String::new())]
    placeholder: String,
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
    let (label, placeholder_text) = generate_label_placeholder(label, optional, placeholder);

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
                name=name
                data-testid=data_testid
                placeholder=placeholder_text
                on:input:target=move |ev| {
                    update_on.commit_duration_input(ev, set_draft, action, set_parse_err, unit)
                }
                on:change:target=move |ev| {
                    update_on.commit_duration_change(ev, set_draft, action, set_parse_err, unit)
                }
                // USER LEAVES FIELD: Reset draft to sync with core
                on:blur=move |_| {
                    if parse_err.get().is_some() {
                        set_parse_err.set(None);
                        set_draft.set(None);
                    }
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

#[component]
pub fn EnumSelect<E>(
    /// Label text for the input
    #[prop(into)]
    label: String,
    /// Name attribute for the input (also used for test-id)
    /// If None, input will not be submitted in forms.
    #[prop(into, optional)]
    name: Option<String>,
    /// Optional data-testid attribute for testing
    #[prop(into, optional)]
    data_testid: Option<String>,
    /// Reactive read-access to Option<T>.
    /// Using Signal<Option<E>> allows passing ReadSignal, Memo, or derived closures.
    #[prop(into)]
    value: Signal<Option<E>>,
    /// Defines the action to take when the value changes.
    action: InputCommitAction<E>,
    /// Reactive read-access to validation results
    /// Using Signal<ValidationResult<()>> allows passing ReadSignal, Memo, or derived closures.
    #[prop(into, default = Ok(()).into())]
    validation_result: Signal<ValidationResult<()>>,
    /// Object ID for field error lookup
    #[prop(into, default = None.into())]
    object_id: Signal<Option<Uuid>>,
    /// Field name for field error lookup
    #[prop(into, default = String::new())]
    field: String,
    /// Whether the field is optional (affects label and placeholder)
    #[prop(into, default = false)]
    optional: bool,
    /// Optional label for a "Clear selection" option.
    /// If Some("Text"), an option to clear the selection (set to None) is displayed.
    /// Note: Make sure that the "clear" option value does not conflict with any Enum option value.
    #[prop(into, optional)]
    clear_label: Option<String>,
    /// Placeholder text for the input
    #[prop(into, default = String::new())]
    placeholder: String,
) -> impl IntoView
where
    E: SelectableOption,
{
    // StoredValue helper for optional props
    let clear_label = StoredValue::new(clear_label);

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

    // Auto-generate data-testid, label, and placeholder text based on name, label, and optionality
    let (label, placeholder_text) = generate_label_placeholder(label, optional, placeholder);

    view! {
        <div class="form-control w-full">
            <label class="label">
                <span class="label-text">{label}</span>
            </label>
            <select
                class="select select-bordered w-full"
                aria-invalid=move || show_error().to_string()
                prop:value=move || {
                    match value.get().as_ref() {
                        Some(v) => v.value(),
                        None => clear_label.get_value().unwrap_or_default(),
                    }
                }
                name=name
                data-testid=data_testid
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
                    let submit = if new_val.is_empty()
                        || clear_label.get_value().as_deref() == Some(&new_val)
                    {
                        action.execute(None)
                    } else if let Some(selected_variant) = value
                        .with(|v| {
                            v.as_ref()
                                .map_or_else(|| E::static_options(), |current| current.options())
                                .into_iter()
                                .find(|o| o.value() == new_val)
                        })
                    {
                        action.execute(Some(selected_variant))
                    } else {
                        action.execute(None)
                    };
                    if submit {
                        ev.target().form().map(|f| f.request_submit());
                    }
                    set_is_selecting.set(false);
                }
            >
                {move || {
                    clear_label
                        .with_value(|maybe_label| match maybe_label {
                            Some(label) => {
                                let val = label.clone();
                                if value
                                    .with(|v| {
                                        v.as_ref()
                                            .map_or_else(
                                                || E::static_options(),
                                                |current| current.options(),
                                            )
                                            .into_iter()
                                            .any(|o| o.value() == val)
                                    })
                                {
                                    return ().into_any();
                                }
                                // Avoid rendering the clear option if its value conflicts
                                // with any Enum option value
                                view! { <option value=val>{label.to_owned()}</option> }
                                    .into_any()
                            }
                            None => {
                                let placeholder_text = placeholder_text.clone();
                                // Render placeholder, if no clear_label is provided
                                view! {
                                    <option disabled selected value="">
                                        {placeholder_text}
                                    </option>
                                }
                                    .into_any()
                            }
                        })
                }}

                {move || {
                    value
                        .get()
                        .map_or_else(|| E::static_options(), |current| current.options())
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
                        .collect_view()
                }}
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

/// Auto-generate label and placeholder text based on label and optionality
fn generate_label_placeholder(
    label: String,
    optional: bool,
    place_holder: String,
) -> (String, String) {
    let (label, placeholder_text) = if optional {
        (
            format!("{} (optional)", label.clone()),
            format!("Enter {} (optional)...", label.to_lowercase()),
        )
    } else {
        (label.clone(), format!("Enter {}...", label.to_lowercase()))
    };
    if !place_holder.is_empty() {
        (label, place_holder)
    } else {
        (label, placeholder_text)
    }
}
