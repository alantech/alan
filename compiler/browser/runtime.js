const r = require('alan-js-runtime')

// Redefined stdoutp and exitop to work in the browser
module.exports = {
  ...r,
  stdoutp: console.log,
  exitop: () => {
    r.emitter.removeAllListeners()
  }, // Clean up the event emitter, later we'll want a hook into the playground to show this
}
