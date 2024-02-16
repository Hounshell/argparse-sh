use std::collections::VecDeque;

use super::argument::Argument;
use super::argument_common::ArgumentCommon;
use super::argument_common::ArgumentCommonBuilder;

pub struct StringArgument {
  common: ArgumentCommon,
}

impl StringArgument {
  pub fn new(args: &mut VecDeque<String>) -> Self {
    let mut common = ArgumentCommon::new_builder();
    match common.parse_arguments(args) {
      None => { }
      Some(other) => {
        args.push_front(other);
      }
    }

    return StringArgument {
      common: common.build(),
    };
  }
}

impl Argument for StringArgument {
  fn get_common(&self) -> &ArgumentCommon {
    &self.common
  }

  fn get_debug_info(&self) -> String {
    return format!("type: String; {}", self.common.get_debug_info());
  }

  fn consume(&self, arg: Option<String>, other_args: &mut VecDeque<String>) -> Option<String> {
    self.consume_with_parser(
      arg,
      other_args,
      |_name, value: &String| value.clone())
  }
}

