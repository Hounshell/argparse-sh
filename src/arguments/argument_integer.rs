use std::collections::VecDeque;

use super::argument::Argument;
use super::argument_common::ArgumentCommon;
use super::argument_common::ArgumentCommonBuilder;
use super::errors::OptionExt;
use super::errors::USER_ERROR;

pub struct IntegerArgument {
  common: ArgumentCommon,
}

impl IntegerArgument {
  pub fn new(args: &mut VecDeque<String>) -> Self {
    let mut common = ArgumentCommon::new_builder();
    match common.parse_arguments(args) {
      None => { }
      Some(other) => {
        args.push_front(other);
      }
    }

    return IntegerArgument {
      common: common.build(),
    };
  }
}

impl Argument for IntegerArgument {
  fn get_common(&self) -> &ArgumentCommon {
    &self.common
  }

  fn get_debug_info(&self) -> String {
    return format!("type: Integer; {}", self.common.get_debug_info());
  }

  fn consume(&self, arg: Option<String>, other_args: &mut VecDeque<String>) -> Option<String> {
    self.consume_with_parser(
      arg,
      other_args,
      |name, value: &String| value
          .parse::<i64>()
          .unwrap_or_error(USER_ERROR, format!("Non-integer value '{value}' provided for argument {name}"))
          .to_string())
  }
}

