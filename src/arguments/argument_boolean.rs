use std::collections::VecDeque;

use super::argument::Argument;
use super::argument_common::ArgumentCommon;
use super::argument_common::ArgumentCommonBuilder;
use super::argument_common::MatchResult;
use super::errors::error;
use super::errors::OptionExt;
use super::errors::DEFINITION_ERROR;
use super::errors::USER_ERROR;

pub struct BooleanArgument {
  common: ArgumentCommon,
  negative_flags: Vec<String>,
}

impl BooleanArgument {
  pub fn new(args: &mut VecDeque<String>) -> Self {
    let mut common = ArgumentCommon::new_builder();
    let mut negative_flags = Vec::new();

    loop {
      match common.parse_arguments(args).as_deref() {
        None => {
          break;
        }
        Some("--negative-flag") | Some("--negative") | Some("--neg") => {
          negative_flags.push(args.pop_front()
                .unwrap_or_error(DEFINITION_ERROR, String::from("flag must be provided after --negative-flag"))
                .to_string());
        }
        Some(other) => {
          args.push_front(other.to_string());
          break;
        }
      }
    }

    let common = common.build();

    if common.get_repeated() {
      error(DEFINITION_ERROR, format!("Boolean argument {} can not be repeated", common.get_name()));

    } else if common.get_catch_all() {
      error(DEFINITION_ERROR, format!("Boolean argument {} can not be catch-all", common.get_name()));

    } else if !common.get_ordinals().is_empty() {
      error(DEFINITION_ERROR, format!("Boolean argument {} can not be ordinal", common.get_name()));
    }

    return BooleanArgument {
      common: common,
      negative_flags: negative_flags,
    };
  }
}

impl Argument for BooleanArgument {
  fn get_help_flags(&self) -> Vec<String> {
    [
        self.common
          .get_all_flags()
          .iter()
          .map(|flag| format!("{flag}[=<true|false>]"))
          .collect::<Vec<String>>(),
        self.negative_flags
          .iter()
          .map(|flag| format!("{flag}"))
          .collect::<Vec<String>>()
    ].concat()
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
    match self.common.check_flag_match(arg.clone()) {
      MatchResult::NoMatch => {}
      MatchResult::MatchWithoutValue => {
        return Some(String::from("true"));
      }
      MatchResult::MatchWithValue(value) => {
        return Some(value
          .parse::<bool>()
          .unwrap_or_error(USER_ERROR, format!("Non-boolean value '{value}' provided for argument {}", self.get_name()))
          .to_string());
       }
    };

    if self.negative_flags.contains(&arg.unwrap()) {
      return Some(String::from("false"));
    }

    return None
  }
}

