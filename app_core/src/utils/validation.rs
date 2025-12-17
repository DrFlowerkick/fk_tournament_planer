// tools for validation of input

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{self, Display},
};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldError {
    field: String,
    // e.g. "required", "invalid_format"
    code: String,
    // human-friendly (or build from code+params)
    message: String,
    // e.g. { "min": "5" }
    params: HashMap<String, String>,
}

impl Display for FieldError {
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

impl FieldError {
    pub fn get_field(&self) -> &str {
        &self.field
    }
    pub fn get_code(&self) -> &str {
        self.code.as_str()
    }
    pub fn get_message(&self) -> &str {
        &self.message
    }
    pub fn get_params(&self) -> &HashMap<String, String> {
        &self.params
    }
}

impl FieldError {
    pub fn builder() -> FieldErrorBuilder<NoField> {
        FieldErrorBuilder {
            field: NoField {},
            code: "".into(),
            message: "".into(),
            params: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Error, Default, Serialize, Deserialize, PartialEq, Eq)]
#[error("validation failed with {} error(s)", errors.len())]
pub struct ValidationErrors {
    pub errors: Vec<FieldError>,
}

impl From<FieldError> for ValidationErrors {
    fn from(value: FieldError) -> Self {
        Self {
            errors: vec![value],
        }
    }
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }
    pub fn add(&mut self, err: FieldError) {
        self.errors.push(err);
    }
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

pub type ValidationResult<T> = Result<T, ValidationErrors>;

pub struct NoField {}
pub struct Field(String);

pub struct FieldErrorBuilder<F> {
    field: F,
    code: String,
    message: String,
    params: HashMap<String, String>,
}

impl FieldErrorBuilder<NoField> {
    pub fn set_field(self, field: impl Into<String>) -> FieldErrorBuilder<Field> {
        FieldErrorBuilder {
            field: Field(field.into()),
            code: self.code,
            message: self.message,
            params: self.params,
        }
    }
}

impl FieldErrorBuilder<Field> {
    /// set code to required
    pub fn add_required(mut self) -> Self {
        self.code = "required".into();
        self
    }
    /// set code to invalid_format
    pub fn add_invalid_format(mut self) -> Self {
        self.code = "invalid_format".into();
        self
    }
    /// set user defined code
    pub fn add_user_defined_code(mut self, code: &str) -> Self {
        self.code = code.into();
        self
    }
    /// set user defined code
    pub fn add_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }
    /// set user defined code
    pub fn add_params(mut self, key: String, value: impl Into<String>) -> Self {
        self.params.insert(key, value.into());
        self
    }
    /// build FieldError
    pub fn build(self) -> FieldError {
        FieldError {
            field: self.field.0,
            code: self.code,
            message: self.message,
            params: self.params,
        }
    }
}
