eval "$(target/debug/argparse \
  --bool is-sunny \
  --bool is-rainy rainy r --name kinda----rainy \
  --int --name degrees temperature temp t --repeated \
  --int wind-speed windspeed wind w --default 0 \
  --float rainfall rain --desc "How much rain is expected to fall." \
  --choice units unit u --option imperial "ft, mi, °F" --map us imperial --option metric --default metric \
  --string text --required \
  --string source --secret --catch-all \
  --autohelp \
  --prefix "TEST_ARG_" \
  --export \
  --debug \
  --program-name "$0" \
  --program-summary "ArgParse Test Script" \
  --program-description "This is a test script, used to demonstrate various ArgParse features." \
  -- "$@")"

