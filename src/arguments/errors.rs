pub const HELP_ERROR: i32 = 1;
pub const DEFINITION_ERROR: i32 = 2;
pub const USER_ERROR: i32 = 3;

pub fn error<S: AsRef<str>>(exit_code: i32, message: S) {
  println!("echo \"\"");
  println!("echo \"!!! ArgParse-sh Error: {} !!!\"", message.as_ref());
  println!("echo \"\"");
  println!("( exit {exit_code} )");
  std::process::exit(exit_code);
}

pub trait OptionExt<T> {
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

