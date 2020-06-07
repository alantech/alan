const lntoamm = require('../lntoamm')
const { ammTextToAga, } = require('../ammtoaga')

module.exports = (filename) => ammTextToAga(lntoamm(filename))
