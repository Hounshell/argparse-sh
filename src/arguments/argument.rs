use std::collections::VecDeque;

use super::argument_common::ArgumentCommon;
use super::argument_common::MatchResult;
use super::errors::OptionExt;
use super::errors::USER_ERROR;

pub trait Argument {
  /// Provides a terse representation of the argument, suitable for debugging.
  fn get_debug_info(&self) -> String;

  /// Gets the ArgumentCommon pieces of the Argument.
  fn get_common(&self) -> &ArgumentCommon;

  /// Attempts to consume the provided argument. 
  ///
  /// Return value is None if the argument couldn't be consumed, Some(value) if it could. This
  /// may or may not remove additional items from the `other_args` queue.
  fn consume(&self, arg: Option<String>, other_args: &mut VecDeque<String>) -> Option<String>;

  fn consume_with_parser(
      &self,
      arg: Option<String>,
      other_args: &mut VecDeque<String>,
      parser: fn(&String, &String) -> String) -> Option<String> {
    match self.get_common().check_flag_match(arg) {
      MatchResult::NoMatch => None,
      MatchResult::MatchWithValue(_flag, value) => Some(parser(self.get_name(), &value)),
      MatchResult::MatchWithoutValue => Some(parser(
          self.get_name(),
          &other_args.pop_front()
            .unwrap_or_error(USER_ERROR, format!("No value provided for argument {}", self.get_name()))))
    }
  }

  fn get_help_details(&self) -> Vec<String> {
    vec![self.get_description().clone().unwrap_or(String::from("No details available."))]
  }

  fn get_help_flags(&self) -> Vec<String> {
    self.get_common()
        .get_all_flags()
        .iter()
        .map(|flag| format!("{} <{}>", flag, self.get_name().to_lowercase()))
        .collect()
  }

  fn get_help_default(&self) -> Option<String> {
    if self.get_default().is_some() {
      Some(format!(
          "When this option is not provided it will default to '{}'.",
          self.get_default().clone().unwrap()))
    } else {
      None
    }
  }

  fn get_name(&self) -> &String {
    self.get_common().get_name()
  }

  fn get_description(&self) -> &Option<String> {
    self.get_common().get_description()
  }

  fn get_default(&self) -> &Option<String> {
    self.get_common().get_default()
  }

  fn is_secret(&self) -> bool {
    self.get_common().get_secret()
  }

  fn is_repeated(&self) -> bool {
    self.get_common().get_repeated()
  }

  fn is_required(&self) -> bool {
    self.get_common().get_required()
  }

  fn is_ordinal(&self, ordinal: u16) -> bool {
    self.get_common().get_ordinals().contains(&ordinal)
  }

  fn is_catch_all(&self) -> bool {
    self.get_common().get_catch_all()
  }
}

