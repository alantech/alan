import {
  LP,
  LPish,
  NamedAnd,
  NulLP,
} from '../lp'

import amm from '../amm'

// This project depends on BigNum and associated support in Node's Buffer, so must be >= Node 10.20
// and does not work in the browser. It would be possible to implement a browser-compatible version
// but there is no need for it and it would make it harder to work with.
const ceil8 = (n: number) => Math.ceil(n / 8) * 8

const loadGlobalMem = (globalMemAst: LPish[], addressMap: object) => {
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
      console.error(rec.get('fulltypename').t + " not yet implemented")
      process.exit(1)
    }
  }
  return globalMem
}

const loadEventDecs = (eventAst: LPish[]) => {
  const eventMem = {}
  for (const evt of eventAst) {
    const rec = evt.get()
    if (!(rec instanceof NamedAnd)) continue
    const evtName = rec.get('variable').t.trim()
    // TODO: Add event support for Arrays
    const evtSize = rec.get('fulltypename').t.trim() === 'void' ? 0 :
      rec.get('fulltypename').t.trim() === 'string' ? -1 : 8
    eventMem[evtName] = evtSize
  }
  return eventMem
}

const getFunctionbodyMem = (functionbody: LPish) => {
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
          memSize += 8
        }
      } else {
        addressMap[
          statement.get('declarations').get('letdeclaration').get('decname').t.trim()
        ] = memSize
        memSize += 8
      }
    }
  }
  return {
    memSize,
    addressMap,
  }
}

const getHandlersMem = (handlers: LPish[]) => handlers
  .map(h => h.get())
  .filter(h => h instanceof NamedAnd)
  .map(handler => {
    const handlerMem = getFunctionbodyMem(handler.get('functions').get('functionbody'))
    if (!(handler.get('functions').get('arg') instanceof NulLP)) {
      // Increase the memory usage and shift *everything* down, then add the new address
      handlerMem.memSize += 8
      Object.keys(handlerMem.addressMap).forEach(name => handlerMem.addressMap[name] += 8)
      handlerMem.addressMap[handler.get('functions').get('arg').get('variable').t.trim()] = 0
    }
    return handlerMem
  })

const closuresFromDeclaration = (declaration: LPish, closureMem: object, eventDecs: object) => {
  const name = declaration.get('constdeclaration').get('decname').t.trim()
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
    s => closuresFromDeclaration(s.get('declarations'), closureMem, eventDecs)
  ).reduce((obj, rec) => ({
    ...obj,
    ...rec,
  }), {})
  eventDecs[name] = 0

  return {
    [name]: {
      name,
      statements,
      closureMem,
    },
    ...otherClosures,
  }
}

const extractClosures = (handlers: LPish[], handlerMem: object, eventDecs: object) => {
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

const loadStatements = (statements: LPish[], localMem: object, globalMem: object) => {
  let vec = []
  let line = 0
  let localMemToLine = {}
  for (const statement of statements) {
    if (statement.has('whitespace')) continue
    if (
      statement.has('declarations') &&
      statement.get('declarations').has('constdeclaration') &&
      statement.get('declarations').get('constdeclaration').get('assignables').has('functions')
    ) {
      // It's a closure, skip it
      continue
    }
    let s = ''
    if (statement.has('declarations')) {
      const dec = statement.get('declarations').has('constdeclaration') ?
        statement.get('declarations').get('constdeclaration') :
        statement.get('declarations').get('letdeclaration')
      let resultAddress = localMem[dec.get('decname').t.trim()]
      localMemToLine[dec.get('decname').t.trim()] = line
      const assignables = dec.get('assignables')
      if (assignables.has('functions')) {
        console.error("This shouldn't be possible!")
        process.exit(2)
      } else if (assignables.has('calls')) {
        const call = assignables.get('calls')
        const fn = call.get('variable').t.trim()
        // TODO: Absolute hackery that must be removed soon
        if (fn === 'pusharr') {
          switch (dec.get('fulltypename').t.trim()) {
          case 'int8':
          case 'bool':
            resultAddress = 1
            break
          case 'int16':
            resultAddress = 2
            break
          case 'int32':
          case 'float32':
            resultAddress = 4
            break
          case 'int64':
          case 'float64':
            resultAddress = 8
            break
          default:
            resultAddress = 0
            break
          }
        }
        const vars = (call.has('calllist') ? call.get('calllist').getAll() : []).map(
          v => v.get('variable').t.trim()
        )
        const args = vars.map(v => localMem.hasOwnProperty(v) ?
          localMem[v] :
          globalMem.hasOwnProperty(v) ?
            globalMem[v] :
            v
        ).map(a => typeof a === 'string' ? a : `@${a}`)
        while (args.length < 2) args.push('@0')
        const deps = vars
          .filter(v => localMem.hasOwnProperty(v))
          .map(v => localMemToLine[localMem[v]])
          .filter(v => v !== undefined) // Filter out the handler arg from the dep list
          .map(v => `#${v}`)
        s += `@${resultAddress} = ${fn}(${args.join(', ')}) #${line}`
        if (deps.length > 0) {
          s += ` <- [${deps.join(', ')}]`
        }
      } else if (assignables.has('constants')) {
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
        s += `@${resultAddress} = ${fn}(${val}, @0) #${line}`
      } else if (assignables.has('variable')) {
        console.error("This should have been squashed")
        process.exit(5)
      }
    } else if (statement.has('assignments')) {
      const asgn = statement.get('assignments')
      const resultAddress = localMem[asgn.get('decname').t.trim()]
      localMemToLine[resultAddress] = line
      const assignables = asgn.get('assignables')
      if (assignables.has('functions')) {
        console.error("This shouldn't be possible!")
        process.exit(2)
      } else if (assignables.has('calls')) {
        const call = assignables.get('calls')
        const fn = call.get('variable').t.trim()
        const vars = (call.has('calllist') ? call.get('calllist').getAll() : []).map(
          v => v.get('variable').t.trim()
        )
        const args = vars.map(v => localMem.hasOwnProperty(v) ?
          localMem[v] :
          globalMem.hasOwnProperty(v) ?
            globalMem[v] :
            v
        ).map(a => typeof a === 'string' ? a : `@${a}`)
        while (args.length < 2) args.push('@0')
        const deps = vars
          .filter(v => localMem.hasOwnProperty(v))
          .map(v => localMemToLine[localMem[v]])
          .filter(v => v !== undefined) // Filter out the handler arg from the dep list
          .map(v => `#${v}`)
        s += `@${resultAddress} = ${fn}(${args.join(', ')}) #${line}`
        if (deps.length > 0) {
          s += ` <- [${deps.join(', ')}]`
        }
      } else if (assignables.has('constants')) {
        console.error("This should have been hoisted")
        process.exit(3)
      } else if (assignables.has('variable')) {
        console.error("This should have been squashed")
        process.exit(5)
      }
    } else if (statement.has('calls')) {
      const call = statement.get('calls')
      const fn = call.get('variable').t.trim()
      const vars = (call.has('calllist') ? call.get('calllist').getAll() : []).map(v => v.t.trim())
      const args = vars.map(v => localMem.hasOwnProperty(v) ?
        localMem[v] :
        globalMem.hasOwnProperty(v) ?
          globalMem[v] :
          v
      ).map(a => typeof a === 'string' ? a : `@${a}`)
      while (args.length < 2) args.push('0')
      const deps = vars
        .filter(v => localMem.hasOwnProperty(v))
        .map(v => localMemToLine[localMem[v]])
        .filter(v => v !== undefined) // Filter out the handler arg from the dep list
        .map(v => `#${v}`)
      s += `${fn}(${args.join(', ')}) #${line}`
      if (deps.length > 0) {
        s += ` <- [${deps.join(', ')}]`
      }
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
      const deps = (
        !payloadVar ? [] :
          localMem.hasOwnProperty(payloadVar) ?
            [localMemToLine[payloadVar]].filter(v => v !== undefined) :
            []
      ).map(v => `#${v}`)
      s += `emit(${evtName}, ${typeof payload === 'string' ? payload : `@${payload}`}) #${line}`
      if (deps.length > 0) {
        s += ` <- [${deps.join(', ')}]`
      }
    }
    vec.push(s)
    line += 1
  }
  return vec
}

const loadHandlers = (handlers: LPish[], handlerMem: object, globalMem: object) => {
  const vec = []
  const recs = handlers.filter(h => h.get() instanceof NamedAnd)
  for (let i = 0; i < recs.length; i++) {
    const handler = recs[i].get()
    const eventName = handler.get('variable').t.trim()
    const memSize = handlerMem[i].memSize
    const localMem = handlerMem[i].addressMap
    let h = `handler for ${eventName} with size ${memSize}\n`
    const statements = loadStatements(
      handler.get('functions').get('functionbody').get('statements').getAll(),
      localMem,
      globalMem,
    )
    statements.forEach(s => h += `  ${s}\n`)
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
    let c = `handler for ${eventName} with size ${memSize}\n`
    const statements = loadStatements(
      closure.statements,
      localMem,
      globalMem,
    )
    statements.forEach(s => c += `  ${s}\n`)
    vec.push(c)
  }
  return vec
}

const ammToAga = (amm: LPish) => {
  // Declare the AGA header
  let outStr = 'Alan Graphcode Assembler v0.0.1\n\n'
  // Get the global memory and the memory address map (var name to address ID)
  const addressMap = {}
  const globalMem = loadGlobalMem(amm.get('globalMem').getAll(), addressMap)
  // Output the global memory
  outStr += 'globalMem\n'
  Object.keys(globalMem).forEach(addr => outStr += `  ${addr}: ${globalMem[addr]}\n`)
  outStr += '\n'
  // Load the events, get the event id offset (for reuse with closures) and the event declarations
  let eventDecs = loadEventDecs(amm.get('eventDec').getAll())
  // Determine the amount of memory to allocate per handler and map declarations to addresses
  const handlerMem = getHandlersMem(amm.get('handlers').getAll())
  const closures = extractClosures(amm.get('handlers').getAll(), handlerMem, eventDecs)
  // Then output the custom events, which may include closures, if needed
  if (Object.keys(eventDecs).length > 0) {
    outStr += 'customEvents\n'
    Object.keys(eventDecs).forEach(evt => outStr += `  ${evt}: ${eventDecs[evt]}\n`)
    outStr += '\n'
  }
  // Load the handlers
  const handlerVec = loadHandlers(amm.get('handlers').getAll(), handlerMem, addressMap)
  outStr += handlerVec.join('\n')
  // And load the closures (as handlers) if present
  const closureVec = loadClosures(closures, addressMap)
  if (closureVec.length > 0) {
    outStr += '\n'
    outStr += closureVec.join('\n')
  }
  return outStr
}

export const fromFile = (filename: string) => {
  const lp = new LP(filename)
  const ast = amm.apply(lp)
  if (ast instanceof Error) {
    console.error(ast)
    process.exit(1)
  }
  return ammToAga(ast)
}
export const fromString = (str: string) => {
  const lp = LP.fromText(str)
  const ast = amm.apply(lp)
  if (ast instanceof Error) {
    console.error(ast)
    process.exit(1)
  }
  return ammToAga(ast)
}
