use std::env;

mod arguments;

fn main() {
  arguments::handle_all_arguments(env::args().collect());
}

