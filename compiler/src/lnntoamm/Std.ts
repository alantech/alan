import * as fs from 'fs'
import * as path from 'path'

import * as Ast from './Ast'
import Module from './Module'
import opcodes from './opcodes'
import { LPNode, } from '../lp'

interface AstRec {
  name: string
  ast: LPNode
}

export const loadStdModules = (stdImports: Set<string>) => {
  const stdDir = path.join(__dirname, '../../std')
  const allStdAsts = fs.readdirSync(stdDir).filter(n => /.lnn$/.test(n)).map(n => ({
    name: n,
    ast: Ast.fromFile(path.join(__dirname, '../../std', n)),
  }))
  const stdAsts = allStdAsts.filter(ast => stdImports.has(ast.name) || ast.name === 'root.lnn')
  // Load the rootScope first, all the others depend on it
  let rootModule: Module
  stdAsts.forEach((moduleAst) => {
    if (moduleAst.name === 'root.lnn') {
      rootModule = Module.populateModule('<root>', moduleAst.ast, opcodes(), true)
      Module.getAllModules()['<root>'] = rootModule
    }
  })
  // Put the remaining ASTs in a loadable order
  const orderedAsts = []
  let i = 0
  while (stdAsts.length > 0) {
    const stdAst: AstRec = stdAsts[i]
    // Just remove the root node, already processed
    if (stdAst.name === 'root.lnn') {
      stdAsts.splice(i, 1)
      i = i % stdAsts.length
      continue
    }
    // Immediately add any node with no imports and remove from this list
    if (stdAst.ast.get('imports').getAll().length === 0) {
      orderedAsts.push(stdAst)
      stdAsts.splice(i, 1)
      i = i % stdAsts.length
      continue
    }
    // For everything else, check if the dependencies are already queued up
    const importAsts = stdAst.ast.get('imports').getAll()
    let safeToAdd = true
    for (const importAst of importAsts) {
      const depName = (
        importAst.has('standardImport') ?
          importAst.get('standardImport').get('dependency').t.trim() :
          importAst.get('fromImport').get('dependency').t.trim()
        ).replace('@std/', '').replace(/$/, '.lnn')
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
    if (moduleAst.name !== 'root.lnn') {
      moduleAst.name = '@std/' + moduleAst.name.replace(/.lnn$/, '')
      const stdModule = Module.populateModule(
        moduleAst.name,
        moduleAst.ast,
        rootModule.exportScope,
        true
      )
      Module.getAllModules()[moduleAst.name] = stdModule
    }
  })
}
