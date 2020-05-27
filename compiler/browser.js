const ammtojs = require('./dist/ammtojs').ammTextToJs
const lntoamm = require('./dist/lntoamm').lnTextToAmm
const lntojs = require('./dist/lntojs').lnTextToJs

// We won't support AGC for now because of the complexities of moving off the Buffer API
const convert = {
  ln: {
    amm: lntoamm,
    js: (text) => { // Hackery for browserify for now, will clean this up later after some thought
      return lntojs(text).replace(/alan-js-runtime/g, 'alan-runtime')
    },
  },
  amm: {
    js: (text) => { // Similar hackery
      return ammtojs(text).repalce(/alan-js-runtime/g, 'alan-runtime')
    },
  }
}

module.exports = (inFormat, outFormat, text) => {
  if (convert[inFormat] && convert[inFormat][outFormat]) {
    return convert[inFormat][outFormat](text)
  } else {
    throw new Error(`${inFormat} to ${outFormat} is not supported`)
  }
}
