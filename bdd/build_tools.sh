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
  alan-compile temp.amm temp.agc 1>/dev/null
}

sourceToAgc() {
  sourceToTemp "$1"
  tempToAgc
}

tempToJs() {
  alan-compile temp.amm temp.js 1>/dev/null
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

cleanTemp() {
  rm -f temp.ln
  rm -f temp.amm
  rm -f temp.agc
  rm -f temp.js
}

