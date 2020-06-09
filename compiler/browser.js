const { default: buildPipeline, } = require('./dist/pipeline')
const ammtojs = require('./dist/ammtojs')
const lntoamm = require('./dist/lntoamm')
const ammtoaga = require('./dist/ammtoaga')

// We won't support AGC for now because of the complexities of moving off the Buffer API
const convert = buildPipeline([
  ['ln', 'amm', lntoamm],
  ['amm', 'aga', ammtoaga],
  ['amm', 'js', ammtojs],
])

module.exports = (inFormat, outFormat, text) => {
  if (convert[inFormat] && convert[inFormat][outFormat]) {
    const out = convert[inFormat][outFormat].fromString(text)
    if (outFormat === 'js') { // Hackery for browserify for now, will clean this up later
      return out.replace(/alan-js-runtime/g, 'alan-runtime')
    }
    return out
  } else {
    throw new Error(`${inFormat} to ${outFormat} is not supported`)
  }
}
