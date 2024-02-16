use regex::Regex;
use std::collections::VecDeque;

use crate::arguments::errors::error;
use crate::arguments::errors::OptionExt;
use crate::arguments::errors::DEFINITION_ERROR;

struct ArgumentCommonBuilderData {
  name: Option<String>,
  all_flags: Vec<String>,
  default: Option<String>,
  description: Option<String>,
  required: bool,
  secret: bool,
  repeated: bool,
  ordinals: Vec<u16>,
  catch_all: bool,
}

pub trait ArgumentCommonBuilder {
  fn parse_arguments(&mut self, args: &mut VecDeque<String>) -> Option<String>;
  fn add_flag(&mut self, flag: String);
  fn build(self) -> ArgumentCommon;
}

impl ArgumentCommonBuilder for ArgumentCommonBuilderData { 
  fn parse_arguments(&mut self, args: &mut VecDeque<String>) -> Option<String> {
    loop {
      match args.pop_front().as_deref() {
        None => { return None; },
        Some("--required") => { self.required = true; },
        Some("--secret") => { self.secret = true; },
        Some("--repeated") | Some("--repeat") => { self.repeated = true; },
        Some("--catch-all") => { self.catch_all = true; },
        Some("--ordinal") | Some("--order") | Some("--ord") => {
            self.ordinals.push(args.pop_front() 
              .unwrap_or_error(DEFINITION_ERROR, String::from("ordinal position must be provided after --ordinal or --order or --ord"))
              .to_string()
              .parse::<u16>()
              .unwrap_or_error(DEFINITION_ERROR, String::from("ordinal position must be an integer between 0 and 65,535")));
        }
        Some("--name") => {
            self.name = Some(args.pop_front()
              .unwrap_or_error(DEFINITION_ERROR, String::from("name must be provided after --name"))
              .to_string());
          },
        Some("--default") => {
            self.default = Some(args.pop_front()
              .unwrap_or_error(DEFINITION_ERROR, String::from("default value must be provided after --default"))
              .to_string());
          },
        Some("--description") | Some("--desc") => {
            self.description = Some(args.pop_front()
              .unwrap_or_error(DEFINITION_ERROR, String::from("description must be provided after --desc or --description"))
              .to_string());
          },
        Some("--flag") => {
            self.all_flags.push(args.pop_front()
              .unwrap_or_error(DEFINITION_ERROR, String::from("flag name must be provided after --flag"))
              .to_string());
          },
        Some(other) => {
          if other.starts_with("-") {
            return Some(other.to_string());
          } else {
            self.all_flags.push(format!("--{other}"));
          }
        },
      }
    }
  }

  fn add_flag(&mut self, flag: String) {
    self.all_flags.push(flag);
  }

  fn build(self) -> ArgumentCommon {
    let mut name = self.name;
    if name.is_none() {
      name = Some(fix_name(self.all_flags.get(0)
          .cloned()
          .unwrap_or_error(DEFINITION_ERROR, String::from("no name or flags provided for argument"))));
    }
    let name = name.unwrap();

    if self.all_flags.is_empty() && !self.catch_all && self.ordinals.is_empty() {
      error(DEFINITION_ERROR, format!("{name} argument can not be set - no flags, no ordinal, and not a catch-all argument"))
    }

    ArgumentCommon {
      name: name,
      all_flags: self.all_flags,
      default: self.default,
      description: self.description,
      required: self.required,
      secret: self.secret,
      repeated: self.repeated,
      ordinals: self.ordinals,
      catch_all: self.catch_all,
    }
  }
}

pub struct ArgumentCommon {
  name: String,
  all_flags: Vec<String>,
  default: Option<String>,
  description: Option<String>,
  required: bool,
  secret: bool,
  repeated: bool,
  ordinals: Vec<u16>,
  catch_all: bool,
}

impl ArgumentCommon {
  pub fn get_name(&self) -> &String { &self.name }
  pub fn get_all_flags(&self) -> &Vec<String> { &self.all_flags }
  pub fn get_default(&self) -> &Option<String> { &self.default }
  pub fn get_description(&self) -> &Option<String> { &self.description }
  pub fn get_required(&self) -> bool { self.required }
  pub fn get_secret(&self) -> bool { self.secret }
  pub fn get_repeated(&self) -> bool { self.repeated }
  pub fn get_ordinals(&self) -> &Vec<u16> { &self.ordinals }
  pub fn get_catch_all(&self) -> bool { self.catch_all }

  pub fn new_builder() -> impl ArgumentCommonBuilder {
    ArgumentCommonBuilderData {
      name: None,
      all_flags: Vec::new(),
      default: None,
      description: None,
      required: false,
      secret: false,
      repeated: false,
      ordinals: Vec::new(),
      catch_all: false,
    }
  }

  pub fn get_debug_info(&self) -> String {
    let mut description = format!("name: {}", self.name);
    description.push_str("; flags: ");
    description.push_str(&self.all_flags.join(", "));
    if self.required {
      description.push_str("; required");
    }
    if self.repeated {
      description.push_str("; repeated");
    }
    if self.secret {
      description.push_str("; secret");
    }
    if self.catch_all {
      description.push_str("; catch-all");
    }
    if self.default.is_some() {
      description.push_str("; default: ");
      description.push_str(&self.default.as_ref().unwrap());
    }

    if self.description.is_some() {
      description.push_str("; description: ");
      description.push_str(&self.description.as_ref().unwrap());
    }

    return description;
  }

  pub fn check_flag_match(&self, flag: Option<String>) -> MatchResult {
    match flag {
      None => { return MatchResult::MatchWithoutValue; },
      Some(flag) => {
        match &flag.to_string().split_once("=") {
          None =>
            if self.all_flags.contains(&flag.to_string()) {
              return MatchResult::MatchWithoutValue;
            },

          Some((name, value)) =>
            if self.all_flags.contains(&name.to_string()) {
              return MatchResult::MatchWithValue(name.to_string(), value.to_string());
            }
        }
      }
    }

    return MatchResult::NoMatch;
  }
}

pub enum MatchResult {
  MatchWithValue(String, String),
  MatchWithoutValue,
  NoMatch,
}

fn fix_name(name: String) -> String {
  Regex::new(r"[a-zA-Z0-9]+")
      .unwrap()
      .find_iter(name.as_str())
      .map(|m| m.as_str())
      .collect::<Vec<&str>>()
      .join("_")
      .to_uppercase()
}

