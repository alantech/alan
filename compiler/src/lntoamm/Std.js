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
      if (moduleAst.name === 'root.ln') {
        rootModule = Module.populateModule('<root>', moduleAst.ast, opcodeScope)
        Module.getAllModules()['<root>'] = rootModule
      }
    })
    // Put the remaining ASTs in a loadable order
    const orderedAsts = []
    let i = 0
    while (stdAsts.length > 0) {
      const stdAst = stdAsts[i]
      // Just remove the root node, already processed
      if (stdAst.name === 'root.ln') {
        stdAsts.splice(i, 1)
        i = i % stdAsts.length
        continue
      }
      // Immediately add any node with no imports and remove from this list
      if (!stdAst.ast.imports()) {
        orderedAsts.push(stdAst)
        stdAsts.splice(i, 1)
        i = i % stdAsts.length
        continue
      }
      // For everything else, check if the dependencies are already queued up
      const importAsts = stdAst.ast.imports()
      let safeToAdd = true
      for (importAst of importAsts) {
        const depName = (
          importAst.standardImport() ?
            importAst.standardImport().dependency().getText().trim() :
            importAst.fromImport().dependency().getText().trim()
          ).replace('@std/', '').replace(/$/, '.ln')
        if (!orderedAsts.some((ast) => ast.name === depName)) {
          safeToAdd = false
          break
        }
      }
      // If it's safe, add it
      if (safeToAdd) {
        orderedAsts.push(stdAst)
        stdAsts.splice(i, 1)
        i = i % stdAsts.length
        continue
      }
      // Otherwise, skip this one
      i = (i + 1) % stdAsts.length
    }
    // Now load the remainig modules based on the root scope
    orderedAsts.forEach((moduleAst) => {
      if (moduleAst.name !== 'root.ln') {
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
