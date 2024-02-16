use std::collections::VecDeque;

use super::argument::Argument;
use super::argument_common::ArgumentCommon;
use super::argument_common::ArgumentCommonBuilder;
use super::argument_common::MatchResult;
use super::errors::OptionExt;
use super::errors::USER_ERROR;

pub struct BooleanArgument {
  common: ArgumentCommon,
}

impl BooleanArgument {
  pub fn new(args: &mut VecDeque<String>) -> Self {
    let mut common = ArgumentCommon::new_builder();
    match common.parse_arguments(args) {
      None => { }
      Some(other) => {
        args.push_front(other);
      }
    }

    return BooleanArgument {
      common: common.build(),
    };
  }
}

impl Argument for BooleanArgument {
  fn get_help_flags(&self) -> Vec<String> {
    self.common
        .get_all_flags()
        .iter()
        .map(|flag| format!("{flag}[=<true|false>]"))
        .collect()
  }

  fn get_help_default(&self) -> Option<String> {
    Some(String::from("When this option is not provided it will default to false. ") +
         &String::from("If provided without a value it will be set to true."))
  }

  fn get_common(&self) -> &ArgumentCommon {
    &self.common
  }

  fn get_debug_info(&self) -> String {
    return format!("type: Boolean; {}", self.common.get_debug_info());
  }

  fn consume(&self, arg: Option<String>, _other_args: &mut VecDeque<String>) -> Option<String> {
    match self.common.check_flag_match(arg) {
      MatchResult::NoMatch => None,
      MatchResult::MatchWithoutValue => Some(String::from("true")),
      MatchResult::MatchWithValue(value) => Some(value
        .parse::<bool>()
        .unwrap_or_error(USER_ERROR, format!("Non-boolean value '{value}' provided for argument {}", self.get_name()))
        .to_string()),
    }
  }
}

