import { v4 as uuid, } from 'uuid'

import Event from './Event'
import Microstatement from './Microstatement'
import Module from './Module'
import Scope from './Scope'
import StatementType from './StatementType'
import { Interface, Type, } from './Type'
import UserFunction from './UserFunction'

const opcodeScope = new Scope()
const opcodeModule = new Module(opcodeScope)

// Base types
const addBuiltIn = (name: string) => {
  opcodeScope.put(name, Type.builtinTypes[name])
}
([
  'void', 'int8', 'int16', 'int32', 'int64', 'float32', 'float64', 'bool', 'string', 'function',
  'operator', 'Error', 'Maybe', 'Result', 'Either', 'Array', 'ExecRes', 'InitialReduce',
  'InternalResponse', 'Seq', 'Self',
].map(addBuiltIn))
Type.builtinTypes['Array'].solidify(['string'], opcodeScope)
opcodeScope.put('any', new Type('any', true, false, {}, {}, null, new Interface('any')))
opcodeScope.put(
  'anythingElse',
  new Type('anythingElse', true, false, {}, {}, null, new Interface('anythingElse')),
)
Type.builtinTypes['Array'].solidify(['any'], opcodeScope)
Type.builtinTypes['Array'].solidify(['anythingElse'], opcodeScope)
Type.builtinTypes.Maybe.solidify(['any'], opcodeScope)
Type.builtinTypes.Result.solidify(['any'], opcodeScope)
Type.builtinTypes.Result.solidify(['anythingElse'], opcodeScope)
Type.builtinTypes.Result.solidify(['int64'], opcodeScope)
Type.builtinTypes.Result.solidify(['string'], opcodeScope)
Type.builtinTypes.Either.solidify(['any', 'anythingElse'], opcodeScope)
Type.builtinTypes.InitialReduce.solidify(['any', 'anythingElse'], opcodeScope)
opcodeScope.put("start", new Event("_start", Type.builtinTypes.void, true))
opcodeScope.put("__conn", new Event("__conn", Type.builtinTypes.InternalRequest, true))
const t = (str: string) => opcodeScope.get(str)

// opcode declarations
const addopcodes = (opcodes: object) => {
  const opcodeNames = Object.keys(opcodes)
  opcodeNames.forEach((opcodeName) => {
    const opcodeDef = opcodes[opcodeName]
    const [args, returnType] = opcodeDef
    if (!returnType) { // This is a three-arg, 0-return opcode
      const opcodeObj = {
        getName: () => opcodeName,
        getArguments: () => args,
        getReturnType: () => Type.builtinTypes.void,
        isPure: () => true,
        microstatementInlining: (
          realArgNames: Array<string>,
          scope: Scope,
          microstatements: Array<Microstatement>,
        ) => {
          if (['seqwhile'].includes(opcodeName)) {
            const inputs = realArgNames.map(n => Microstatement.fromVarName(n, scope, microstatements))
            const condfn = UserFunction.dispatchFn(inputs[1].fns, [], scope)
            const condidx = microstatements.indexOf(inputs[1])
            const condm = microstatements.slice(0, condidx)
            Microstatement.closureFromUserFunction(condfn, condfn.scope || scope, condm, new Map())
            const condclosure = condm[condm.length - 1]
            microstatements.splice(condidx, 0, condclosure)
            realArgNames[1] = condclosure.outputName
            const bodyfn = UserFunction.dispatchFn(inputs[2].fns, [], scope)
            const bodyidx = microstatements.indexOf(inputs[2])
            const bodym = microstatements.slice(0, bodyidx)
            Microstatement.closureFromUserFunction(bodyfn, bodyfn.scope || scope, bodym, new Map())
            const bodyclosure = bodym[bodym.length - 1]
            microstatements.splice(bodyidx, 0, bodyclosure)
            realArgNames[2] = bodyclosure.outputName
          }
          microstatements.push(new Microstatement(
            StatementType.CALL,
            scope,
            true,
            null,
            opcodeObj.getReturnType(),
            realArgNames,
            [opcodeObj],
          ))
        },
      }
      // Add each opcode
      opcodeScope.put(opcodeName, [opcodeObj])
    } else {
      const opcodeObj = {
        getName: () => opcodeName,
        getArguments: () => args,
        getReturnType: () => returnType,
        isPure: () => true,
        microstatementInlining: (
          realArgNames: Array<string>,
          scope: Scope,
          microstatements: Array<Microstatement>,
        ) => {
          const inputs = realArgNames.map(n => Microstatement.fromVarName(n, scope, microstatements))
          const inputTypes = inputs.map(i => i.outputType)
          const interfaceMap: Map<Type, Type> = new Map()
          Object.values(args).forEach((t: Type, i) => t.typeApplies(inputTypes[i], scope, interfaceMap))
          microstatements.push(new Microstatement(
            StatementType.CONSTDEC,
            scope,
            true,
            "_" + uuid().replace(/-/g, "_"),
            ((inputTypes, scope) => {
              if (!!returnType.iface) {
                // Path 1: the opcode returns an interface based on the interface type of an input
                let replacementType: Type
                Object.values(args).forEach((a: Type, i: number) => {
                  if (inputs[i].statementType === StatementType.CLOSUREDEF) {
                    const idx = microstatements.indexOf(inputs[i])
                    const m = microstatements.slice(0, idx)
                    let fn: any
                    // TODO: Remove this hackery after function types are more than just 'function'
                    if ([
                      'map', 'mapl', 'each', 'eachl', 'find', 'findl', 'every', 'everyl', 'some',
                      'somel', 'filter', 'filterl', 'seqeach',
                    ].includes(opcodeName)) {
                      // TODO: Try to re-unify these blocks from above
                      const arrayInnerType = scope.deepGet(
                        inputTypes[0].typename.replace(/^Array<(.*)>$/, "$1")
                      ) as Type
                      const innerType = inputTypes[0].originalType ?
                        arrayInnerType :
                        Type.builtinTypes.int64 // Hackery for seqeach
                      try {
                        fn = UserFunction.dispatchFn(inputs[i].fns, [innerType], scope)
                        (Object.values(fn.getArguments())[0] as Type)
                          .typeApplies(innerType, scope, interfaceMap)
                      } catch {
                        try {
                          fn = UserFunction.dispatchFn(inputs[i].fns, [], scope)
                        } catch {
                          fn = UserFunction.dispatchFn(
                            inputs[i].fns,
                            [arrayInnerType, Type.builtinTypes.int64],
                            scope,
                          )
                          const closureArgs = Object.values(fn.getArguments()) as Type[]
                          closureArgs[0].typeApplies(arrayInnerType, scope, interfaceMap)
                          closureArgs[1].typeApplies(Type.builtinTypes.int64, scope, interfaceMap)
                        }
                      }
                    } else if (['reducel', 'reducep'].includes(opcodeName)) {
                      const arrayInnerType = scope.deepGet(
                        inputTypes[0].typename.replace(/^Array<(.*)>$/, "$1")
                      ) as Type
                      fn = UserFunction.dispatchFn(
                        inputs[i].fns,
                        [arrayInnerType, arrayInnerType],
                        scope
                      )
                      const closureArgs = Object.values(fn.getArguments()) as Type[]
                      closureArgs[0].typeApplies(arrayInnerType, scope, interfaceMap)
                      closureArgs[1].typeApplies(arrayInnerType, scope, interfaceMap)
                    } else if (['foldl'].includes(opcodeName)) {
                      const reducerTypes = Object.values(inputTypes[0].properties) as Type[]
                      const inType = scope.deepGet(
                        reducerTypes[0].typename.replace(/^Array<(.*)>$/, "$1")
                      ) as Type
                      const fnArgTypes = [
                        reducerTypes[1],
                        inType,
                      ]
                      fn = UserFunction.dispatchFn(
                        inputs[i].fns,
                        fnArgTypes,
                        scope,
                      )
                      const closureArgs = Object.values(fn.getArguments()) as Type[]
                      closureArgs[0].typeApplies(reducerTypes[1], scope, interfaceMap)
                      closureArgs[1].typeApplies(inType, scope, interfaceMap)
                    } else if (['foldp'].includes(opcodeName)) {
                      const reducerTypes = Object.values(inputTypes[0].properties) as Type[]
                      const inType = scope.deepGet(
                        reducerTypes[0].typename.replace(/^Array<(.*)>$/, "$1")
                      ) as Type
                      const fnArgTypes = [
                        reducerTypes[1],
                        inType,
                      ]
                      fn = UserFunction.dispatchFn(
                        inputs[i].fns,
                        fnArgTypes,
                        scope,
                      )
                      const closureArgs = Object.values(fn.getArguments()) as Type[]
                      closureArgs[0].typeApplies(reducerTypes[1], scope, interfaceMap)
                      closureArgs[1].typeApplies(inType, scope, interfaceMap)
                    } else if (['seqrec'].includes(opcodeName)) {
                      // TODO: Is this even reachable?
                      // TODO: How would multiple dispatch even work here?
                      fn = inputs[1].fns[0]
                    } else if (['selfrec'].includes(opcodeName)) {
                      // TODO: Is this even reachable?
                      fn = inputs[0].fns[0]
                    } else {
                      fn = UserFunction.dispatchFn(inputs[i].fns, [], scope)
                    }
                    Microstatement.closureFromUserFunction(fn, fn.scope || scope, m, interfaceMap)
                    const closure = m[m.length - 1]
                    microstatements.splice(idx, 0, closure)
                    realArgNames[i] = closure.outputName
                  }
                  if (!!a.iface && a.iface.interfacename === returnType.iface.interfacename) {
                    replacementType = inputTypes[i]
                  }
                  if (Object.values(a.properties).some(
                    p => !!p.iface && p.iface.interfacename === returnType.iface.interfacename
                  )) {
                    Object.values(a.properties).forEach((p, j) => {
                      if (!!p.iface && p.iface.interfacename === returnType.iface.interfacename) {
                        replacementType = Object.values(inputTypes[i].properties)[j] as Type
                      }
                    })
                  }
                })
                if (!replacementType) return returnType
                return replacementType
              } else if (
                returnType.originalType &&
                Object.values(returnType.properties).some((p: Type) => !!p.iface)
              ) {
                // TODO: Remove this hackery after function types are more than just 'function'
                if ([
                  'map', 'mapl', 'each', 'eachl', 'find', 'findl', 'every', 'everyl', 'some',
                  'somel', 'filter', 'filterl', 'seqeach',
                ].includes(opcodeName)) {
                  // The ideal `map` opcode type declaration is something like:
                  // `map(Array<any>, fn (any): anythingElse): Array<anythingElse>` and then the
                  // interface matching logic figures out what the return type of the opcode is
                  // based on the return type of the function given to it.
                  // For now, we just do that "by hand."
                  const arrayInnerType = scope.deepGet(
                    inputTypes[0].typename.replace(/^Array<(.*)>$/, "$1")
                  ) as Type
                  const innerType = inputTypes[0].originalType ?
                    arrayInnerType :
                    Type.builtinTypes.int64 // Hackery for seqeach
                  let fn: any
                  try {
                    fn = UserFunction.dispatchFn(inputs[1].fns, [innerType], scope)
                  } catch {
                    try {
                      fn = UserFunction.dispatchFn(inputs[1].fns, [], scope)
                    } catch {
                      fn = UserFunction.dispatchFn(
                        inputs[1].fns,
                        [arrayInnerType, Type.builtinTypes.int64],
                        scope,
                      )
                    }
                  }
                  const closureArgs = Object.values(fn.getArguments()) as Type[]
                  if (closureArgs[0]) {
                    closureArgs[0].typeApplies(innerType, scope, interfaceMap)
                  }
                  if (closureArgs[1]) {
                    closureArgs[1].typeApplies(Type.builtinTypes.int64, scope, interfaceMap)
                  }
                  const idx = microstatements.indexOf(inputs[1])
                  const m = microstatements.slice(0, idx)
                  Microstatement.closureFromUserFunction(fn, fn.scope || scope, m, interfaceMap)
                  const closure = m[m.length - 1]
                  microstatements.splice(idx, 0, closure)
                  realArgNames[1] = closure.outputName
                  if (['filter', 'filterl'].includes(opcodeName)) {
                    return inputs[0].outputType
                  } else {
                    const innerType = closure.closureOutputType
                    const newInnerType = innerType.realize(interfaceMap, scope) // Necessary?
                    const baseType = returnType.originalType
                    const newReturnType = baseType ?
                      baseType.solidify([newInnerType.typename], scope) :
                      returnType
                    return newReturnType
                  }
                } else if (['reducel', 'reducep'].includes(opcodeName)) {
                  const arrayInnerType = scope.deepGet(
                    inputTypes[0].typename.replace(/^Array<(.*)>$/, "$1")
                  ) as Type
                  let fn = UserFunction.dispatchFn(
                    inputs[1].fns,
                    [arrayInnerType, arrayInnerType],
                    scope
                  )
                  const closureArgs = Object.values(fn.getArguments()) as Type[]
                  closureArgs[0].typeApplies(arrayInnerType, scope, interfaceMap)
                  closureArgs[1].typeApplies(arrayInnerType, scope, interfaceMap)
                  const idx = microstatements.indexOf(inputs[1])
                  const m = microstatements.slice(0, idx)
                  Microstatement.closureFromUserFunction(fn, fn.scope || scope, m, interfaceMap)
                  const closure = m[m.length - 1]
                  microstatements.splice(idx, 0, closure)
                  realArgNames[1] = closure.outputName
                  return arrayInnerType
                } else if (['foldl'].includes(opcodeName)) {
                  const reducerTypes = Object.values(inputTypes[0].properties) as Type[]
                  const inType = scope.deepGet(
                    reducerTypes[0].typename.replace(/^Array<(.*)>$/, "$1")
                  ) as Type
                  const fnArgTypes = [
                    reducerTypes[1],
                    inType,
                  ]
                  let fn = UserFunction.dispatchFn(
                    inputs[1].fns,
                    fnArgTypes,
                    scope,
                  )
                  const closureArgs = Object.values(fn.getArguments()) as Type[]
                  closureArgs[0].typeApplies(reducerTypes[1], scope, interfaceMap)
                  closureArgs[1].typeApplies(inType, scope, interfaceMap)
                  const idx = microstatements.indexOf(inputs[1])
                  const m = microstatements.slice(0, idx)
                  Microstatement.closureFromUserFunction(fn, fn.scope || scope, m, interfaceMap)
                  const closure = m[m.length - 1]
                  microstatements.splice(idx, 0, closure)
                  realArgNames[1] = closure.outputName
                  return closure.closureOutputType
                } else if (['foldp'].includes(opcodeName)) {
                  const reducerTypes = Object.values(inputTypes[0].properties) as Type[]
                  const inType = scope.deepGet(
                    reducerTypes[0].typename.replace(/^Array<(.*)>$/, "$1")
                  ) as Type
                  const fnArgTypes = [
                    reducerTypes[1],
                    inType,
                  ]
                  const fn = UserFunction.dispatchFn(
                    inputs[1].fns,
                    fnArgTypes,
                    scope,
                  )
                  const closureArgs = Object.values(fn.getArguments()) as Type[]
                  closureArgs[0].typeApplies(reducerTypes[1], scope, interfaceMap)
                  closureArgs[1].typeApplies(inType, scope, interfaceMap)
                  const idx = microstatements.indexOf(inputs[1])
                  const m = microstatements.slice(0, idx)
                  Microstatement.closureFromUserFunction(fn, fn.scope || scope, m, interfaceMap)
                  const closure = m[m.length - 1]
                  microstatements.splice(idx, 0, closure)
                  realArgNames[1] = closure.outputName
                  return Type.builtinTypes['Array'].solidify(
                    [closure.closureOutputType.typename],
                    scope,
                  )
                } else if (['seqrec'].includes(opcodeName)) {
                  // TODO: How would multiple dispatch even work here?
                  const fn = inputs[1].inputNames[1].fns[0]
                  const idx = microstatements.indexOf(inputs[1])
                  const m = microstatements.slice(0, idx)
                  Microstatement.closureFromUserFunction(fn, fn.scope || scope, m, interfaceMap)
                  const closure = m[m.length - 1]
                  microstatements.splice(idx, 0, closure)
                  realArgNames[1] = closure.outputName
                  // TODO: How do interface types work here?
                  return closure.closureOutputType.typename
                } else if (['selfrec'].includes(opcodeName)) {
                  // TODO: This is absolute crap. How to fix?
                  return inputs[0].inputNames[1] ? Microstatement.fromVarName(
                    inputs[0].inputNames[1], scope, microstatements
                  ).closureOutputType : returnType
                } else {
                  // Path 2: the opcode returns solidified generic type with an interface generic
                  // that mathces the interface type of an input
                  const returnIfaces = Object.values(returnType.properties)
                    .filter((p: Type) => !!p.iface).map((p: Type) => p.iface)
                  if (returnIfaces.length > 0) {
                    const newReturnType = returnType.realize(interfaceMap, scope)
                    return newReturnType
                  } else {
                    return returnType
                  }
                }
              } else {
                // No need to adjust the return type, but may still need to lazy eval a closure
                Object.values(args).forEach((_a: Type, i: number) => {
                  if (inputs[i].statementType === StatementType.CLOSUREDEF) {
                    const idx = microstatements.indexOf(inputs[i])
                    const m = microstatements.slice(0, idx)
                    let fn: any
                    // TODO: Remove this hackery after function types are more than just 'function'
                    if ([
                      'map', 'mapl', 'each', 'eachl', 'find', 'findl', 'every', 'everyl', 'some',
                      'somel', 'filter', 'filterl', 'seqeach',
                    ].includes(opcodeName)) {
                      // TODO: Try to re-unify these blocks from above
                      const arrayInnerType = scope.deepGet(
                        inputTypes[0].typename.replace(/^Array<(.*)>$/, "$1")
                      ) as Type
                      const innerType = inputTypes[0].originalType ?
                        arrayInnerType :
                        Type.builtinTypes.int64 // Hackery for seqeach
                      try {
                        fn = UserFunction.dispatchFn(inputs[i].fns, [innerType], scope)
                      } catch {
                        try {
                          fn = UserFunction.dispatchFn(inputs[i].fns, [], scope)
                        } catch {
                          fn = UserFunction.dispatchFn(
                            inputs[i].fns,
                            [arrayInnerType, Type.builtinTypes.int64],
                            scope,
                          )
                        }
                      }
                      const closureArgs = Object.values(fn.getArguments()) as Type[]
                      if (closureArgs[0]) {
                        closureArgs[0].typeApplies(innerType, scope, interfaceMap)
                      }
                      if (closureArgs[1]) {
                        closureArgs[1].typeApplies(Type.builtinTypes.int64, scope, interfaceMap)
                      }
                    } else if (['reducel', 'reducep'].includes(opcodeName)) {
                      const arrayInnerType = scope.deepGet(
                        inputTypes[0].typename.replace(/^Array<(.*)>$/, "$1")
                      ) as Type
                      fn = UserFunction.dispatchFn(
                        inputs[1].fns,
                        [arrayInnerType, arrayInnerType],
                        scope
                      )
                      const closureArgs = Object.values(fn.getArguments()) as Type[]
                      closureArgs[0].typeApplies(arrayInnerType, scope, interfaceMap)
                      closureArgs[1].typeApplies(arrayInnerType, scope, interfaceMap)
                    } else if (['foldl'].includes(opcodeName)) {
                      const reducerTypes = Object.values(inputTypes[0].properties) as Type[]
                      const inType = scope.deepGet(
                        reducerTypes[0].typename.replace(/^Array<(.*)>$/, "$1")
                      ) as Type
                      const fnArgTypes = [
                        reducerTypes[1],
                        inType,
                      ]
                      let fn = UserFunction.dispatchFn(
                        inputs[1].fns,
                        fnArgTypes,
                        scope,
                      )
                      const closureArgs = Object.values(fn.getArguments()) as Type[]
                      closureArgs[0].typeApplies(reducerTypes[1], scope, interfaceMap)
                      closureArgs[1].typeApplies(inType, scope, interfaceMap)
                    } else if (['seqrec'].includes(opcodeName)) {
                      // TODO: How would multiple dispatch even work here?
                      fn = inputs[1].fns[0]
                    } else if (['selfrec'].includes(opcodeName)) {
                      // TODO: Is this even reachable?
                      fn = inputs[0].inputNames[1].fns[0]
                    } else {
                      fn = UserFunction.dispatchFn(inputs[i].fns, [], scope)
                    }
                    Microstatement.closureFromUserFunction(fn, fn.scope || scope, m, interfaceMap)
                    const closure = m[m.length - 1]
                    microstatements.splice(idx, 0, closure)
                    realArgNames[i] = closure.outputName
                  }
                })
              }
              return returnType
            })(inputTypes, scope),
            realArgNames,
            [opcodeObj],
          ))
        },
      }
      // Add each opcode
      opcodeScope.put(opcodeName, [opcodeObj])
    }
  })
}

addopcodes({
  i8f64: [{ number: t('int8'), }, t('float64')],
  i16f64: [{ number: t('int16'), }, t('float64')],
  i32f64: [{ number: t('int32'), }, t('float64')],
  i64f64: [{ number: t('int64'), }, t('float64')],
  f32f64: [{ number: t('float32'), }, t('float64')],
  strf64: [{ str: t('string'), }, t('float64')],
  boolf64: [{ boo: t('bool'), }, t('float64')],
  i8f32: [{ number: t('int8'), }, t('float32')],
  i16f32: [{ number: t('int16'), }, t('float32')],
  i32f32: [{ number: t('int32'), }, t('float32')],
  i64f32: [{ number: t('int64'), }, t('float32')],
  f64f32: [{ number: t('float64'), }, t('float32')],
  strf32: [{ str: t('string'), }, t('float32')],
  boolf32: [{ boo: t('bool'), }, t('float32')],
  i8i64: [{ number: t('int8'), }, t('int64')],
  i16i64: [{ number: t('int16'), }, t('int64')],
  i32i64: [{ number: t('int32'), }, t('int64')],
  f32i64: [{ number: t('float32'), }, t('int64')],
  f64i64: [{ number: t('float64'), }, t('int64')],
  stri64: [{ str: t('string'), }, t('int64')],
  booli64: [{ boo: t('bool'), }, t('int64')],
  i8i32: [{ number: t('int8'), }, t('int32')],
  i16i32: [{ number: t('int16'), }, t('int32')],
  i64i32: [{ number: t('int64'), }, t('int32')],
  f32i32: [{ number: t('float32'), }, t('int32')],
  f64i32: [{ number: t('float64'), }, t('int32')],
  stri32: [{ str: t('string'), }, t('int32')],
  booli32: [{ boo: t('bool'), }, t('int32')],
  i8i16: [{ number: t('int8'), }, t('int16')],
  i32i16: [{ number: t('int32'), }, t('int16')],
  i64i16: [{ number: t('int64'), }, t('int16')],
  f32i16: [{ number: t('float32'), }, t('int16')],
  f64i16: [{ number: t('float64'), }, t('int16')],
  stri16: [{ str: t('string'), }, t('int16')],
  booli16: [{ boo: t('bool'), }, t('int16')],
  i16i8: [{ number: t('int16'), }, t('int8')],
  i32i8: [{ number: t('int32'), }, t('int8')],
  i64i8: [{ number: t('int64'), }, t('int8')],
  f32i8: [{ number: t('float32'), }, t('int8')],
  f64i8: [{ number: t('float64'), }, t('int8')],
  stri8: [{ str: t('string'), }, t('int8')],
  booli8: [{ boo: t('bool'), }, t('int8')],
  i8bool: [{ number: t('int8'), }, t('bool')],
  i16bool: [{ number: t('int16'), }, t('bool')],
  i32bool: [{ number: t('int32'), }, t('bool')],
  i64bool: [{ number: t('int64'), }, t('bool')],
  f32bool: [{ number: t('float32'), }, t('bool')],
  f64bool: [{ number: t('float64'), }, t('bool')],
  strbool: [{ str: t('string'), }, t('bool')],
  i8str: [{ number: t('int8'), }, t('string')],
  i16str: [{ number: t('int16'), }, t('string')],
  i32str: [{ number: t('int32'), }, t('string')],
  i64str: [{ number: t('int64'), }, t('string')],
  f32str: [{ number: t('float32'), }, t('string')],
  f64str: [{ number: t('float64'), }, t('string')],
  boolstr: [{ boo: t('bool'), }, t('string')],
  addi8: [{ a: t('int8'), b: t('int8'), }, t('int8')],
  addi16: [{ a: t('int16'), b: t('int16'), }, t('int16')],
  addi32: [{ a: t('int32'), b: t('int32'), }, t('int32')],
  addi64: [{ a: t('int64'), b: t('int64'), }, t('int64')],
  addf32: [{ a: t('float32'), b: t('float32'), }, t('float32')],
  addf64: [{ a: t('float64'), b: t('float64'), }, t('float64')],
  subi8: [{ a: t('int8'), b: t('int8'), }, t('int8')],
  subi16: [{ a: t('int16'), b: t('int16'), }, t('int16')],
  subi32: [{ a: t('int32'), b: t('int32'), }, t('int32')],
  subi64: [{ a: t('int64'), b: t('int64'), }, t('int64')],
  subf32: [{ a: t('float32'), b: t('float32'), }, t('float32')],
  subf64: [{ a: t('float64'), b: t('float64'), }, t('float64')],
  negi8: [{ a: t('int8'), }, t('int8')],
  negi16: [{ a: t('int16'), }, t('int16')],
  negi32: [{ a: t('int32'), }, t('int32')],
  negi64: [{ a: t('int64'), }, t('int64')],
  negf32: [{ a: t('float32'), }, t('float32')],
  negf64: [{ a: t('float64'), }, t('float64')],
  absi8: [{ a: t('int8'), }, t('int8')],
  absi16: [{ a: t('int16'), }, t('int16')],
  absi32: [{ a: t('int32'), }, t('int32')],
  absi64: [{ a: t('int64'), }, t('int64')],
  absf32: [{ a: t('float32'), }, t('float32')],
  absf64: [{ a: t('float64'), }, t('float64')],
  muli8: [{ a: t('int8'), b: t('int8'), }, t('int8')],
  muli16: [{ a: t('int16'), b: t('int16'), }, t('int16')],
  muli32: [{ a: t('int32'), b: t('int32'), }, t('int32')],
  muli64: [{ a: t('int64'), b: t('int64'), }, t('int64')],
  mulf32: [{ a: t('float32'), b: t('float32'), }, t('float32')],
  mulf64: [{ a: t('float64'), b: t('float64'), }, t('float64')],
  divi8: [{ a: t('int8'), b: t('int8'), }, t('int8')],
  divi16: [{ a: t('int16'), b: t('int16'), }, t('int16')],
  divi32: [{ a: t('int32'), b: t('int32'), }, t('int32')],
  divi64: [{ a: t('int64'), b: t('int64'), }, t('int64')],
  divf32: [{ a: t('float32'), b: t('float32'), }, t('float32')],
  divf64: [{ a: t('float64'), b: t('float64'), }, t('float64')],
  modi8: [{ a: t('int8'), b: t('int8'), }, t('int8')],
  modi16: [{ a: t('int16'), b: t('int16'), }, t('int16')],
  modi32: [{ a: t('int32'), b: t('int32'), }, t('int32')],
  modi64: [{ a: t('int64'), b: t('int64'), }, t('int64')],
  powi8: [{ a: t('int8'), b: t('int8'), }, t('int8')],
  powi16: [{ a: t('int16'), b: t('int16'), }, t('int16')],
  powi32: [{ a: t('int32'), b: t('int32'), }, t('int32')],
  powi64: [{ a: t('int64'), b: t('int64'), }, t('int64')],
  powf32: [{ a: t('float32'), b: t('float32'), }, t('float32')],
  powf64: [{ a: t('float64'), b: t('float64'), }, t('float64')],
  sqrtf32: [{ a: t('float32'), }, t('float32')],
  sqrtf64: [{ a: t('float64'), }, t('float64')],
  andi8: [{ a: t('int8'), b: t('int8'), }, t('int8')],
  andi16: [{ a: t('int16'), b: t('int16'), }, t('int16')],
  andi32: [{ a: t('int32'), b: t('int32'), }, t('int32')],
  andi64: [{ a: t('int64'), b: t('int64'), }, t('int64')],
  andbool: [{ a: t('bool'), b: t('bool'), }, t('bool')],
  ori8: [{ a: t('int8'), b: t('int8'), }, t('int8')],
  ori16: [{ a: t('int16'), b: t('int16'), }, t('int16')],
  ori32: [{ a: t('int32'), b: t('int32'), }, t('int32')],
  ori64: [{ a: t('int64'), b: t('int64'), }, t('int64')],
  orbool: [{ a: t('bool'), b: t('bool'), }, t('bool')],
  xori8: [{ a: t('int8'), b: t('int8'), }, t('int8')],
  xori16: [{ a: t('int16'), b: t('int16'), }, t('int16')],
  xori32: [{ a: t('int32'), b: t('int32'), }, t('int32')],
  xori64: [{ a: t('int64'), b: t('int64'), }, t('int64')],
  xorbool: [{ a: t('bool'), b: t('bool'), }, t('bool')],
  noti8: [{ a: t('int8'), }, t('int8')],
  noti16: [{ a: t('int16'), }, t('int16')],
  noti32: [{ a: t('int32'), }, t('int32')],
  noti64: [{ a: t('int64'), }, t('int64')],
  notbool: [{ a: t('bool'), }, t('bool')],
  nandi8: [{ a: t('int8'), b: t('int8'), }, t('int8')],
  nandi16: [{ a: t('int16'), b: t('int16'), }, t('int16')],
  nandi32: [{ a: t('int32'), b: t('int32'), }, t('int32')],
  nandi64: [{ a: t('int64'), b: t('int64'), }, t('int64')],
  nandboo: [{ a: t('bool'), b: t('bool'), }, t('bool')],
  nori8: [{ a: t('int8'), b: t('int8'), }, t('int8')],
  nori16: [{ a: t('int16'), b: t('int16'), }, t('int16')],
  nori32: [{ a: t('int32'), b: t('int32'), }, t('int32')],
  nori64: [{ a: t('int64'), b: t('int64'), }, t('int64')],
  norbool: [{ a: t('bool'), b: t('bool'), }, t('bool')],
  xnori8: [{ a: t('int8'), b: t('int8'), }, t('int8')],
  xnori16: [{ a: t('int16'), b: t('int16'), }, t('int16')],
  xnori32: [{ a: t('int32'), b: t('int32'), }, t('int32')],
  xnori64: [{ a: t('int64'), b: t('int64'), }, t('int64')],
  xnorboo: [{ a: t('bool'), b: t('bool'), }, t('bool')],
  eqi8: [{ a: t('int8'), b: t('int8'), }, t('bool')],
  eqi16: [{ a: t('int16'), b: t('int16'), }, t('bool')],
  eqi32: [{ a: t('int32'), b: t('int32'), }, t('bool')],
  eqi64: [{ a: t('int64'), b: t('int64'), }, t('bool')],
  eqf32: [{ a: t('float32'), b: t('float32'), }, t('bool')],
  eqf64: [{ a: t('float64'), b: t('float64'), }, t('bool')],
  eqbool: [{ a: t('bool'), b: t('bool'), }, t('bool')],
  eqstr: [{ a: t('string'), b: t('string'), }, t('bool')],
  neqi8: [{ a: t('int8'), b: t('int8'), }, t('bool')],
  neqi16: [{ a: t('int16'), b: t('int16'), }, t('bool')],
  neqi32: [{ a: t('int32'), b: t('int32'), }, t('bool')],
  neqi64: [{ a: t('int64'), b: t('int64'), }, t('bool')],
  neqf32: [{ a: t('float32'), b: t('float32'), }, t('bool')],
  neqf64: [{ a: t('float64'), b: t('float64'), }, t('bool')],
  neqbool: [{ a: t('bool'), b: t('bool'), }, t('bool')],
  neqstr: [{ a: t('string'), b: t('string'), }, t('bool')],
  lti8: [{ a: t('int8'), b: t('int8'), }, t('bool')],
  lti16: [{ a: t('int16'), b: t('int16'), }, t('bool')],
  lti32: [{ a: t('int32'), b: t('int32'), }, t('bool')],
  lti64: [{ a: t('int64'), b: t('int64'), }, t('bool')],
  ltf32: [{ a: t('float32'), b: t('float32'), }, t('bool')],
  ltf64: [{ a: t('float64'), b: t('float64'), }, t('bool')],
  ltstr: [{ a: t('string'), b: t('string'), }, t('bool')],
  ltei8: [{ a: t('int8'), b: t('int8'), }, t('bool')],
  ltei16: [{ a: t('int16'), b: t('int16'), }, t('bool')],
  ltei32: [{ a: t('int32'), b: t('int32'), }, t('bool')],
  ltei64: [{ a: t('int64'), b: t('int64'), }, t('bool')],
  ltef32: [{ a: t('float32'), b: t('float32'), }, t('bool')],
  ltef64: [{ a: t('float64'), b: t('float64'), }, t('bool')],
  ltestr: [{ a: t('string'), b: t('string'), }, t('bool')],
  gti8: [{ a: t('int8'), b: t('int8'), }, t('bool')],
  gti16: [{ a: t('int16'), b: t('int16'), }, t('bool')],
  gti32: [{ a: t('int32'), b: t('int32'), }, t('bool')],
  gti64: [{ a: t('int64'), b: t('int64'), }, t('bool')],
  gtf32: [{ a: t('float32'), b: t('float32'), }, t('bool')],
  gtf64: [{ a: t('float64'), b: t('float64'), }, t('bool')],
  gtstr: [{ a: t('string'), b: t('string'), }, t('bool')],
  gtei8: [{ a: t('int8'), b: t('int8'), }, t('bool')],
  gtei16: [{ a: t('int16'), b: t('int16'), }, t('bool')],
  gtei32: [{ a: t('int32'), b: t('int32'), }, t('bool')],
  gtei64: [{ a: t('int64'), b: t('int64'), }, t('bool')],
  gtef32: [{ a: t('float32'), b: t('float32'), }, t('bool')],
  gtef64: [{ a: t('float64'), b: t('float64'), }, t('bool')],
  gtestr: [{ a: t('string'), b: t('string'), }, t('bool')],
  httpget: [{ a: t('string')}, t('Result<string>')],
  httppost: [{ a: t('string'), b: t('string')}, t('Result<string>')],
  httplsn: [{ a: t('int64'), }, t('Result<string>')],
  httpsend: [{ a: t('InternalResponse'), }, t('Result<string>')],
  execop: [{ a: t('string')}, t('ExecRes')],
  waitop: [{ a: t('int64')}, t('void')],
  catstr: [{ a: t('string'), b: t('string'), }, t('string')],
  catarr: [{ a: t('Array<any>'), b: t('Array<any>')}, t('Array<any>')],
  split: [{ str: t('string'), spl: t('string'), }, t('Array<string>')],
  repstr: [{ s: t('string'), n: t('int64'), }, t('string')],
  reparr: [{ arr: t('Array<any>'), n: t('int64'), }, t('Array<any>')],
  matches: [{ s: t('string'), t: t('string'), }, t('bool')],
  indstr: [{ s: t('string'), t: t('string'), }, t('Result<int64>')],
  indarrf: [{ arr: t('Array<any>'), val: t('any'), }, t('Result<int64>')],
  indarrv: [{ arr: t('Array<any>'), val: t('any'), }, t('Result<int64>')],
  lenstr: [{ s: t('string'), }, t('int64')],
  lenarr: [{ arr: t('Array<any>'), }, t('int64')],
  trim: [{ s: t('string'), }, t('string')],
  condfn: [{ cond: t('bool'), optional: t('function'), }, t('any')],
  pusharr: [{ arr: t('Array<any>'), val: t('any'), size: t('int64')}],
  poparr: [{ arr: t('Array<any>')}, t('Result<any>')],
  each: [{ arr: t('Array<any>'), cb: t('function'), }, t('void')],
  eachl: [{ arr: t('Array<any>'), cb: t('function'), }, t('void')],
  map: [{ arr: t('Array<any>'), cb: t('function'), }, t('Array<any>')],
  mapl: [{ arr: t('Array<any>'), cb: t('function'), }, t('Array<any>')],
  reducel: [{ arr: t('Array<any>'), cb: t('function'), }, t('any')],
  reducep: [{ arr: t('Array<any>'), cb: t('function'), }, t('any')],
  foldl: [{ arr: t('InitialReduce<any, anythingElse>'), cb: t('function'), }, t('anythingElse')],
  foldp: [{ arr: t('InitialReduce<any, anythingElse>'), cb: t('function'), }, t('Array<anythingElse>')],
  filter: [{ arr: t('Array<any>'), cb: t('function'), }, t('Array<any>')],
  filterl: [{ arr: t('Array<any>'), cb: t('function'), }, t('Array<any>')],
  find: [{ arr: t('Array<any>'), cb: t('function'), }, t('Result<any>')],
  every: [{ arr: t('Array<any>'), cb: t('function'), }, t('bool')],
  everyl: [{ arr: t('Array<any>'), cb: t('function'), }, t('bool')],
  some: [{ arr: t('Array<any>'), cb: t('function'), }, t('bool')],
  somel: [{ arr: t('Array<any>'), cb: t('function'), }, t('bool')],
  join: [{ arr: t('Array<string>'), sep: t('string'), }, t('string')],
  newarr: [{ size: t('int64'), }, t('Array<any>')],
  stdoutp: [{ out: t('string'), }, t('void')],
  exitop: [{ code: t('int8'), }, t('void')],
  copyfrom: [{ arr: t('Array<any>'), addr: t('int64') }, t('any')],
  copytof: [{ arr: t('Array<any>'), addr: t('int64'), val: t('any') }],
  copytov: [{ arr: t('Array<any>'), addr: t('int64'), val: t('any') }],
  register: [{ arr: t('Array<any>'), addr: t('int64') }, t('Array<any>')],
  copyi8: [{ a: t('int8'), }, t('int8')],
  copyi16: [{ a: t('int16'), }, t('int16')],
  copyi32: [{ a: t('int32'), }, t('int32')],
  copyi64: [{ a: t('int64'), }, t('int64')],
  copyvoid: [{ a: t('void'), }, t('void')],
  copyf32: [{ a: t('float32'), }, t('float32')],
  copyf64: [{ a: t('float64'), }, t('float64')],
  copybool: [{ a: t('bool'), }, t('bool')],
  copystr: [{ a: t('string'), }, t('string')],
  copyarr: [{ a: t('any'), }, t('any')],
  zeroed: [{ }, t('any')],
  lnf64: [{ a: t('float64'), }, t('float64')],
  logf64: [{ a: t('float64'), }, t('float64')],
  sinf64: [{ a: t('float64'), }, t('float64')],
  cosf64: [{ a: t('float64'), }, t('float64')],
  tanf64: [{ a: t('float64'), }, t('float64')],
  asinf64: [{ a: t('float64'), }, t('float64')],
  acosf64: [{ a: t('float64'), }, t('float64')],
  atanf64: [{ a: t('float64'), }, t('float64')],
  sinhf64: [{ a: t('float64'), }, t('float64')],
  coshf64: [{ a: t('float64'), }, t('float64')],
  tanhf64: [{ a: t('float64'), }, t('float64')],
  error: [{ a: t('string'), }, t('Error')],
  ref: [{ a: t('any'), }, t('any')],
  noerr: [{ }, t('Error')],
  errorstr: [{ a: t('Error'), }, t('string')],
  someM:  [{ a: t('any'), size: t('int64'), }, t('Maybe<any>')],
  noneM:  [{ }, t('Maybe<any>')],
  isSome: [{ a: t('Maybe<any>'), }, t('bool')],
  isNone: [{ a: t('Maybe<any>'), }, t('bool')],
  getOrM: [{ a: t('Maybe<any>'), b: t('any'), }, t('any')],
  okR: [{ a: t('any'), size: t('int64'), }, t('Result<any>')],
  err: [{ a: t('string'), }, t('Result<any>')],
  isOk: [{ a: t('Result<any>'), }, t('bool')],
  isErr: [{ a: t('Result<any>'), }, t('bool')],
  getOrR: [{ a: t('Result<any>'), b: t('any'), }, t('any')],
  getOrRS: [{ a: t('Result<any>'), b: t('string'), }, t('string')],
  getR: [{ a: t('Result<any>'), }, t('any')],
  getErr: [{ a: t('Result<any>'), b: t('Error'), }, t('Error')],
  resfrom: [{ arr: t('Array<any>'), addr: t('int64') }, t('Result<any>')],
  mainE: [{ a: t('any'), size: t('int64'), }, t('Either<any, anythingElse>')],
  altE: [{ a: t('anythingElse'), size: t('int64'), }, t('Either<any, anythingElse>')],
  isMain: [{ a: t('Either<any, anythingElse>'), }, t('bool')],
  isAlt: [{ a: t('Either<any, anythingElse>'), }, t('bool')],
  mainOr: [{ a: t('Either<any, anythingElse>'), b: t('any'), }, t('any')],
  altOr: [{ a: t('Either<any, anythingElse>'), b: t('anythingElse'), }, t('anythingElse')],
  hashf: [{ a: t('any'), }, t('int64')],
  hashv: [{ a: t('any'), }, t('int64')],
  dssetf: [{ ns: t('string'), key: t('string'), val: t('any'), }],
  dssetv: [{ ns: t('string'), key: t('string'), val: t('any'), }],
  dshas: [{ ns: t('string'), key: t('string'), }, t('bool')],
  dsdel: [{ ns: t('string'), key: t('string'), }, t('bool')],
  dsgetf: [{ ns: t('string'), key: t('string'), }, t('Result<any>')],
  dsgetv: [{ ns: t('string'), key: t('string'), }, t('Result<any>')],
  newseq: [{ limit: t('int64'), }, t('Seq')],
  seqnext: [{ seq: t('Seq'), }, t('Result<int64>')],
  seqeach: [{ seq: t('Seq'), func: t('function'), }, t('void')],
  seqwhile: [{ seq: t('Seq'), condFn: t('function'), bodyFn: t('function'), }],
  seqdo: [{ seq: t('Seq'), bodyFn: t('function'), }, t('void')],
  selfrec: [{ self: t('Self'), arg: t('any'), }, t('Result<anythingElse>')],
  seqrec: [{ seq: t('Seq'), recurseFn: t('function'), }, t('Self')],
})

export default opcodeModule