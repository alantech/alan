import Event from './Event'
import Float32 from './Float32'
import Float64 from './Float64'
import Int16 from './Int16'
import Int32 from './Int32'
import Int64 from './Int64'
import Int8 from './Int8'
import Operator from './Operator'
import Scope from './Scope'
import Type from './Type'
import UserFunction from './UserFunction'

class Box {
  type: Type
  readonly: boolean
  typeval: Type | undefined
  scopeval: Scope | undefined
  microstatementval: any | undefined // TODO: Port Microstatement to TS
  operatorval: Array<Operator> | undefined
  int8val: Int8 | undefined
  int16val: Int16 | undefined
  int32val: Int32 | undefined
  int64val: Int64 | undefined
  float32val: Float32 | undefined
  float64val: Float64 | undefined
  boolval: boolean | undefined
  stringval: string | undefined
  functionval: Array<UserFunction> | undefined
  eventval: Event | undefined
  arrayval: Array<any> | undefined
  mapval: Map<any, any> | undefined
  typevalval: object | undefined

  constructor(...args: Array<any>) {
    // Work around circular deps in another way
    const Microstatement = require('./Microstatement')
    if (args.length === 0) {
      this.type = Type.builtinTypes.void
      this.readonly = true
    } else if (args.length === 1) {
      if (typeof args[0] === "boolean") {
        this.type = Type.builtinTypes.void
        this.readonly = args[0]
      } else if (args[0] instanceof Type) {
        this.type = Type.builtinTypes.type
        this.typeval = args[0]
        this.readonly = true // Type declarations are always read-only
      } else if (args[0] instanceof Scope) {
        this.type = Type.builtinTypes.scope
        this.scopeval = args[0]
        this.readonly = true // Boxed scopes are always read-only
      } else if (args[0] instanceof Microstatement) {
        this.type = Type.builtinTypes.microstatement
        this.microstatementval = args[0]
        this.readonly = true
      } else if (args[0] instanceof Array) {
        // This is only operator declarations right now
        this.type = Type.builtinTypes.operator
        this.operatorval = args[0]
        this.readonly = true
      }
    } else if (args.length === 2) {
      if (args[0] instanceof Int8) {
        this.type = Type.builtinTypes.int8
        this.int8val = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Int16) {
        this.type = Type.builtinTypes.int16
        this.int16val = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Int32) {
        this.type = Type.builtinTypes.int32
        this.int32val = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Int64) {
        this.type = Type.builtinTypes.int64
        this.int64val = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Float32) {
        this.type = Type.builtinTypes.float32
        this.float32val = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Float64) {
        this.type = Type.builtinTypes.float64
        this.float64val = args[0]
        this.readonly = args[1]
      } else if (typeof args[0] === "boolean") {
        this.type = Type.builtinTypes.bool
        this.boolval = args[0]
        this.readonly = args[1]
      } else if (typeof args[0] === "string") {
        this.type = Type.builtinTypes.string
        this.stringval = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Array) {
        // This is only function declarations right now
        this.type = Type.builtinTypes["function"]
        this.functionval = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Event) {
        this.type = Type.builtinTypes.Event
        this.eventval = args[0]
        this.readonly = args[1]
      }
      // Technically there's also supposed to be something for an "Object blank" but I don't know
      // what that is supposed to be for
    } else if (args.length === 3) {
      if (args[0] instanceof Array) {
        // It's an array, like a real one
        this.type = args[1]
        this.arrayval = args[0]
        this.readonly = args[2]
      } else if (args[0] instanceof Map) {
        // It's a map, also a real one
        this.type = args[1]
        this.mapval = args[0]
        this.readonly = args[2]
      } else if (args[0] instanceof Object) {
        // It's an event or user-defined type
        if (args[1].originalType == Type.builtinTypes.Event) {
          let eventval = new Event(args[0].name.stringval, args[1].properties.type, false)
          this.eventval = eventval
          this.readonly = args[2]
        } else {
          this.type = args[1]
          this.typevalval = args[0]
          this.readonly = args[2]
        }
      }
    }
  }
 
  // TODO: There are so many Java-isms in this method, check if it's even being used
  static fromConstantsAst(
    constantsAst: any, // TODO: Port from ANTLR to improve AST typing
    _scope: Scope, // TODO: Remove this arg from calling functions
    expectedType: Type,
    readonly: boolean
  ) {
    if (constantsAst.BOOLCONSTANT() != null) {
      if (constantsAst.BOOLCONSTANT().getText() === "true") {
        return new Box(true, readonly);
      } else {
        return new Box(false, readonly);
      }
    }
    if (constantsAst.STRINGCONSTANT() != null) {
      return new Box(
        constantsAst
          .STRINGCONSTANT()
          .getText()
          .substring(1, constantsAst.STRINGCONSTANT().getText().length - 1)
          .replace("\\t", "\t")
          .replace("\\b", "\b")
          .replace("\\n", "\n")
          .replace("\\r", "\r")
          .replace("\\f", "\f")
          .replace("\\'", "'")
          .replace("\\\"", "\"")
          .replace("\\\\", "\\"),
        readonly
      );
    }
    if (constantsAst.NUMBERCONSTANT() != null) {
      // TODO: Add support for hex, octal, scientific, etc
      const numberConst = constantsAst.NUMBERCONSTANT().getText();
      let typename = expectedType != null ? expectedType.typename : null
      if (typename != null && typename === "void") typename = null
      if (numberConst.indexOf('.') > -1) { // It's a float
        // TODO: How to handle other float constants like NaN, Infinity, -0, etc
        if (typename == null) {
          return new Box(new Float64(numberConst), readonly)
        } else if (typename === "float32") {
          return new Box(new Float32(numberConst), readonly)
        } else if (typename === "float64") {
          return new Box(new Float64(numberConst), readonly)
        } else {
          // Bad assignment
          console.error("Assigning floating point number to non-floating point type")
          process.exit(-6)
        }
      } else { // It's an integer
        // TODO: Should we error on overflowing constants in integer mode?
        if (typename == null) {
          return new Box(new Int64(numberConst), readonly)
        } else if (typename === "int8") {
          return new Box(new Int8(numberConst), readonly)
        } else if (typename === "int16") {
          return new Box(new Int16(numberConst), readonly)
        } else if (typename === "int32") {
          return new Box(new Int32(numberConst), readonly)
        } else if (typename === "int64") {
          return new Box(new Int64(numberConst), readonly)
        } else if (typename === "float32") { // We'll allow floats to get integer constants
          return new Box(new Float32(numberConst), readonly)
        } else if (typename === "float64") {
          return new Box(new Float64(numberConst), readonly)
        } else {
          // Bad assignment
          console.error("Assigning integer number to non-numeric type")
          console.error("Variable type: " + typename)
          process.exit(-7)
        }
      }
    }
    // This should never be reached
    return null
  }

  static fromConstAst(constAst: any, scope: Scope) { // TODO: Eliminate ANTLR
    const assignment = constAst.assignments()
    return Box.fromAssignmentAst(assignment, scope, true)
  }

  static fromAssignmentAst(assignmentAst: any, scope: Scope, readonly: boolean) {
    // TODO: This code is becoming very overloaded with different meanings in different contexts
    // Should probably split this up into multiple functions instead of trying to have this function
    // guess which context it's running in.
    
    // TODO: Review if any of the extra logic after deepGet is needed anymore
    const typename = assignmentAst.varn().getText();
    let typeBox = scope.deepGet(assignmentAst.varn());

    let type: Type

    if (typeBox == null) {
      const nameSegments = typename.split(".");
      let parentName = nameSegments[0]
      for (let i = 1; i < nameSegments.length - 1; i++) {
        parentName += "." + nameSegments[i]
      }
      const childName = nameSegments[nameSegments.length - 1]
      typeBox = scope.deepGet(parentName)
      if (typeBox == null) {
        // Assignment to an undefined variable. This can legitimately happen in cases of type
        // inference, but not in other cases. This whole bit really needs to be rethought.
        return Box.fromAssignableAst(assignmentAst.assignables(), scope, null, readonly)
      }
      type = typeBox.type.properties[childName]
    } else if (typeBox.type.typename !== "type") {
      // This is actually a secondary assignment to an existing variable
      if (typeBox.readonly) {
        console.error("Invalid reassignment to constant: " + typename)
        process.exit(-30)
      }
      return Box.fromAssignableAst(assignmentAst.assignables(), scope, typeBox.type, false)
    } else {
      type = typeBox.typeval
    }

    if (type.generics.length > 0 && assignmentAst.typegenerics() != null) {
      let solidTypes = []
      for (const fulltypenameAst of assignmentAst.typegenerics().fulltypename()) {
        solidTypes.push(fulltypenameAst.getText())
      }
      type = type.solidify(solidTypes, scope)
    }

    return Box.fromAssignableAst(assignmentAst.assignables(), scope, type, readonly)
  }

  static fromAssignableAst(
    assignableAst: any, // TODO: Eliminate ANTLR
    scope: Scope,
    expectedType: Type,
    readonly: boolean
  ) {
    if (assignableAst == null) {
      return new Box(null, expectedType)
    }
    if (assignableAst.basicassignables() != null) {
      return Box.fromBasicAssignableAst(
        assignableAst.basicassignables(),
        scope,
        expectedType,
        readonly
      )
    }
    if (assignableAst.withoperators() != null) {
      // TODO: How to support this in the compiler
      // Operators are another form a function, to evaluate them requires a full interpreter, so
      // we'll come back to this later
      /* return Box.fromWithOperatorsAst(
        assignableAst.withoperators(),
        scope,
        expectedType,
        readonly
      ) */
      return new Box() // void it for now
    }
    // Just to prevent complains, but this should not be reachable
    return null
  }

  static fromBasicAssignableAst(
    basicAssignable: any, // TODO: Eliminate ANTLR
    scope: Scope,
    expectedType: Type,
    readonly: boolean
  ) {
    if (basicAssignable.functions() != null) {
      const assignedFunction = UserFunction.fromAst(basicAssignable.functions(), scope)
      return new Box([assignedFunction], readonly)
    }
    if (basicAssignable.calls() != null) {
      // TODO: Support generating global constants from function calls at some point
      // return Function.callFromAst(basicAssignable.calls(), scope);
      return new Box() // Void it for now
    }
    if (basicAssignable.varn() != null) {
      return scope.deepGet(basicAssignable.varn());
    }
    if (basicAssignable.groups() != null) {
      // TODO: Suppor this later
      /* return Box.fromWithOperatorsAst(
        basicAssignable.groups().withoperators(),
        scope,
        expectedType,
        readonly
      ) */
      return new Box() // void it for now
    }
    if (basicAssignable.typeofn() != null) {
      // Potentially add a bunch of guards around this
      return new Box(Box.fromBasicAssignableAst(
        basicAssignable.typeofn().basicassignables(),
        scope,
        null,
        readonly
      ).type.typename, readonly)
    }
    if (basicAssignable.objectliterals() != null) {
      return Box.fromObjectLiteralsAst(
        basicAssignable.objectliterals(),
        scope,
        null,
        readonly
      )
    }
    if (basicAssignable.constants() != null) {
      return Box.fromConstantsAst(
        basicAssignable.constants(),
        scope,
        expectedType,
        readonly
      );
    }
    // Shouldn't be possible
    console.error("Something went wrong parsing the syntax")
    process.exit(-8)
  }

  static fromObjectLiteralsAst(
    objectliteralsAst: any, // TODO: Eliminate ANTLR
    scope: Scope,
    _expectedType: Type, // TODO: Eliminate this arg from calling code
    readonly: boolean
  ) {
    const typename = objectliteralsAst.othertype().getText()
    const typeBox = scope.deepGet(typename)
    let type = null
    if (objectliteralsAst.othertype().typegenerics() != null && typeBox == null) {
      const originalTypeBox = scope.deepGet(objectliteralsAst.othertype().typename().getText())
      if (originalTypeBox == null) {
        console.error(objectliteralsAst.othertype().typename().getText() + " is referenced but not defined. Unexpected runtime error!")
        process.exit(-46)
      }
      let solidTypes = []
      for (const fulltypenameAst of objectliteralsAst.othertype().typegenerics().fulltypename()) {
        solidTypes.push(fulltypenameAst.getText())
      }
      type = originalTypeBox.typeval.solidify(solidTypes, scope)
    } else {
      type = typeBox.typeval
    }
    if (type == null) {
      console.error(objectliteralsAst.othertype().getText() + " is not a valid type")
      process.exit(-45)
    }
    if (objectliteralsAst.arrayliteral() != null) {
      let arrayval = []
      for (const assignableAst of objectliteralsAst.arrayliteral().assignablelist().assignables()) {
        arrayval.push(Box.fromAssignableAst(
          assignableAst,
          scope,
          type.properties["records"], // Special for Arrays (and Trees and Sets later)
          readonly
        ))
      }
      return new Box(arrayval, type, readonly)
    }
    if (objectliteralsAst.typeliteral() != null) {
      let typevalval = {}
      for (const assignmentsAst of objectliteralsAst.typeliteral().assignments()) {
        const property = assignmentsAst.varn().getText()
        const assignmentType = type.properties[property]
        if (assignmentsAst.assignables() == null) {
          // TODO: this can only happen if parts of the `assignments` syntax not valid here are used
          // This should be eliminated in the future, but for now just crash
          console.error("Invalid literal assignment for " + type.typename + " on the "
            + property + " property.")
          process.exit(-46)
        }
        typevalval[assignmentsAst.varn().getText()] = Box.fromAssignableAst(
          assignmentsAst.assignables(),
          scope,
          assignmentType,
          readonly
        )
      }
      return new Box(typevalval, type, readonly)
    }
    if (objectliteralsAst.mapliteral() != null) {
      let mapval = {}
      if (objectliteralsAst.mapliteral().mapline() != null) {
        for (const mapline of objectliteralsAst.mapliteral().mapline()) {
          const keyBox = Box.fromAssignableAst(
            mapline.assignables(0),
            scope,
            type.properties["key"], // Special for Maps
            readonly
          )
          const valBox = Box.fromAssignableAst(
            mapline.assignables(1),
            scope,
            type.properties["value"], // Special for Maps
            readonly
          )
          mapval[keyBox] = valBox
        }
      }
      return new Box(mapval, type, readonly)
    }
    // Should never reach here
    return null
  }
}

export default Box
