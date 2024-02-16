# ArgParse

Utility for parsing arguments to shell scripts and providing the results as environment variables.

# Usage

ArgParse takes two sets of arguments. The first set defines all of the arguments that will be
parsed. The second set is the arguments to parse. This is probably best clarified via a minimal
demonstration:

```sh
$ argparse --string text -- --text "Hello, world!"
TEXT="Hello, world!"
```

What did this do? It defined a set of arguments (one string argument called "text") and then
parsed a set of argument values. The output is a script that sets environment variables for the
arguments provided.

This is a minimal example, but it isn't a very useful one. Instead lets drop this into a script
and pass arguments on the script through to ArgParse:

**demo.sh**

```sh
eval "$(argparse --string text -- "$@")";
echo "$TEXT"
```

Let's break this script up into various pieces:

- `argparse` - This invokes the ArgParse program

- `--string text` - This indicates to ArgParse that we can accept an optional argument called
  "text". The value is a string and there is no default value.

- `--` - This indicates to ArgParse that we are done defining arguments. All remaining command line
  parameters should be treated as argument values.

- `"$@"` - This forwards all of the parameters that the script was called with to ArgParse. Because
  this is after the `--`, ArgParse will interpret these parameters.

- `eval "$(...)";` - The output of ArgParse is script commands. `eval` executes these commands in
  the current shell.

- `echo "$TEXT"` - The script commands that ArgParse will output set environment variables with the
  value of the arguments. This will print the value of whatever was passed in for the "text"
  argument.

If we run `demo.sh` we can see this all work in action:

```sh
$ ./demo.sh --text "Hello, world!"
Hello, world!
```

ArgParse recognizes the `--text` argument and expects a string. That string ("Hello, world!") is
written to an environment variable called "TEXT" (based on the argument's name), which can then be
used by the remainder of the script.

In several examples below we will omit the `eval` part of the argparse command. This will cause the
ArgParse output to dump to the screen, allowing us to see what is happening more clearly.

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
$ argparse --string given-name first-name name-- --first-name "Alice"
GIVEN_NAME="Alice"
```

This creates a single string argument called "GIVEN\_NAME" that can be set using either
`--given-name`, `--first-name`, or `--name`. This is the shorthand form of:

```sh
$ argparse --string --name GIVEN_NAME \
    --flag "--given-name" \
    --flag "--first-name" \
    --flag "--name" \
    -- --first-name "Alice"
GIVEN_NAME="Alice"
```

#### --name <name>

Provide the name of the environment variable the value for this argument will be stored in. If the
name is not provided, the name will be a normalized version of the first flag name that is
provided (capitalized and non-alphanumeric sequences will be replaced with "\_").

If the name is not provided, the first flag defined (either using --flag or the shorthand above)
will be normalized and used. Normalization removes any preceeding or trailing non-alphanumeric
characters. All other sequences of non-alphanumeric sequences will be replaced with "\_" and the
entire string will be capitalized. For example, if the first flag is "--first-name" the normalized
name would be "FIRST\_NAME".

##### Example:

```sh
$ argparse --string name --name FIRST_NAME -- --name "Alice"
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
$ argparse --string name --flag "-n" -- -n Alice
NAME="Alice"
```

This defines a string argument called "NAME" using the shorthand method, but an alternate flag
("-n") is also defined as available for use.

#### --default \<default>

Provide the default value to use if this argument is not specified. 

**Warning:** This default value is not parsed or validated; invalid values will be passed on
through.

##### Example:

```sh
$ argparse --string name --default "Alice"
NAME="Alice"
```

If `-- --name "Bob"` had been provided then `NAME` would have been set to "Bob" instead of "Alice".

#### --desc\[ription] \<description>

Provide a description to use for this argument when generating help text.

##### Example:

```sh
$ eval "$(argparse --string name --desc "The user's first name" --autohelp -- --help)"

OPTIONS
       --name <name>
           The user's first name

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
$ argparse --string name --repeated -- --name "Alice" --name "Bob" --name "Carol"
NAME="3"
NAME_0="Alice"
NAME_1="Bob"
NAME_2="Carol"
```

Here we can see that three names were supplied. Each value for `--name` was included in order.

#### --required

Indicates that this argument is required. If not provided ArgParse will fail.

##### Example:

```sh
$ argparse --string name --required
echo ""
echo "!!! ArgParse Error: Value for argument NAME is missing !!!"
echo ""

$ echo $?
2
```

An error message is shown indicating that the "name" argument wasn't supplied. The exit code from
ArgParse when there is an error parsing the arguments is 2.

#### --secret

Marks an argument for non-inclusion in generated help text.

##### Example:

```sh
$ eval "$(target/debug/argparse --string name --secret --string age --autohelp -- --help)"

OPTIONS
       --age <age>
           No details available.

```

Again we use `eval` for clarity. Note that help text is generated for the "age" argument, but not
for the "name" argument.

#### --catch-all

This is used to mark an argument that will be get any unrecognized values. This is particularly
useful for repeated arguments where you don't want to require the user to specify the flag name.

Note that catch-all arguments don't ***require*** any flags, but it is advised that you still
provide some to the user so that they can use the `--flag=value` syntax. If not using any flags
then the argument **must** have a name.

##### Example:

```sh
$ argparse --string name --catch-all -- "Bob"
NAME="Bob"
```

### String Arguments (--string or --str)

String arguments do not perform any validation or re-writing of their values. These are simply
passed through to the environment variable. String arguments support all of the common argument
parameters.

#### Example:

```
$ argparse \
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
If an invalid argument is provided then argparse will fail with a message and an error code of 2.
Integer arguments support all of the common argument parameters.

**Important:** If a default value is provided it is not validated. You are responsible for ensuring
that the provided value resolves to an integer, or your script is able to handle non-integer values.

#### Example:

```
$ argparse \
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
number. If an invalid argument is provided then argparse will fail with a message and an error code
of 2. Float arguments support all of the common argument parameters.

**Important:** If a default value is provided it is not validated. You are responsible for ensuring
that the provided value resolves to a number, or your script is able to handle non-numeric values.

#### Example:

```
$ argparse \
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
If an unrecognized value is provided then argparse will fail with a message and an error code of 2.

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
$ argparse \
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

$ eval "$(argparse \
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

Boolean arguments are significantly different than other argument types. By default boolean
arguments have a value of "false", and specifying the argument is enough to change the value to
"true". You can specify a value of "false" for this flag by using the `--flag=false` syntax. When
using this syntax the only acceptable values are "true" and "false".

Boolean arguments support a very limited subset of the common argument parameters. The only
supported parameters are:

- `--name`

- `--secret`

- `--desc[ription]`

All other parameters are ignored.

#### Example:

```
$ argparse \
    --boolean is-happy \
    --boolean is-sad \
    -- \
    --is-happy \
    --is-sad=false
IS_HAPPY="true"
IS_SAD="false"
```

#### Advanced Options

Because of the limitations on Boolean arguments, it's not uncommon to want slightly different
functionality, or to use one of the common argument parameters that are not permitted on Boolean
arguments (like `--default`). In this case we recommend creating a Choice argument that looks
like a Boolean argument:

```
$ argparse \
    --choice is-happy \
        --default false \
        --option true \
        --option false \
        --map yes true \
        --map no false \
    -- \
    --is-happy true
IS_HAPPY="true"
```

This gives you an argument that mostly looks like a boolean argument. The help text is a bit
different, but you can make it required, provide defaults, make it repeated, and add additional
mappings to "true" and "false".

## Other Runtime Options

There are a handful of other options that can be used when running argparse. These can be included
anywhere in the list of arguments, but it is recommended that you put these at the beginning or end
of the arguments. They can not be placed inside an argument definition (e.g. 
`argparse --bool --debug --name bad-example` is ***not*** allowed).

### --debug

Writes debugging information out via echo. This is useful when trying to determine why an argument
is not behaving the way you expected.

#### Example:

```
$ eval "$(argparse \
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
[ArgParse] ArgParse debugging enabled with --debug flag
[ArgParse] Arguments are not exported to child processes
[ArgParse] 
[ArgParse] Definition - type: Choice; name: GENDER; flags: gender; default: none; options: male, female, boy -> male, girl -> female, other, none
[ArgParse] 
[ArgParse] Parsing argument values
[ArgParse] 
[ArgParse] Parsed argument GENDER = 'male'
[ArgParse] 
[ArgParse] Setting GENDER = "male"
[ArgParse] 
[ArgParse] ArgParse completed successfully
```

### --auto-help

### --export

### --prefix \<arg\_prefix>

### --program-name \<name>

### --program-summary \<summary>

### --program-description \<description>

## Exit Codes

- 0 - Success

  When ArgParse completes successfully the exit code will be 0. If this happens you can be sure
  that all required arguments have been set, all provided arguments have been parsed, and all type
  checks completed successfully.

- 1 - Help

  If the `--autohelp` flag was used and the user passed in `--help` then help text will be written
  to screen (using the user's PAGER if set) and ArgParse will exit with a code of 1.

- 2 - Definition Error

  If there's an issue with the definition of the arguments the exit code will be 2. For example,
  not including an argument name after `--name` would generate this error.

- 3 - User Error

  This error code is returned if there is a problem with the arguments that the user provided. An
  omitted argument that is marked as required, or multiple values for arguments that are not
  repeated are examples of this.

If you run `set -e`

## Putting it all together.

This is an example of a script using a wide variety of functionality along with best practices.

**demo.sh**

```sh
# Set shell to exit immediately after failed command.
set -e;

# The description is long, so we pulled it out into a variable for clarity.
PROGRAM_DESCRIPTION="This demo program provides a number of examples of how to use ArgParse.
You can provide a number of arguments that are parsed and sent back to the wrapper script as
environment variables.

Feel free to save this script and run it with a variety of parameters to test things out.";

# Run ArgParse with argument definitions and pass command line through.
eval "$(argparse \
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
  --program-summary "Sample script that uses argparse to parse command line arguments." \
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
