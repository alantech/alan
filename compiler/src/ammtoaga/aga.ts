import { stat } from "fs"
import { DepGraph, DepNode } from "./depgraph"

export class Block {
  type: string
  name: string
  memSize: number
  statements: Statement[]
  deps: string[]

  constructor(
    type: string,
    name: string,
    memSize: number,
    statements: Statement[],
    deps: string[],
  ) {
    this.type = type
    this.name = name
    this.memSize = memSize
    this.statements = statements
    this.deps = deps
  }

  build(): string {
    const dependencies = []
    const idxByNode: Map<DepNode, number> = new Map()
    for (let ii = 0; ii < this.statements.length; ii++) {
      let stmt = this.statements[ii]
      if (stmt.depNode === null) continue;
      idxByNode.set(stmt.depNode, ii)
      for (let upstream of stmt.depNode.upstream) {
        if (idxByNode.get(upstream) !== null && idxByNode.get(upstream) !== undefined) {
          stmt.deps.push(idxByNode.get(upstream))
          dependencies.push({
            in: this.name,
            stmt: ii,
            dependsOn: idxByNode.get(upstream),
          })
        }
      }
    }
    return JSON.stringify(dependencies)
  }

  toString() {
    let b = `${this.type} for ${this.name} with size ${this.memSize}\n`
    this.statements.forEach(s => b += `  ${s.toString()}\n`)
    return b
  }
}

export class Statement {
  fn: string
  inArgs: [string, string] | [string, string, string]
  outArg: string | null
  line: number
  deps: number[]
  depNode: DepNode

  constructor(
    fn: string,
    inArgs: [string, string] | [string, string, string],
    outArg: string | null,
    line: number,
    deps: number[],
    depNode: DepNode,
  ) {
    this.fn = fn
    this.inArgs = inArgs
    this.outArg = outArg
    this.line = line
    this.deps = deps
    this.depNode = depNode || null
  }

  toString() {
    let s = ''
    if (this.outArg !== null) {
      s += `${this.outArg} = `
    }
    s += `${this.fn}(${this.inArgs.join(', ')}) #${this.line}`
    if (this.deps.length > 0) {
      s += ` <- [${this.deps.map(d => `#${d}`).join(', ')}]`
    }
    return s
  }
}
