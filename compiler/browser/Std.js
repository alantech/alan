const stdlibs = require('./stdlibs.json')

const Ast = require('../dist/lntoamm/Ast')
const Module = require('../dist/lntoamm/Module').default
const Scope = require('../dist/lntoamm/Scope').default
const opcodeScope = require('../dist/lntoamm/opcodes').default.exportScope

module.exports = {
  loadStdModules: (modules) => {
    const stdAsts = Object.keys(stdlibs).map(n => ({
      name: n,
      ast: Ast.fromString(stdlibs[n]),
    }))
    // Load the rootScope first, all the others depend on it
    let rootModule
    stdAsts.forEach((moduleAst) => {
      if (moduleAst.name == 'root.ln') {
        rootModule = Module.populateModule('<root>', moduleAst.ast, opcodeScope, true)
        Module.getAllModules()['<root>'] = rootModule
      }
    })
    // Now load the remaining modules based on the root scope
    while (stdAsts.length > 0) {
      const moduleAst = stdAsts.shift();
      if (moduleAst.name !== 'root.ln') {
        const currName = moduleAst.name;
        try {
          moduleAst.name = '@std/' + moduleAst.name.replace(/.ln$/, '')
          const stdModule = Module.populateModule(
            moduleAst.name,
            moduleAst.ast,
            rootModule.exportScope,
            true
          )
          Module.getAllModules()[moduleAst.name] = stdModule
        } catch (e) { // Failed to load, throw it back on the list to try again
          moduleAst.name = currName;
          stdAsts.push(moduleAst);
        }
      }
    }
  },
}
