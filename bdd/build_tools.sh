sourceToFile() {
  mkdir -p test_$$
  echo "$2" > test_$$/$1
}

sourceToTemp() {
  mkdir -p test_$$
  echo "$1" > test_$$/temp.ln
}

lnn_sourceToTemp() {
  mkdir -p test_$$
  echo "$1" > test_$$/temp.lnn
}

tempToAmm() {
  alan compile test_$$/temp.ln test_$$/temp.amm 1>/dev/null
}

lnn_tempToAmm() {
  alan compile test_$$/temp.lnn test_$$/temp.amm 1>/dev/null
}

sourceToAmm() {
  sourceToTemp "$1"
  tempToAmm
}

lnn_sourceToAmm() {
  lnn_sourceToTemp "$1"
  lnn_tempToAmm
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

lnn_sourceToJs() {
  lnn_sourceToTemp "$1"
  tempToJs
}

sourceToAll() {
  sourceToTemp "$1"
  tempToAmm
  tempToAgc
  tempToJs
}

lnn_sourceToAll() {
  lnn_sourceToTemp "$1"
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

