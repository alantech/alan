const lntoamm = require('../lntoamm')
const { ammTextToAgc, } = require('../ammtoagc')

module.exports = (filename) => ammTextToAgc(lntoamm(filename))
