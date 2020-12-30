import { v4 as uuid, } from 'uuid'

import * as Ast from './Ast'
import Event from './Event'
import Operator from './Operator'
import Constant from './Constant'
import Scope from './Scope'
import Statement from './Statement'
import StatementType from './StatementType'
import Type from './Type'
import UserFunction from './UserFunction'
import { Args, Fn, } from './Function'
import { LPNode, ZeroOrMore, } from '../lp'

const FIXED_TYPES = [ 'int64', 'int32', 'int16', 'int8', 'float64', 'float32', 'bool', 'void' ]

class Microstatement {
  statementType: StatementType
  scope: Scope
  pure: boolean
  outputName: string
  alias: string
  outputType: Type
  inputNames: Array<string>
  fns: Array<Fn>
  closurePure: boolean
  closureStatements: Array<Microstatement>
  closureArgs: Args
  closureOutputType: Type

  constructor(
    statementType: StatementType,
    scope: Scope,
    pure: boolean,
    outputName: string,
    outputType: Type = Type.builtinTypes.void,
    inputNames: Array<string> = [],
    fns: Array<Fn> = [],
    alias: string = '',
    closurePure: boolean = true,
    closureStatements: Array<Microstatement> = [],
    closureArgs: Args = {},
    closureOutputType: Type = Type.builtinTypes.void,
  ) {
    this.statementType = statementType
    this.scope = scope
    this.pure = pure
    this.outputName = outputName
    this.outputType = outputType
    this.inputNames = inputNames
    this.fns = fns
    this.alias = alias
    this.closurePure = closurePure
    this.closureStatements = closureStatements
    this.closureArgs = closureArgs
    this.closureOutputType = closureOutputType
  }

  toString() {
    let outString = ""
    switch (this.statementType) {
      case StatementType.CONSTDEC:
        outString = "const " + this.outputName + ": " + this.outputType.typename
        if (this.fns.length > 0) {
          outString += " = " + this.fns[0].getName() + "(" + this.inputNames.join(", ") + ")"
        } else if (this.inputNames.length > 0) {
          outString += " = " + this.inputNames[0] // Doesn't appear the list is ever used here
        }
        break
      case StatementType.LETDEC:
        outString = "let " + this.outputName + ": " + this.outputType.typename
        if (this.fns.length > 0) {
          outString += " = " + this.fns[0].getName() + "(" + this.inputNames.join(", ") + ")"
        } else if (this.inputNames.length > 0) {
          outString += " = " + this.inputNames[0] // Doesn't appear the list is ever used here
        }
        break
      case StatementType.ASSIGNMENT:
        outString = this.outputName
        if (this.fns.length > 0) {
          outString += " = " + this.fns[0].getName() + "(" + this.inputNames.join(", ") + ")"
        } else if (this.inputNames.length > 0) {
          outString += " = " + this.inputNames[0] // Doesn't appear the list is ever used here
        } else {
          outString += "NO!"
        }
        break
      case StatementType.CALL:
        if (this.fns.length > 0) {
          outString += this.fns[0].getName() + "(" + this.inputNames.join(", ") + ")"
        }
        break
      case StatementType.EMIT:
        outString = "emit " + this.outputName + " "
        if (this.fns.length > 0) {
          outString += this.fns[0].getName() + "(" + this.inputNames.join(", ") + ")"
        } else if (this.inputNames.length > 0) {
          outString += this.inputNames[0] // Doesn't appear the list is ever used here
        }
        break
      case StatementType.EXIT:
        outString = "return " + this.outputName
        break
      case StatementType.CLOSURE:
        outString = "const " + this.outputName + ": function = fn ("
        let args = []
        for (const [name, type] of Object.entries(this.closureArgs)) {
          if (name !== "" && type.typename != "") {
            args.push(name + ": " + type.typename)
          }
        }
        outString += args.join(",")
        outString += "): " + this.closureOutputType.typename + " {\n"
        for (const m of this.closureStatements) {
          const s = m.toString()
          if (s !== "") {
            outString += "    " + m.toString() + "\n"
          }
        }
        outString += "  }"
        break
      case StatementType.REREF:
      case StatementType.ARG:
      case StatementType.CLOSUREDEF:
        // Intentionally never output anything, this is metadata for the transpiler algo only
        break
    }
    return outString
  }

  static fromVarName(varName: string, scope: Scope, microstatements: Array<Microstatement>) {
    let original = null
    for (let i = microstatements.length - 1; i > -1; i--) {
      const microstatement = microstatements[i]
      // TODO: var resolution is complex. Need to revisit this.
      if (microstatement.outputName === varName) {
        original = microstatement
        if (microstatement.statementType !== StatementType.REREF) {
          break
        }
      }
      if (microstatement.alias === varName) {
        original = microstatement
        for (let j = i - 1; j >= 0; j--) {
          if (
            microstatements[j].outputName === original.outputName &&
            microstatements[j].statementType !== StatementType.REREF
          ) {
            original = microstatements[j]
            break
          }
        }
        break
      }
    }
    // Check if this is a module constant that should be un-hoisted
    if (
      original === null &&
      !!scope.deepGet(varName) &&
      scope.deepGet(varName) instanceof Constant
    ) {
      const globalConst = scope.deepGet(varName) as Constant
      Microstatement.fromAssignablesAst(
        globalConst.assignablesAst,
        globalConst.scope, // Eval this in its original scope in case it was an exported const
        microstatements    // that was dependent on unexported internal functions or constants
      )
      const last = microstatements[microstatements.length - 1]
      microstatements.push(new Microstatement(
        StatementType.REREF,
        scope,
        true,
        last.outputName,
        last.outputType,
        [],
        [],
        globalConst.name,
      ))
    }
    return original
  }

  // TODO: Eliminate ANTLR
  static fromVarAst(varAst: any, scope: Scope, microstatements: Array<Microstatement>) {
    // Short-circuit if this exact var was already loaded
    let original = Microstatement.fromVarName(varAst.getText(), scope, microstatements)
    if (!original) {
      // Otherwise, we're digging in piece by piece to find the relevant microstatement.
      const segments = varAst.varsegment()
      let name = ''
      for (const segment of segments) {
        // A 'normal' segment. Either append it to the name and attempt to get the underlying
        // sub-name or rewrite it into an array access if it's a user-defined type field name
        if (segment.VARNAME()) {
          // Decide if this is an access on a user-defined type that should be rewritten as an array
          // access instead
          name += segment.VARNAME().getText()
          if (!original || !original.outputType || original.outputType.builtIn) {
            original = Microstatement.fromVarName(name, scope, microstatements)
          } else {
            // Next, figure out which field number this is
            const fieldName = segment.VARNAME().getText()
            const fields = Object.keys(original.outputType.properties)
            const fieldNum = fields.indexOf(fieldName)
            if (fieldNum < 0) {
              // Invalid object access
              throw new Error(`${name} does not have a field named ${fieldName}
${varAst.getText()} on line ${varAst.start.line}:${varAst.start.column}`)
            }
            // Create a new variable to hold the address within the array literal
            const addrName = "_" + uuid().replace(/-/g, "_")
            microstatements.push(new Microstatement(
              StatementType.CONSTDEC,
              scope,
              true,
              addrName,
              Type.builtinTypes['int64'],
              [`${fieldNum}`],
              [],
            ))
            // Insert a `register` opcode.
            const opcodes = require('./opcodes').default
            opcodes.exportScope.get('register')[0].microstatementInlining(
              [original.outputName, addrName],
              scope,
              microstatements,
            )
            // We'll need a reference to this for later
            const typeRecord = original
            // Set the original to this newly-generated microstatement
            original = microstatements[microstatements.length - 1]
            // Now we do something odd, but correct here; we need to replace the `outputType` from
            // `any` to the type that was actually copied so function resolution continues to work
            original.outputType = typeRecord.outputType.properties[fieldName]
          }
        }
        // A separator, just append it and do nothing else
        if (segment.METHODSEP()) {
          name += segment.METHODSEP().getText()
        }
        // An array access. This requires resolving the contents of the array access variable and
        // then using that value to find the correct index to read from. For now, that will be
        // emitting a `resfrom` opcode call. Also for now it is an error if the resolved type is
        // anything but `int64` for the array access path. Maps use the same syntax with the type
        // being the Map's Key type.
        if (segment.arrayaccess()) {
          if (original == null || !(original instanceof Microstatement)) {
            // This is all moot if we didn't resolve a variable to dig into
            throw new Error(`${name} cannot be found
${varAst.getText()} on line ${varAst.start.line}:${varAst.start.column}`)
          }
          // We're still ID'ing it with the raw text to make the short-circuit work
          name += segment.arrayaccess().getText()
          const assignables = segment.arrayaccess().assignables()
          Microstatement.fromAssignablesAst(assignables, scope, microstatements)
          const lookup = microstatements[microstatements.length - 1]
          // TODO: Map support, which requires figuring out if the outer memory object is an array
          // or a map.
          if (lookup.outputType.typename === 'int64') {
            const opcodes = require('./opcodes').default
            // Create a new variable to hold the `okR` size value
            const sizeName = "_" + uuid().replace(/-/g, "_")
            microstatements.push(new Microstatement(
              StatementType.CONSTDEC,
              scope,
              true,
              sizeName,
              Type.builtinTypes['int64'],
              ['8'],
              [],
            ))
            // Insert an `okR` opcode.
            opcodes.exportScope.get('okR')[0].microstatementInlining(
              [lookup.outputName, sizeName],
              scope,
              microstatements,
            )
            const wrapped = microstatements[microstatements.length - 1]
            // Insert a `resfrom` opcode.
            opcodes.exportScope.get('resfrom')[0].microstatementInlining(
              [original.outputName, wrapped.outputName],
              scope,
              microstatements,
            )
          } else if (lookup.outputType.typename === 'Result<int64>') {
            const opcodes = require('./opcodes').default
            // Insert a `resfrom` opcode.
            opcodes.exportScope.get('resfrom')[0].microstatementInlining(
              [original.outputName, lookup.outputName],
              scope,
              microstatements,
            )
          } else {
            throw new Error(`${segment.getText()} cannot be used in an array lookup as it is not an int64 or Result<int64>
${varAst.getText()} on line ${varAst.start.line}:${varAst.start.column}`)
          }
          // We'll need a reference to this for later
          const arrayRecord = original
          // Set the original to this newly-generated microstatement
          original = microstatements[microstatements.length - 1]
          // Now we do something odd, but correct here; we need to replace the `outputType` from
          // `any` to the type that was actually copied so function resolution continues to work
          original.outputType = Type.builtinTypes.Result.solidify(
            [Object.values(arrayRecord.outputType.properties)[0].typename],
            scope,
          )
        }
      }
    }
    if (original == null || !(original instanceof Microstatement)) {
      throw new Error(`${varAst.getText()} cannot be found
${varAst.getText()} on line ${varAst.start.line}:${varAst.start.column}`)
    }
    // When a variable is reassigned (or was referenced in a function call or operator statement,
    // instead of duplicating its data, add a microstatement to rereference that data (all of the
    // function and operator calls expect their arguments to be the N statements preceding them).
    microstatements.push(new Microstatement(
      StatementType.REREF,
      scope,
      true,
      original.outputName,
      original.outputType,
      [],
      [],
    ))
  }

  // TODO: Eliminate ANTLR
  static fromConstantsAst(constantsAst: any, scope: Scope, microstatements: Array<Microstatement>) {
    const constName = "_" + uuid().replace(/-/g, "_")
    let constType: string = 'void'
    if (constantsAst.BOOLCONSTANT() != null) {
      constType = 'bool'
    }
    if (constantsAst.STRINGCONSTANT() != null) {
      constType = 'string'
    }
    if (constantsAst.NUMBERCONSTANT() != null) {
      // TODO: Add support for hex, octal, scientific, etc
      const numberConst = constantsAst.NUMBERCONSTANT().getText()
      if (numberConst.indexOf('.') > -1) { // It's a float
        constType = 'float64'
      } else { // It's an integer
        constType = 'int64'
      }
    }
    let constVal: string
    try {
      JSON.parse(constantsAst.getText()) // Will fail on strings with escape chars
      constVal = constantsAst.getText()
    } catch (e) {
      // It may be a zero-padded number
      if (
        ['int8', 'int16', 'int32', 'int64'].includes(constType) &&
        constantsAst.getText()[0] === '0'
      ) {
        constVal = parseInt(constantsAst.getText(), 10).toString()
      } else if (
        ['float32', 'float64'].includes(constType) &&
        constantsAst.getText()[0] === '0'
      ) {
        constVal = parseFloat(constantsAst.getText()).toString()
      } else {
        // Hackery to get these strings to work
        constVal = JSON.stringify(constantsAst.getText()
          .replace(/^["']/, '').replace(/["']$/, ''))
      }
    }
    microstatements.push(new Microstatement(
      StatementType.CONSTDEC,
      scope,
      true,
      constName,
      scope.deepGet(constType) as Type,
      [constVal],
      [],
    ))
  }

  static fromObjectLiteralsAst(
    objectLiteralsAst: any, // TODO: Eliminate ANTLR
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    if (objectLiteralsAst.arrayliteral()) {
      // Array literals first need all of the microstatements of the array contents defined, then
      // a `newarr` opcode call is inserted for the object literal itself, then `pusharr` opcode
      // calls are emitted to insert the relevant data into the array, and finally the array itself
      // is REREFed for the outer microstatement generation call.
      const arrayLiteralContents = []
      const assignablelist = objectLiteralsAst.arrayliteral().arraybase().assignablelist()
      const assignableLen = assignablelist ? assignablelist.assignables().length : 0
      for (let i = 0; i < assignableLen; i++) {
        Microstatement.fromAssignablesAst(assignablelist.assignables(i), scope, microstatements)
        arrayLiteralContents.push(microstatements[microstatements.length - 1])
      }
      let typeBox = null
      if (objectLiteralsAst.arrayliteral().literaldec()) {
        typeBox = scope.deepGet(
          objectLiteralsAst.arrayliteral().literaldec().fulltypename().getText().trim()
        ) as Type
        if (!typeBox) {
          // Try to define it if it's a generic type
          if (objectLiteralsAst.arrayliteral().literaldec().fulltypename().typegenerics()) {
            const outerTypeBox = scope.deepGet(
              objectLiteralsAst.arrayliteral().literaldec().fulltypename().typename().getText().trim()
            ) as Type
            if (!outerTypeBox) {
              throw new Error(`${objectLiteralsAst.arrayliteral().literaldec().fulltypename().getText()}  is not defined
${objectLiteralsAst.getText()} on line ${objectLiteralsAst.start.line}:${objectLiteralsAst.start.column}`)
            }
            outerTypeBox.solidify(
              objectLiteralsAst.arrayliteral().literaldec().fulltypename().typegenerics().fulltypename().map(
                (t: any) => t.getText() // TODO: Eliminate ANTLR
              ),
              scope
            )
            typeBox = scope.deepGet(objectLiteralsAst.arrayliteral().literaldec().fulltypename().getText().trim())
          }
        }
        if (!(typeBox instanceof Type)) {
          throw new Error(`${objectLiteralsAst.arrayliteral().literaldec().fulltypename().getText().trim()} is not a type
${objectLiteralsAst.getText()} on line ${objectLiteralsAst.start.line}:${objectLiteralsAst.start.column}`)
        }
      } else if (arrayLiteralContents.length > 0) {
        const innerType = arrayLiteralContents[0].outputType.typename
        Type.builtinTypes['Array'].solidify([innerType], scope)
        typeBox = scope.deepGet(`Array<${innerType}>`) as Type
      } else {
        throw new Error(`Ambiguous array type, please specify the type for an empty array with the syntax \`new Array<MyType> []\`
${objectLiteralsAst.getText()} on line ${objectLiteralsAst.start.line}:${objectLiteralsAst.start.column}`)
      }
      // Create a new variable to hold the size of the array literal
      const lenName = "_" + uuid().replace(/-/g, "_")
      microstatements.push(new Microstatement(
        StatementType.CONSTDEC,
        scope,
        true,
        lenName,
        Type.builtinTypes['int64'],
        [`${arrayLiteralContents.length}`],
        [],
      ))
      // Add the opcode to create a new array with the specified size
      const opcodes = require('./opcodes').default
      opcodes.exportScope.get('newarr')[0].microstatementInlining(
        [lenName],
        scope,
        microstatements,
      )
      // Get the array microstatement and extract the name and insert the correct type
      const array = microstatements[microstatements.length - 1]
      array.outputType = typeBox
      // Try to use the "real" type if knowable
      if (arrayLiteralContents.length > 0) {
        array.outputType = Type.builtinTypes['Array'].solidify(
          [arrayLiteralContents[0].outputType.typename],
          scope,
        )
      }
      const arrayName = array.outputName
      // Push the values into the array
      for (let i = 0; i < arrayLiteralContents.length; i++) {
        // Create a new variable to hold the size of the array value
        const size = FIXED_TYPES.includes(arrayLiteralContents[i].outputType.typename) ? "8" : "0"
        const sizeName = "_" + uuid().replace(/-/g, "_")
        microstatements.push(new Microstatement(
          StatementType.CONSTDEC,
          scope,
          true,
          sizeName,
          Type.builtinTypes['int64'],
          [size],
          [],
        ))
        // Push the value into the array
        const opcodes = require('./opcodes').default
        opcodes.exportScope.get('pusharr')[0].microstatementInlining(
          [arrayName, arrayLiteralContents[i].outputName, sizeName],
          scope,
          microstatements,
        )
      }
      // REREF the array
      microstatements.push(new Microstatement(
        StatementType.REREF,
        scope,
        true,
        arrayName,
        array.outputType,
        [],
        [],
      ))
    } else if (!!objectLiteralsAst.typeliteral()) {
      // User types are represented in AMM and lower as `Array<any>`. This reduces the number of
      // concepts that have to be maintained in the execution layer (and is really what C structs
      // are, anyways). The order of the properties on the specified type directly map to the
      // order that they are inserted into the Array, not the order they're defined in the object
      // literal notation, so reads and updates later on can occur predictably by mapping the name
      // of the property to its array index.
      //
      // If the type literal is missing any fields, that's a hard compile error to make sure
      // accessing undefined data is impossible. If a value might not be needed, they should use
      // the `Option` type and provide a `None` value there.
      let typeBox = scope.deepGet(
        objectLiteralsAst.typeliteral().literaldec().fulltypename().getText().trim()
      ) as Type
      if (typeBox === null) {
        // Try to define it if it's a generic type
        if (objectLiteralsAst.typeliteral().literaldec().fulltypename().typegenerics()) {
          const outerTypeBox = scope.deepGet(
            objectLiteralsAst.typeliteral().literaldec().fulltypename().typename().getText().trim()
          )
          if (outerTypeBox === null) {
            throw new Error(`${objectLiteralsAst.typeliteral().literaldec().fulltypename().getText()}  is not defined
${objectLiteralsAst.getText()} on line ${objectLiteralsAst.start.line}:${objectLiteralsAst.start.column}`)
          }
          (outerTypeBox as Type).solidify(
            objectLiteralsAst.typeliteral().literaldec().fulltypename().typegenerics().fulltypename().map(
              (t: any) => t.getText() // TODO: Eliminate ANTLR
            ),
            scope
          )
          typeBox = scope.deepGet(objectLiteralsAst.typeliteral().literaldec().fulltypename().getText().trim()) as Type
        }
      }
      if (!(typeBox instanceof Type)) {
        throw new Error(`${objectLiteralsAst.typeliteral().literaldec().fulltypename().getText().trim()} is not a type
${objectLiteralsAst.getText()} on line ${objectLiteralsAst.start.line}:${objectLiteralsAst.start.column}`)
      }
      const assignlist = objectLiteralsAst.typeliteral().typebase().typeassignlist()
      const assignfields = assignlist.VARNAME().map((f: any) => f.getText())
      const assignvals = assignlist.assignables()
      const fields = Object.keys(typeBox.properties)
      let missingFields = []
      let foundFields = []
      let extraFields = []
      let astLookup = {}
      for (let i = 0; i < assignfields.length; i++) {
        const assignfield = assignfields[i]
        const assignval = assignvals[i]
        astLookup[assignfield] = assignval
        if (!fields.includes(assignfield)) {
          extraFields.push(assignfield)
        }
        if (foundFields.includes(assignfield)) {
          extraFields.push(assignfield)
        }
        foundFields.push(assignfield)
      }
      for (const field of fields) {
        if (!foundFields.includes(field)) {
          missingFields.push(field)
        }
      }
      if (missingFields.length > 0 || extraFields.length > 0) {
        let errMsg = `${objectLiteralsAst.typeliteral().literaldec().fulltypename().getText().trim()} object literal improperly defined`
        if (missingFields.length > 0) {
          errMsg += '\n' + `Missing fields: ${missingFields.join(', ')}`
        }
        if (extraFields.length > 0) {
          errMsg += '\n' + `Extra fields: ${extraFields.join(', ')}`
        }
        errMsg += '\n' +
          objectLiteralsAst.getText() +
          " on line " +
          objectLiteralsAst.start.line +
          ":" +
          objectLiteralsAst.start.column
        throw new Error(errMsg)
      }
      // The assignment looks good, now we'll mimic the array literal logic mostly
      const arrayLiteralContents = []
      for (let i = 0; i < fields.length; i++) {
        Microstatement.fromAssignablesAst(
          astLookup[fields[i]],
          scope,
          microstatements
        )
        arrayLiteralContents.push(microstatements[microstatements.length - 1])
      }
      // Create a new variable to hold the size of the array literal
      const lenName = "_" + uuid().replace(/-/g, "_")
      microstatements.push(new Microstatement(
        StatementType.CONSTDEC,
        scope,
        true,
        lenName,
        Type.builtinTypes['int64'],
        [`${fields.length}`],
        [],
      ))
      // Add the opcode to create a new array with the specified size
      const opcodes = require('./opcodes').default
      opcodes.exportScope.get('newarr')[0].microstatementInlining(
        [lenName],
        scope,
        microstatements,
      )
      // Get the array microstatement and extract the name and insert the correct type
      const array = microstatements[microstatements.length - 1]
      array.outputType = typeBox
      const arrayName = array.outputName
      // Push the values into the array
      for (let i = 0; i < arrayLiteralContents.length; i++) {
        // Create a new variable to hold the size of the array value
        const size = FIXED_TYPES.includes(arrayLiteralContents[i].outputType.typename) ? "8" : "0"
        const sizeName = "_" + uuid().replace(/-/g, "_")
        microstatements.push(new Microstatement(
          StatementType.CONSTDEC,
          scope,
          true,
          sizeName,
          Type.builtinTypes['int64'],
          [size],
          [],
        ))
        // Push the value into the array
        const opcodes = require('./opcodes').default
        opcodes.exportScope.get('pusharr')[0].microstatementInlining(
          [arrayName, arrayLiteralContents[i].outputName, sizeName],
          scope,
          microstatements,
        )
      }
      // REREF the array
      microstatements.push(new Microstatement(
        StatementType.REREF,
        scope,
        true,
        arrayName,
        array.outputType,
        [],
        [],
      ))
    }
  }

  static closureDef(
    fns: Array<Fn>,
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    const closuredefName = "_" + uuid().replace(/-/g, "_")
    // Keep any rerefs around as closure references
    const rerefs = microstatements.filter(m => m.statementType === StatementType.REREF)
    microstatements.push(new Microstatement(
      StatementType.CLOSUREDEF,
      scope,
      true, // TODO: What should this be?
      closuredefName,
      Type.builtinTypes['function'],
      [],
      fns,
      '',
      true,
      rerefs,
    ))
  }

  static closureFromUserFunction(
    userFunction: UserFunction,
    scope: Scope,
    microstatements: Array<Microstatement>,
    interfaceMap: Map<Type, Type>,
  ) {
    const fn = userFunction.maybeTransform(interfaceMap)
    const idx = microstatements.length
    const args = Object.entries(fn.args)
    for (const [name, type] of args) {
      if (name !== "" && type.typename != "") {
        microstatements.push(new Microstatement(
          StatementType.CONSTDEC,
          scope,
          true,
          name,
          type,
        ))
      }
    }
    const len = microstatements.length - args.length
    for (const s of fn.statements) {
      Microstatement.fromStatementsAst(s.statementAst, scope, microstatements)
    }
    microstatements.splice(idx, args.length)
    const newlen = microstatements.length
    // There might be off-by-one bugs in the conversion here
    const innerMicrostatements = microstatements.slice(len, newlen)
    microstatements.splice(len, newlen - len)
    const constName = "_" + uuid().replace(/-/g, "_")
    // if closure is not void return the last inner statement
    // TODO: Revisit this, if the closure doesn't have a type defined, sometimes it can only be
    // determined in the calling context and shouldn't be assumed to be `void`
    if (innerMicrostatements.length > 0 && fn.getReturnType() !== Type.builtinTypes.void) {
      const last = innerMicrostatements[innerMicrostatements.length - 1]
      innerMicrostatements.push(new Microstatement(
        StatementType.EXIT,
        scope,
        true,
        last.outputName,
        last.outputType
      ))
    }
    microstatements.push(new Microstatement(
      StatementType.CLOSURE,
      scope,
      true, // TODO: Figure out if this is true or not
      constName,
      Type.builtinTypes['function'],
      [],
      [],
      '',
      fn.pure,
      innerMicrostatements,
      fn.args,
      fn.getReturnType(),
    ))
  }

  static fromEmitsAst(
    emitsAst: any, // TODO: Eliminate ANTLR
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    if (emitsAst.assignables() != null) {
      // If there's an assignable value here, add it to the list of microstatements first, then
      // rewrite the final const assignment as the emit statement.
      Microstatement.fromAssignablesAst(emitsAst.assignables(), scope, microstatements)
      const eventBox = scope.deepGet(emitsAst.eventref().getText()) // TODO: Port to fromVarAst when Box is removed
      if (!(eventBox instanceof Event)) {
        throw new Error(`${emitsAst.eventref().getText()} is not an event!
${emitsAst.getText()} on line ${emitsAst.start.line}:${emitsAst.start.column}`)
      }
      const last = microstatements[microstatements.length - 1]
      if (
        last.outputType != eventBox.type &&
        !eventBox.type.castable(last.outputType)
      ) {
        throw new Error(`Attempting to assign a value of type ${last.outputType.typename} to an event of type ${eventBox.type.typename}
${emitsAst.getText()} on line ${emitsAst.start.line}:${emitsAst.start.column}`)
      }
      microstatements.push(new Microstatement(
        StatementType.EMIT,
        scope,
        true,
        eventBox.name,
        eventBox.type,
        [last.outputName],
        [],
      ))
    } else {
      // Otherwise, create an emit statement with no value
      const eventBox = scope.deepGet(emitsAst.eventref().getText()) as Event // TODO: Port to fromVarAst
      if (!(eventBox instanceof Event)) {
        throw new Error(`${emitsAst.eventref().getText()} is not an event!
${emitsAst.getText()} on line ${emitsAst.start.line}:${emitsAst.start.column}`)
      }
      if (eventBox.type != Type.builtinTypes.void) {
        throw new Error(`${emitsAst.eventref().getText()} must have a value emitted to it!
${emitsAst.getText()} on line ${emitsAst.start.line}:${emitsAst.start.column}`)
      }
      microstatements.push(new Microstatement(
        StatementType.EMIT,
        scope,
        true,
        eventBox.name,
        Type.builtinTypes.void,
        [],
        [],
      ))
    }
  }

  static fromExitsAst(
    exitsAst: any, // TODO: Eliminate ANTLR
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    // `alan--` handlers don't have the concept of a `return` statement, the functions are all inlined
    // and the last assigned value for the function *is* the return statement
    if (exitsAst.assignables() != null) {
      // If there's an assignable value here, add it to the list of microstatements
      Microstatement.fromAssignablesAst(
        exitsAst.assignables(),
        scope,
        microstatements
      )
    } else {
      // Otherwise, create a microstatement with no value
      const constName = "_" + uuid().replace(/-/g, "_")
      microstatements.push(new Microstatement(
        StatementType.CONSTDEC,
        scope,
        true,
        constName,
        Type.builtinTypes.void,
        ["void"],
        null
      ))
    }
  }

  static fromAssignmentsAst(
    assignmentsAst: any, // TODO: Eliminate ANTLR
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    // For reassigning to a variable, we need to determine that the root variable is a
    // `let`-defined mutable variable and then tease out if any array or property accesses are done,
    // and if so we need to `register` a mutable reference to the array memory space and then update
    // the value with a `register` call from the assignables result address to the relevant inner
    // address of the last access argument. The format of a `varn` can only be the following:
    // `{moduleScope}.varName[arrayAccess].userProperty` where the array accesses and userProperties
    // can come in any order after the preamble. *Fortunately,* for this scenario, any situation
    // where `moduleScope` is included is invalid since only constants can be exported out of a
    // module, not mutable values, so we only need to read the *first* segment to immediately
    // determine if it is relevant or not -- if it comes back as a `Scope` object we abort with an
    // error. If not, then we find the relevant `Microstatement` and determine if it is a `const`
    // or a `let` declaration and abort if it is a `const`. After that, if there are no segments
    // beyond the first one, we simply take the `assignable` microstatement output and turn it into
    // an `ASSIGNMENT` StatementType, otherwise we need to go through a more complicated procedure
    // to `register` the `n-1` remaining inner array segments to new variables as references and
    // finally `register` the `assignable` into the location the last segment indicates.
    const segments = assignmentsAst.varn().varsegment()
    // Now, find the original variable and confirm that it actually is a let declaration
    const letName = segments[0].getText()
    let actualLetName: string
    let original: Microstatement
    for (let i = microstatements.length - 1; i >= 0; i--) {
      const microstatement = microstatements[i]
      if (microstatement.alias === letName) {
        actualLetName = microstatement.outputName
        continue
      }
      if (microstatement.outputName === actualLetName) {
        if (microstatement.statementType === StatementType.LETDEC) {
          original = microstatement
          break
        } else if (microstatement.statementType === StatementType.REREF) {
          original = Microstatement.fromVarName(microstatement.outputName, scope, microstatements)
          break
        } else if (microstatement.statementType === StatementType.ASSIGNMENT) {
          // We could treat this as evidence that it's cool, but let's just skip it.
          continue
        } else {
          throw new Error(`Attempting to reassign a non-let variable.
${letName} on line ${assignmentsAst.start.line}:${assignmentsAst.start.column}`)
        }
      }
    }
    if (!original) {
      throw new Error(`Attempting to reassign to an undeclared variable
${letName} on line ${assignmentsAst.line}:${assignmentsAst.start.column}`)
    }
    if (segments.length === 1) { // Could be a simple let variable
      const letName = segments[0].getText()
      let actualLetName: string
      for (let i = microstatements.length - 1; i >= 0; i--) {
        const microstatement = microstatements[i]
        if (microstatement.alias === letName) {
          actualLetName = microstatement.outputName
          continue
        }
        if (microstatement.outputName === actualLetName) {
          if (microstatement.statementType === StatementType.LETDEC) {
            break
          } else if (microstatement.statementType === StatementType.REREF) {
            original = Microstatement.fromVarName(microstatement.outputName, scope, microstatements)
            break
          } else if (microstatement.statementType === StatementType.ASSIGNMENT) {
            // Could treat this as evidence that it's okay, but let's be sure about that
            continue
          } else {
            throw new Error(`Attempting to reassign a non-let variable.
${letName} on line ${assignmentsAst.line}:${assignmentsAst.start.column}`)
          }
        }
      }
      Microstatement.fromAssignablesAst(
        assignmentsAst.assignables(),
        scope,
        microstatements
      )
      // By definition the last microstatement is the const assignment we care about, so we can
      // just mutate its object to rename the output variable name to the name we need instead.
      let last = microstatements[microstatements.length - 1]
      if (last.statementType === StatementType.REREF) {
        // Find what it's rereferencing and adjust that, instead
        for (let i = microstatements.length - 2; i >=0; i--) {
          let m = microstatements[i]
          if (m.outputName === last.outputName && m.statementType !== StatementType.REREF) {
            last = m
            break
          }
        }
      }
      if (last.statementType === StatementType.LETDEC) {
        // Insert a ref call for this instead of mutating the original assignment
        Microstatement.fromAssignablesAst(
          Ast.assignablesAstFromString(`ref(${last.outputName})`),
          scope,
          microstatements
        )
        last = microstatements[microstatements.length - 1]
        if (last.statementType === StatementType.REREF) {
          // Find what it's rereferencing and adjust that, instead
          for (let i = microstatements.length - 2; i >=0; i--) {
            let m = microstatements[i]
            if (m.outputName === last.outputName && m.statementType !== StatementType.REREF) {
              last = m
              break
            }
          }
        }
      }
      last.outputName = actualLetName
      last.statementType = StatementType.ASSIGNMENT
      // Attempt to "merge" the output types, useful for multiple branches assigning into the same
      // variable but only part of the type information is known in each branch (like in `Result`
      // or `Either` with the result value only in one branch or one type in each of the branches
      // for `Either`).
      if (original.outputType.typename !== last.outputType.typename) {
        if (!!original.outputType.iface) {
          // Just overwrite if it's an interface type
          original.outputType = last.outputType
        } else if (
          !!original.outputType.originalType &&
          !!last.outputType.originalType &&
          original.outputType.originalType.typename === last.outputType.originalType.typename
        ) {
          // The tricky path, let's try to merge the two types together
          const baseType = original.outputType.originalType
          const originalTypeAst = Ast.fulltypenameAstFromString(original.outputType.typename)
          const lastTypeAst = Ast.fulltypenameAstFromString(last.outputType.typename)
          const originalSubtypes = []
          if (originalTypeAst.has('opttypegenerics')) {
            const originalTypeGenerics = originalTypeAst.get('opttypegenerics')
            originalSubtypes.push(originalTypeGenerics.get('fulltypename').t);
            (originalTypeGenerics.get('cdr') as ZeroOrMore).zeroOrMore.forEach(r => {
              originalSubtypes.push(r.get('fulltypename').t)
            })
          }
          const lastSubtypes = []
          if (lastTypeAst.has('opttypegenerics')) {
            const lastTypeGenerics = lastTypeAst.get('opttypegenerics')
            lastSubtypes.push(lastTypeGenerics.get('fulltypename').t);
            (lastTypeGenerics.get('cdr') as ZeroOrMore).zeroOrMore.forEach(r => {
              lastSubtypes.push(r.get('fulltypename').t)
            })
          }
          const newSubtypes = []
          for (let i = 0; i < originalSubtypes.length; i++) {
            if (originalSubtypes[i] === lastSubtypes[i]) {
              newSubtypes.push(originalSubtypes[i])
            } else {
              let originalSubtype = scope.deepGet(originalSubtypes[i]) as Type
              if (!!originalSubtype.iface) {
                newSubtypes.push(lastSubtypes[i])
              } else if (!!originalSubtype.originalType) {
                // TODO: Support nesting
                newSubtypes.push(originalSubtypes[i])
              } else {
                newSubtypes.push(originalSubtypes[i])
              }
            }
          }
          const newType = baseType.solidify(newSubtypes, scope)
          original.outputType = newType
        } else {
          // Hmm... what to do here?
          original.outputType = last.outputType
        }
      }
      return
    }
    // The more complicated path. First, rule out that the first segment is not a `scope`.
    const testBox = scope.deepGet(segments[0].getText())
    if (!!testBox && testBox instanceof Scope) {
      throw new Error(`Atempting to reassign to variable from another module
${assignmentsAst.varn().getText()} on line ${assignmentsAst.start.line}:${assignmentsAst.start.column}`)
    }
    let nestedLetType = original.outputType
    for (let i = 1; i < segments.length - 1; i++) {
      const segment = segments[i]
      // A separator, just do nothing else this loop
      if (segment.METHODSEP()) continue
      // An array access. Until the grammar definition is reworked, this will parse correctly, but
      // it is banned in alan (due to being unable to catch and report assignment errors to arrays)
      if (segment.arrayaccess()) {
        throw new Error(`${segments.join('')} cannot be written to. Please use 'set' to mutate arrays and hash tables`)
      }
      // If it's a varname here, then we're accessing an inner property type. We need to figure out
      // which index it is in the underlying array structure and then `register` that piece (since
      // this is an intermediate access and not the final access point)
      if (segment.VARNAME()) {
        const fieldName = segment.VARNAME().getText()
        const fields = Object.keys(nestedLetType.properties)
        const fieldNum = fields.indexOf(fieldName)
        if (fieldNum < 0) {
          // Invalid object access
          throw new Error(`${letName} does not have a field named ${fieldName}
${assignmentsAst.varn().getText()} on line ${assignmentsAst.varn().start.line}:${assignmentsAst.varn().start.column}`)
        }
        // Create a new variable to hold the address within the array literal
        const addrName = "_" + uuid().replace(/-/g, "_")
        microstatements.push(new Microstatement(
          StatementType.CONSTDEC,
          scope,
          true,
          addrName,
          Type.builtinTypes['int64'],
          [`${fieldNum}`],
          [],
        ))
        // Insert a `register` opcode.
        const opcodes = require('./opcodes').default
        opcodes.exportScope.get('register')[0].microstatementInlining(
          [original.outputName, addrName],
          scope,
          microstatements,
        )
        // Now, we need to update the type we're working with.
        nestedLetType = Object.values(nestedLetType.properties)[fieldNum]
        // Now update the `original` record to the new `register` result
        original = microstatements[microstatements.length - 1]
      }
    }
    Microstatement.fromAssignablesAst(
      assignmentsAst.assignables(),
      scope,
      microstatements
    )
    // Grab a reference to the final assignment variable.
    const assign = microstatements[microstatements.length - 1]
    // Next, determine which kind of final segment this is and perform the appropriate action to
    // insert into with a `copytof` or `copytov` opcode.
    const copytoop = [
      'int8', 'int16', 'int32', 'int64', 'float32', 'float64', 'bool'
    ].includes(assign.outputType.typename) ? 'copytof' : 'copytov'
    const finalSegment = segments[segments.length - 1]
    if (finalSegment.arrayaccess()) {
      const assignables = finalSegment.arrayaccess().assignables()
      Microstatement.fromAssignablesAst(assignables, scope, microstatements)
      const lookup = microstatements[microstatements.length - 1]
      // TODO: Map support, which requires figuring out if the outer memory object is an array
      // or a map.
      if (lookup.outputType.typename !== 'int64') {
        throw new Error(`${finalSegment.getText()} cannot be used in an array lookup as it is not an int64
${letName} on line ${assignmentsAst.start.line}:${assignmentsAst.start.column}`)
      }
      // Insert a `copytof` or `copytov` opcode.
      const opcodes = require('./opcodes').default
      opcodes.exportScope.get(copytoop)[0].microstatementInlining(
        [original.outputName, lookup.outputName, assign.outputName],
        scope,
        microstatements,
      )
    } else if (finalSegment.VARNAME()) {
      const fieldName = finalSegment.VARNAME().getText()
      const fields = Object.keys(nestedLetType.properties)
      const fieldNum = fields.indexOf(fieldName)
      if (fieldNum < 0) {
        // Invalid object access
        throw new Error(`${letName} does not have a field named ${fieldName}
${letName} on line ${assignmentsAst.start.line}:${assignmentsAst.start.column}`)
      }
      // Check if the new variable is allowed to be assigned to this object
      const originalType = nestedLetType.properties[fieldName]
      if (!originalType.typeApplies(assign.outputType, scope)) {
        throw new Error(`${letName}.${fieldName} is of type ${originalType.typename} but assigned a value of type ${assign.outputType.typename}`)
      }
      // Create a new variable to hold the address within the array literal
      const addrName = "_" + uuid().replace(/-/g, "_")
      microstatements.push(new Microstatement(
        StatementType.CONSTDEC,
        scope,
        true,
        addrName,
        Type.builtinTypes['int64'],
        [`${fieldNum}`],
        [],
      ))
      // Insert a `copytof` or `copytov` opcode.
      const opcodes = require('./opcodes').default
      opcodes.exportScope.get(copytoop)[0].microstatementInlining(
        [original.outputName, addrName, assign.outputName],
        scope,
        microstatements,
      )
    } else {
      throw new Error(`${finalSegment.getText()} cannot be the final piece in a reassignment statement
${letName} on line ${assignmentsAst.start.line}:${assignmentsAst.start.column}`)
    }
  }

  static fromLetdeclarationAst(
    letdeclarationAst: any, // TODO: Eliminate ANTLR
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    const letAlias = letdeclarationAst.VARNAME().getText()
    const letTypeHint = letdeclarationAst.fulltypename() ? letdeclarationAst.fulltypename().getText() : ''
    const typeBox = scope.deepGet(letTypeHint)
    if (typeBox === null && letTypeHint !== '') {
      // Try to define it if it's a generic type
      if (letdeclarationAst.fulltypename().typegenerics()) {
        const outerTypeBox = scope.deepGet(
          letdeclarationAst.fulltypename().typename().getText()
        ) as Type
        if (outerTypeBox === null) {
          throw new Error(`${letdeclarationAst.fulltypename().typename().getText()}  is not defined
${letdeclarationAst.getText()} on line ${letdeclarationAst.start.line}:${letdeclarationAst.start.column}`)
        }
        outerTypeBox.solidify(
          letdeclarationAst.fulltypename().typegenerics().fulltypename().map(
            (t: any) =>t.getText() // TODO: Eliminate ANTLR
          ),
          scope
        )
      }
    }
    Microstatement.fromAssignablesAst(
      letdeclarationAst.assignables(),
      scope,
      microstatements,
    )
    // By definition the last microstatement is the const assignment we care about, so we can just
    // mutate its object to rename the output variable name to the name we need instead.
    // EXCEPT with Arrays and User Types. The last is a REREF, so follow it back to the original
    // and mutate that, instead
    let val = microstatements[microstatements.length - 1]
    if (val.statementType === StatementType.REREF) {
      val = Microstatement.fromVarName(val.alias, scope, microstatements)
    }
    val.statementType = StatementType.LETDEC
    microstatements.push(new Microstatement(
      StatementType.REREF,
      scope,
      true,
      val.outputName,
      val.outputType,
      [],
      [],
      letAlias,
    ))
  }

  static fromConstdeclarationAst(
    constdeclarationAst: any, // TODO: Eliminate ANTLR
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    const constName = "_" + uuid().replace(/-/g, "_")
    const constAlias = constdeclarationAst.VARNAME().getText()
    const constTypeHint = constdeclarationAst.fulltypename() ?
      constdeclarationAst.fulltypename().getText() :
      ''
    const typeBox = scope.deepGet(constTypeHint)
    if (typeBox === null && constTypeHint !== '') {
      // Try to define it if it's a generic type
      if (constdeclarationAst.fulltypename().typegenerics()) {
        const outerTypeBox = scope.deepGet(
          constdeclarationAst.fulltypename().typename().getText()
        ) as Type
        if (outerTypeBox === null) {
          throw new Error(`${constdeclarationAst.fulltypename().typename().getText()}  is not defined
${constdeclarationAst.getText()} on line ${constdeclarationAst.start.line}:${constdeclarationAst.start.column}`)
        }
        outerTypeBox.solidify(
          constdeclarationAst.fulltypename().typegenerics().fulltypename().map(
            (t: any) => t.getText() // TODO: Eliminate ANTLR
          ),
          scope
        )
      }
    }
    Microstatement.fromAssignablesAst(
      constdeclarationAst.assignables(),
      scope,
      microstatements,
    )
    // By definition the last microstatement is the const assignment we care about, so we can just
    // mutate its object to rename the output variable name to the name we need instead.
    microstatements.push(new Microstatement(
      StatementType.REREF,
      scope,
      true,
      microstatements[microstatements.length - 1].outputName,
      microstatements[microstatements.length - 1].outputType,
      [],
      [],
      constAlias,
    ))
  }

  // DFS recursive algo to get the microstatements in a valid ordering
  static fromStatementsAst(
    statementAst: any, // TODO: Eliminate ANTLR
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    if (statementAst.declarations() != null) {
      if (statementAst.declarations().constdeclaration() != null) {
        Microstatement.fromConstdeclarationAst(
          statementAst.declarations().constdeclaration(),
          scope,
          microstatements
        )
      } else {
        Microstatement.fromLetdeclarationAst(
          statementAst.declarations().letdeclaration(),
          scope,
          microstatements
        )
      }
    }
    if (statementAst.assignments() != null) {
      Microstatement.fromAssignmentsAst(
        statementAst.assignments(),
        scope,
        microstatements
      )
    }
    if (statementAst.assignables() != null) {
      Microstatement.fromAssignablesAst(
        statementAst.assignables(),
        scope,
        microstatements
      )
    }
    if (statementAst.exits() != null) {
      Microstatement.fromExitsAst(
        statementAst.exits(),
        scope,
        microstatements
      )
    }
    if (statementAst.emits() != null) {
      Microstatement.fromEmitsAst(
        statementAst.emits(),
        scope,
        microstatements
      )
    }

    return microstatements
  }

  static fromBaseAssignableAst(
    baseAssignableAsts: any, // TODO: Eliminate ANTLR
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    // The base assignables array are a lightly annotated set of primitives that can be combined
    // together to produce an assignable value. Certain combinations of these primitives are invalid
    // and TODO provide good error messaging when these are encountered. A state machine of valid
    // transitions is defined below:
    //
    // null -> { var, obj, fn, const, group }
    // var -> { dot, arraccess, call, eos }
    // obj -> { dot, arraccess, eos }
    // fn -> { call, eos }
    // const -> { dot, eos }
    // group -> { dot, arraccess, eos }
    // call -> { call, arraccess, dot, eos }
    // arraccess -> { arraccess, dot, call, eos }
    //
    // Where `null` is the initial state and `eos` is end-of-statement terminating state. `var` is
    // some variable-name-like value (could be a scope, variable, property, or function name). `obj`
    // is object literal syntax, `fn` is function literal syntax, `const` is a constant literal.
    // `group)` is re-using the function call syntax to handle operator grouping (eg `2 * (3 + 4)`).
    // Because of how operators are mixed in with the assignables, the only time this syntax is used
    // as an operator grouping syntax is if it is the first element in the array. Otherwise it is
    // being used as a function call for a given function (either defined by a variable, an
    // inline-defined function, or a returned function from another call or array access) as `call`.
    // Finally `arraccess` is when an array (and ideally later a HashMap) is accessed. This mode is
    // also abusing the `obj` syntax, but only when it's an array literal with only one value and no
    // `new Array<foo>` type definition *and* when there are prior elements in the list. This means
    // `[0][0]` is unambiguous and would return a Result-wrapped zero value, for instance.
    //
    // The exact meaning of `var.var...` chains varies based on the elements of the array both
    // before and after such a chain. If the start of such a list, and if a `call` is at the end, it
    // could be something like `scope.variable.property.functionName(args)` where `.property` can
    // repeat multiple times over. Basically, to properly parse any `.var` requires both the prior
    // state *and* look-ahead to the next element in the list.
    //
    // All of this to re-iterate that for the sake of compile time, some of the complexities of the
    // grammar have been moved from the ANTLR definition into the compiler itself for performance
    // reasons, explaining the complicated iterative logic that follows.

    let currVal: any = null
    for (let i = 0; i < baseAssignableAsts.length; i++) {
      const baseassignable = baseAssignableAsts[i]
      if (!!baseassignable.METHODSEP()) {
        if (i === 0) {
          throw new Error(`Invalid start of assignable statement. Cannot begin with a dot (.)
${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
        }
        const prevassignable = baseAssignableAsts[i - 1]
        if (!!prevassignable.METHODSEP()) {
          throw new Error(`Invalid property access. You accidentally typed a dot twice in a row.
${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
        } else if (!!prevassignable.functions()) {
          throw new Error(`Invalid property access. Functions do not have properties.
${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
        }
        // TODO: Do we even do anything else in this branch?
      } else if (!!baseassignable.VARNAME()) {
        const nextassignable = baseAssignableAsts[i + 1]
        if (!!nextassignable && !!nextassignable.fncall()) {
          // This is a function call path
          const fncall = nextassignable.fncall()
          const argAsts = fncall.assignablelist() ? fncall.assignablelist().assignables() : []
          const argMicrostatements = argAsts.map(arg => {
            Microstatement.fromAssignablesAst(arg, scope, microstatements)
            return microstatements[microstatements.length - 1]
          })
          if (currVal === null) {
            // This is a basic function call
            const realArgNames = argMicrostatements.map(arg => arg.outputName)
            const realArgTypes = argMicrostatements.map(arg => arg.outputType)
            // Do a scan of the microstatements for an inner defined closure that might exist.
            const fn = scope.deepGet(baseassignable.VARNAME().getText()) as Array<Fn>
            if (
              !fn ||
              !(fn instanceof Array && fn[0].microstatementInlining instanceof Function)
            ) {
              const fnName = baseassignable.VARNAME().getText()
              let actualFnName: string
              let inlinedClosure = false
              for (let i = microstatements.length - 1; i >= 0; i--) {
                if (microstatements[i].alias === fnName) {
                  actualFnName = microstatements[i].outputName
                  continue
                }
                if (
                  microstatements[i].outputName === actualFnName &&
                  microstatements[i].statementType === StatementType.CLOSUREDEF) {
                  const m = [...microstatements, ...microstatements[i].closureStatements]
                  const fn = UserFunction.dispatchFn(microstatements[i].fns, realArgTypes, scope)
                  const interfaceMap = new Map()
                  Object.values(fn.getArguments()).forEach(
                    (t: Type, i) => t.typeApplies(realArgTypes[i], scope, interfaceMap)
                  )
                  Microstatement.closureFromUserFunction(fn, fn.scope || scope, m, interfaceMap)
                  const closure = m.pop()
                  microstatements.push(...closure.closureStatements.filter(
                    s => s.statementType !== StatementType.EXIT)
                  )
                  currVal = microstatements[microstatements.length - 1]
                  inlinedClosure = true
                  break
                }
              }
              if (!inlinedClosure) {
                throw new Error(`${baseassignable.VARNAME().getText()} is not a function but used as one.
${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
              }
            } else {
              // Generate the relevant microstatements for this function. UserFunctions get inlined
              // with the return statement turned into a const assignment as the last statement,
              // while built-in functions are kept as function calls with the correct renaming.
              UserFunction
                .dispatchFn(fn, realArgTypes, scope)
                .microstatementInlining(realArgNames, scope, microstatements)
              currVal = microstatements[microstatements.length - 1]
            }
          } else if (currVal instanceof Scope) {
            // This is calling a function by its parent scope
            const realArgNames = argMicrostatements.map(arg => arg.outputName)
            const realArgTypes = argMicrostatements.map(arg => arg.outputType)
            const fn = currVal.deepGet(baseassignable.VARNAME().getText()) as Array<Fn>
            if (
              !fn ||
              !(fn instanceof Array && fn[0].microstatementInlining instanceof Function)
            ) {
              throw new Error(`${baseassignable.VARNAME().getText()} is not a function but used as one.
${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
            }
            // Generate the relevant microstatements for this function. UserFunctions get inlined
            // with the return statement turned into a const assignment as the last statement,
            // while built-in functions are kept as function calls with the correct renaming.
            UserFunction
              .dispatchFn(fn, realArgTypes, scope)
              .microstatementInlining(realArgNames, scope, microstatements)
            currVal = microstatements[microstatements.length - 1]
          } else { // It's a method-style function call
            const realArgNames = [
              currVal.outputName,
              ...argMicrostatements.map(arg => arg.outputName)
            ]
            const realArgTypes = [
              currVal.outputType,
              ...argMicrostatements.map(arg => arg.outputType)
            ]
            const fn = scope.deepGet(baseassignable.VARNAME().getText()) as Array<Fn>
            if (
              !fn ||
              !(fn instanceof Array && fn[0].microstatementInlining instanceof Function)
            ) {
              throw new Error(`${baseassignable.VARNAME().getText()} is not a function but used as one.
${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
            }
            // Generate the relevant microstatements for this function. UserFunctions get inlined
            // with the return statement turned into a const assignment as the last statement,
            // while built-in functions are kept as function calls with the correct renaming.
            UserFunction
              .dispatchFn(fn, realArgTypes, scope)
              .microstatementInlining(realArgNames, scope, microstatements)
            currVal = microstatements[microstatements.length - 1]
          }
          // Intentionally skip over the `fncall` block on the next iteration
          i++
        } else {
          if (currVal === null) {
            let thing = Microstatement.fromVarName(
              baseassignable.VARNAME().getText(),
              scope,
              microstatements,
            )
            if (!thing) {
              thing = scope.deepGet(baseassignable.VARNAME().getText())
            }
            if (!thing) {
              throw new Error(`${baseassignable.VARNAME().getText()} not found.
  ${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
            }
            currVal = thing
          } else if (currVal instanceof Scope) {
            const thing = currVal.deepGet(baseassignable.VARNAME().getText())
            if (!thing) {
              throw new Error(`${baseassignable.VARNAME().getText()} not found in other scope.
  ${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
            }
            currVal = thing
          } else if (currVal instanceof Microstatement) {
            const fieldName = baseassignable.VARNAME().getText()
            const fields = Object.keys(currVal.outputType.properties)
            const fieldNum = fields.indexOf(fieldName)
            if (fieldNum < 0) {
              // Invalid object access
              throw new Error(`${fieldName} property not found.
  ${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
            }
            // Create a new variable to hold the address within the array literal
            const addrName = "_" + uuid().replace(/-/g, "_")
            microstatements.push(new Microstatement(
              StatementType.CONSTDEC,
              scope,
              true,
              addrName,
              Type.builtinTypes['int64'],
              [`${fieldNum}`],
              [],
            ))
            // Insert a `register` opcode.
            const opcodes = require('./opcodes').default
            opcodes.exportScope.get('register')[0].microstatementInlining(
              [currVal.outputName, addrName],
              scope,
              microstatements,
            )
            // We'll need a reference to this for later
            const typeRecord = currVal 
            // Set the original to this newly-generated microstatement
            currVal = microstatements[microstatements.length - 1]
            // Now we do something odd, but correct here; we need to replace the `outputType` from
            // `any` to the type that was actually copied so function resolution continues to work
            currVal.outputType = typeRecord.outputType.properties[fieldName]
          } else {
            // What is this?
            throw new Error(`Impossible path found. Bug in compiler, please report!
Previous value type: ${typeof currVal}
${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
          }
        }
      } else if (!!baseassignable.constants()) {
        if (currVal !== null) {
          throw new Error(`Unexpected constant value detected.
Previous value type: ${typeof currVal}
${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
        }
        Microstatement.fromConstantsAst(baseassignable.constants(), scope, microstatements)
        currVal = microstatements[microstatements.length - 1]
      } else if (!!baseassignable.functions()) {
        if (currVal !== null) {
          throw new Error(`Unexpected function definition detected.
Previous value type: ${typeof currVal}
${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
        }
        // So the closures eval correctly, we add the alias microstatements to the scope
        // TODO: Is this the right approach?
        microstatements.filter(m => !!m.alias).forEach(m => scope.put(m.alias, m))
        const fn = UserFunction.fromFunctionsAst(baseassignable.functions(), scope)
        currVal = fn // TODO: Is this the right choice here?
      } else if (!!baseassignable.objectliterals()) {
        if (currVal === null) {
          // Has to be a "normal" object literal in this case
          Microstatement.fromObjectLiteralsAst(
            baseassignable.objectliterals(),
            scope,
            microstatements
          )
          currVal = microstatements[microstatements.length - 1]
        } else {
          // Can only be an array accessor syntax
          const objlit = baseassignable.objectliterals()
          if (!!objlit.typeliteral() || !!objlit.arrayliteral().literaldec()) {
            throw new Error(`Unexpected object literal definition detected.
Previous value type: ${typeof currVal}
${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
          }
          const arrbase = objlit.arrayliteral().arraybase()
          if (!arrbase.assignablelist() || arrbase.assignablelist().assignables().length !== 1) {
            throw new Error(`Array access must provide only one index value to query the array with
${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
          }
          const assignableAst = arrbase.assignablelist().assignables(0)
          Microstatement.fromAssignablesAst(assignableAst, scope, microstatements)
          const arrIndex = microstatements[microstatements.length - 1]
          if (
            !(currVal instanceof Microstatement) ||
            currVal.outputType.originalType.typename !== 'Array'
          ) {
            throw new Error(`Array access may only be performed on arrays.
Previous value type: ${currVal.outputType.typename}
${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
          }
          if (arrIndex.outputType.typename === 'int64') {
            const opcodes = require('./opcodes').default
            // Create a new variable to hold the `okR` size value
            const sizeName = "_" + uuid().replace(/-/g, "_")
            microstatements.push(new Microstatement(
              StatementType.CONSTDEC,
              scope,
              true,
              sizeName,
              Type.builtinTypes['int64'],
              ['8'],
              [],
            ))
            // Insert an `okR` opcode.
            opcodes.exportScope.get('okR')[0].microstatementInlining(
              [arrIndex.outputName, sizeName],
              scope,
              microstatements,
            )
            const wrapped = microstatements[microstatements.length - 1]
            // Insert a `resfrom` opcode.
            opcodes.exportScope.get('resfrom')[0].microstatementInlining(
              [currVal.outputName, wrapped.outputName],
              scope,
              microstatements,
            )
          } else if (arrIndex.outputType.typename === 'Result<int64>') {
            const opcodes = require('./opcodes').default
            // Insert a `resfrom` opcode.
            opcodes.exportScope.get('resfrom')[0].microstatementInlining(
              [currVal.outputName, arrIndex.outputName],
              scope,
              microstatements,
            )
          } else {
            throw new Error(`Array access must be done with an int64 or Result<int64> value
${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
          }
          // We'll need a reference to this for later
          const arrayRecord = currVal
          // Update to this newly-generated microstatement
          currVal = microstatements[microstatements.length - 1]
          // Now we do something odd, but correct here; we need to replace the `outputType` from
          // `any` to the type that was actually copied so function resolution continues to work
          currVal.outputType = Type.builtinTypes.Result.solidify(
            [Object.values(arrayRecord.outputType.properties)[0].typename],
            scope,
          )
        }
      } else if (!!baseassignable.fncall()) {
        // It's a `fncall` syntax block but it wasn't caught in a function call before, so it's
        // either a function call on a returned function type, or it's an assignable group
        if (!currVal) {
          // It's probably an assignable group
          if (
            !baseassignable.fncall().assignablelist() ||
            baseassignable.fncall().assignablelist().assignables().length !== 1
          ) {
            throw new Error(`Expected a group of assignable values, but got a function call signature.
${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
          }
          // It *is* an assignable group!
          Microstatement.fromAssignablesAst(
            baseassignable.fncall().assignablelist().assignables(0),
            scope,
            microstatements,
          )
          currVal = microstatements[microstatements.length - 1]
        } else {
          // TODO: handle functions/closures being called from access out of other function returns
          // and the like
        }
      } else {
        throw new Error(`Compiler error! Completely unhandled input.
${baseassignable.getText()} on line ${baseassignable.start.line}:${baseassignable.start.column}`)
      }
    }
    if (!(currVal instanceof Microstatement)) {
      if (currVal instanceof UserFunction) {
        Microstatement.closureDef([currVal], currVal.scope || scope, microstatements)
      } else if (currVal instanceof Array && currVal[0] instanceof UserFunction) {
        Microstatement.closureDef(currVal, currVal[0].scope || scope, microstatements)
      }
    } else if (currVal.statementType !== StatementType.EMIT) {
      microstatements.push(new Microstatement(
        StatementType.REREF,
        scope,
        true,
        currVal.outputName,
        currVal.outputType,
        [],
        [],
        currVal.alias,
      ))
    }
  }

  static fromAssignablesAst(
    assignablesAst: any, // TODO: Eliminate ANTLR
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    const withoperators = assignablesAst.withoperators()
    let withOperatorsList = []
    for (const operatorOrAssignable of withoperators) {
      if (!!operatorOrAssignable.operators()) {
        const operator = operatorOrAssignable.operators()
        const op = scope.deepGet(operator.getText())
        if (op == null || !(op instanceof Array && op[0] instanceof Operator)) {
          throw new Error("Operator " + operator.getText() + " is not defined")
        }
        withOperatorsList.push(op)
      } else if (
        !!operatorOrAssignable.baseassignable() &&
        operatorOrAssignable.baseassignable().length > 0
      ) {
        Microstatement.fromBaseAssignableAst(
          operatorOrAssignable.baseassignable(),
          scope,
          microstatements,
        )
        const last = microstatements[microstatements.length - 1]
        withOperatorsList.push(last)
      }
    }
    // Now to combine these operators and values in the correct order. A compiled language could
    // never do something so inefficient, but I don't care about performance right now, so here's
    // the algorithm: while the list length is greater than 1, perform the two steps:
    // 1. Find the operator with the greatest precedence
    // 2. Apply the underlying function to the values on either side of the operator (or just the
    //    right side if the operator is a prefix operator), then replace the operator with the
    //    returned value in the list and delete the impacted values.
    while (withOperatorsList.length > 1) {
      let maxPrecedence = -1
      let maxOperatorLoc = -1
      let maxOperatorListLoc = -1
      for (let i = 0; i < withOperatorsList.length; i++) {
        if (withOperatorsList[i] instanceof Array && withOperatorsList[i][0] instanceof Operator) {
          const ops = withOperatorsList[i]
          let op = null
          let operatorListLoc = -1
          let operatorPrecedence = -127
          if (ops.length == 1) {
            op = ops[0]
            operatorListLoc = 0
          } else {
            // TODO: We need to identify which particular operator applies in this case.
            // We're just going to short-circuit this process on the first operator that matches
            // but we need to come up with a "best match" behavior (ie, if one argument is an int8
            // it may choose the int64-based operator because it was first and it can cast int8 to
            // int64 and then miss the specialized int8 version of the function).
            let left = null
            if (i != 0) left = withOperatorsList[i - 1]
            let right = null
            if (i != withOperatorsList.length - 1) right = withOperatorsList[i + 1]
            // Skip over any operator that is followed by another operator as it must be a prefix
            // operator (or a syntax error, but we'll catch that later)
            if (right === null || right instanceof Microstatement) {
              for (let j = 0; j < ops.length; j++) {
                if (
                  ops[j].precedence > operatorPrecedence &&
                  ops[j].applicableFunction(
                    !left ? // Left is special, if two operators are in a row, this one
                      null :        // needs to be a prefix operator for this to work at all
                      left instanceof Microstatement ?
                        left.outputType :
                        null,
                    right === null ? null : right.outputType,
                    scope
                  ) != null
                ) {
                  op = ops[j]
                  operatorListLoc = j
                  operatorPrecedence = op.precedence
                }
              }
            }
            // During the process of determining the operator ordering, there may be tests that
            // will not match because operator precedence will convert the neighboring types into
            // types that will match. This is complicated and doing this statically will be more
            // difficult, but for now, just skip over these.
            if (op == null) continue
          }

          if (op.precedence > maxPrecedence) {
            maxPrecedence = op.precedence
            maxOperatorLoc = i
            maxOperatorListLoc = operatorListLoc
          }
        }
      }
      if (maxPrecedence == -1 || maxOperatorLoc == -1) {
        let errMsg = `Cannot resolve operators with remaining statement
${assignablesAst.getText()}`
        let withOperatorsTranslation = []
        for (let i = 0; i < withOperatorsList.length; i++) {
          const node = withOperatorsList[i]
          if (node instanceof Array && node[0] instanceof Operator) {
            withOperatorsTranslation.push(node[0].name)
          } else {
            withOperatorsTranslation.push("<" + node.outputType.typename + ">")
          }
        }
        errMsg += '\n' + withOperatorsTranslation.join(' ')
        throw new Error(errMsg)
      }
      const op = withOperatorsList[maxOperatorLoc][maxOperatorListLoc]
      let realArgNames = []
      let realArgTypes = []
      if (!op.isPrefix) {
        const left = withOperatorsList[maxOperatorLoc - 1]
        realArgNames.push(left.outputName)
        realArgTypes.push(left.outputType)
      }
      const right = withOperatorsList[maxOperatorLoc + 1]
      realArgNames.push(right.outputName)
      realArgTypes.push(right.outputType)
      UserFunction
        .dispatchFn(op.potentialFunctions, realArgTypes, scope)
        .microstatementInlining(realArgNames, scope, microstatements)
      const last = microstatements[microstatements.length - 1]
      withOperatorsList[maxOperatorLoc] = last
      withOperatorsList.splice(maxOperatorLoc + 1, 1)
      if (!op.isPrefix) {
        withOperatorsList.splice(maxOperatorLoc - 1, 1)
      }
    }
  }

  static fromStatement(
    statement: Statement,
    microstatements: Array<Microstatement>,
    secondaryScope: Scope | null = null,
  ) {
    let actualStatement = statement
    if (secondaryScope !== null) {
      const newScope = new Scope(statement.scope)
      newScope.secondaryPar = secondaryScope
      actualStatement = new Statement(
        statement.statementAst,
        newScope,
        statement.pure,
      )
    }
    Microstatement.fromStatementsAst(
      actualStatement.statementAst,
      actualStatement.scope,
      microstatements
    )
  }
}

export default Microstatement
