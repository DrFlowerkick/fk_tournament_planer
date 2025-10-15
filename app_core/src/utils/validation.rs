// tools for validation of input

use std::{
    collections::HashMap,
    fmt::{self, Display},
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct FieldError<F> {
    field: F,
    // e.g. "required", "invalid_format"
    code: &'static str,
    // human-friendly (or build from code+params)
    message: String,
    // e.g. { "min": "5" }
    params: HashMap<&'static str, String>,
}

impl<F: Display> Display for FieldError<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.message.is_empty() {
            write!(f, "{}: {}", self.field, self.code)?;
        } else {
            write!(f, "{}", self.message)?;
        }
        if !self.params.is_empty() {
            write!(f, "\nparams:")?;
        }
        for (key, value) in self.params.iter() {
            write!(f, "\n{key}: {value}")?;
        }
        Ok(())
    }
}

impl<F> FieldError<F> {
    pub fn get_field(&self) -> &F {
        &self.field
    }
    pub fn get_code(&self) -> &'static str {
        self.code
    }
    pub fn get_message(&self) -> &str {
        &self.message
    }
    pub fn get_params(&self) -> &HashMap<&'static str, String> {
        &self.params
    }
}

impl FieldError<NoField> {
    pub fn builder() -> FieldErrorBuilder<NoField> {
        FieldErrorBuilder {
            field: NoField {},
            code: "",
            message: "".into(),
            params: HashMap::new(),
        }
    }
}

#[derive(Debug, Error)]
#[error("validation failed with {} error(s)", errors.len())]
pub struct ValidationErrors<F> {
    pub errors: Vec<FieldError<F>>,
}

impl<F> ValidationErrors<F> {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }
    pub fn add(&mut self, err: FieldError<F>) {
        self.errors.push(err);
    }
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

pub struct NoField {}
pub struct Field<F>(F);

pub struct FieldErrorBuilder<F> {
    field: F,
    code: &'static str,
    message: String,
    params: HashMap<&'static str, String>,
}

impl FieldErrorBuilder<NoField> {
    pub fn set_field<F>(self, field: F) -> FieldErrorBuilder<Field<F>> {
        FieldErrorBuilder {
            field: Field(field),
            code: self.code,
            message: self.message,
            params: self.params,
        }
    }
}

impl<F> FieldErrorBuilder<Field<F>> {
    /// set code to required
    pub fn add_required(mut self) -> Self {
        self.code = "required";
        self
    }
    /// set code to invalid_format
    pub fn add_invalid_format(mut self) -> Self {
        self.code = "invalid_format";
        self
    }
    /// set user defined code
    pub fn add_user_defined_code(mut self, code: &'static str) -> Self {
        self.code = code;
        self
    }
    /// set user defined code
    pub fn add_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }
    /// set user defined code
    pub fn add_params(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.params.insert(key, value.into());
        self
    }
    /// build FieldError
    pub fn build(self) -> FieldError<F> {
        FieldError {
            field: self.field.0,
            code: self.code,
            message: self.message,
            params: self.params,
        }
    }
}
