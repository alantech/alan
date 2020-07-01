import { v4 as uuid, } from 'uuid'

import Box from './Box' // TODO: Eliminate Box
import Event from './Event'
import Interface from './Interface'
import Microstatement from './Microstatement'
import Module from './Module'
import Scope from './Scope'
import StatementType from './StatementType'
import Type from './Type'

const opcodeScope = new Scope()
const opcodeModule = new Module(opcodeScope)

// Base types
const addBuiltIn = (name: string) => { opcodeScope.put(name, new Box(Type.builtinTypes[name])) }
([
  'void', 'int8', 'int16', 'int32', 'int64', 'float32', 'float64', 'bool', 'string', 'function',
  'operator', 'Error', 'Array', 'Map', 'KeyVal',
].map(addBuiltIn))
Type.builtinTypes['Array'].solidify(['string'], opcodeScope)
Type.builtinTypes['Map'].solidify(['string', 'string'], opcodeScope)
opcodeScope.put('any', new Box(new Type('any', true, new Interface('any'))))
Type.builtinTypes['Array'].solidify(['any'], opcodeScope)
Type.builtinTypes['Map'].solidify(['any', 'any'], opcodeScope)
Type.builtinTypes['KeyVal'].solidify(['any', 'any'], opcodeScope)
Type.builtinTypes['Array'].solidify(['KeyVal<any, any>'], opcodeScope)
opcodeScope.put("start", new Box(new Event("_start", Type.builtinTypes.void, true), true))
const t = (str: string) => opcodeScope.get(str).typeval

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
        isNary: () => false,
        isPure: () => true,
        microstatementInlining: (
          realArgNames: Array<string>,
          scope: Scope,
          microstatements: Array<Microstatement>,
        ) => {
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
      opcodeScope.put(opcodeName, new Box([opcodeObj], true))
    } else {
      const opcodeObj = {
        getName: () => opcodeName,
        getArguments: () => args,
        getReturnType: () => returnType,
        isNary: () => false,
        isPure: () => true,
        microstatementInlining: (
          realArgNames: Array<string>,
          scope: Scope,
          microstatements: Array<Microstatement>,
        ) => {
          const inputs = realArgNames.map(n => Microstatement.fromVarName(n, microstatements))
          const inputTypes = inputs.map(i => i.outputType)
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
                return replacementType
              } else if (
                returnType.originalType &&
                Object.values(returnType.properties).some((p: Type) => !!p.iface)
              ) {
                // Path 2: the opcode returns solidified generic type with an interface generic that
                // mathces the interface type of an input
                const returnIfaces = Object.values(returnType.properties)
                  .filter((p: Type) => !!p.iface).map((p: Type) => p.iface)
                const ifaceMap = {}
                Object.values(args).forEach((a: Type, i: number) => {
                  if (!!a.iface) {
                    ifaceMap[a.iface.interfacename] = inputTypes[i]
                  }
                })
                const baseType = returnType.originalType
                if (Object.keys(ifaceMap).length >= Object.keys(baseType.generics).length) {
                  const solidTypes = returnIfaces.map(i => ifaceMap[i.interfacename])
                  const newReturnType = baseType.solidify(solidTypes, scope)
                  return newReturnType
                } else {
                  return returnType
                }
              }
              return returnType
            })(inputTypes, scope),
            realArgNames,
            [opcodeObj],
          ))
        },
      }
      // Add each opcode
      opcodeScope.put(opcodeName, new Box([opcodeObj], true))
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
  execop: [{ a: t('string')}, t('void')],
  waitop: [{ a: t('int64')}, t('void')],
  catstr: [{ a: t('string'), b: t('string'), }, t('string')],
  catarr: [{ a: t('Array<any>'), b: t('string')}, t('Array<any>')],
  split: [{ str: t('string'), spl: t('string'), }, t('Array<string>')],
  repstr: [{ s: t('string'), n: t('int64'), }, t('string')],
  reparr: [{ arr: t('Array<any>'), n: t('int64'), }, t('Array<any>')],
  templ: [{ str: t('string'), map: t('Map<string, string>'), }, t('string')],
  matches: [{ s: t('string'), t: t('string'), }, t('bool')],
  indstr: [{ s: t('string'), t: t('string'), }, t('int64')],
  indarrf: [{ arr: t('Array<any>'), val: t('any'), }, t('int64')],
  indarrv: [{ arr: t('Array<any>'), val: t('any'), }, t('int64')],
  lenstr: [{ s: t('string'), }, t('int64')],
  lenarr: [{ arr: t('Array<any>'), }, t('int64')],
  lenmap: [{ map: t('Map<any, any>'), }, t('int64')],
  trim: [{ s: t('string'), }, t('string')],
  condfn: [{ cond: t('bool'), optional: t('function'), }, t('any')],
  pusharr: [{ arr: t('Array<any>'), val: t('any'), size: t('int64')}],
  poparr: [{ arr: t('Array<any>')}, t('any')],
  each: [{ arr: t('Array<any>'), cb: t('function'), }, t('void')],
  map: [{ arr: t('Array<any>'), cb: t('function'), }, t('Array<any>')],
  reduce: [{ arr: t('Array<any>'), cb: t('function'), }, t('any')],
  filter: [{ arr: t('Array<any>'), cb: t('function'), }, t('Array<any>')],
  find: [{ arr: t('Array<any>'), cb: t('function'), }, t('any')],
  every: [{ arr: t('Array<any>'), cb: t('function'), }, t('bool')],
  some: [{ arr: t('Array<any>'), cb: t('function'), }, t('bool')],
  join: [{ arr: t('Array<string>'), sep: t('string'), }, t('string')],
  newarr: [{ size: t('int64'), }, t('Array<any>')],
  keyVal: [{ map: t('Map<any, any>'), }, t('Array<KeyVal<any, any>>')],
  keys: [{ map: t('Map<any, any>'), }, t('Array<any>')],
  values: [{ map: t('Map<any, any>'), }, t('Array<any>')],
  stdoutp: [{ out: t('string'), }, t('void')],
  exitop: [{ code: t('int8'), }, t('void')],
  copyfrom: [{ arr: t('Array<any>'), addr: t('int64') }, t('any')],
  copytof: [{ arr: t('Array<any>'), val: t('any'), addr: t('int64') }],
  copytov: [{ arr: t('Array<any>'), val: t('any'), addr: t('int64') }],
  register: [{ arr: t('Array<any>'), addr: t('int64') }, t('Array<any>')],
  copyi8: [{ a: t('int8'), }, t('int8')],
  copyi16: [{ a: t('int16'), }, t('int16')],
  copyi32: [{ a: t('int32'), }, t('int32')],
  copyi64: [{ a: t('int64'), }, t('int64')],
  copyf32: [{ a: t('float32'), }, t('float32')],
  copyf64: [{ a: t('float64'), }, t('float64')],
  copybool: [{ a: t('bool'), }, t('bool')],
  copystr: [{ a: t('string'), }, t('string')],
  copyarr: [{ a: t('Array<any>'), }, t('Array<any>')],
})

export default opcodeModule