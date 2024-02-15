use std::env;

fn main() {
  arguments::handle_all_arguments(env::args().collect());
}

mod arguments {
  use regex::Regex;
  use std::collections::HashMap;
  use std::collections::VecDeque;

  const HELP_ERROR: i32 = 1;
  const DEFINITION_ERROR: i32 = 2;
  const USER_ERROR: i32 = 3;

  trait Argument {
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
        MatchResult::MatchWithValue(value) => Some(parser(self.get_name(), &value)),
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
          .all_flags
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
      &self.get_common().name
    }

    fn get_description(&self) -> &Option<String> {
      &self.get_common().description
    }

    fn get_default(&self) -> &Option<String> {
      &self.get_common().default
    }

    fn is_secret(&self) -> bool {
      self.get_common().secret
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

  struct ArgumentCommonBuilder {
    name: Option<String>,
    all_flags: Vec<String>,
    default: Option<String>,
    description: Option<String>,
    required: bool,
    secret: bool,
    repeated: bool,
    catch_all: bool,
  }

  impl ArgumentCommonBuilder {
    fn parse_arguments(&mut self, args: &mut VecDeque<String>) -> Option<String> {
      loop {
        match args.pop_front().as_deref() {
          None => { return None; },
          Some("--required") => { self.required = true; },
          Some("--secret") => { self.secret = true; },
          Some("--repeated") | Some("--repeat") => { self.repeated = true; },
          Some("--catch-all") => { self.catch_all = true; },
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

    fn build(self) -> ArgumentCommon {
      let mut name = self.name;
      if name.is_none() {
        name = Some(fix_name(self.all_flags.get(0)
            .cloned()
            .unwrap_or_error(DEFINITION_ERROR, String::from("no name or flags provided for argument"))));
      }
      let name = name.unwrap();

      if self.all_flags.is_empty() && !self.catch_all {
        error(DEFINITION_ERROR, format!("{name} argument can not be set - no flags and not a catch-all argument"))
      }

      ArgumentCommon {
        name: name,
        all_flags: self.all_flags,
        default: self.default,
        description: self.description,
        required: self.required,
        secret: self.secret,
        repeated: self.repeated,
        catch_all: self.catch_all,
      }
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
    fn new_builder() -> ArgumentCommonBuilder {
      ArgumentCommonBuilder {
        name: None,
        all_flags: Vec::new(),
        default: None,
        description: None,
        required: false,
        secret: false,
        repeated: false,
        catch_all: false,
      }
    }

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

    fn check_flag_match(&self, flag: Option<String>) -> MatchResult {
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
                return MatchResult::MatchWithValue(value.to_string());
              }
          }
        }
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
    all_options: Vec<(String, OptionType)>,
  }

  #[derive(Clone)]
  enum OptionType {
    Mapping(String),
    Actual(Option<String>),
  }

  trait OptionExt<T> {
    fn unwrap_or_error(self, exit_code: i32, message: String) -> T;
  }

  impl<T> OptionExt<T> for Option<T> {
    fn unwrap_or_error(self, exit_code: i32, message: String) -> T {
      if self.is_none() {
        error(exit_code, message);
      }
      return self.unwrap();
    }
  }

  impl<T, E: std::fmt::Debug> OptionExt<T> for Result<T, E> {
    fn unwrap_or_error(self, exit_code: i32, message: String) -> T {
      if self.is_err() {
        error(exit_code, message);
      }
      return self.unwrap();
    }
  }


  impl BooleanArgument {
    fn new(args: &mut VecDeque<String>) -> Self {
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
          .all_flags
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
          .unwrap_or_error(USER_ERROR, format!("Non-boolean value \"{value}\" provided for argument {}", self.get_name()))
          .to_string()),
      }
    }
  }

  impl IntegerArgument {
    fn new(args: &mut VecDeque<String>) -> Self {
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
            .unwrap_or_error(USER_ERROR, format!("Non-integer value \"{value}\" provided for argument {name}"))
            .to_string())
    }
  }

  impl FloatArgument {
    fn new(args: &mut VecDeque<String>) -> Self {
      let mut common = ArgumentCommon::new_builder();
      match common.parse_arguments(args) {
        None => { }
        Some(other) => {
          args.push_front(other);
        }
      }

      return FloatArgument {
        common: common.build(),
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

    fn consume(&self, arg: Option<String>, other_args: &mut VecDeque<String>) -> Option<String> {
      self.consume_with_parser(
        arg,
        other_args,
        |name, value: &String| value
            .parse::<f64>()
            .unwrap_or_error(USER_ERROR, format!("Non-numeric value \"{value}\" provided for argument {name}"))
            .to_string())
    }
  }


  impl StringArgument {
    fn new(args: &mut VecDeque<String>) -> Self {
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

  impl ChoiceArgument {
    fn new(args: &mut VecDeque<String>) -> Self {
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

    fn get_help_details(&self) -> Vec<String> {
      let mut lines = vec![
          self.get_description().clone().unwrap_or(String::from("No details available.")),
          String::from("The possible options are:"),
      ];

      for (option, info) in &self.all_options {
        lines.push(format!("•   {} - {}", option,
            match info {
              OptionType::Actual(description) => description.clone().unwrap_or(String::from("No details available.")),
              OptionType::Mapping(actual) => format!("Identical to '{actual}'"),
            }));
      }

      lines
    }

    fn consume(&self, arg: Option<String>, other_args: &mut VecDeque<String>) -> Option<String> {
      let value = match self.common.check_flag_match(arg) {
        MatchResult::NoMatch => return None,
        MatchResult::MatchWithValue(value) => value,
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

  struct Settings {
    arguments: Vec<Box<dyn Argument>>,
    prefix: Option<String>,
    auto_help: bool,
    export: bool,
    debug: bool,
    program_name: Option<String>,
    program_summary: Option<String>,
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
    let mut program_summary = None;
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
              .unwrap_or_error(DEFINITION_ERROR, String::from("program name prefix must be provided after --program-name"))
              .to_string());
        }
        Some("--program-summary") => {
          program_summary = Some(args.pop_front()
              .unwrap_or_error(DEFINITION_ERROR, String::from("program summary prefix must be provided after --program-summary"))
              .to_string());
        }
        Some("--program-description") => {
          program_description = Some(args.pop_front()
              .unwrap_or_error(DEFINITION_ERROR, String::from("program description prefix must be provided after --program-description"))
              .to_string());
        }
        Some("--export") => {
          export = true;
        }
        Some("--prefix") => {
          prefix = Some(args.pop_front()
              .unwrap_or_error(DEFINITION_ERROR, String::from("argument name prefix must be provided after --prefix"))
              .to_string());
        }
        Some("--debug") => {
          debug = true;
        }
        Some(other) => {
          error(DEFINITION_ERROR, format!("Unrecognized option: {other}"));
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
      program_summary: program_summary,
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

  fn parse_argument_values(settings: &Settings) -> HashMap<String, Vec<String>> {
    let mut args = VecDeque::from(settings.remaining_args.clone());

    output_debug(settings, "");
    output_debug(settings, "Parsing argument values");
    output_debug(settings, "");

    let mut result = HashMap::new();

    while !args.is_empty() {
      let arg = args.pop_front().unwrap();
      let (name, value) = parse_argument_value(&settings, &arg, &mut args, &result);

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
      known_values: &HashMap<String, Vec<String>>,
  ) -> (String, String) {
    // First pass handles `--arg value` and `--arg=value` cases.
    for argument in settings.arguments.iter() {
      match argument.consume(Some(first.clone()), rest) {
        None => {}
        Some(value) => {
          let name = argument.get_name().to_string();
          output_debug(settings, format!("Parsed argument {name} = '{value}'"));
          return (name, value);
        }
      }
    }

    // Second pass handles catch-all cases.
    for argument in settings.arguments.iter() {
      let name = argument.get_name().to_string();
      if argument.is_catch_all() && (argument.is_repeated() || !known_values.contains_key(&name)) {
        let value = argument.consume(None, &mut VecDeque::from(vec![first.clone()])).unwrap();
        output_debug(settings, format!("Parsed argument {name} = '{value}'"));
        return (name, value);
      }
    }

    error(USER_ERROR, format!("Extra argument \"{first}\" passed and no catch-all argument found"));
    panic!("");
  }

  fn validate_argument_values(settings: &Settings, arg_values: &HashMap<String, Vec<String>>) {
    output_debug(settings, "");

    for argument in settings.arguments.iter() {
      let values = arg_values.get(argument.get_name());
      if values.is_some() {
        let values = values.unwrap();
        if !argument.is_repeated() && values.len() > 1 {
          error(USER_ERROR, format!("Multiple values found for argument {}", argument.get_name()));
        }
      } else if argument.is_required() {
        error(USER_ERROR, format!("Value for argument {} is missing", argument.get_name()));
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
      } else if argument.get_default().is_some() {
        output_argument(settings, argument.get_name(), argument.get_default().clone().unwrap());
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

    println!("HELP_PAGER=\"${{PAGER:-\"less -R\"}}\"");
    println!("HELP_TEXT=\"");

    if settings.program_name.is_some() && settings.program_summary.is_some() {
      println!("${{bold}}NAME${{unbold}}");
      println!("       {} - {}", settings.program_name.clone().unwrap(), settings.program_summary.clone().unwrap());
      println!("");
    } else if settings.program_name.is_some() {
      println!("${{bold}}NAME${{unbold}}");
      println!("       {}", settings.program_name.clone().unwrap());
      println!("");
    } else if settings.program_summary.is_some() {
      println!("${{bold}}SUMMARY${{unbold}}");
      println!("       {}", settings.program_summary.clone().unwrap());
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
        if !arg.is_secret() {
          println!("       {}", arg.get_help_flags().join(", "));

          for detail in arg.get_help_details() {
            println!("           {detail}\n");
          }

          match arg.get_help_default() {
            None => {},
            Some(text) => { println!("           {text}\n"); }
          }
        }
      }
    }

    println!("\"");
    println!("echo \"$HELP_TEXT\" | $HELP_PAGER");
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

    if settings.remaining_args.len() == 1 && settings.remaining_args.get(0) == Some(&String::from("--help")) {
      print_help_text(&settings);
      println!("( exit {HELP_ERROR} )");
      std::process::exit(HELP_ERROR);

    } else {
      let values = parse_argument_values(&settings);

      validate_argument_values(&settings, &values);
      output_argument_settings(&settings, &values);
    }
  }

  fn error<S: AsRef<str>>(exit_code: i32, message: S) {
    println!("echo \"\"");
    println!("echo \"!!! ArgParse Error: {} !!!\"", message.as_ref());
    println!("echo \"\"");
    println!("( exit {exit_code} )");
    std::process::exit(exit_code);
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
}

