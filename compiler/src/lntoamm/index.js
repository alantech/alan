const fs = require('fs')
const path = require('path')

const { v4: uuid, } = require('uuid')

const opcodes = require('./opcodes')
const Ast = require('./Ast')
const Std = require('./Std')
const Module = require('./Module')
const Event = require('./Event')
const UserFunction = require('./UserFunction')
const Microstatement = require('./Microstatement')
const StatementType = require('./StatementType')

const hoistConst = (microstatements, constantDedupeLookup, constants) => {
  let i = 0
  while (i < microstatements.length) {
    const m = microstatements[i]
    if (
      m.statementType === StatementType.CONSTDEC &&
      m.fns.length === 0
    ) {
      const original = constantDedupeLookup[m.inputNames[0]]
      if (!original) {
        constants.add(m)
        if (!m.outputType.builtIn) {
          eventTypes.add(m.outputType)
        }
        microstatements.splice(i, 1)
        constantDedupeLookup[m.inputNames[0]] = m
      } else {
        // Rewrite with the replaced name
        for(let j = i + 1; j < microstatements.length; j++) {
          const n = microstatements[j]
          for (let k = 0; k < n.inputNames.length; k++) {
            if (n.inputNames[k] === m.outputName) {
              n.inputNames[k] = original.outputName
            }
          }
        }
        microstatements.splice(i, 1);
      }
    } else if (m.statementType === StatementType.CLOSURE) {
      hoistConst(m.closureStatements, constantDedupeLookup, constants)
      i++
    } else {
      i++
    }
  }
}

const moduleAstsFromFile = (filename) => {
  let moduleAsts = {}
  let paths = []
  const rootPath = fs.realpathSync(filename)
  paths.push(rootPath)
  while (paths.length > 0) {
    const modulePath = paths.shift()
    let module = null
    try {
      module = Ast.fromFile(modulePath)
    } catch (e) {
      console.error("Could not load " + modulePath)
      console.error(e)
      throw e
    }
    moduleAsts[modulePath] = module
    const imports = Ast.resolveImports(modulePath, module)
    for (let i = 0; i < imports.length; i++) {
      if (!moduleAsts[imports[i]] && !(imports[i].substring(0, 5) === "@std/")) {
        paths.push(imports[i])
      }
    }
  }
  return moduleAsts
}

const moduleAstsFromString = (str) => {
  let moduleAsts = {}
  const fakeRoot = '/fake/root/test.ln'
  let module = null
  try {
    module = Ast.fromString(str)
  } catch (e) {
    console.error("Could not load test.ln")
    console.error(e)
    throw e
  }
  moduleAsts[fakeRoot] = module
  const imports = Ast.resolveImports(fakeRoot, module)
  for (let i = 0; i < imports.length; i++) {
    if (moduleAsts[imports[i]] === null && !(imports[i].substring(0, 5) === "@std/")) {
      console.error('Only @std imports allowed in the playground')
      throw new Error('Import declaration error')
    }
  }
  return moduleAsts
}

const ammFromModuleAsts = (moduleAsts) => {
  // Load the standard library
  Std.loadStdModules(Module.getAllModules())
  const rootScope = Module.getAllModules()['<root>'].exportScope
  // Load all modules
  const modules = Module.modulesFromAsts(moduleAsts, rootScope)

  // This implicitly populates the `allEvents` static property on the `Event` type, which we can
  // use to serialize out the definitions, skipping the built-in events. In the process we're need
  // to check a hashset for duplicate event names and rename as necessary. We also need to get the
  // list of user-defined types that we need to emit.
  let eventNames = new Set()
  let eventTypeNames = new Set()
  let eventTypes = new Set()
  for (const evt of Event.allEvents) {
    // Skip built-in events
    if (evt.builtIn) continue
    // Check if there's a collision
    if (eventNames.has(evt.name)) {
      // We modify the event name by attaching a UUIDv4 to it
      evt.name = evt.name + "_" + uuid().replace(/-/g, "_")
    }
    // Add the event to the list
    eventNames.add(evt.name)
    // Now on to event type processing
    const type = evt.type
    // Skip built-in types, too
    if (type.builtIn) continue
    // Check if there's a collision
    if (eventTypeNames.has(type.typename)) {
      // An event type may be seen multiple times, make sure this is an actual collision
      if (eventTypes.has(type)) continue // This event was already processed, so we're done
      // Modify the type name by attaching a UUIDv4 to it
      type.typename = type.typename + "_" + uuid().replace(/-/g, "_")
    }
    // Add the type to the list
    eventTypeNames.add(type.typename)
    eventTypes.add(type)
    // Determine if the event type is a union type, if so do the same checks for each subtype
    for (const unionType of type.unionTypes) {
      // Skip built-in types, too
      if (unionType.builtIn) continue
      // Check if there's a collision
      if (eventTypeNames.has(unionType.typename)) {
        // A type may be seen multiple times, make sure this is an actual collision
        if (eventTypes.has(unionType)) continue // This event was already processed, so we're done
        // Modify the type name by attaching a UUIDv4 to it
        unionType.typename = unionType.typename + "_" + uuid().replace(/-/g, "_")
      }
      // Add the type to the list
      eventTypeNames.add(unionType.typename)
      eventTypes.add(unionType)
    } // TODO: DRY this all up
    // Determine if any of the properties of the type should be added to the list
    for (const propType of Object.values(type.properties)) {
      // Skip built-in types, too
      if (propType.builtIn) continue
      // Check if there's a collision
      if (eventTypeNames.has(propType.typename)) {
        // A type may be seen multiple times, make sure this is an actual collision
        if (eventTypes.has(propType)) continue // This event was already processed, so we're done
        // Modify the type name by attaching a UUIDv4 to it
        propType.typename = propType.typename + "_" + uuid().replace(/-/g, "_")
      }
      // Add the type to the list
      eventTypeNames.add(propType.typename)
      eventTypes.add(propType)
    }
  }
  // Extract the handler definitions and constant data
  let handlers = {} // String to array of Microstatement objects
  let constantDedupeLookup = {} // String to Microstatement object
  let constants = new Set() // Microstatment objects
  for (let evt of Event.allEvents) {
    for (let handler of evt.handlers) {
      if (handler instanceof UserFunction) {
        // Define the handler preamble
        let handlerDec = "on " + evt.name + " fn ("
        let argList = []
        let microstatements = []
        for (const arg of Object.keys(handler.getArguments())) {
          argList.push(arg + ": " + handler.getArguments()[arg].typename)
          microstatements.push(new Microstatement(
            StatementType.ARG,
            handler.closureScope,
            true,
            arg,
            handler.getArguments()[arg],
            [],
            [],
          ))
        }
        handlerDec += argList.join(", ")
        handlerDec += "): " + handler.getReturnType().typename + " {"
        // Extract the handler statements and compile into microstatements
        const statements = handler.maybeTransform().statements;
        for (const s of statements) {
          Microstatement.fromStatement(s, microstatements)
        }
        // Pull the constants out of the microstatements into the constants set.
        hoistConst(microstatements, constantDedupeLookup, constants)
        // Register the handler and remaining statements
        handlers[handlerDec] = microstatements
      }
    }
  }
  let outStr = ""
  // Print the event types
  for (const eventType of eventTypes) {
    outStr += eventType.toString() + "\n"
  }
  // Print the constants
  for (const constant of constants) {
    outStr += constant.toString() + "\n"
  }
  // Print the user-defined event declarations
  for (const evt of Event.allEvents) {
    if (evt.builtIn) continue // Skip built-in events
    if (evt.handlers.length == 0) continue // Skip events that are never handled
    outStr += evt.toString() + "\n"
  }
  // Print the user-defined event handlers
  for (const handlerDec of Object.keys(handlers)) {
    outStr += handlerDec + "\n"
    const microstatements = handlers[handlerDec]
    for (const m of microstatements) {
      const mString = m.toString()
      if (mString === "") continue
      outStr += "  " + mString + "\n"
    }
    outStr += "}\n"
  }
  return outStr
}

module.exports = (filename) => ammFromModuleAsts(moduleAstsFromFile(filename))
module.exports.lnTextToAmm = (str) => ammFromModuleAsts(moduleAstsFromString(str))
