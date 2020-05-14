sourceToTemp() {
  echo "$1" > temp.ln
}

tempToAmm() {
  alan-compile -m lntoamm -i temp.ln -o temp.amm 1>/dev/null
}

sourceToAmm() {
  sourceToTemp "$1" 
  tempToAmm
}

tempToAgc() {
  alan-runtime ammtoagc temp.amm -o temp.agc 1>/dev/null
}

sourceToAgc() {
  sourceToAmm "$1"
  tempToAgc
}

jsRuntime() {
  npm install git+ssh://git@github.com:/alantech/alan-js-runtime 2>/dev/null 1>/dev/null
}

tempToJs() {
  alan-compile -m ammtojs -i temp.amm -o temp.js 1>/dev/null
}

sourceToJs() {
  sourceToAmm "$1"
  tempToJs
}

cleanTemp() {
  rm -f temp.ln
  rm -f temp.amm
  rm -f temp.agc
  rm -f temp.js
}

