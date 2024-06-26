# ArgParse-sh

Utility for parsing arguments to shell scripts and providing the results as environment variables.

# Installation

## Cargo Install

The preferred method of install uses `cargo`, the Rust tool. If you have Rust installed on your
local machine, run:

```sh
cargo install argparse-sh
```

The binary will be built and put in your `~/.cargo/bin/` folder. Add that to your `PATH` or
reference it directly, and you are all set.

## Building From Source

You can download and build the application from source code. This also requires Rust to be
installed on your local machine, as well as Git. To do this, run:

```sh
git clone git@github.com:Hounshell/argparse-sh.git
cd argparse-sh
cargo build --release
```

The compiled binary will be in `target/release/argparse-sh`. Put this wherever you like and add it
to your `PATH` or reference it directly.

# Usage

ArgParse-sh takes two sets of arguments. The first set defines all of the arguments that will be
parsed. The second set is the arguments to parse. This is probably best clarified via a minimal
demonstration:

```sh
$ argparse-sh --string text -- --text "Hello, world!"
TEXT="Hello, world!"
```

What did this do? It defined a set of arguments (one string argument called "text") and then
parsed a set of argument values. The output is a script that sets environment variables for the
arguments provided.

This is a minimal example, but it isn't a very useful one. Instead lets drop this into a script
and pass arguments on the script through to ArgParse-sh:

**demo.sh**

```sh
eval "$(argparse-sh --string text -- "$@")";
echo "$TEXT"
```

Let's break this script up into various pieces:

- `argparse-sh` - This invokes the ArgParse-sh program

- `--string text` - This indicates to ArgParse-sh that we can accept an optional argument called
  "text". The value is a string and there is no default value.

- `--` - This indicates to ArgParse-sh that we are done defining arguments. All remaining command line
  parameters should be treated as argument values.

- `"$@"` - This forwards all of the parameters that the script was called with to ArgParse-sh. Because
  this is after the `--`, ArgParse-sh will interpret these parameters.

- `eval "$(...)";` - The output of ArgParse-sh is script commands. `eval` executes these commands in
  the current shell.

- `echo "$TEXT"` - The script commands that ArgParse-sh will output set environment variables with the
  value of the arguments. This will print the value of whatever was passed in for the "text"
  argument.

If we run `demo.sh` we can see this all work in action:

```sh
$ ./demo.sh --text "Hello, world!"
Hello, world!
```

ArgParse-sh recognizes the `--text` argument and expects a string. That string ("Hello, world!") is
written to an environment variable called "TEXT" (based on the argument's name), which can then be
used by the remainder of the script.

In several examples below we will omit the `eval` part of the argparse-sh command. This will cause the
ArgParse-sh output to dump to the screen, allowing us to see what is happening more clearly.

## Argument Types

There are several supported argument types. Generally they share the same set of options (where it
makes sense for them to do so), and some have additional options or forms.

### Common Argument Parameters

There is a common pattern to defining arguments, as well as a number of common parameters that can
be provided:

#### --\<type> \[\<name / key 1> \<key 2> \<key 3> ...]

To begin defining an argument we need to specify the argument type. After the type we may provide
shorthand for the name and the keys that can be used to provide this argument. This shorthand
covers the most common cases, but you are not required to use it; the name and flags can be defined
individually.

The types available are:

- **--boolean** or **--bool** - A "true" or "false" value.
- **--choice** or **--pick** - One selection from a list of options.
- **--float** or **--number** - A 64 bit floating point number.
- **--integer** or **--int** - A 64 bit signed integer.
- **--string** or **--str** - Free-form text.

##### Example:

```sh
$ argparse-sh --string given-name first-name name -- --first-name "Alice"
GIVEN_NAME="Alice"
```

This creates a single string argument called "GIVEN\_NAME" that can be set using either
`--given-name`, `--first-name`, or `--name`. This is the shorthand form of:

```sh
$ argparse-sh --string --name GIVEN_NAME \
    --flag "--given-name" \
    --flag "--first-name" \
    --flag "--name" \
    -- --first-name "Alice"
GIVEN_NAME="Alice"
```

#### --name \<name>

Provide the name of the environment variable the value for this argument will be stored in.

If the name is not provided, the first flag defined (either using --flag or the shorthand above)
will be normalized and used. Normalization removes any preceeding or trailing non-alphanumeric
characters. All other sequences of non-alphanumeric sequences will be replaced with "\_" and the
entire string will be capitalized. For example, if the first flag is "--first-name" the normalized
name would be "FIRST\_NAME".

##### Example:

```sh
$ argparse-sh --string name --name FIRST_NAME -- --name "Alice"
FIRST_NAME="Alice"
```

This will configure a string argument that can be set using `--name "Name"`, but will be stored in
the `FIRST_NAME` environment variable.

#### --flag \<flag>

Provides a flag that can be used to specify this argument's value. The flag will be used as-is,
with capitalization and hyphens. This is a good way to support shorter flags. You are allowed to
specify a flag name without a hyphen at all.

##### Example:

```sh
$ argparse-sh --string name --flag "--first-name" --flag "-n" -- -n Alice
NAME="Alice"
```

This defines a string argument called "NAME" using the shorthand method, but alternate flags
("--first-name" and "-n") are also defined as available for use.

#### --default \<default>

Provide the default value to use if this argument is not specified. 

**Warning:** This default value is not parsed or validated; invalid values will be passed on
through.

##### Example:

```sh
$ argparse-sh --string name --default "Alice"
NAME="Alice"
```

If `-- --name "Bob"` had been provided then `NAME` would have been set to "Bob" instead of "Alice".

#### --desc\[ription] \<description>

Provide a description to use for this argument when generating help text.

##### Example:

```sh
$ eval "$(argparse-sh --string name --desc "The user's first name." --autohelp -- --help)"

OPTIONS
       --name <name>
           The user's first name.

```

We used `eval` in this example because it's hard to see the output otherwise. Help was generated
for all of the arguments (in this case only `--name` was specified), and the help text was used in
the description of that argument.

#### --repeated

Indicates that this argument may be repeated. If an argument is repeated then the environment
variable will be set to the number of values that exist, and each value will be set as its own
environment variable, with a 0-based suffix for the index.

##### Example:

```sh
$ argparse-sh --string name --repeated -- --name "Alice" --name "Bob" --name "Carol"
NAME="3"
NAME_0="Alice"
NAME_1="Bob"
NAME_2="Carol"
```

Here we can see that three names were supplied. Each value for `--name` was included in order.

#### --required

Indicates that this argument is required. If not provided ArgParse-sh will fail.

##### Example:

```sh
$ argparse-sh --string name --required
echo ""
echo "!!! ArgParse-sh Error: Value for argument NAME is missing !!!"
echo ""

$ echo $?
2
```

An error message is shown indicating that the "name" argument wasn't supplied. The exit code from
ArgParse-sh when there is an error parsing the arguments is 2.

#### --secret

Marks an argument for non-inclusion in generated help text.

##### Example:

```sh
$ eval "$(argparse-sh --string name --secret --string age --autohelp -- --help)"

OPTIONS
       --age <age>
           No details available.

```

Again we use `eval` for clarity. Note that help text is generated for the "age" argument, but not
for the "name" argument.

#### --catch-all

This is used to mark an argument that will be get any unrecognized values. This is particularly
useful for repeated arguments where you don't want to require the user to specify the flag name.

Catch-all arguments do not need to be repeated, and multiple arguments can be marked as catch-all.
If an argument is not repeated it can only catch a single value. If a second unrecognized value is
provided, it will be consumed by the next catch-all argument that is repeated or does not already
have a value assigned to it.

Note that catch-all arguments don't ***require*** any flags, but it is advised that you still
provide some to the user so that they can use the `--flag=value` syntax. If not using any flags
then the argument **must** have a name.

##### Example:

```sh
$ argparse-sh --string name --catch-all -- "Bob"
NAME="Bob"
```

#### --ordinal \<order>

Makes this argument act like a catch-all argument, except it will only take a single value, and
only if a value hasn't been explicitly provided. The `order` is an integer that provides the order
that ordinal arguments are filled. The lowest argument that does not already have a value (e.g. the
user hasn't explicitly provided a value for this argument via a flag) will be used next. This means
that ordinals can start at whatever number you like, and can have gaps between the numbers.

##### Example:

```sh
$ argparse-sh \
    --string first_name --ordinal 1 --required \
    --string middle_name --ordinal 2 --required \
    --string last_name --ordinal 3 --required \
    -- --middle_name "Quincy" "Alice" "Smith"
FIRST_NAME="Alice"
MIDDLE_NAME="Qunicy"
LAST_NAME="Smith"
```

### String Arguments (--string or --str)

String arguments do not perform any validation or re-writing of their values. These are simply
passed through to the environment variable. String arguments support all of the common argument
parameters.

#### Example:

```
$ argparse-sh \
    --string first-name --required \
    --string last-name --default "Doe" \
    --string nickname --repeated --catch-all \
    -- \
    --first-name "John" \
    --nickname "Sticky Fingers" \
    --nickname "Tight Lips"
FIRST_NAME="John"
LAST_NAME="Doe"
NICKNAME="2"
NICKNAME_0="Sticky Fingers"
NICKNAME_1="Tight Lips"
```

### Integer Arguments (--integer or --int)

Integer arguments are validated. The value provided must be parseable as a 64 bit signed integer.
If an invalid argument is provided then argparse-sh will fail with a message and an error code of 2.
Integer arguments support all of the common argument parameters.

**Important:** If a default value is provided it is not validated. You are responsible for ensuring
that the provided value resolves to an integer, or your script is able to handle non-integer values.

#### Example:

```
$ argparse-sh \
    --integer age --required \
    --integer children --default 0 \
    --integer pockets --required --catch-all \
    -- \
    --age 42 \
    7
AGE="42"
CHILDREN="0"
POCKETS="7"
```

### Float Arguments (--float or --number)

Float arguments are also validated. The value provided must be parseable as a 64 bit floating point
number. If an invalid argument is provided then argparse-sh will fail with a message and an error code
of 2. Float arguments support all of the common argument parameters.

**Important:** If a default value is provided it is not validated. You are responsible for ensuring
that the provided value resolves to a number, or your script is able to handle non-numeric values.

#### Example:

```
$ argparse-sh \
    --float height --required \
    --float weight --default 0 \
    --float cash-on-hand --catch-all \
    -- \
    --height 180.4 \
    72.34
HEIGHT="180.4"
WEIGHT="0"
CASH_ON_HAND="72.34"
```

### Choice Arguments (--choice or --pick)

Choice arguments are a little different than other argument types, but they are most similar to
String arguments. With Choice arguments you supply a list of valid choices and alternate mappings.
If an unrecognized value is provided then argparse-sh will fail with a message and an error code of 2.

**Important:** If a default value is provided it is not validated and it is not mapped. You are
responsible for ensuring that the provided value resolves to one of your choices, or your script
is able to handle this value.

#### --option \<name> \[\<help\_text>]

Choice arguments expect one or more option parameters. After `--option` you must include the option
name. You may also provide help text that is shown after that option.

#### --map \<from> \<to>

Maps from one option to another. This provides an easy way to have multiple names for a specific
option.

**Important:** You can map to an option that does not exist. You are responsible for ensuring that
the option that you map to exists. Mappings are not chained; if you map from "a" to "b" and "b" to
"c" and the user provides "a", the value will be "b".

#### Example:

```
$ argparse-sh \
    --choice gender --default "none" \
        --option male "Person identifies as male" \
        --option female "Person identifies as female" \
        --map boy male \
        --map girl female \
        --option other "Person identifies as something else" \
        --option none "Person declines to identify" \
    -- \
    --gender boy
GENDER="male"

$ eval "$(argparse-sh \
    --choice gender --default "none" \
        --option male "Person identifies as male" \
        --option female "Person identifies as female" \
        --map boy male \
        --map girl female \
        --option other "Person identifies as something else" \
        --option none "Person declines to identify" \
    --autohelp \
    -- \
    --help)"

OPTIONS
       --gender <gender>
           No details available.

           The possible options are:

           •   male - Person identifies as male

           •   female - Person identifies as female

           •   boy - Identical to 'male'

           •   girl - Identical to 'female'

           •   other - Person identifies as something else

           •   none - Person declines to identify

           When this option is not provided it will default to 'none'.

```

### Boolean Arguments (--boolean or --bool)

Boolean arguments, like Choice arguments, have additional behavior.

By default boolean arguments do not have a value. If the user does not specify the argument then
the variable will not be set. If the user provides the flag with `--flag-name` then the value of
the boolean argument will be "true". However, users can also explicitly specify the value by using
`--flag-name=false`. The user input will only accept "true" and "false". You can utilize the
`--default` flag to ensure that this is always set.

Boolean arguments can not be repeated, can not have any ordinals, and can not be a catch-all. If
you attempt to define one with any of these characteristics you will get a definition error.

##### Example:

```
$ argparse-sh --boolean happy -- --happy
HAPPY="true"
```

#### --negative-flag \<flag>

Boolean arguments allow defining negative flags. These are flags that force the value to "false".
Negation flags are explicitly defined.

#### Example:

```sh
$ argparse-sh --boolean happy --negative-flag "--not-happy" -- --not-happy
HAPPY="false"
```

This technique is particular useful when combined with either a `--default` paramater or a
`--required` parameter:

```sh
$ argparse-sh --boolean happy --negative-flag "--sad" --required -- --sad
HAPPY="false"

$ argparse-sh --boolean --name "HAPPY" --negative-flag "--sad" --default "true" --
HAPPY="true"
```

The first line requires that you include either `--happy` or `--sad`. If you don't include either
you will get a user error. If you include both you will get an error due to having multiple values
for "HAPPY".

The second line defines "HAPPY" as a boolean that defaults to "true", but can be made "false" by
including the `--sad` argument.

## Other Runtime Options

There are a handful of other options that can be used when running argparse-sh. These can be included
anywhere in the list of arguments, but it is recommended that you put these at the beginning or end
of the arguments. They can not be placed inside an argument definition (e.g. 
`argparse-sh --bool --debug --name bad-example` is ***not*** allowed).

### --debug

Writes debugging information out via echo. This is useful when trying to determine why an argument
is not behaving the way you expected.

#### Example:

```
$ eval "$(argparse-sh \
    --choice gender --default "none" \
        --option male "Person identifies as male" \
        --option female "Person identifies as female" \
        --map boy male \
        --map girl female \
        --option other "Person identifies as something else" \
        --option none "Person declines to identify" \
    --debug \
    -- \
    --gender boy)"
[ArgParse-sh] ArgParse-sh debugging enabled with --debug flag
[ArgParse-sh] Arguments are not exported to child processes
[ArgParse-sh] 
[ArgParse-sh] Definition - type: Choice; name: GENDER; flags: gender; default: none; options: male, female, boy -> male, girl -> female, other, none
[ArgParse-sh] 
[ArgParse-sh] Parsing argument values
[ArgParse-sh] 
[ArgParse-sh] Parsed argument GENDER = 'male'
[ArgParse-sh] 
[ArgParse-sh] Setting GENDER = "male"
[ArgParse-sh] 
[ArgParse-sh] ArgParse-sh completed successfully
```

### --prefix \<arg\_prefix>

Provides a prefix that is put before all parameter names. This is a good way to effectively
namespace the environment variables that get created so you don't overwrite common names.

#### Example:

```
$ argparse-sh \
    --string first_name --catch-all \
    --string last_name --catch-all \
    --prefix "DEMO_" \
    -- Alice Smith
DEMO_FIRST_NAME="Alice"
DEMO_LAST_NAME="Smith"
```

### --export

TODO: This might be changing to `--format <format>`.

### Help Options

ArgParse-sh can auto-generate help text for your command and all of its options. There are a number
of parameters you can supply to aid or direct this process.

#### --auto-help

Indicates that ArgParse-sh should print out the generated help text if the first user argument is
`--help`. This is the simplest way of providing help text for your users.

Text is displayed using the user's `PAGER` variable. If `PAGER` is unset or blank then `less -R` is
used.

##### Example:

```sh
$ eval "$(argparse-sh \
    --string first_name --required \
    --string last_name \
    --auto-help \
    -- \
    --help)"

OPTIONS
       --first_name <first_name>
           No details available.

       --last_name <last_name>
           No details available.
```

#### --help-function \<name>

This directs the script to create a function with the given name that will print out the help text.
This function will not automatically be called, you may call it yourself.

This is useful if you want to put some conditions on your arguments that the program can't validate.
Maybe you expect a string parameter to be at least 4 letters long, and if it isn't you want to
display the help text.

##### Example:

```sh
$ eval "$(argparse-sh \
    --string first_name --required \
    --string last_name \
    --help-function help_me \
    -- )"

$ help_me

OPTIONS
       --first_name <first_name>
           No details available.

       --last_name <last_name>
           No details available.
```

#### --columns \<cols>

Provides the width of the user's screen. This usually can't be determined automatically because
ArgParse-sh is wrapped in the `eval` command. The most common way to provide this is by specifying
`--columns "$(tput cols)"` to read it from the current environment. If this is not provided and
argparse-sh can't determine how many columns the screen has, 80 will be used.

ArgParse-sh will attempt to wrap text at word boundaries, but may not be perfect, especially with
very long words or very low numbers of columns.

##### Example:

```sh
$ eval "$(argparse-sh \
    --program-description "This is a really neat program I wrote." \
    --auto-help \
    --columns 25 \
    -- --help )"

DESCRIPTION
       This is a really
       neat program I
       wrote.
```

#### --program-name \<name>, --program-summary \<summary>, --program-description \<description>

These parameters are all optional, and can provide extra text that shows up in the generated help
text.

For `--program-name` you can use `--program-name "$(basename "$0")"` to automatically get the name
of the script that is executing.

##### Example:

```sh
$ eval "$(argparse-sh \
    --program-name "argparse-sh-demo" \
    --program-summary "Demos of argparse-sh features" \
    --program-description "These are a bunch of demos for how argparse-sh works." \
    --auto-help \
    -- --help )"

NAME
       argparse-sh-demo - Demos of argparse-sh features

DESCRIPTION
       These are a bunch of demos for how argparse-sh works.
```

## Exit Codes

- 0 - Success

  When ArgParse-sh completes successfully the exit code will be 0. If this happens you can be sure
  that all required arguments have been set, all provided arguments have been parsed, and all type
  checks completed successfully.

- 1 - Help

  If the `--autohelp` flag was used and the user passed in `--help` then help text will be written
  to screen (using the user's PAGER if set) and ArgParse-sh will exit with a code of 1.

- 2 - Definition Error

  If there's an issue with the definition of the arguments the exit code will be 2. For example,
  not including an argument name after `--name` would generate this error.

- 3 - User Error

  This error code is returned if there is a problem with the arguments that the user provided. An
  omitted argument that is marked as required, or multiple values for arguments that are not
  repeated are examples of this.

If you run `set -e` before calling argparse-sh, your script will automatically exit if ArgParse-sh
returns an exit code other than 0. You can also trap this error to recover more gracefully.

## Putting it all together.

This is an example of a script using a wide variety of functionality along with best practices.

**demo.sh**

```sh
# Set shell to exit immediately after failed command.
set -e;

# The description is long, so we pulled it out into a variable for clarity.
PROGRAM_DESCRIPTION="This demo program provides a number of examples of how to use ArgParse-sh.
You can provide a number of arguments that are parsed and sent back to the wrapper script as
environment variables.

Feel free to save this script and run it with a variety of parameters to test things out.";

# Run ArgParse-sh with argument definitions and pass command line through.
eval "$(argparse-sh \
  --string given-name first-name \
      --description "Name given to you. In western cultures this is usually your first name." \
      --required \
  --string family-name last-name \
      --description "Name inherited from your family. In western cultures this is usually your last name." \
      --required \
  --string nickname \
      --name NICKNAMES \
      --description "Nicknames that you are commonly known by." \
      --repeated \
  --integer age \
      --description "Your age in years." \
      --required \
  --integer children \
      --default 0 \
      --secret \
  --choice gender \
      --description "The gender that you identify as." \
      --option male "Person identifies as male" \
      --map boy male \
      --option female "Person identifies as female" \
      --map girl female \
      --option other "Person identifies as something else" \
      --option none "Person declines to identify" \
      --default none \
  --boolean basic-data one-line single-line \
      --description "Include this argument if you only want to see the first line in the output." \
  --string quote \
      --name QUOTES \
      --description "Include one or more quotes that you find inspirational." \
      --repeated \
      --required \
      --catch-all \
  --auto-help \
  --prefix "DEMO_" \
  --program-name "$(basename "$0")" \
  --program-summary "Sample script that uses argparse-sh to parse command line arguments." \
  --program-description "$PROGRAM_DESCRIPTION" \
  -- "$@")";

# Dump some of the basic variables to the screen.
echo "Hello $DEMO_GIVEN_NAME. I see you are $DEMO_AGE years old and have $DEMO_CHILDREN children.";

# We can test for existence of boolean variables
if [ "$DEMO_BASIC_DATA" = "true" ]; then
  exit
fi

# We can use if statements to switch logic based on the results of a flag.
if [ "$DEMO_GENDER" = "male" ]; then
  echo "You identify as male."
elif [ "$DEMO_GENDER" = "female" ]; then
  echo "You identify as female."
elif [ "$DEMO_GENDER" = "other" ]; then
  echo "You identify as something other than strictly male or female."
else 
  echo "You have declined to provide your gender identity."
fi

# We can test to see if variables are set, even for repeated arguments.
if [ -n "$DEMO_NICKNAMES" ]; then
  echo ""
  echo "You have $DEMO_NICKNAMES nickname(s):";
  for (( i=0; i<$DEMO_NICKNAMES; i++ )); do
    # We need to do this expansion to iterate over all of the values in the list.
    NICKNAME=DEMO_NICKNAMES_$i
    echo "  ${!NICKNAME}";
  done
fi

# If an argument is required you can be guaranteed that the value is set, even for repeated args.
echo ""
echo "You have $DEMO_QUOTES favorite quote(s):";
for (( i=0; i<$DEMO_QUOTES; i++ )); do
  QUOTE=DEMO_QUOTES_$i
  echo "  ${!QUOTE}";
done
```

You can run this script with `--help` to get the help text
