const Ast = require('../amm/Ast')

// This project depends on BigNum and associated support in Node's Buffer, so must be >= Node 10.20
// and does not work in the browser. It would be possible to implement a browser-compatible version
// but there is no need for it and it would make it harder to work with.
const header   = Buffer.from('agc00001', 'utf8').readBigUInt64LE(0)
const eventdd  = Buffer.from('eventdd:', 'utf8').readBigUInt64LE(0)
const handlerd = Buffer.from('handler:', 'utf8').readBigUInt64LE(0)
const lineno   = Buffer.from('lineno: ', 'utf8').readBigUInt64LE(0)
const emitd    = Buffer.from('emit to:', 'utf8').readBigUInt64LE(0)

const ceil8 = n => Math.ceil(n / 8) * 8
const int64ToUint64 = n => {
  const buf = Buffer.alloc(8)
  buf.writeBigInt64LE(n, 0)
  return buf.readBigUInt64LE(0)
}

const loadGlobalMem = (globalMemAst, addressMap) => {
  const globalMem = []
  let currentOffset = -1n
  for (const globalConst of globalMemAst) {
    let val
    switch (globalConst.fulltypename().getText().trim()) {
    case "int64":
      val = BigInt(globalConst.assignables().getText())
      globalMem.push(val)
      addressMap[globalConst.decname().getText()] = int64ToUint64(currentOffset)
      currentOffset -= 8n
      break
    case "float64":
      const buf = Buffer.alloc(8)
      buf.writeDoubleLE(parseFloat(globalConst.assignables().getText()))
      val = buf.readBigUInt64LE(0)
      globalMem.push(val)
      addressMap[globalConst.decname().getText()] = int64ToUint64(currentOffset)
      currentOffset -= 8n
      break
    case "string":
      let str
      try {                                                                                               
        str = JSON.parse(globalConst.assignables().getText()) // Will fail on strings with escape chars
      } catch (e) {
        // Hackery to get these strings to work
        str = JSON.stringify(globalConst.assignables().getText().replace(/^["']/, '').replace(/["']$/, ''))
      }
      let len = BigInt(ceil8(str.length) + 8)
      val = Buffer.alloc(Number(len))
      val.writeBigInt64LE(BigInt(str.length), 0)
      for (let i = 8; i < str.length + 8; i++) {
        val.writeInt8(str.charCodeAt(i - 8), i)
      }
      for (let i = 0; i < Number(len) / 8; i++) {
        globalMem.push(val.readBigUInt64LE(i * 8))
      }
      addressMap[globalConst.decname().getText()] = int64ToUint64(currentOffset)
      currentOffset -= len
      break
    case "bool":
      val = globalConst.assignables().getText() == "true" ? 1n : 0n
      globalMem.push(val)
      addressMap[globalConst.decname().getText()] = int64ToUint64(currentOffset)
      currentOffset -= 8n
      break
    default:
      console.error(globalConst.fulltypename().getText() + " not yet implemented")
      process.exit(1)
    }
  }
  return globalMem
}

const loadEventDecs = (eventAst, eventLookup) => {
  let customEventIdOffset = 0n
  const eventMem = []
  for (const evt of eventAst) {
    const evtName = evt.typename().getText().trim()
    const evtSize = evtName === "void" ? 0n : (evtName === "string" ? int64ToUint64(-1n) : 8n);
    eventMem.push(eventdd, customEventIdOffset, evtSize)
    eventLookup[evt.VARNAME().getText().trim()] = {
      eventId: customEventIdOffset,
    }
    customEventIdOffset++
  }
  return [customEventIdOffset, eventMem]
}

const getFunctionbodyMem = (functionbody) => {
  let memSize = 0n
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
          memSize += 8n
        }
      } else {
        addressMap[statement.declarations().letdeclaration().decname().getText().trim()] = memSize
        memSize += 8n
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
    handlerMem.memSize += 8n
    Object.keys(handlerMem.addressMap).forEach(name => handlerMem.addressMap[name] += 8n)
    handlerMem.addressMap[handler.functions().VARNAME().getText().trim()] = 0n
  }
  return handlerMem
})

const closuresFromDeclaration = (declaration, closureMem, customEventIdOffset) => {
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
    (s, i) => closuresFromDeclaration(s.declarations(), closureMem, customEventIdOffset + BigInt(i))
  ).reduce((obj, rec) => ({
    ...obj,
    ...rec,
  }), {})
  customEventIdOffset += BigInt(allStatements.length - statements.length)

  return {
    [name]: {
      statements,
      closureMem,
      eventId: customEventIdOffset,
    },
    ...otherClosures,
  }
}

const extractClosures = (handlers, handlerMem, customEventIdOffset, closureMap) => {
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
          customEventIdOffset,
        )
        customEventIdOffset = Object.values(innerClosures)
          .map(c => c.eventId)
          .reduce((m, i) => i > m ? i : m, -1n)
        closures = {
          ...closures,
          ...innerClosures,
        }
        customEventIdOffset++
      }
    }
  }
  Object.keys(closures).forEach(name => closureMap[name] = closures[name].eventId)
  return Object.values(closures)
}

const fakeEventsForClosures = (closures) => {
  const vec = []
  for (const closure of closures) {
    vec.push(eventdd, closure.eventId, 0n)
  }
  return vec
}

const fill8 = name => {
  const buf = Buffer.alloc(8, ' '.charCodeAt(0))
  for (let i = 0; i < name.length; i++) {
    buf.writeInt8(name.charCodeAt(i), i)
  }
  return buf.readBigUInt64LE(0)
}

const loadStatements = (statements, localMem, globalMem, eventLookup, closureMap) => {
  let vec = []
  let line = 0n
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
    vec.push(lineno, line)
    if (statement.declarations()) {
      const dec = statement.declarations().constdeclaration() || statement.declarations().letdeclaration()
      const resultAddress = localMem[dec.decname().getText().trim()]
      localMemToLine[resultAddress] = line
      const assignables = dec.assignables()
      if (assignables.functions()) {
        console.error("This shouldn't be possible!")
        process.exit(2)
      } else if (assignables.calls()) {
        const call = assignables.calls()
        const fn = fill8(call.VARNAME().getText().trim())
        const vars = (call.calllist() ? call.calllist().VARNAME() : []).map(v => v.getText().trim())
        const args = vars.map(v => localMem.hasOwnProperty(v) ?
          localMem[v] :
          closureMap.hasOwnProperty(v) ?
            closureMap[v] :
            globalMem[v]
        )
        while (args.length < 2) args.push(0n)
        const deps = vars
          .filter(v => localMem.hasOwnProperty(v))
          .map(v => localMemToLine[localMem[v]])
          .filter(v => v !== undefined) // Filter out the handler arg from the dep list
        vec.push(BigInt(deps.length), ...deps, fn, ...args, resultAddress)
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
    } else if (statement.assignments()) {
      const asgn = statement.assignments()
      const resultAddress = localMem[asgn.decname().getText().trim()]
      localMemToLine[resultAddress] = asgn // This is safe because future references to this var
                                           // should depend on the mutated form

      const assignables = dec.assignables()
      if (assignables.functions()) {
        console.error("This shouldn't be possible!")
        process.exit(2)
      } else if (assignables.calls()) {
        const call = assignables.calls()
        const fn = fill8(call.VARNAME().getText().trim())
        const vars = (call.calllist() ? call.calllist().VARNAME() : [])
        const args = vars.map(v => v.getText().trim()).map(v => localMem.hasOwnProperty(v) ?
          localMem[v] :
          closureMap.hasOwnProperty(v) ?
            closureMap[v] :
            globalMem[v]
        )
        while (args.length < 2) args.push(0n)
        const deps = vars
          .filter(v => localMem.hasOwnProperty(v))
          .map(v => localMemToLine[localMem[v]])
          .filter(v => v !== undefined) // Filter out the handler arg from the dep list
        vec.push(BigInt(deps.length), ...deps, fn, ...args, resultAddress)
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
      const fn = fill8(call.VARNAME().getText().trim())
      const vars = (call.calllist() ? call.calllist().VARNAME() : [])
      const args = vars.map(v => v.getText().trim()).map(v => localMem.hasOwnProperty(v) ?
        localMem[v] :
        closureMap.hasOwnProperty(v) ?
          closureMap[v] :
          globalMem[v]
      )
      while (args.length < 2) args.push(0n)
      const deps = vars
        .filter(v => localMem.hasOwnProperty(v))
        .map(v => localMemToLine[localMem[v]])
        .filter(v => v !== undefined) // Filter out the handler arg from the dep list
      vec.push(BigInt(deps.length), ...deps, fn, ...args, 0n)
    } else if (statement.emits()) {
      const emit = statement.emits()
      const { eventId, } = eventLookup[emit.VARNAME(0).getText().trim()]
      const payloadVar = emit.VARNAME(1)
      const payload = !payloadVar ?
        0n :
        localMem.hasOwnProperty(payloadVar) ?
          localMem[payloadVar] :
          closureMap.hasOwnProperty(payloadVar) ?
            closureMap[payloadVar] :
            globalMem[payloadVar]
      const dep = !payloadVar ? [] :
        localMem.hasOwnProperty(payloadVar) ?
          [localMemToLine[payloadVar]].filter(v => v !== undefined) :
          []
      vec.push(BigInt(dep.length), ...dep, emitd, eventId, payload, 0n)
    }
    line += 1n
  }
  return vec
}

const loadHandlers = (handlers, handlerMem, globalMem, eventLookup, closureMap) => {
  const vec = []
  for (let i = 0; i < handlers.length; i++) {
    const handler = handlers[i]
    const { eventId } = eventLookup[handler.VARNAME().getText().trim()]
    const memSize = handlerMem[i].memSize
    const localMem = handlerMem[i].addressMap
    vec.push(handlerd, eventId, memSize)
    let line = 0n
    let localMemToLine = {}
    const statementVec = loadStatements(
      handler.functions().functionbody().statements(),
      localMem,
      globalMem,
      eventLookup,
      closureMap
    )
    vec.push(...statementVec)
  }
  return vec
}
          
const loadClosures = (closures, globalMem, eventLookup, closureMap) => {
  const vec = []
  for (let i = 0; i < closures.length; i++) {
    const closure = closures[i]
    const eventId = closure.eventId
    const memSize = closure.closureMem.memSize
    const localMem = closure.closureMem.addressMap
    vec.push(handlerd, eventId, memSize)
    let line = 0n
    let localMemToLine = {}
    const statementVec = loadStatements(
      closure.statements,
      localMem,
      globalMem,
      eventLookup,
      closureMap
    )
    vec.push(...statementVec)
  }
  return vec
}

const ammToAgc = (amm) => {
  // Declare the AGC header
  const vec = [header]
  // Get the global memory and the memory address map (var name to address ID)
  const addressMap = {}
  const globalMem = loadGlobalMem(amm.constdeclaration(), addressMap)
  // Compute the global memory size and declare that and add all of the global memory
  const memSize = BigInt(globalMem.length * 8)
  vec.push(memSize, ...globalMem)
  // Declare the event lookup table (event string to id) with the singular special event `"start"`
  const eventLookup = {
    _start: {
      eventId: (() => {
        const buf = Buffer.from('"start" ', 'utf8')
        buf.writeUInt8(0x80, 7)
        return buf.readBigUInt64LE(0)
      })(),
    },
  }
  // Load the events, get the event id offset (for reuse with closures) and the event declarations
  let [customEventIdOffset, eventDecs] = loadEventDecs(amm.events(), eventLookup)
  // Then add that to the output vector
  vec.push(...eventDecs)
  // Skipping types for now as exactly how we deal with them and what metadata the runtime needs is
  // not yet decided.

  // Determine the amount of memory to allocate per handler and map declarations to addresses
  const handlerMem = getHandlersMem(amm.handlers())
  const closureMap = {}
  const closures = extractClosures(amm.handlers(), handlerMem, customEventIdOffset, closureMap)
  // Generate event records for the closures for the runtime to register them to
  const fakeEvents = fakeEventsForClosures(closures)
  vec.push(...fakeEvents)
  // Load the handlers
  const handlerVec = loadHandlers(amm.handlers(), handlerMem, addressMap, eventLookup, closureMap)
  vec.push(...handlerVec)
  // And load the closures (as handlers)
  const closureVec = loadClosures(closures, addressMap, eventLookup, closureMap)
  vec.push(...closureVec)
  // All done, convert the BigInt array to a big buffer to write to a file
  const outBuf = Buffer.alloc(vec.length * 8)
  vec.forEach((n, i) => {
    outBuf.writeBigUInt64LE(n, i * 8)
  })
  return outBuf
}

module.exports = (filename) => ammToAgc(Ast.fromFile(filename))
module.exports.ammTextToAgc = (str) => ammToAgc(Ast.fromString(str))
