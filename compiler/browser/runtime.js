const r = require('alan-js-runtime')

// Redefined stdoutp and exitop to work in the browser
module.exports = {
  ...r,
  execop: a => {}, // no-op in the browser
  stdoutp: (...args) => console.log(...args), // Lazy binding to replace `console.log` at will
  exitop: () => {
    r.emitter.removeAllListeners()
  }, // Clean up the event emitter, later we'll want a hook into the playground to show this
}
