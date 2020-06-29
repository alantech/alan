class Type {
  constructor(...args) {
    // Circular dependency 'fix'
    const Interface = require('./Interface').default
    // Simulate multiple dispatch by duck typing the args
    if (args.length === 1) {
      this.typename = args[0]
      this.builtIn = false
      this.isGenericStandin = false
      this.properties = {}
      this.generics = {}
      this.originalType = null
      this.unionTypes = null
      this.iface = null
    } else if (args.length === 2) {
      this.typename = args[0]
      this.builtIn = args[1]
      this.isGenericStandin = false
      this.properties = {}
      this.generics = {}
      this.originalType = null
      this.unionTypes = null
      this.iface = null
    } else if (args.length === 3) {
      if (typeof args[2] === "boolean") {
        this.typename = args[0]
        this.builtIn = args[1]
        this.isGenericStandin = args[2]
        this.properties = {}
        this.generics = {}
        this.originalType = null
        this.unionTypes = null
        this.iface = null
      } else if (args[2] instanceof Interface) {
        this.typename = args[0]
        this.builtIn = args[1]
        this.isGenericStandin = false
        this.properties = {}
        this.generics = {}
        this.originalType = null
        this.unionTypes = null
        this.iface = args[2]
      } else if (args[2] instanceof Array) {
        this.typename = args[0]
        this.builtIn = args[1]
        this.isGenericStandin = false
        this.properties = {}
        this.generics = {}
        this.originalType = null
        this.unionTypes = args[2]
        this.iface = null
      } else if (args[2] instanceof Object) {
        this.typename = args[0]
        this.builtIn = args[1]
        this.isGenericStandin = false
        this.properties = args[2]
        this.generics = {}
        this.originalType = null
        this.unionTypes = null
        this.iface = null
      }
    } else if (args.length === 4) {
      this.typename = args[0]
      this.builtIn = args[1]
      this.isGenericStandin = false
      this.properties = args[2]
      this.generics = args[3]
      this.originalType = null
      this.unionTypes = null
      this.iface = null
    }
  }

  toString() {
    // TODO: Handle interfaces union types appropriately
    if (this.iface != null) return "// Interfaces TBD"
    if (this.unionTypes != null) return "// Union types TBD"
    let outString = "type " + this.typename
    if (this.alias != null) {
      outString += " = " + this.alias.typename
      return outString
    }
    if (this.generics.length > 0) {
      outString += "<" + Object.keys(this.generics).join(", ") + ">"
    }
    outString += "{\n"
    for (const propName of Object.keys(this.properties)) {
      outString += "  " + propName + ": " + this.properties[propName].typename + "\n"
    }
    outString += "}\n"
    return outString
  }

  static fromAst(typeAst, scope) {
    let type = new Type(typeAst.typename().getText())
    if (typeAst.typegenerics() != null) {
      const generics = typeAst.typegenerics().fulltypename()
      for (let i = 0; i < generics.length; i++) {
        type.generics[generics[i].getText()] = i
      }
    }
    if (typeAst.typebody() != null) {
      const lines = typeAst.typebody().typeline()
      for (const lineAst of lines) {
        const propertyName = lineAst.VARNAME().getText()
        const typeName = lineAst.varn().getText()
        const property = scope.deepGet(lineAst.varn())
        if (property == null || !property.type.typename === "type") {
          if (type.generics.hasOwnProperty(typeName)) {
            type.properties[propertyName] = new Type(typeName, true, true)
          } else {
            console.error(lineAst.varn().getText() + " is not a type")
            process.exit(-4)
          }
        } else {
          type.properties[propertyName] = property.typeval
        }
      }
    }
    if (typeAst.othertype() != null && typeAst.othertype().length == 1) {
      const otherTypebox = scope.deepGet(typeAst.othertype(0).typename().getText())

      if (otherTypebox == null) {
        console.error("Type " + typeAst.othertype(0).getText() + " not defined")
        process.exit(-38)
      }
      if (otherTypebox.typeval == null) {
        console.error(typeAst.othertype(0).getText() + " is not a valid type")
        process.exit(-39)
      }

      let othertype = otherTypebox.typeval
      if (Object.keys(othertype.generics).length > 0 && typeAst.othertype(0).typegenerics() != null) {
        let solidTypes = []
        for (const fulltypenameAst of typeAst.othertype(0).typegenerics().fulltypename()) {
          solidTypes.push(fulltypenameAst.getText())
        }
        othertype = othertype.solidify(solidTypes, scope)
      }

      // For simplification of the type aliasing functionality, the other type is attached as
      // an alias. The module construction will, if present, perfer the alias over the actual
      // type, to make sure built-in types that are aliased continue to work. This means that
      // `type varA == type varB` will work if `varA` is assigned to an alias and `varB` to the
      // orignal type. I can see the argument either way on this, but the simplicity of this
      // approach is why I will go with this for now.
      type.alias = othertype
    } else if (typeAst.othertype() != null) { // It's a union type
      const othertypes = typeAst.othertype()
      let unionTypes = []
      for (const othertype of othertypes) {
        const othertypeBox = scope.deepGet(othertype.typename().getText())

        if (othertypeBox == null) {
          console.error("Type " + othertype.getText() + " not defined")
          process.exit(-48)
        }
        if (othertypeBox.typeval == null) {
          console.error(othertype.getText() + " is not a valid type")
          process.exit(-49)
        }

        let othertypeVal = othertypeBox.typeval
        if (othertypeVal.generics.length > 0 && othertype.typegenerics() != null) {
          let solidTypes = []
          for (fulltypenameAst of othertype.typegenerics().fulltypename()) {
            solidTypes.push(fulltypenameAst.getText())
          }
          othertypeVal = othertypeVal.solidify(solidTypes, scope)
        }
        unionTypes.push(othertypeVal)
      }
      type.unionTypes = unionTypes
    }
    return type
  }

  solidify(genericReplacements, scope) {
    const Box = require('./Box') // To solve circular dependency issues
    let replacementTypes = []
    for (const typename of genericReplacements) {
      const typebox = scope.deepGet(typename)
      if (typebox == null || typebox.type.typename !== "type") {
        console.error(typename + " type not found")
        process.exit(-35)
      }
      replacementTypes.push(typebox.typeval)
    }
    const solidifiedName = this.typename + "<" + genericReplacements.join(", ") + ">"
    let solidified = new Type(solidifiedName, this.builtIn)
    solidified.originalType = this
    for (const propKey of Object.keys(this.properties)) {
      const propValue = this.properties[propKey]
      if (propValue.isGenericStandin) {
        const genericLoc = this.generics[propValue.typename]
        if (genericLoc == null) {
          console.error("Generic property not described but not found. Should be impossible")
          process.exit(-36)
        }
        const replacementType = replacementTypes[genericLoc]
        solidified.properties[propKey] = replacementType
      } else {
        solidified.properties[propKey] = propValue
      }
    }
    scope.put(solidifiedName, new Box(solidified))
    return solidified
  }

  // This is only necessary for the numeric types. TODO: Can we eliminate it?
  castable(otherType) {
    const intTypes = ["int8", "int16", "int32", "int64"]
    const floatTypes = ["float32", "float64"]
    if (intTypes.includes(this.typename) && intTypes.includes(otherType.typename)) return true
    if (floatTypes.includes(this.typename) && floatTypes.includes(otherType.typename)) return true
    if (floatTypes.includes(this.typename) && intTypes.includes(otherType.typename)) return true
    return false
  }
}

Type.builtinTypes = {
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

module.exports = Type
