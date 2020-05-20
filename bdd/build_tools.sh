sourceToTemp() {
  echo "$1" > temp.ln
}

tempToAmm() {
  alan-compile temp.ln temp.amm 1>/dev/null
}

sourceToAmm() {
  sourceToTemp "$1" 
  tempToAmm
}

tempToAgc() {
  alan-compile temp.ln temp.agc 1>/dev/null
}

sourceToAgc() {
  sourceToTemp "$1"
  tempToAgc
}

tempToJs() {
  alan-compile temp.ln temp.js 1>/dev/null
}

sourceToJs() {
  sourceToTemp "$1"
  tempToJs
}

cleanTemp() {
  rm -f temp.ln
  rm -f temp.amm
  rm -f temp.agc
  rm -f temp.js
}

