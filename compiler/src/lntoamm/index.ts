import * as fs from 'fs'

import { v4 as uuid, } from 'uuid'

import * as Ast from './Ast'
import * as Std from './Std'
import Module from './Module'
import Event from './Event'
import UserFunction = require('./UserFunction')
import Microstatement = require('./Microstatement')
import StatementType from './StatementType'

const hoistConst = (
  microstatements: any, // TODO: Convert `Microstatement` to TS
  constantDedupeLookup: object,
  constants: Set<any>, // TODO: Convert `Microstatement` to TS
  eventTypes: Set<any>, // Convert `Type` to TS
) => {
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
        microstatements.splice(i, 1)
      }
    } else if (m.statementType === StatementType.CLOSURE) {
      hoistConst(m.closureStatements, constantDedupeLookup, constants, eventTypes)
      i++
    } else {
      i++
    }
  }
}

const moduleAstsFromFile = (filename: string) => {
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

const moduleAstsFromString = (str: string) => {
  // If loading from a string, it's in the browser and some internal state needs cleaning. Some of
  // this doesn't appear to affect things, but better to compile from a known state
  Event.allEvents = [Event.allEvents[0]] // Keep the `start` event
  Event.allEvents[0].handlers = [] // Reset the registered handlers on the `start` event
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

const ammFromModuleAsts = (moduleAsts: any) => { // TODO: Migrate from ANTLR
  // Load the standard library
  let stdFiles = new Set()
  for (const [modulePath, module] of Object.entries(moduleAsts)) {
    for (const importt of Ast.resolveImports(modulePath, module)) {
      if (importt.substring(0, 5) === "@std/") {
        stdFiles.add(importt.substring(5, importt.length) + '.ln')
      }
    }
  }
  Std.loadStdModules(stdFiles)
  const rootScope = Module.getAllModules()['<root>'].exportScope
  // Load all modules
  Module.modulesFromAsts(moduleAsts, rootScope)
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
      // TODO: Convert `Type` to TS
      // Skip built-in types, too
      if ((propType as any).builtIn) continue
      // Check if there's a collision
      if (eventTypeNames.has((propType as any).typename)) {
        // A type may be seen multiple times, make sure this is an actual collision
        if (eventTypes.has(propType)) continue // This event was already processed, so we're done
        // Modify the type name by attaching a UUIDv4 to it
        (propType as any).typename = (propType as any).typename + "_" + uuid().replace(/-/g, "_")
      }
      // Add the type to the list
      eventTypeNames.add((propType as any).typename)
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
        hoistConst(microstatements, constantDedupeLookup, constants, eventTypes)
        // Register the handler and remaining statements
        handlers.hasOwnProperty(handlerDec) ? handlers[handlerDec].push(microstatements) : handlers[handlerDec] = [microstatements]
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
    outStr += evt.toString() + "\n"
  }
  // Print the user-defined event handlers
  for (const [handlerDec, handlersList] of Object.entries(handlers)) {
    for (const microstatements of (handlersList as Array<any>)) {
      outStr += handlerDec + "\n"
      for (const m of microstatements) {
        const mString = m.toString()
        if (mString === "") continue
        outStr += "  " + mString + "\n"
      }
      outStr += "}\n"
    }
  }
  return outStr
}

export const fromFile = (filename: string) => ammFromModuleAsts(moduleAstsFromFile(filename))
export const fromString = (str: string) => ammFromModuleAsts(moduleAstsFromString(str))

