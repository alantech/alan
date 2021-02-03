import {
  LP,
  LPNode,
  LPError,
  NamedAnd,
  NulLP,
} from './lp'

import amm from './amm'
import { exit } from 'process'

// This project depends on BigNum and associated support in Node's Buffer, so must be >= Node 10.20
// and does not work in the browser. It would be possible to implement a browser-compatible version
// but there is no need for it and it would make it harder to work with.
const ceil8 = (n: number) => Math.ceil(n / 8) * 8
const CLOSURE_ARG_MEM_START = BigInt(Math.pow(-2,63))

const loadGlobalMem = (globalMemAst: LPNode[], addressMap: object) => {
  const globalMem = {}
  let currentOffset = -1
  for (const globalConst of globalMemAst) {
    const rec = globalConst.get()
    if (!(rec instanceof NamedAnd)) continue
    let val: string
    switch (rec.get('fulltypename').t.trim()) {
    case "int64":
      val = rec.get('assignables').t.trim() + 'i64'
      globalMem[`@${currentOffset}`] = val
      addressMap[rec.get('decname').t] = currentOffset
      currentOffset -= 8
      break
    case "float64":
      val = rec.get('assignables').t.trim() + 'f64'
      globalMem[`@${currentOffset}`] = val
      addressMap[rec.get('decname').t] = currentOffset
      currentOffset -= 8
      break
    case "string":
      let str: string
      try {
        // Will fail on strings with escape chars
        str = JSON.parse(rec.get('assignables').t.trim())
      } catch (e) {
        // Hackery to get these strings to work
        str = JSON.stringify(
          rec.get('assignables').t.trim().replace(/^["']/, '').replace(/["']$/, '')
        )
      }
      let len = ceil8(str.length) + 8
      val = rec.get('assignables').t.trim()
      globalMem[`@${currentOffset}`] = val
      addressMap[rec.get('decname').t] = currentOffset
      currentOffset -= len
      break
    case "bool":
      val = rec.get('assignables').t.trim()
      globalMem[`@${currentOffset}`] = val
      addressMap[rec.get('decname').t] = currentOffset
      currentOffset -= 8
      break
    default:
      throw new Error(rec.get('fulltypename').t + ' not yet implemented')
    }
  }
  return globalMem
}

const loadEventDecs = (eventAst: LPNode[]) => {
  const eventMem = {}
  for (const evt of eventAst) {
    const rec = evt.get()
    if (!(rec instanceof NamedAnd)) continue
    const evtName = rec.get('variable').t.trim()
    const evtSize = rec.get('fulltypename').t.trim() === 'void' ? 0 : [
      'int8', 'int16', 'int32', 'int64', 'float32', 'float64', 'bool',
    ].includes(rec.get('fulltypename').t.trim()) ? 8 : -1
    eventMem[evtName] = evtSize
  }
  return eventMem
}

const getFunctionbodyMem = (functionbody: LPNode) => {
  let memSize = 0
  const addressMap = {}
  for (const statement of functionbody.get('statements').getAll()) {
    if (statement.has('declarations')) {
      if (statement.get('declarations').has('constdeclaration')) {
        if (statement.get('declarations').get('constdeclaration').get('assignables').has('functions')) {
          // Because closures re-use their parent memory space, their own memory needs to be included
          const closureMem = getFunctionbodyMem(
            statement
              .get('declarations')
              .get('constdeclaration')
              .get('assignables')
              .get('functions')
              .get('functionbody')
          )
          Object.keys(closureMem.addressMap).forEach(
            name => addressMap[name] = closureMem.addressMap[name] + memSize
          )
          memSize += closureMem.memSize
        } else {
          addressMap[
            statement.get('declarations').get('constdeclaration').get('decname').t.trim()
          ] = memSize
          memSize += 1
        }
      } else {
        addressMap[
          statement.get('declarations').get('letdeclaration').get('decname').t.trim()
        ] = memSize
        memSize += 1
      }
    }
  }
  return {
    memSize,
    addressMap,
  }
}

const getHandlersMem = (handlers: LPNode[]) => handlers
  .map(h => h.get())
  .filter(h => h instanceof NamedAnd)
  .map(handler => {
    const handlerMem = getFunctionbodyMem(handler.get('functions').get('functionbody'))
    let arg = handler.get('functions').get('args').get(0).get(0).get('arg')
    if (arg instanceof NulLP) {
      arg = handler.get('functions').get('args').get(1).get('arg')
    }
    if (!(arg instanceof NulLP)) {
      // Increase the memory usage and shift *everything* down, then add the new address
      handlerMem.memSize += 1
      Object.keys(handlerMem.addressMap).forEach(name => handlerMem.addressMap[name] += 1)
      handlerMem.addressMap[arg.get('variable').t.trim()] = 0
    }
    return handlerMem
  })

const unhandled = (val) => {
  console.log('========== UNHANDLED')
  console.log(val)
  console.log()
  throw new Error()
}

class HandlerGraph {
  build(stmts: LPNode[]) {}

  getLastMutationFor(varName: String): HandlerNode { return null }
}

class HandlerNode {
  stmt: string
  upstream: HandlerNode[]
  downstream: HandlerNode[]
  closure?: HandlerGraph
  mutates: string[]

  constructor(stmt: LPNode, graph: HandlerGraph) {
    if (stmt.has('declarations')) {
      let dec = stmt.get('declarations')
      if (dec.has('constdeclaration')) dec = dec.get('constdeclaration')
      else if (dec.has('letdeclaration')) dec = dec.get('letdeclaration')
      else unhandled(dec)

      if (dec.get('fulltypename').t.trim() == 'function') {
        // TODO: recurse into functions
        unhandled(dec)
      } else if (dec.has('assignables') && dec.get('assignables').has('calls')) {
        this.fromCall(dec.get('assignables').get('calls'), graph)
      } else unhandled(dec)
    } else unhandled(stmt)
  }

  fromCall(call: LPNode, graph: HandlerGraph) {
    let opcodeName = call.get('variable').t.trim()
    let args = call.get('calllist').getAll()
    let mutated = []
    let opMutability = opcodeParamMutabilities[opcodeName]
    for (let ii = 0; ii < opMutability.length; ii++) {
      if (opMutability[ii]) mutated.push(args[ii].t.trim())
    }
    this.mutates = mutated
    for (let arg of args) {
      let upstream = graph.getLastMutationFor(arg.t.trim())
      if (upstream) {
        this.upstream.push(upstream)
        upstream.downstream.push(this)
      }
    }
  }
}

const opcodeParamMutabilities = [
]

const closuresFromDeclaration = (
  declaration: LPNode,
  closureMem: object,
  eventDecs: object,
  addressMap: object,
  // For each scope branch, determine a unique argument rereference so nested scopes can access
  // parent scope arguments
  argRerefOffset: number,
  scope: string[],
) => {
  const name = declaration.get('constdeclaration').get('decname').t.trim()
  const fn = declaration.get('constdeclaration').get('assignables').get('functions')
  let fnArgs = []
  fn.get('args').getAll()[0].getAll().forEach((argdef) => {
    fnArgs.push(argdef.get('arg').get('variable').t)
  })
  if (fn.get('args').getAll()[1].has()) {
    fnArgs.push(...fn.get('args').getAll()[1].getAll().map(t => t.get('variable').t))
    fnArgs = fnArgs.filter(t => t !== '')
  }
  fnArgs.forEach(arg => {
    addressMap[arg + name] = CLOSURE_ARG_MEM_START + BigInt(argRerefOffset)
    argRerefOffset++
  })
  const allStatements = declaration
    .get('constdeclaration')
    .get('assignables')
    .get('functions')
    .get('functionbody')
    .get('statements')
    .getAll()
  const statements = allStatements.filter(statement => !(statement.has('declarations') &&
    statement.get('declarations').has('constdeclaration') &&
    statement.get('declarations').get('constdeclaration').get('assignables').has('functions')
  ))
  const otherClosures = allStatements.filter(statement => statement.has('declarations') &&
    statement.get('declarations').has('constdeclaration') &&
    statement.get('declarations').get('constdeclaration').get('assignables').has('functions')
  ).map(
    s => closuresFromDeclaration(
      s.get('declarations'),
      closureMem,
      eventDecs,
      addressMap,
      argRerefOffset,
      [ name, ...scope, ], // Newest scope gets highest priority
    )
  ).reduce((obj, rec) => ({
    ...obj,
    ...rec,
  }), {})
  eventDecs[name] = 0

  return {
    [name]: {
      name,
      fn,
      statements,
      closureMem,
      scope: [ name, ...scope, ],
    },
    ...otherClosures,
  }
}

const extractClosures = (handlers: LPNode[], handlerMem: object, eventDecs: object, addressMap: object) => {
  let closures = {}
  let recs = handlers.filter(h => h.get() instanceof NamedAnd)
  for (let i = 0; i < recs.length; i++) {
    const rec = recs[i].get()
    const closureMem = handlerMem[i]
    for (const statement of rec.get('functions').get('functionbody').get('statements').getAll()) {
      if (
        statement.has('declarations') &&
        statement.get('declarations').has('constdeclaration') &&
        statement.get('declarations').get('constdeclaration').get('assignables').has('functions')
      ) {
        // It's a closure, first try to extract any inner closures it may have
        const innerClosures = closuresFromDeclaration(
          statement.get('declarations'),
          closureMem,
          eventDecs,
          addressMap,
          5,
          [],
        )
        closures = {
          ...closures,
          ...innerClosures,
        }
      }
    }
  }
  return Object.values(closures)
}

class Statement {
  fn: string
  inArgs: [string, string] | [string, string, string]
  outArg: string | null
  line: number
  deps: number[]

  constructor(
    fn: string,
    inArgs: [string, string] | [string, string, string],
    outArg: string | null,
    line: number,
    deps: number[],
  ) {
    this.fn = fn
    this.inArgs = inArgs
    this.outArg = outArg
    this.line = line
    this.deps = deps
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

const loadStatements = (
  statements: LPNode[],
  localMem: object,
  globalMem: object,
  fn: LPNode,
  fnName: string,
  isClosure: boolean,
  closureScope: string[],
) => {
  let vec = []
  let line = 0
  let localMemToLine = {}
  statements = statements.filter(s => !s.has('whitespace'))
  let fnArgs = []
  fn.get('args').getAll()[0].getAll().forEach((argdef) => {
    fnArgs.push(argdef.get('arg').get('variable').t)
  })
  if (fn.get('args').getAll()[1].has()) {
    fnArgs.push(...fn.get('args').getAll()[1].getAll().map(t => t.get('variable').t))
    fnArgs = fnArgs.filter(t => t !== '')
  }
  fnArgs.forEach((arg, i) => {
    if (globalMem.hasOwnProperty(arg + fnName)) {
      let resultAddress = globalMem[arg + fnName]
      let val = CLOSURE_ARG_MEM_START + BigInt(1) + BigInt(i)
      let s = new Statement('refv', [`@${val}`, '@0'], `@${resultAddress}`, line, [])
      vec.push(s)
      line += 1
    }
  })
  for (let idx = 0; idx < statements.length; idx++) {
    const statement = statements[idx]
    if (
      statement.has('declarations') &&
      statement.get('declarations').has('constdeclaration') &&
      statement.get('declarations').get('constdeclaration').get('assignables').has('functions')
    ) {
      // It's a closure, skip it
      continue
    }
    const hasClosureArgs = isClosure && fnArgs.length > 0
    let s: Statement
    if (statement.has('declarations')) {
      const dec = statement.get('declarations').has('constdeclaration') ?
        statement.get('declarations').get('constdeclaration') :
        statement.get('declarations').get('letdeclaration')
      let resultAddress = localMem[dec.get('decname').t.trim()]
      localMemToLine[dec.get('decname').t.trim()] = line
      const assignables = dec.get('assignables')
      if (assignables.has('functions')) {
        throw new Error("This shouldn't be possible!")
      } else if (assignables.has('calls')) {
        const call = assignables.get('calls')
        const fnName = call.get('variable').t.trim()
        const vars = (call.has('calllist') ? call.get('calllist').getAll() : []).map(
          v => v.get('variable').t.trim()
        )
        const args = vars.map(v => {
          if (localMem.hasOwnProperty(v)) {
            return localMem[v]
          } else if (globalMem.hasOwnProperty(v)) {
            return globalMem[v]
          } else if (Object.keys(globalMem).some(k => closureScope.map(s => v + s).includes(k))) {
            return globalMem[
              closureScope.map(s => v + s).find(k => Object.keys(globalMem).includes(k))
            ]
          } else if (hasClosureArgs) {
            return CLOSURE_ARG_MEM_START + BigInt(1) + BigInt(fnArgs.indexOf(v))
          } else {
            return v
          }
        }).map(a => typeof a === 'string' ? a : `@${a}`)
        while (args.length < 2) args.push('@0')
        s = new Statement(fnName, args as [string, string], `@${resultAddress}`, line, [])
      } else if (assignables.has('value')) {
        // Only required for `let` statements
        let fn: string
        let val: string
        switch (dec.get('fulltypename').t.trim()) {
        case 'int64':
          fn = 'seti64'
          val = assignables.t + 'i64'
          break
        case 'int32':
          fn = 'seti32'
          val = assignables.t + 'i32'
          break
        case 'int16':
          fn = 'seti16'
          val = assignables.t + 'i16'
          break
        case 'int8':
          fn = 'seti8'
          val = assignables.t + 'i8'
          break
        case 'float64':
          fn = 'setf64'
          val = assignables.t + 'f64'
          break
        case 'float32':
          fn = 'setf32'
          val = assignables.t + 'f32'
          break
        case 'bool':
          fn = 'setbool'
          val = assignables.t === 'true' ? '1i8' : '0i8' // Bools are bytes in the runtime
          break
        case 'string':
          fn = 'setestr'
          val = '0i64'
          break
        default:
          throw new Error(`Unsupported variable type ${dec.get('fulltypename').t}`)
        }
        s = new Statement(fn, [val, '@0'], `@${resultAddress}`, line, [])
      } else if (assignables.has('variable')) {
        throw new Error('This should have been squashed')
      }
    } else if (statement.has('assignments')) {
      const asgn = statement.get('assignments')
      const resultAddress = localMem[asgn.get('decname').t.trim()]
      localMemToLine[resultAddress] = line
      const assignables = asgn.get('assignables')
      if (assignables.has('functions')) {
        throw new Error("This shouldn't be possible!")
      } else if (assignables.has('calls')) {
        const call = assignables.get('calls')
        const fnName = call.get('variable').t.trim()
        const vars = (call.has('calllist') ? call.get('calllist').getAll() : []).map(
          v => v.get('variable').t.trim()
        )
        const hasClosureArgs = isClosure && vars.length > 0
        const args = vars.map(v => {
          if (localMem.hasOwnProperty(v)) {
            return localMem[v]
          } else if (globalMem.hasOwnProperty(v)) {
            return globalMem[v]
          } else if (Object.keys(globalMem).some(k => closureScope.map(s => v + s).includes(k))) {
            return globalMem[
              closureScope.map(s => v + s).find(k => Object.keys(globalMem).includes(k))
            ]
          } else if (hasClosureArgs) {
            return CLOSURE_ARG_MEM_START + BigInt(1) + BigInt(fnArgs.indexOf(v))
          } else return v
        }).map(a => typeof a === 'string' ? a : `@${a}`)
        while (args.length < 2) args.push('@0')
        s = new Statement(fnName, args as [string, string], `@${resultAddress}`, line, [])
      } else if (assignables.has('value')) {
        // Only required for `let` statements
        let fn: string
        let val: string
        // TODO: Relying on little-endian trimming integers correctly and doesn't support float32
        // correctly. Need to find the correct type data from the original variable.
        const valStr = assignables.t
        if (valStr[0] === '"' || valStr[0] === "'") { // It's a string, which doesn't work here...
          fn = 'setestr'
          val = '0i64'
        } else if (valStr[0] === 't' || valStr[0] === 'f') { // It's a bool
          fn = 'setbool'
          val = assignables.t === 'true' ? '1i8' : '0i8' // Bools are bytes in the runtime
        } else if (valStr.indexOf('.') > -1) { // It's a floating point number, assume 64-bit
          fn = 'setf64'
          val = valStr + 'f64'
        } else { // It's an integer. i64 will "work" for now
          fn = 'seti64'
          val = valStr + 'i64'
        }
        s = new Statement(fn, [val, '@0'], `@${resultAddress}`, line, [])
      } else if (assignables.has('variable')) {
        throw new Error('This should have been squashed')
      }
    } else if (statement.has('calls')) {
      const call = statement.get('calls')
      const fnName = call.get('variable').t.trim()
      const vars = (call.has('calllist') ? call.get('calllist').getAll() : []).map(
        v => v.get('variable').t.trim()
      )
      const hasClosureArgs = isClosure && vars.length > 0
      const args = vars.map(v => {
        if (localMem.hasOwnProperty(v)) {
          return localMem[v]
        } else if (globalMem.hasOwnProperty(v)) {
          return globalMem[v]
        } else if (Object.keys(globalMem).some(k => closureScope.map(s => v + s).includes(k))) {
          return globalMem[
            closureScope.map(s => v + s).find(k => Object.keys(globalMem).includes(k))
          ]
        } else if (hasClosureArgs) {
          return CLOSURE_ARG_MEM_START + BigInt(1) + BigInt(fnArgs.indexOf(v))
        } else return v
      }).map(a => typeof a === 'string' ? a : `@${a}`)
      while (args.length < 3) args.push('@0')
      s = new Statement(fnName, args as [string, string, string], null, line, [])
    } else if (statement.has('emits')) {
      const emit = statement.get('emits')
      const evtName = emit.get('variable').t.trim()
      const payloadVar = emit.has('value') ? emit.get('value').t.trim() : undefined
      const payload = !payloadVar ?
        0 :
        localMem.hasOwnProperty(payloadVar) ?
          localMem[payloadVar] :
          globalMem.hasOwnProperty(payloadVar) ?
            globalMem[payloadVar] :
            payloadVar
      s = new Statement(
        'emit',
        [evtName, typeof payload === 'string' ? payload : `@${payload}`],
        null,
        line,
        [],
      )
    } else if (statement.has('exits')) {
      const exit = statement.get('exits')
      const exitVar = exit.get('variable').t.trim()
      let exitVarType = localMem.hasOwnProperty(exitVar) ? 'variable' : (
        globalMem.hasOwnProperty(exitVar) && typeof globalMem[exitVar] !== 'string' ?
        'fixed' : 'variable'
      )
      const vars = [exitVar]
      const args = vars.map(v => {
        if (localMem.hasOwnProperty(v)) {
          return localMem[v]
        } else if (globalMem.hasOwnProperty(v)) {
          return globalMem[v]
        } else if (Object.keys(globalMem).some(k => closureScope.map(s => v + s).includes(k))) {
          return globalMem[
            closureScope.map(s => v + s).find(k => Object.keys(globalMem).includes(k))
          ]
        } else if (hasClosureArgs) {
          return CLOSURE_ARG_MEM_START + BigInt(1) + BigInt(fnArgs.indexOf(v))
        } else return v
      }).map(a => typeof a === 'string' ? a : `@${a}`)
      while (args.length < 2) args.push('@0')
      const ref = exitVarType === 'variable' ? 'refv' : 'reff'
      s = new Statement(ref, args as [string, string], `@${CLOSURE_ARG_MEM_START}`, line, [])
    }
    vec.push(s)
    line += 1
  }
  return vec
}

class Block {
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
    deps: string[]
  ) {
    this.type = type
    this.name = name
    this.memSize = memSize
    this.statements = statements
    this.deps = deps
  }

  toString() {
    let b = `${this.type} for ${this.name} with size ${this.memSize}\n`
    this.statements.forEach(s => b += `  ${s.toString()}\n`)
    return b
  }
}

const loadHandlers = (handlers: LPNode[], handlerMem: object, globalMem: object) => {
  const vec = []
  const recs = handlers.filter(h => h.get() instanceof NamedAnd)
  for (let i = 0; i < recs.length; i++) {
    const handler = recs[i].get()
    const eventName = handler.get('variable').t.trim()
    const memSize = handlerMem[i].memSize
    const localMem = handlerMem[i].addressMap
    const h = new Block('handler', eventName, memSize, loadStatements(
      handler.get('functions').get('functionbody').get('statements').getAll(),
      localMem,
      globalMem,
      handler.get('functions'),
      eventName,
      false,
      [],
    ), [])
    vec.push(h)
  }
  return vec
}

const loadClosures = (closures: any[], globalMem: object) => {
  const vec = []
  for (let i = 0; i < closures.length; i++) {
    const closure = closures[i]
    const eventName = closure.name
    const memSize = closure.closureMem.memSize
    const localMem = closure.closureMem.addressMap
    const c = new Block('closure', eventName, memSize, loadStatements(
      closure.statements,
      localMem,
      globalMem,
      closure.fn,
      eventName,
      true,
      closure.scope,
    ), [])
    vec.push(c)
  }
  return vec
}

// Perform basic dependency stitching within a single block, but also attach unknown dependencies
// to the block object for later "stitching"
const innerBlockDeps = (block: Block) => {
  const depMap = {}
  let lastEmit = null
  const statements = block.statements
  for (const s of statements) {
    for (const a of s.inArgs) {
      if (depMap.hasOwnProperty(a)) {
        s.deps.push(depMap[a])
      } else if (/^@/.test(a)) {
        block.deps.push(a)
      }
    }
    if (s.fn === 'emit') {
      if (lastEmit !== null) {
        s.deps.push(lastEmit)
      }
      lastEmit = s.line
    }
    if (s.outArg !== null) {
      depMap[s.outArg] = s.line
    }
  }
  return block
}

// Use the unknown dependencies attached to the block scope and attach them in the outer level
// TODO: Handle dependencies many nested levels deep, perhaps with an iterative approach?
const closureDeps = (blocks: Block[]) => {
  const blockMap = {}
  for (const b of blocks) {
    blockMap[b.name] = b
  }
  const blockNames = Object.keys(blockMap)
  for (const b of blocks) {
    let argMap = {}
    for (const s of b.statements) {
      if (s.outArg !== null) {
        argMap[s.outArg] = s.line
      }
      for (const a of s.inArgs) {
        if (blockNames.includes(a)) {
          const blockDeps = blockMap[a].deps
          for (const bd of blockDeps) {
            if (argMap.hasOwnProperty(bd)) {
              s.deps.push(argMap[bd])
            }
          }
        }
      }
      s.deps = [...new Set(s.deps)] // Dedupe the final dependencies list
    }
  }
  return blocks
}

const ammToAga = (amm: LPNode) => {
  // Declare the AGA header
  let outStr = 'Alan Graphcode Assembler v0.0.1\n\n'
  // Get the global memory and the memory address map (var name to address ID)
  const addressMap = {}
  const globalMem = loadGlobalMem(amm.get('globalMem').getAll(), addressMap)
  if (Object.keys(globalMem).length > 0) {
    // Output the global memory
    outStr += 'globalMem\n'
    Object.keys(globalMem).forEach(addr => outStr += `  ${addr}: ${globalMem[addr]}\n`)
    outStr += '\n'
  }
  // Load the events, get the event id offset (for reuse with closures) and the event declarations
  let eventDecs = loadEventDecs(amm.get('eventDec').getAll())
  // Determine the amount of memory to allocate per handler and map declarations to addresses
  const handlerMem = getHandlersMem(amm.get('handlers').getAll())
  // const depGraph = getDepGraph(amm.get('handlers').getAll())
  const closures = extractClosures(amm.get('handlers').getAll(), handlerMem, eventDecs, addressMap)
  // Make sure closures are accessible as addresses for statements to use
  closures.forEach((c: any) => addressMap[c.name] = c.name)
  // Then output the custom events, which may include closures, if needed
  if (Object.keys(eventDecs).length > 0) {
    outStr += 'customEvents\n'
    Object.keys(eventDecs).forEach(evt => outStr += `  ${evt}: ${eventDecs[evt]}\n`)
    outStr += '\n'
  }
  // Load the handlers and load the closures (as handlers) if present
  const handlerVec = loadHandlers(amm.get('handlers').getAll(), handlerMem, addressMap)
  const closureVec = loadClosures(closures, addressMap)
  const blockVec = closureDeps([...handlerVec, ...closureVec].map(b => innerBlockDeps(b)))
    .map(b => b.toString())
  outStr += blockVec.join('\n')
  return outStr
}

export const fromFile = (filename: string) => {
  const lp = new LP(filename)
  const ast = amm.apply(lp)
  if (ast instanceof LPError) {
    throw new Error(ast.msg)
  }
  return ammToAga(ast)
}
export const fromString = (str: string) => {
  const lp = LP.fromText(str)
  const ast = amm.apply(lp)
  if (ast instanceof LPError) {
    throw new Error(ast.msg)
  }
  return ammToAga(ast)
}
