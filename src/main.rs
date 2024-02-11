/*
    secret: bool,
  --bool is_sunny \
  --bool is_rainy rainy r \
  --bool is_snowy snowy s --default true \
  --bool tornados twisters --map true yes --map false no --map "" maybe \
  --int temperature temp t \
  --int wind_speed wind w --default 0 \
  --float rainfall rain --desc "How much rain is expected to fall" \
  --choice units unit u --options imperial metric --default metric \
  --autohelp \
  --error_code 7 \
  --export \
  -- "$@"
*/

use std::env;

fn main() {
  arguments::handle_all_arguments(env::args().collect());
}

mod arguments {
  use regex::Regex;
  use std::collections::HashMap;
  use std::collections::VecDeque;

  trait Argument {
    fn get_debug_info(&self) -> String;
    fn get_common(&self) -> &ArgumentCommon;
    fn consume(&self, arg: &String, other_args: &mut VecDeque<String>) -> Option<String>;

    fn consume_with_parser(
        &self, 
        arg: &String, 
        other_args: &mut VecDeque<String>, 
        parse: fn(&String, &String) -> String) -> Option<String> {
      fn parse2(arg: &FloatArgument, value: &String) -> String {
        value
            .parse::<f64>()
            .unwrap_or_error(format!("Non-numeric value \"{value}\" provided for argument {}", arg.get_name()))
            .to_string()
      }

      match self.get_common().check_flag_match(arg) {
        MatchResult::NoMatch => None,
        MatchResult::MatchWithValue(value) => Some(parse(self.get_name(), &value)),
        MatchResult::MatchWithoutValue => Some(parse(
            self.get_name(),
            &other_args.pop_front()
              .unwrap_or_error(format!("No value provided for argument {}", self.get_name()))))
      }
    }


    fn get_options(&self) -> Option<HashMap<String, OptionType>> {
      None
    }

    fn get_help_suggestions(&self) -> String {
      let mut result = String::from("");
      for flag in self.get_common().all_flags.iter() {
        result += format!(
            "{}--{} <{}>",
            if result.is_empty() { "" } else { ", " },
            flag,
            self.get_common().name.to_lowercase()).as_str();
      }
      return result;
    }

    fn get_name(&self) -> &String {
      &self.get_common().name
    }

    fn is_repeated(&self) -> bool {
      self.get_common().repeated
    }

    fn is_required(&self) -> bool {
      self.get_common().required
    }

    fn is_catch_all(&self) -> bool {
      self.get_common().catch_all
    }
  }

  struct ArgumentCommon {
    name: String,
    all_flags: Vec<String>,
    default: Option<String>,
    description: Option<String>,
    required: bool,
    secret: bool,
    repeated: bool,
    catch_all: bool,
  }

  impl ArgumentCommon {
    fn get_debug_info(&self) -> String {
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

    fn check_flag_match(&self, flag: &String) -> MatchResult {
      if flag.starts_with("--") {
        let flag = &flag[2..];

        match &flag.to_string().split_once("=") {
          None =>
            if self.all_flags.contains(&flag.to_string()) {
              return MatchResult::MatchWithoutValue;
            },

          Some((name, value)) =>
            if self.all_flags.contains(&name.to_string()) {
              return MatchResult::MatchWithValue(value.to_string());
            }
        }
      } else if self.catch_all {
        return MatchResult::MatchWithValue(flag.to_string());
      }

      return MatchResult::NoMatch;
    }
  }

  enum MatchResult {
    MatchWithValue(String),
    MatchWithoutValue,
    NoMatch,
  }

  struct BooleanArgument {
    common: ArgumentCommon,
  }

  struct IntegerArgument {
    common: ArgumentCommon,
  }

  struct FloatArgument {
    common: ArgumentCommon,
  }

  struct StringArgument {
    common: ArgumentCommon,
  }

  struct ChoiceArgument {
    common: ArgumentCommon,
    all_options: HashMap<String, OptionType>,
  }

  #[derive(Clone)]
  enum OptionType {
    Mapping(String),
    Actual(Option<String>),
  }

  trait OptionExt<T> {
    fn unwrap_or_error(self, message: String) -> T;
  }

  impl<T> OptionExt<T> for Option<T> {
    fn unwrap_or_error(self, message: String) -> T {
      if self.is_none() {
        error(message);
      }
      return self.unwrap();
    }
  }

  impl<T, E: std::fmt::Debug> OptionExt<T> for Result<T, E> {
    fn unwrap_or_error(self, message: String) -> T {
      if self.is_err() {
        error(message);
      }
      return self.unwrap();
    }
  }


  impl BooleanArgument {
    fn new(args: &mut VecDeque<String>) -> Self {
      let mut name = None;
      let mut collecting_flags = true;
      let mut all_flags = Vec::new();
      let mut description = None;
      let mut secret = false;

      loop {
        match args.pop_front().as_deref() {
          None => {
            break;
          }
          Some("--name") => {
            name = Some(args.pop_front()
                .unwrap_or_error(String::from("name must be provided after --name"))
                .to_string());
          }
          Some("--secret") => {
            collecting_flags = false;
            secret = true;
          }
          Some("--desc") | Some("--description") => {
            collecting_flags = false;
            description = Some(args.pop_front()
                .unwrap_or_error(String::from("description must be provided after --desc or --description"))
                .to_string());
          }
          Some(other) => {
            if collecting_flags && !other.starts_with("-") {
              all_flags.push(other.to_string());
              if name.is_none() {
                name = Some(other.to_string());
              }
            } else {
              args.push_front(other.to_string());
              break;
            }
          }
        }
      }

      return BooleanArgument {
        common: ArgumentCommon {
          name: fix_name(name.unwrap()),
          all_flags: all_flags,
          default: None,
          description: description,
          required: false,
          secret: secret,
          repeated: false,
          catch_all: false,
        },
      };
    }
  }

  impl Argument for BooleanArgument {
    fn get_help_suggestions(&self) -> String {
      let mut result = String::from("");
      for flag in self.common.all_flags.iter() {
        result += format!("{}--{}", if result.is_empty() { "" } else { ", " }, flag).as_str();
      }
      return result;
    }

    fn get_common(&self) -> &ArgumentCommon {
      &self.common
    }

    fn get_debug_info(&self) -> String {
      return format!("type: Boolean; {}", self.common.get_debug_info());
    }

    fn consume(&self, arg: &String, _other_args: &mut VecDeque<String>) -> Option<String> {
      match self.common.check_flag_match(arg) {
        MatchResult::NoMatch => None,
        MatchResult::MatchWithoutValue => Some(String::from("true")),
        MatchResult::MatchWithValue(value) => Some(value
            .parse::<bool>()
            .unwrap_or_error(format!("Non-boolean value \"{value}\" provided for argument {}", self.common.name))
            .to_string()),
      }
    }
  }

  impl IntegerArgument {
    fn new(args: &mut VecDeque<String>) -> Self {
      let mut name = None;
      let mut collecting_flags = true;
      let mut all_flags = Vec::new();
      let mut default = None;
      let mut description = None;
      let mut required = false;
      let mut secret = false;
      let mut repeated = false;
      let mut catch_all = false;

      loop {
        match args.pop_front().as_deref() {
          None => {
            break;
          }
          Some("--name") => {
            name = Some(args.pop_front()
                .unwrap_or_error(String::from("name must be provided after --name"))
                .to_string());
          }
          Some("--required") => {
            collecting_flags = false;
            required = true;
          }
          Some("--secret") => {
            collecting_flags = false;
            secret = true;
          }
          Some("--repeated") | Some("--repeat") => {
            collecting_flags = false;
            repeated = true;
          }
          Some("--catch-all") => {
            collecting_flags = false;
            catch_all = true;
          }
          Some("--default") => {
            collecting_flags = false;
            default = Some(args.pop_front()
                .unwrap_or_error(String::from("default value must be provided after --default"))
                .to_string());
          }
          Some("--desc") | Some("--description") => {
            collecting_flags = false;
            description = Some(args.pop_front()
                .unwrap_or_error(String::from("description must be provided after --desc or --description"))
                .to_string());
          }
          Some(other) => {
            if collecting_flags && !other.starts_with("-") {
              all_flags.push(other.to_string());
              if name.is_none() {
                name = Some(other.to_string());
              }
            } else {
              args.push_front(other.to_string());
              break;
            }
          }
        }
      }

      return IntegerArgument {
        common: ArgumentCommon {
          name: fix_name(name.unwrap()),
          all_flags: all_flags,
          default: default,
          description: description,
          required: required,
          secret: secret,
          repeated: repeated,
          catch_all: catch_all,
        }
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

    fn consume(&self, arg: &String, other_args: &mut VecDeque<String>) -> Option<String> {
      self.consume_with_parser(
        arg, 
        other_args, 
        |name, value: &String| value
            .parse::<i64>()
            .unwrap_or_error(format!("Non-integer value \"{value}\" provided for argument {name}"))
            .to_string())
    }
  }

  impl FloatArgument {
    fn new(args: &mut VecDeque<String>) -> Self {
      let mut name = None;
      let mut collecting_flags = true;
      let mut all_flags = Vec::new();
      let mut default = None;
      let mut description = None;
      let mut required = false;
      let mut secret = false;
      let mut repeated = false;
      let mut catch_all = false;

      loop {
        match args.pop_front().as_deref() {
          None => {
            break;
          }
          Some("--name") => {
            name = Some(args.pop_front()
                .unwrap_or_error(String::from("name must be provided after --name"))
                .to_string());
          }
          Some("--required") => {
            collecting_flags = false;
            required = true;
          }
          Some("--secret") => {
            collecting_flags = false;
            secret = true;
          }
          Some("--repeated") | Some("--repeat") => {
            collecting_flags = false;
            repeated = true;
          }
          Some("--catch-all") => {
            collecting_flags = false;
            catch_all = true;
          }
          Some("--default") => {
            collecting_flags = false;
            default = Some(args.pop_front()
                .unwrap_or_error(String::from("default value must be provided after --default"))
                .to_string());
          }
          Some("--desc") | Some("--description") => {
            collecting_flags = false;
            description = Some(args.pop_front()
                .unwrap_or_error(String::from("description must be provided after --desc or --description"))
                .to_string());
          }
          Some(other) => {
            if collecting_flags && !other.starts_with("-") {
              all_flags.push(other.to_string());
              if name.is_none() {
                name = Some(other.to_string());
              }
            } else {
              args.push_front(other.to_string());
              break;
            }
          }
        }
      }

      return FloatArgument {
        common: ArgumentCommon {
          name: fix_name(name.unwrap()),
          all_flags: all_flags,
          default: default,
          description: description,
          required: required,
          secret: secret,
          repeated: repeated,
          catch_all: catch_all,
        }
      };
    }
  }

  impl Argument for FloatArgument {
    fn get_common(&self) -> &ArgumentCommon {
      &self.common
    }

    fn get_debug_info(&self) -> String {
      return format!("type: Float; {}", self.common.get_debug_info());
    }

    fn consume(&self, arg: &String, other_args: &mut VecDeque<String>) -> Option<String> {
      self.consume_with_parser(
        arg, 
        other_args, 
        |name, value: &String| value
            .parse::<f64>()
            .unwrap_or_error(format!("Non-numeric value \"{value}\" provided for argument {name}"))
            .to_string())
    }
  }

  impl StringArgument {
    fn new(args: &mut VecDeque<String>) -> Self {
      let mut name = None;
      let mut collecting_flags = true;
      let mut all_flags = Vec::new();
      let mut default = None;
      let mut description = None;
      let mut required = false;
      let mut secret = false;
      let mut repeated = false;
      let mut catch_all = false;

      loop {
        match args.pop_front().as_deref() {
          None => {
            break;
          }
          Some("--name") => {
            name = Some(args.pop_front()
                .unwrap_or_error(String::from("name must be provided after --name"))
                .to_string());
          }
          Some("--required") => {
            collecting_flags = false;
            required = true;
          }
          Some("--secret") => {
            collecting_flags = false;
            secret = true;
          }
          Some("--repeated") | Some("--repeat") => {
            collecting_flags = false;
            repeated = true;
          }
          Some("--catch-all") => {
            collecting_flags = false;
            catch_all = true;
          }
          Some("--default") => {
            collecting_flags = false;
            default = Some(args.pop_front()
                .unwrap_or_error(String::from("default value must be provided after --default"))
                .to_string());
          }
          Some("--desc") | Some("--description") => {
            collecting_flags = false;
            description = Some(args.pop_front()
                .unwrap_or_error(String::from("description must be provided after --desc or --description"))
                .to_string());
          }
          Some(other) => {
            if collecting_flags && !other.starts_with("-") {
              all_flags.push(other.to_string());
              if name.is_none() {
                name = Some(other.to_string());
              }
            } else {
              args.push_front(other.to_string());
              break;
            }
          }
        }
      }

      return StringArgument {
        common: ArgumentCommon {
          name: fix_name(name.unwrap()),
          all_flags: all_flags,
          default: default,
          description: description,
          required: required,
          secret: secret,
          repeated: repeated,
          catch_all: catch_all,
        }
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

    fn consume(&self, arg: &String, other_args: &mut VecDeque<String>) -> Option<String> {
      self.consume_with_parser(
        arg, 
        other_args, 
        |name, value: &String| value.to_string())
    }
  }

  impl ChoiceArgument {
    fn new(args: &mut VecDeque<String>) -> Self {
      let mut name = None;
      let mut collecting_flags = true;
      let mut all_flags = Vec::new();
      let mut all_options = HashMap::new();
      let mut default = None;
      let mut description = None;
      let mut required = false;
      let mut secret = false;
      let mut repeated = false;
      let mut catch_all = false;

      loop {
        match args.pop_front().as_deref() {
          None => {
            break;
          }
          Some("--name") => {
            name = Some(args.pop_front()
                .unwrap_or_error(String::from("name must be provided after --name"))
                .to_string());
          }
          Some("--required") => {
            collecting_flags = false;
            required = true;
          }
          Some("--secret") => {
            collecting_flags = false;
            secret = true;
          }
          Some("--repeated") | Some("--repeat") => {
            collecting_flags = false;
            repeated = true;
          }
          Some("--catch-all") => {
            collecting_flags = false;
            catch_all = true;
          }
          Some("--default") => {
            collecting_flags = false;
            default = Some(args.pop_front()
                .unwrap_or_error(String::from("default value must be provided after --default"))
                .to_string());
          }
          Some("--desc") | Some("--description") => {
            collecting_flags = false;
            description = Some(args.pop_front()
                .unwrap_or_error(String::from("description must be provided after --desc or --description"))
                .to_string());
          }
          Some("--map") => {
            collecting_flags = false;
            let from = args.pop_front()
                .unwrap_or_error(String::from("pair of values ({from} {to}) must be provided after --map"))
                .to_string();
            let to = args.pop_front()
                .unwrap_or_error(String::from("pair of values ({from} {to}) must be provided after --map"))
                .to_string();
            all_options.insert(from, OptionType::Mapping(to));
          }
          Some("--option") => {
            collecting_flags = false;
            let from = args.pop_front()
                .unwrap_or_error(String::from("option must be provided after --option"))
                .to_string();
            let description = args.pop_front();
            if description.is_none() {
              all_options.insert(from, OptionType::Actual(None));
            } else if description.clone().unwrap().starts_with("-") {
              args.push_front(description.unwrap());
              all_options.insert(from, OptionType::Actual(None));
            } else {
              all_options.insert(from, OptionType::Actual(Some(description.unwrap())));
            }
          }
          Some(other) => {
            if collecting_flags && !other.starts_with("-") {
              all_flags.push(other.to_string());
              if name.is_none() {
                name = Some(other.to_string());
              }
            } else {
              args.push_front(other.to_string());
              break;
            }
          }
        }
      }

      return ChoiceArgument {
        common: ArgumentCommon {
          name: fix_name(name.unwrap()),
          all_flags: all_flags,
          default: default,
          description: description,
          required: required,
          secret: secret,
          repeated: repeated,
          catch_all: catch_all,
        },
        all_options: all_options,
      };
    }
  }

  impl Argument for ChoiceArgument {
    fn get_common(&self) -> &ArgumentCommon {
      &self.common
    }

    fn get_options(&self) -> Option<HashMap<String, OptionType>> {
      Some(self.all_options.clone())
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

    fn consume(&self, arg: &String, other_args: &mut VecDeque<String>) -> Option<String> {
      let value = match self.common.check_flag_match(arg) {
        MatchResult::NoMatch => return None,
        MatchResult::MatchWithValue(value) => value,
        MatchResult::MatchWithoutValue => other_args.pop_front()
              .unwrap_or_error(format!("No value provided for argument {}", self.get_name()))
      };

      match self.all_options.get(&value)
            .unwrap_or_error(format!("Value \"{value}\" not recognized for argument {}", self.get_name())) {
        OptionType::Mapping(actual) => Some(actual.to_string()),
        _ => Some(value),
      }
    }
  }

  struct Settings {
    arguments: Vec<Box<dyn Argument>>,
    prefix: Option<String>,
    auto_help: bool,
    export: bool,
    debug: bool,
    program_name: Option<String>,
    program_description: Option<String>,
    remaining_args: Vec<String>,
  }

  fn parse_settings(args: Vec<String>) -> Settings {
    let mut args = VecDeque::from(args);
    args.pop_front();

    let mut arguments: Vec<Box<dyn Argument>> = Vec::new();
    let mut prefix = None;
    let mut auto_help = false;
    let mut export = false;
    let mut debug = false;
    let mut program_name = None;
    let mut program_description = None;

    loop {
      match args.pop_front().as_deref() {
        None | Some("--") => {
          break;
        }
        Some("--boolean") | Some("--bool") => {
          arguments.push(Box::new(BooleanArgument::new(&mut args)));
        }
        Some("--integer") | Some("--int") => {
          arguments.push(Box::new(IntegerArgument::new(&mut args)));
        }
        Some("--float") | Some("--number") => {
          arguments.push(Box::new(FloatArgument::new(&mut args)));
        }
        Some("--string") | Some("--str") => {
          arguments.push(Box::new(StringArgument::new(&mut args)));
        }
        Some("--choice") | Some("--pick") => {
          arguments.push(Box::new(ChoiceArgument::new(&mut args)));
        }
        Some("--autohelp") | Some("--auto-help") => {
          auto_help = true;
        }
        Some("--program-name") => {
          program_name = Some(args.pop_front()
              .unwrap_or_error(String::from("program name prefix must be provided after --program-name"))
              .to_string());
        }
        Some("--program-description") => {
          program_description = Some(args.pop_front()
              .unwrap_or_error(String::from("program description prefix must be provided after --program-description"))
              .to_string());
        }
        Some("--export") => {
          export = true;
        }
        Some("--prefix") => {
          prefix = Some(args.pop_front()
              .unwrap_or_error(String::from("argument name prefix must be provided after --prefix"))
              .to_string());
        }
        Some("--debug") => {
          debug = true;
        }
        Some(other) => {
          error(format!("Unrecognized option: {other}"));
        }
      };
    }

    Settings {
      arguments: arguments,
      prefix: prefix,
      auto_help: auto_help,
      export: export,
      debug: debug,
      program_name: program_name,
      program_description: program_description,
      remaining_args: Vec::from(args),
    }
  }

  fn debug_setup(settings: &Settings) {
    output_debug(settings, "ArgParse debugging enabled with --debug flag");
    output_debug(settings, format!(
        "Arguments {} exported to child processes", 
        if settings.export { "are" } else { "are not" }));

    if settings.prefix.is_some() {
      output_debug(settings, format!("All variables will be prefixed with '{}'", settings.prefix.clone().unwrap()));
    }

    if settings.auto_help {
      output_debug(settings, "Help text will be printed if '--help' is found in arguments");
    }

    output_debug(settings, "");

    for arg in settings.arguments.iter() {
      output_debug(settings, format!("Definition - {}", arg.get_debug_info()));
    }
  }

  fn validate_setup(settings: &Settings) {
    let catch_all_args: Vec<&String> = settings.arguments.iter()
        .filter(|a| a.is_catch_all())
        .map(|a| a.get_name())
        .collect();

    if catch_all_args.len() > 1 {
      error(format!("More than one catch-all argument found: {:?}", catch_all_args));
    }
  }

  fn parse_argument_values(settings: &Settings) -> HashMap<String, Vec<String>> {
    let mut args = VecDeque::from(settings.remaining_args.clone());

    output_debug(settings, "");
    output_debug(settings, "Parsing argument values");
    output_debug(settings, "");

    let mut result = HashMap::new();

    while !args.is_empty() {
      let arg = args.pop_front().unwrap();
      let (name, value) = parse_argument_value(&settings, &arg, &mut args);

      let mut all_values = result.remove(&name).unwrap_or(Vec::new());
      all_values.push(value);
      result.insert(name, all_values);
    }

    return result;
  }

  fn parse_argument_value(
      settings: &Settings,
      first: &String,
      rest: &mut VecDeque<String>,
  ) -> (String, String) {
    for argument in settings.arguments.iter() {
      match argument.consume(first, rest) {
        None => {}
        Some(value) => {
          let name = argument.get_name().to_string();
          output_debug(settings, format!("Parsed argument {name} = '{value}'"));
          return (name, value);
        }
      }
    }

    error(format!("Extra argument \"{first}\" passed and no catch-all argument found"));
    panic!("");
  }

  fn validate_argument_values(settings: &Settings, arg_values: &HashMap<String, Vec<String>>) {
    output_debug(settings, "");

    for argument in settings.arguments.iter() {
      let values = arg_values.get(argument.get_name());
      if values.is_some() {
        let values = values.unwrap();
        if !argument.is_repeated() && values.len() > 1 {
          error(format!("Multiple values found for argument {}", argument.get_name()));
        }
      } else if argument.is_required() {
        error(format!("Value for argument {} is missing", argument.get_name()));
      }
    }
  }

  fn output_argument_settings(settings: &Settings, arg_values: &HashMap<String, Vec<String>>) {
    for argument in settings.arguments.iter() {
      let values = arg_values.get(argument.get_name());
      if values.is_some() {
        let values = values.unwrap();

        if argument.is_repeated() {
          output_argument(settings, argument.get_name(), values.len());
          for i in 0..values.len() {
            output_argument(settings, &format!("{}_{}", argument.get_name(), i), values.get(i).unwrap());
          }
        } else {
          output_argument(settings, argument.get_name(), values.get(0).unwrap());
        }
      } else if argument.get_common().default.is_some() {
        output_argument(settings, argument.get_name(), argument.get_common().default.clone().unwrap());
      }
    }

    output_debug(settings, "");
    output_debug(settings, "ArgParse completed successfully");
  }

  fn print_help_text(settings: &Settings) {
    println!("(");
    println!("if [ -t 1 ]; then");
    println!("  bold=\"$(tput bold)\"");
    println!("  unbold=\"$(tput sgr0)\"");
    println!("else");
    println!("  bold=\"\"");
    println!("  unbold=\"\"");
    println!("fi");
    println!("HELP_TEXT=\"");

    if settings.program_name.is_some() {
      println!("${{bold}}NAME${{unbold}}");
      println!("       {}", settings.program_name.clone().unwrap());
      println!("");
    }

    if settings.program_description.is_some() {
      println!("${{bold}}DESCRIPTION${{unbold}}");
      println!("       {}", settings.program_description.clone().unwrap());
      println!("");
    }

    if !settings.arguments.is_empty() {
      println!("${{bold}}OPTIONS${{unbold}}");

      for arg in settings.arguments.iter() {
        if !arg.get_common().secret {
          println!("       {}", arg.get_help_suggestions());
          if arg.get_common().description.is_some() {
            println!("           {}", arg.get_common().description.clone().unwrap());
          } else {
            println!("           No details available.");
          }
          let options = arg.get_options();
          if options.is_some() {
            println!("");
            println!("           The possible options are:");
            for (option, info) in options.unwrap() {
              println!("");
              match info {
                OptionType::Actual(description) => {
                  println!("           •   {} - {}", option, description.unwrap_or(String::from("No details available.")));
                },
                OptionType::Mapping(actual) => {
                  println!("           •   {} - Identical to '{}'", option, actual);
                },
              }
            }
          }
          if arg.get_common().default.is_some() {
            println!("");
            println!("           When this option is not provided it will default to '{}'.", arg.get_common().default.clone().unwrap());
          }
          println!("");
        }
      }
    }

    println!("\"");
    println!("echo \"$HELP_TEXT\"");
    // println!("echo \"$HELP_TEXT\" | less -R");
    println!(")");
  }

  fn echo<S: AsRef<str>>(text: S) {
    println!("echo \"{}\"", text.as_ref());
  }

  fn output_debug<S: AsRef<str>>(settings: &Settings, text: S) {
    if settings.debug {
      echo(format!("[ArgParse] {}", text.as_ref()));
    }
  }

  fn output_argument<V: std::fmt::Display>(settings: &Settings, name: &String, value: V) {
    output_debug(settings, format!(
        "Setting {}{name} = \\\"{value}\\\"", 
        settings.prefix.clone().unwrap_or(String::from(""))));

    println!("{}{}{name}=\"{value}\"",
        if settings.export { "export " } else { "" },
        settings.prefix.clone().unwrap_or(String::from("")));
  }

  pub fn handle_all_arguments(args: Vec<String>) {
    let settings = parse_settings(args);

    debug_setup(&settings);
    validate_setup(&settings);

    let values = parse_argument_values(&settings);

    validate_argument_values(&settings, &values);
    output_argument_settings(&settings, &values);

    print_help_text(&settings);
  }

  fn error<S: AsRef<str>>(message: S) {
    println!("echo \"\"");
    println!("echo \"!!! ArgParse Error: {} !!!\"", message.as_ref());
    println!("echo \"\"");
    std::process::exit(1);
  }

  fn fix_name(name: String) -> String {
    Regex::new(r"[^a-zA-Z0-9]+")
        .unwrap()
        .replace_all(name.as_str(), "_")
        .to_string()
        .to_uppercase()
  }
}

