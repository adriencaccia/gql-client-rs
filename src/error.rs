use std::fmt::{self, Formatter};

use reqwest::Error;
use serde::Deserialize;
use serde_json::Map;

#[derive(Clone)]
pub struct GraphQLError {
  message: String,
  json: Option<Vec<GraphQLErrorMessage>>,
}

// https://spec.graphql.org/June2018/#sec-Errors
#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct GraphQLErrorMessage {
  message: String,
  locations: Option<Vec<GraphQLErrorLocation>>,
  extensions: Option<Map<String, serde_json::Value>>,
  path: Option<Vec<GraphQLErrorPathParam>>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct GraphQLErrorLocation {
  line: u32,
  column: u32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GraphQLErrorPathParam {
  String(String),
  Number(u32),
}

impl GraphQLError {
  /// Check if the provided error message is equal to one of the error messages
  pub fn contains_error_message(&self, message: &str) -> bool {
    self.json.as_ref().is_some_and(|errors| {
      errors.iter().any(|err| err.message == message)
    })
  }

  /// Check if the provided error code is equal to one of the error codes
  pub fn contains_error_code(&self, code: &str) -> bool {
    self.json.as_ref().is_some_and(|errors| {
      errors.iter().any(|err| {
        err.extensions.as_ref().is_some_and(|ext| {
          ext.get("code").is_some_and(|val| val.as_str().unwrap_or_default() == code)
        })
      })
    })
  }


  pub fn with_text(message: impl AsRef<str>) -> Self {
    Self {
      message: message.as_ref().to_string(),
      json: None,
    }
  }

  pub fn with_message_and_json(message: impl AsRef<str>, json: Vec<GraphQLErrorMessage>) -> Self {
    Self {
      message: message.as_ref().to_string(),
      json: Some(json),
    }
  }

  pub fn with_json(json: Vec<GraphQLErrorMessage>) -> Self {
    Self::with_message_and_json("Look at json field for more details", json)
  }

  pub fn message(&self) -> &str {
    &self.message
  }

  pub fn json(&self) -> Option<Vec<GraphQLErrorMessage>> {
    self.json.clone()
  }
}

fn format(err: &GraphQLError, f: &mut Formatter<'_>) -> fmt::Result {
  // Print the main error message
  writeln!(f, "\nGQLClient Error: {}", err.message)?;

  // Check if query errors have been received
  if err.json.is_none() {
    return Ok(());
  }

  let errors = err.json.as_ref();

  for err in errors.unwrap() {
    writeln!(f, "Message: {}", err.message)?;
  }

  Ok(())
}

impl fmt::Display for GraphQLError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    format(self, f)
  }
}

impl fmt::Debug for GraphQLError {
  #[allow(clippy::needless_borrow)]
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    format(&self, f)
  }
}

impl From<Error> for GraphQLError {
  fn from(error: Error) -> Self {
    Self {
      message: error.to_string(),
      json: None,
    }
  }
}
