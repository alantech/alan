import * as fs from 'fs'
import * as path from 'path'

import * as Ast from './Ast'
import Module = require('./Module')
import { exportScope as opcodeScope, } from './opcodes'

export const loadStdModules = (stdImports: Set<any>) => {
  const stdDir = path.join(__dirname, '../../std')
  const allStdAsts = fs.readdirSync(stdDir).filter(n => /.ln$/.test(n)).map(n => ({
    name: n,
    ast: Ast.fromFile(path.join(__dirname, '../../std', n)),
  }))
  const stdAsts = allStdAsts.filter(ast => stdImports.has(ast.name) || ast.name === 'root.ln')
  // Load the rootScope first, all the others depend on it
  let rootModule: any // TODO: convert `Module` to TS
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
    const stdAst: any = stdAsts[i]
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
    for (const importAst of importAsts) {
      const depName = (
        importAst.standardImport() ?
          importAst.standardImport().dependency().getText().trim() :
          importAst.fromImport().dependency().getText().trim()
        ).replace('@std/', '').replace(/$/, '.ln')
      if (!orderedAsts.some((ast) => ast.name === depName)) {
        // add std modules this std module imports if not present
        if (!stdAsts.some((ast) => ast.name === depName)) {
          stdAsts.splice(i, 0, allStdAsts.filter(a => a.name === depName)[0])
        }
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
}
