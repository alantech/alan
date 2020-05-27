const fs = require('fs')

const { InputStream, CommonTokenStream, } = require('antlr4')

const { AmmLexer, AmmParser, } = require('./')

const Ast = {
  fromString: (str) => {
    const inputStream = new InputStream(str)
    const langLexer = new AmmLexer(inputStream)
    const commonTokenStream = new CommonTokenStream(langLexer)
    const langParser = new AmmParser(commonTokenStream)

    return langParser.module()
  },

  fromFile: (filename) => {
    return Ast.fromString(fs.readFileSync(filename, { encoding: 'utf8', }))
  },
}

module.exports = Ast
