const Ast = require('../amm/Ast')

// This project depends on BigNum and associated support in Node's Buffer, so must be >= Node 10.20
// and does not work in the browser. It would be possible to implement a browser-compatible version
// but there is no need for it and it would make it harder to work with.

const loadGlobalMem = (globalMemAst, addressMap) => {
  const globalMem = {}
  let currentOffset = -1
  for (const globalConst of globalMemAst) {
    let val
    switch (globalConst.fulltypename().getText().trim()) {
    case "int64":
      val = BigInt(globalConst.assignables().getText())
      globalMem[currentOffset] = val
      addressMap[globalConst.decname().getText()] = currentOffset
      currentOffset -= 8
      break
    case "float64":
      val = parseFloat(globalConst.assignables().getText())
      globalMem[currentOffset] = val
      addressMap[globalConst.decname().getText()] = currentOffset
      currentOffset -= 8
      break
    case "string":
      val = globalConst.assignables().getText().trim()
      let len = val.length + 8
      globalMem[currentOffset] = val
      addressMap[globalConst.decname().getText()] = currentOffset
      currentOffset -= len
      break
    case "bool":
      val = globalConst.assignables().getText().trim()
      globalMem[currentOffset] = val
      addressMap[globalConst.decname().getText()] = currentOffset
      currentOffset -= 8
      break
    default:
      console.error(globalConst.fulltypename().getText() + " not yet implemented")
      process.exit(1)
    }
  }
  return globalMem
}

const loadEventDecs = (eventAst) => {
  const eventMem = {}
  for (const evt of eventAst) {
    const evtName = evt.VARNAME().getText().trim()
    const evtType = evt.typename().getText().trim()
    eventMem[evtName] = evtType
  }
  return eventMem
}

const getFunctionbodyMem = (functionbody) => {
  let memSize = 0
  const addressMap = {}
  for (const statement of functionbody.statements()) {
    if (statement.declarations()) {
      if (statement.declarations().constdeclaration()) {
        if (statement.declarations().constdeclaration().assignables().functions()) {
          // Because closures re-use their parent memory space, their own memory needs to be included
          const closureMem = getFunctionbodyMem(
            statement.declarations().constdeclaration().assignables().functions().functionbody()
          )
          Object.keys(closureMem.addressMap).forEach(
            name => addressMap[name] = closureMem.addressMap[name] + memSize
          )
          memSize += closureMem.memSize
        } else {
          addressMap[statement.declarations().constdeclaration().decname().getText().trim()] = memSize
          memSize += 8
        }
      } else {
        addressMap[statement.declarations().letdeclaration().decname().getText().trim()] = memSize
        memSize += 8
      }
    }
  }
  return {
    memSize,
    addressMap,
  }
}

const getHandlersMem = handlers => handlers.map(handler => {
  const handlerMem = getFunctionbodyMem(handler.functions().functionbody())
  if (handler.functions().VARNAME()) {
    // Increase the memory usage and shift *everything* down, then add the new address
    handlerMem.memSize += 8
    Object.keys(handlerMem.addressMap).forEach(name => handlerMem.addressMap[name] += 8)
    handlerMem.addressMap[handler.functions().VARNAME().getText().trim()] = 0
  }
  return handlerMem
})

const closuresFromDeclaration = (declaration, closureMem, eventDecs) => {
  const name = declaration.constdeclaration().decname().getText().trim()
  const allStatements = declaration
    .constdeclaration()
    .assignables()
    .functions()
    .functionbody()
    .statements()
  const statements = allStatements.filter(statement => !(statement.declarations() &&
    statement.declarations().constdeclaration() &&
    statement.declarations().constdeclaration().assignables().functions()
  ))
  const otherClosures = allStatements.filter(statement => statement.declarations() &&
    statement.declarations().constdeclaration() &&
    statement.declarations().constdeclaration().assignables().functions()
  ).map(
    s => closuresFromDeclaration(s.declarations(), closureMem, eventDecs)
  ).reduce((obj, rec) => ({
    ...obj,
    ...rec,
  }), {})
  eventDecs[name] = 'void'

  return {
    [name]: {
      name,
      statements,
      closureMem,
    },
    ...otherClosures,
  }
}

const extractClosures = (handlers, handlerMem, eventDecs) => {
  let closures = {}
  for (let i = 0; i < handlers.length; i++) {
    const closureMem = handlerMem[i]
    const handler = handlers[i]
    for (const statement of handler.functions().functionbody().statements()) {
      if (
        statement.declarations() &&
        statement.declarations().constdeclaration() &&
        statement.declarations().constdeclaration().assignables().functions()
      ) {
        // It's a closure, first try to extract any inner closures it may have
        const innerClosures = closuresFromDeclaration(
          statement.declarations(),
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

const loadStatements = (statements, localMem, globalMem) => {
  let vec = []
  let line = 0
  let localMemToLine = {}
  for (const statement of statements) {
    if (
      statement.declarations() &&
      statement.declarations().constdeclaration() &&
      statement.declarations().constdeclaration().assignables().functions()
    ) {
      // It's a closure, skip it
      continue
    }
    let s = `line ${line}`
    if (statement.declarations()) {
      const dec = statement.declarations().constdeclaration() || statement.declarations().letdeclaration()
      const resultAddress = localMem[dec.decname().getText().trim()]
      localMemToLine[dec.decname().getText().trim()] = line
      const assignables = dec.assignables()
      if (assignables.functions()) {
        console.error("This shouldn't be possible!")
        process.exit(2)
      } else if (assignables.calls()) {
        const call = assignables.calls()
        const fn = call.VARNAME().getText().trim()
        const vars = (call.calllist() ? call.calllist().VARNAME() : []).map(v => v.getText().trim())
        const args = vars.map(v => localMem.hasOwnProperty(v) ?
          localMem[v] :
          globalMem.hasOwnProperty(v) ?
            globalMem[v] :
            v
        )
        while (args.length < 2) args.push(0)
        const deps = vars
          .filter(v => localMem.hasOwnProperty(v))
          .map(v => localMemToLine[localMem[v]])
          .filter(v => v !== undefined) // Filter out the handler arg from the dep list
        if (deps.length > 0) {
          s += ` depends on [${deps.join(', ')}]`
        }
        s += `: ${fn}(${args.join(', ')}) -> ${resultAddress}`
      } else if (assignables.constants()) {
        // Only required for `let` statements
        let fn
        let val
        switch (dec.fulltypename().getText().trim()) {
        case 'int64':
          fn = 'seti64'
          val = assignables.getText()
          break
        case 'int32':
          fn = 'seti32'
          val = assignables.getText()
          break
        case 'int16':
          fn = 'seti16'
          val = assignables.getText()
          break
        case 'int8':
          fn = 'seti8'
          val = assignables.getText()
          break
        case 'float64':
          fn = 'setf64'
          val = assignables.getText()
          break
        case 'float32':
          fn = 'setf32'
          val = assignables.getText()
          break
        case 'bool':
          fn = 'setbool'
          val = assignables.getText()
          break
        case 'string':
          throw new Error('TODO: Decide if this is the responsibility of first or second stage')
          break
        default:
          throw new Error(`Unsupported variable type ${dec.fulltypename().getText()}`)
          break
        }
        s += `: ${fn}(${val}, 0) -> ${resultAddress}`
      } else if (assignables.objectliterals()) {
        console.error("Not yet implemented")
        process.exit(4)
      } else if (assignables.VARNAME()) {
        console.error("This should have been squashed")
        process.exit(5)
      }
    } else if (statement.assignments()) {
      const asgn = statement.assignments()
      const resultAddress = localMem[asgn.decname().getText().trim()]
      localMemToLine[resultAddress] = line
      const assignables = asgn.assignables()
      if (assignables.functions()) {
        console.error("This shouldn't be possible!")
        process.exit(2)
      } else if (assignables.calls()) {
        const call = assignables.calls()
        const fn = call.VARNAME().getText().trim()
        const vars = (call.calllist() ? call.calllist().VARNAME() : [])
        const args = vars.map(v => v.getText().trim()).map(v => localMem.hasOwnProperty(v) ?
          localMem[v] :
          globalMem.hasOwnProperty(v) ?
            globalMem[v] :
            v
        )
        while (args.length < 2) args.push(0)
        const deps = vars
          .filter(v => localMem.hasOwnProperty(v))
          .map(v => localMemToLine[localMem[v]])
          .filter(v => v !== undefined) // Filter out the handler arg from the dep list
        if (deps.length > 0) {
          s += ` depends on [${deps.join(', ')}]`
        }
        s += `: ${fn}(${args.join(', ')}) -> ${resultAddress}`
      } else if (assignables.constants()) {
        console.error("This should have been hoisted")
        process.exit(3)
      } else if (assignables.objectliterals()) {
        console.error("Not yet implemented")
        process.exit(4)
      } else if (assignables.VARNAME()) {
        console.error("This should have been squashed")
        process.exit(5)
      }
    } else if (statement.calls()) {
      const call = statement.calls()
      const fn = call.VARNAME().getText().trim()
      const vars = (call.calllist() ? call.calllist().VARNAME() : [])
      const args = vars.map(v => v.getText().trim()).map(v => localMem.hasOwnProperty(v) ?
        localMem[v] :
        globalMem.hasOwnProperty(v) ?
          globalMem[v] :
          v
      )
      while (args.length < 2) args.push(0)
      const deps = vars
        .filter(v => localMem.hasOwnProperty(v))
        .map(v => localMemToLine[localMem[v]])
        .filter(v => v !== undefined) // Filter out the handler arg from the dep list
      if (deps.length > 0) {
        s += ` depends on [${deps.join(', ')}]`
      }
      s += `: ${fn}(${args.join(', ')}) -> 0`
    } else if (statement.emits()) {
      const emit = statement.emits()
      const evtName = emit.VARNAME(0).getText().trim()
      const payloadVar = emit.VARNAME(1).getText().trim()
      const payload = !payloadVar ?
        0 :
        localMem.hasOwnProperty(payloadVar) ?
          localMem[payloadVar] :
          globalMem.hasOwnProperty(payloadVar) ?
            globalMem[payloadVar] :
            payloadVar
      const deps = !payloadVar ? [] :
        localMem.hasOwnProperty(payloadVar) ?
          [localMemToLine[payloadVar]].filter(v => v !== undefined) :
          []
      if (deps.length > 0) {
        s += ` depends on [${deps.join(', ')}]`
      }
      s += `: emit to:(${evtName}, ${payload}) -> 0`
    }
    vec.push(s)
    line += 1
  }
  return vec
}

const loadHandlers = (handlers, handlerMem, globalMem) => {
  const vec = []
  for (let i = 0; i < handlers.length; i++) {
    const handler = handlers[i]
    const eventName = handler.VARNAME().getText().trim()
    const memSize = handlerMem[i].memSize
    const localMem = handlerMem[i].addressMap
    let h = `handler: ${eventName} size ${memSize}\n`
    const statements = loadStatements(
      handler.functions().functionbody().statements(),
      localMem,
      globalMem,
    )
    statements.forEach(s => h += `  ${s}\n`)
    vec.push(h)
  }
  return vec 
}
          
const loadClosures = (closures, globalMem) => {
  const vec = []
  for (let i = 0; i < closures.length; i++) {
    const closure = closures[i]
    const eventName = closure.name
    const memSize = closure.closureMem.memSize
    const localMem = closure.closureMem.addressMap
    let c = `handler: ${eventName} size ${memSize}\n`
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

const ammToAga = (amm) => {
  // Declare the AGA header
  let outStr = 'Alan Graphcode Assembler v0.0.1\n\n'
  // Get the global memory and the memory address map (var name to address ID)
  const addressMap = {}
  const globalMem = loadGlobalMem(amm.constdeclaration(), addressMap)
  // Output the global memory
  outStr += 'globalMem\n'
  Object.keys(globalMem).forEach(addr => outStr += `  ${addr}: ${globalMem[addr]}\n`)
  outStr += '\n'
  // Load the events, get the event id offset (for reuse with closures) and the event declarations
  let eventDecs = loadEventDecs(amm.events())
  // Skipping types for now as exactly how we deal with them and what metadata the runtime needs is
  // not yet decided.
  // Determine the amount of memory to allocate per handler and map declarations to addresses
  const handlerMem = getHandlersMem(amm.handlers())
  const closures = extractClosures(amm.handlers(), handlerMem, eventDecs)
  // Then output the custom events, which may include closures, if needed
  if (Object.keys(eventDecs).length > 0) {
    outStr += 'customEvents\n'
    Object.keys(eventDecs).forEach(evt => outStr += `  ${evt}: ${eventDecs[evt]}\n`)
    outStr += '\n'
  }
  // Load the handlers
  const handlerVec = loadHandlers(amm.handlers(), handlerMem, addressMap)
  outStr += handlerVec.join('\n')
  // And load the closures (as handlers) if present
  const closureVec = loadClosures(closures, addressMap)
  if (closureVec.length > 0) {
    outStr += '\n'
    outStr += closureVec.join('\n')
  }
  return outStr
}

module.exports = (filename) => ammToAga(Ast.fromFile(filename))
module.exports.ammTextToAga = (str) => ammToAga(Ast.fromString(str))
