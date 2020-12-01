sourceToFile() {
  mkdir -p test_$$
  echo "$2" > test_$$/$1
}

sourceToTemp() {
  mkdir -p test_$$
  echo "$1" > test_$$/temp.ln
}

tempToAmm() {
  alan compile test_$$/temp.ln test_$$/temp.amm 1>/dev/null
}

sourceToAmm() {
  sourceToTemp "$1"
  tempToAmm
}

tempToAgc() {
  alan compile test_$$/temp.amm test_$$/temp.agc 1>/dev/null
}

sourceToAgc() {
  sourceToTemp "$1"
  tempToAgc
}

tempToJs() {
  alan compile test_$$/temp.amm test_$$/temp.js 1>/dev/null
}

sourceToJs() {
  sourceToTemp "$1"
  tempToJs
}

sourceToAll() {
  sourceToTemp "$1"
  tempToAmm
  tempToAgc
  tempToJs
}

test_js() {
  node test_$$/temp.js
}

test_agc() {
  alan run test_$$/temp.agc
}

cleanFile() {
  rm -f "test_$$/$1"
}

cleanTemp() {
  rm -rf test_$$
}

