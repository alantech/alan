const fs = require('fs')
const path = require('path')

const Ast = require('./Ast')
const Module = require('./Module')
const Scope = require('./Scope')
const opcodeScope = require('./opcodes').exportScope

module.exports = {
  loadStdModules: (modules) => {
    const stdDir = path.join(__dirname, '../../std')
    const stdAsts = fs.readdirSync(stdDir).filter(n => /.ln$/.test(n)).map(n => ({
      name: n,
      ast: Ast.fromFile(path.join(__dirname, '../../std', n)),
    }))
    // Load the rootScope first, all the others depend on it
    let rootModule
    stdAsts.forEach((moduleAst) => {
      if (moduleAst.name == 'root.ln') {
        rootModule = Module.populateModule('<root>', moduleAst.ast, opcodeScope)
        Module.getAllModules()['<root>'] = rootModule
      }
    })
    // Now load the remainig modules based on the root scope
    stdAsts.forEach((moduleAst) => {
      if (moduleAst.name != 'root.ln') {
        moduleAst.name = '@std/' + moduleAst.name.replace(/.ln$/, '')
        const stdModule = Module.populateModule(
          moduleAst.name,
          moduleAst.ast,
          rootModule.exportScope
        )
        Module.getAllModules()[moduleAst.name] = stdModule
      }
    })
  },
}
