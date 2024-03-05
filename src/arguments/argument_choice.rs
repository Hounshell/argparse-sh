use std::collections::VecDeque;

use super::argument::Argument;
use super::argument::HelpDetailSection;
use super::argument_common::ArgumentCommon;
use super::argument_common::ArgumentCommonBuilder;
use super::argument_common::MatchResult;
use super::errors::error;
use super::errors::OptionExt;
use super::errors::DEFINITION_ERROR;
use super::errors::USER_ERROR;

pub struct ChoiceArgument {
  common: ArgumentCommon,
  all_options: Vec<(String, OptionType)>,
}

#[derive(Clone)]
enum OptionType {
  Mapping(String),
  Actual(Option<String>),
}

impl ChoiceArgument {
  pub fn new(args: &mut VecDeque<String>) -> Self {
    let mut common = ArgumentCommon::new_builder();
    let mut all_options = Vec::new();

    loop {
      match common.parse_arguments(args).as_deref() {
        None => { break; }
        Some("--map") => {
          let from = args.pop_front()
              .unwrap_or_error(DEFINITION_ERROR, String::from("pair of values ({from} {to}) must be provided after --map"))
              .to_string();
          let to = args.pop_front()
              .unwrap_or_error(DEFINITION_ERROR, String::from("pair of values ({from} {to}) must be provided after --map"))
              .to_string();
          all_options.push((from, OptionType::Mapping(to)));
        }
        Some("--option") => {
          let from = args.pop_front()
              .unwrap_or_error(DEFINITION_ERROR, String::from("option must be provided after --option"))
              .to_string();
          let description = args.pop_front();
          if description.is_none() {
            all_options.push((from, OptionType::Actual(None)));
          } else if description.clone().unwrap().starts_with("-") {
            args.push_front(description.unwrap());
            all_options.push((from, OptionType::Actual(None)));
          } else {
            all_options.push((from, OptionType::Actual(Some(description.unwrap()))));
          }
        }
        Some(other) => {
          args.push_front(other.to_string());
          break;
        }
      }
    }

    return ChoiceArgument {
      common: common.build(),
      all_options: all_options,
    };
  }
}

impl Argument for ChoiceArgument {
  fn get_common(&self) -> &ArgumentCommon {
    &self.common
  }

  fn get_debug_info(&self) -> String {
    let mut description = format!("type: Choice; {}", self.common.get_debug_info());
    let mut first = true;
    description.push_str("; options: ");
    for (from, info) in &self.all_options {
      if first {
        first = false;
      } else {
        description.push_str(", ");
      }

      description.push_str(from);

      match info {
        OptionType::Mapping(to) => {
          description.push_str(" -> ");
          description.push_str(to);
        },
        _ => {}
      }
    }
    return description;
  }

  fn get_help_details(&self) -> Vec<HelpDetailSection> {
    let mut lines = vec![
        HelpDetailSection::Text(self.get_description().clone().unwrap_or(String::from("No details available."))),
        HelpDetailSection::Text(String::from("The possible options are:")),
    ];

    for (option, info) in &self.all_options {
      lines.push(HelpDetailSection::ListItem(format!("{} - {}", option,
          match info {
            OptionType::Actual(description) => description.clone().unwrap_or(String::from("No details available.")),
            OptionType::Mapping(actual) => format!("Identical to '{actual}'"),
          })));
    }

    lines
  }

  fn consume(&self, arg: Option<String>, other_args: &mut VecDeque<String>) -> Option<String> {
    let value = match self.common.check_flag_match(arg) {
      MatchResult::NoMatch => return None,
      MatchResult::MatchWithValue(_flag, value) => value,
      MatchResult::MatchWithoutValue => other_args.pop_front()
            .unwrap_or_error(USER_ERROR, format!("No value provided for argument {}", self.get_name()))
    };

    for (option, info) in &self.all_options {
      if option == &value {
        return match info {
          OptionType::Actual(_) => Some(value.clone()),
          OptionType::Mapping(actual) => Some(actual.clone()),
        }
      }
    }

    error(USER_ERROR, format!("Value \"{value}\" not recognized for argument {}", self.get_name()));
    panic!("");
  }
}

