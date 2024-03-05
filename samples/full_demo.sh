# Set shell to exit immediately after failed command.
set -e;

# The description is long, so we pulled it out into a variable for clarity.
PROGRAM_DESCRIPTION="This demo program provides a number of examples of how to use ArgParse.
You can provide a number of arguments that are parsed and sent back to the wrapper script as
environment variables.

Feel free to save this script and run it with a variety of parameters to test things out.";

# Run ArgParse with argument definitions and pass command line through.
eval "$(target/debug/argparse \
  --string given-name first-name \
      --description "Name given to you. In western cultures this is usually your first name." \
      --ordinal 0 \
      --required \
  --string family-name last-name \
      --description "Name inherited from your family. In western cultures this is usually your last name." \
      --ordinal 1 \
      --required \
  --string nickname \
      --name NICKNAMES \
      --description "Nicknames that you are commonly known by." \
      --repeated \
  --integer age \
      --description "Your age in years." \
      --flag "-a" \
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
  --boolean print-help-text \
      --description "Include this argument to print out the help text at the end." \
  --string quote \
      --name QUOTES \
      --description "Include one or more quotes that you find inspirational." \
      --repeated \
      --required \
      --catch-all \
  --auto-help \
  --help-function "print_help" \
  --prefix "DEMO_" \
  --columns "$(tput cols)" \
  --program-name "$(basename "$0")" \
  --program-summary "Sample script that uses argparse to parse command line arguments." \
  --program-description "$PROGRAM_DESCRIPTION" \
  --debug \
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

if [ "$DEMO_PRINT_HELP_TEXT" = "true" ]; then
  print_help
fi

