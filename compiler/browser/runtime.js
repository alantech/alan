const r = require('alan-js-runtime')

// Redefined stdoutp and exitop to work in the browser
module.exports = {
  ...r,
  stdoutp: (...args) => console.log(...args), // Lazy binding to replace `console.log` at will
  stderrp: (...args) => console.error(...args), // Lazy binding to replace `console.error` at will
  exitop: () => {
    r.emitter.removeAllListeners()
  }, // Clean up the event emitter, later we'll want a hook into the playground to show this
}
