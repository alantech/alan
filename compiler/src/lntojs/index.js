const lntoamm = require('../lntoamm')
const { ammTextToJs, } = require('../ammtojs')

module.exports = (filename) => ammTextToJs(lntoamm(filename))
module.exports.lnTextToJs = (str) => ammTextToJs(lntoamm.lnTextToAmm(str))
