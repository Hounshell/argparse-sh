extern crate termsize;

use regex::Regex;
use std::collections::HashMap;
use std::collections::VecDeque;
use textwrap::fill;
use textwrap::Options;
use unicode_width::UnicodeWidthStr;

mod errors;
mod argument;
mod argument_boolean;
mod argument_choice;
mod argument_common;
mod argument_float;
mod argument_integer;
mod argument_string;

use errors::*;

struct Settings {
  arguments: Vec<Box<dyn argument::Argument>>,
  prefix: Option<String>,
  auto_help: bool,
  export: bool,
  debug: bool,
  program_name: Option<String>,
  program_summary: Option<String>,
  program_description: Option<String>,
  remaining_args: Vec<String>,
  columns: usize,
  help_function: Option<String>,
}

fn parse_settings(args: Vec<String>) -> Settings {
  let mut args = VecDeque::from(args);
  args.pop_front();

  let mut arguments: Vec<Box<dyn argument::Argument>> = Vec::new();
  let mut prefix = None;
  let mut auto_help = false;
  let mut export = false;
  let mut debug = false;
  let mut program_name = None;
  let mut program_summary = None;
  let mut program_description = None;
  let mut help_function = None;

  let mut columns = match termsize::get() {
    None => 80_usize,
    Some(size) => size.cols as usize,
  };

  loop {
    match args.pop_front().as_deref() {
      None | Some("--") => {
        break;
      }
      Some("--boolean") | Some("--bool") => {
        arguments.push(Box::new(argument_boolean::BooleanArgument::new(&mut args)));
      }
      Some("--integer") | Some("--int") => {
        arguments.push(Box::new(argument_integer::IntegerArgument::new(&mut args)));
      }
      Some("--float") | Some("--number") => {
        arguments.push(Box::new(argument_float::FloatArgument::new(&mut args)));
      }
      Some("--string") | Some("--str") => {
        arguments.push(Box::new(argument_string::StringArgument::new(&mut args)));
      }
      Some("--choice") | Some("--pick") => {
        arguments.push(Box::new(argument_choice::ChoiceArgument::new(&mut args)));
      }
      Some("--autohelp") | Some("--auto-help") => {
        auto_help = true;
      }
      Some("--help-function") => {
        help_function = Some(args.pop_front()
            .unwrap_or_error(DEFINITION_ERROR, String::from("help function name must be provided after --help-function"))
            .to_string());
      }
      Some("--columns") | Some("--cols") => {
        let value = args.pop_front()
            .unwrap_or_error(DEFINITION_ERROR, String::from("number of columns must be provided after --columns or --cols"));
        columns = value
            .parse::<usize>()
            .unwrap_or_error(DEFINITION_ERROR, format!("Non-numeric value '{value}' provided for number of columns"))
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
    help_function: help_function,
    export: export,
    debug: debug,
    program_name: program_name,
    program_summary: program_summary,
    program_description: program_description,
    remaining_args: Vec::from(args),
    columns: columns
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

  output_debug(settings, format!("Help text will be formatted with {} columns", settings.columns));

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
  let mut ordinal = 0_u16;

  while !args.is_empty() {
    let arg = args.pop_front().unwrap();
    let (name, value, new_ordinal) = parse_argument_value(&settings, ordinal, &arg, &mut args, &result);
    ordinal = new_ordinal;

    let mut all_values = result.remove(&name).unwrap_or(Vec::new());
    all_values.push(value);
    result.insert(name, all_values);
  }

  return result;
}

fn parse_argument_value(
    settings: &Settings,
    ordinal: u16,
    first: &String,
    rest: &mut VecDeque<String>,
    known_values: &HashMap<String, Vec<String>>,
) -> (String, String, u16) {
  // First pass handles flag cases (`--arg value` and `--arg=value`).
  for argument in settings.arguments.iter() {
    match argument.consume(Some(first.clone()), rest) {
      None => {}
      Some(value) => {
        let name = argument.get_name().to_string();
        output_debug(settings, format!("Parsed argument {name} = '{value}' [flag: '{first}']"));
        return (name, value, ordinal);
      }
    }
  }

  // Second pass handles ordinals.
  for argument in settings.arguments.iter() {
    if argument.is_ordinal(ordinal) {
      let name = argument.get_name().to_string();
      let value = argument.consume(None, &mut VecDeque::from(vec![first.clone()])).unwrap();
      output_debug(settings, format!("Parsed argument {name} = '{value}' [ordinal: {ordinal}]"));
      return (name, value, ordinal + 1);
    }
  }

  // Third pass handles catch-all cases.
  for argument in settings.arguments.iter() {
    if argument.is_catch_all() && (argument.is_repeated() || !known_values.contains_key(argument.get_name())) {
      let name = argument.get_name().to_string();
      let value = argument.consume(None, &mut VecDeque::from(vec![first.clone()])).unwrap();
      output_debug(settings, format!("Parsed argument {name} = '{value}' [catch-all]"));
      return (name, value, ordinal + 1);
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

fn cleanup_help_text(text: &Option<String>, options: &Options) -> String {
  let regex = Regex::new(r"(?m)(?P<text>.+?)\s*?(?P<lines>\n+|$)").unwrap();
  let mut result = String::from("");

  for chunk in regex.captures_iter(text.clone().unwrap().as_str()) {
    result.push_str(&chunk["text"]);
    let lines = &chunk["lines"];
    if lines.len() == 1 {
      result.push_str(" ");
    } else {
      result.push_str("\n\n");
    }
  }

  return fill(result.trim_end(), options).to_string();
}

fn print_help_text(settings: &Settings) {
  let shallow_options = Options::new(settings.columns)
      .initial_indent("       ")
      .subsequent_indent("       ");

  let deep_options = Options::new(settings.columns)
      .initial_indent("           ")
      .subsequent_indent("           ");

  let list_options = Options::new(settings.columns)
      .initial_indent("           •   ")
      .subsequent_indent("               ");

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
    println!("{}", cleanup_help_text(
        &Some(format!("{} - {}", settings.program_name.clone().unwrap(), settings.program_summary.clone().unwrap())),
        &shallow_options));
    println!("");
  } else if settings.program_name.is_some() {
    println!("${{bold}}NAME${{unbold}}");
    println!("{}", cleanup_help_text(&settings.program_name, &shallow_options));
    println!("");
  } else if settings.program_summary.is_some() {
    println!("${{bold}}SUMMARY${{unbold}}");
    println!("{}", cleanup_help_text(&settings.program_summary, &shallow_options));
    println!("");
  }

  if settings.program_description.is_some() {
    println!("${{bold}}DESCRIPTION${{unbold}}");
    println!("{}", cleanup_help_text(&settings.program_description, &shallow_options));
    println!("");
  }

  if !settings.arguments.is_empty() {
    println!("${{bold}}OPTIONS${{unbold}}");

    for arg in settings.arguments.iter() {
      if !arg.is_secret() {
        let mut line_so_far = String::from("");
        for (i, flag) in arg.get_help_flags().iter().enumerate() {
          if i == 0 {
            line_so_far = format!("       {flag}");
          } else if UnicodeWidthStr::width(line_so_far.as_str()) + UnicodeWidthStr::width(flag.as_str()) + 4 > settings.columns {
            println!("{line_so_far}, ");
            line_so_far = format!("       {flag}");
          } else {
            line_so_far.push_str(", ");
            line_so_far.push_str(flag);
          }
        }
        println!("{line_so_far}");

        for detail in arg.get_help_details() {
          match detail {
            argument::HelpDetailSection::Text(text) => {
                println!("{}\n", cleanup_help_text(&Some(text), &deep_options));
              },
            argument::HelpDetailSection::ListItem(text) => {
                println!("{}\n", cleanup_help_text(&Some(text), &list_options));
              },
          }
        }

        match arg.get_help_default() {
          None => {},
          Some(text) => { println!("{}\n", cleanup_help_text(&Some(text), &deep_options)); }
        }
      }
    }
  }

  println!("\"");
  println!("echo \"$HELP_TEXT\" | $HELP_PAGER");
  println!(")");
}

fn print_help_function(settings: &Settings) {
  println!("{} () {{", settings.help_function.clone().unwrap());

  print_help_text(settings);

  println!("}}");
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

  if settings.auto_help && settings.remaining_args.len() == 1 && settings.remaining_args.get(0) == Some(&String::from("--help")) {
    print_help_text(&settings);
    println!("( exit {HELP_ERROR} )");
    std::process::exit(HELP_ERROR);

  } else {
    let values = parse_argument_values(&settings);

    validate_argument_values(&settings, &values);
    output_argument_settings(&settings, &values);

    if settings.help_function.is_some() {
      print_help_function(&settings);
    }
  }
}

