const Type = require('./Type')
const Int8 = require('./Int8')
const Int16 = require('./Int16')
const Int32 = require('./Int32')
const Int64 = require('./Int64')
const Float32 = require('./Float32')
const Float64 = require('./Float64')

class Box {
  constructor(...args) {
    // Work around circular deps in another way
    const Scope = require('./Scope')
    const Microstatement = require('./Microstatement')
    const Int8 = require('./Int8')
    const Int16 = require('./Int16')
    const Int32 = require('./Int32')
    const Int64 = require('./Int64')
    const Float32 = require('./Float32')
    const Float64 = require('./Float64')
    const Event = require('./Event')
    if (args.length === 0) {
      this.type = Box.builtinTypes.void
      this.readonly = true
    } else if (args.length === 1) {
      if (typeof args[0] === "boolean") {
        this.type = Box.builtinTypes.void
        this.readonly = args[0]
      } else if (args[0] instanceof Type) {
        this.type = Box.builtinTypes.type
        this.typeval = args[0]
        this.readonly = true // Type declarations are always read-only
      } else if (args[0] instanceof Scope) {
        this.type = Box.builtinTypes.scope
        this.scopeval = args[0]
        this.readonly = true // Boxed scopes are always read-only
      } else if (args[0] instanceof Microstatement) {
        this.type = Box.builtinTypes.microstatement
        this.microstatementval = args[0]
        this.readonly = true
      } else if (args[0] instanceof Array) {
        // This is only operator declarations right now
        this.type = Box.builtinTypes.operator
        this.operatorval = args[0]
        this.readonly = true
      }
    } else if (args.length === 2) {
      if (args[0] instanceof Int8) {
        this.type = Box.builtinTypes.int8
        this.int8val = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Int16) {
        this.type = Box.builtinTypes.int16
        this.int16val = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Int32) {
        this.type = Box.builtinTypes.int32
        this.int32val = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Int64) {
        this.type = Box.builtinTypes.int64
        this.int64val = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Float32) {
        this.type = Box.builtinTypes.float32
        this.float32val = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Float64) {
        this.type = Box.builtinTypes.float64
        this.float64val = args[0]
        this.readonly = args[1]
      } else if (typeof args[0] === "boolean") {
        this.type = Box.builtinTypes.bool
        this.boolval = args[0]
        this.readonly = args[1]
      } else if (typeof args[0] === "string") {
        this.type = Box.builtinTypes.string
        this.stringval = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Array) {
        // This is only function declarations right now
        this.type = Box.builtinTypes["function"]
        this.functionval = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Event) {
        this.type = Box.builtinTypes.Event
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
        if (args[1].originalType == Box.builtinTypes.Event) {
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
  
  static fromConstantsAst(constantsAst, scope, expectedType, readonly) {
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
      const typename = expectedType != null ? expectedType.typename : null
      if (typename != null && typename.equals("void")) typename = null;
      if (numberConst.indexOf('.') > -1) { // It's a float
        // TODO: How to handle other float constants like NaN, Infinity, -0, etc
        if (typename == null) {
          return new Box(new Float64(numberConst), readonly)
        } else if (typename.equals("float32")) {
          return new Box(new Float32(numberConst), readonly)
        } else if (typename.equals("float64")) {
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
        } else if (typename.equals("int8")) {
          return new Box(new Int8(numberConst), readonly)
        } else if (typename.equals("int16")) {
          return new Box(new Int16(numberConst), readonly)
        } else if (typename.equals("int32")) {
          return new Box(new Int32(numberConst), readonly)
        } else if (typename.equals("int64")) {
          return new Box(new Int64(numberConst), readonly)
        } else if (typename.equals("float32")) { // We'll allow floats to get integer constants
          return new Box(new Float32(numberConst), readonly)
        } else if (typename.equals("float64")) {
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

  static fromConstAst(constAst, scope) {
    const assignment = constAst.assignments()
    return Box.fromAssignmentAst(assignment, scope, true)
  }

  static fromAssignmentAst(assignmentAst, scope, readonly) {
    // TODO: This code is becoming very overloaded with different meanings in different contexts
    // Should probably split this up into multiple functions instead of trying to have this function
    // guess which context it's running in.
    
    // TODO: Review if any of the extra logic after deepGet is needed anymore
    const typename = assignmentAst.varn().getText();
    let typeBox = scope.deepGet(assignmentAst.varn());

    let type;

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
      for (fulltypenameAst of assignmentAst.typegenerics().fulltypename()) {
        solidTypes.push(fulltypenameAst.getText())
      }
      type = type.solidify(solidTypes, scope)
    }

    return Box.fromAssignableAst(assignmentAst.assignables(), scope, type, readonly)
  }

  static fromAssignableAst(assignableAst, scope, expectedType, readonly) {
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

  static fromBasicAssignableAst(basicAssignable, scope, expectedType, readonly) {
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

  static fromObjectLiteralsAst(objectliteralsAst, scope, expectedType, readonly) {
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
        typevalval.put(assignmentsAst.varn().getText(), Box.fromAssignableAst(
          assignmentsAst.assignables(),
          scope,
          assignmentType,
          readonly
        ))
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
          mapval.put(keyBox, valBox)
        }
      }
      return new Box(mapval, type, readonly)
    }
    // Should never reach here
    return null
  }

}

Box.builtinTypes = {
  void: new Type("void", true),
  int8: new Type("int8", true),
  int16: new Type("int16", true),
  int32: new Type("int32", true),
  int64: new Type("int64", true),
  float32: new Type("float32", true),
  float64: new Type("float64", true),
  bool: new Type("bool", true),
  string: new Type("string", true),
  Error: new Type("Error", true, {
    message: new Type("string", true, true),
    code: new Type("int64", true, true),
  }),
  "Array": new Type("Array", true, {
    records: new Type("V", true, true),
  }, {
    V: 0,
  }),
  Map: new Type("Map", true, {
    key: new Type("K", true, true),
    value: new Type("V", true, true),
  }, {
    K: 0,
    V: 1,
  }),
  KeyVal: new Type("KeyVal", true, {
    key: new Type("K", true, true),
    value: new Type("V", true, true),
  }, {
    K: 0,
    V: 1,
  }),
  "function": new Type("function", true),
  operator: new Type("operator", true),
  Event: new Type("Event", true, {
    type: new Type("E", true, true),
  }, {
    E: 0,
  }),
  type: new Type("type", true),
  scope: new Type("scope", true),
  microstatement: new Type("microstatement", true),
}

module.exports = Box
