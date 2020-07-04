import Box from './Box'
import Scope from './Scope'

type Properties = {
  [K: string]: Type
}

type Generics = {
  [K: string]: number
}

export class FunctionType {
  functionname: string | null
  args: Array<Type>
  returnType: Type

  constructor(...args: Array<any>) {
    if (args.length === 1) {
      this.functionname = null
      this.args = []
      this.returnType = args[0]
    } else if (args.length === 2) {
      if (typeof args[0] === "string") {
        this.functionname = args[0]
        this.args = []
        this.returnType = args[1]
      } else if (args[0] instanceof Array) {
        this.functionname = null
        this.args = args[0]
        this.returnType = args[1]
      }
    } else if (args.length === 3) {
      this.functionname = args[0]
      this.args = args[1]
      this.returnType = args[2]
    }
  }
}

export class OperatorType {
  operatorname: string | null
  isPrefix: boolean
  args: Array<Type>
  returnType: Type

  constructor(
    operatorname: string,
    isPrefix: boolean = false,
    args: Array<Type>,
    returnType: Type
  ) {
    this.operatorname = operatorname
    this.isPrefix = isPrefix
    this.args = args
    this.returnType = returnType
  }
}

export class Interface {
  interfacename: string
  functionTypes: Array<FunctionType>
  operatorTypes: Array<OperatorType>
  requiredProperties: object

  constructor(
    interfacename: string,
    functionTypes: Array<FunctionType> = [],
    operatorTypes: Array<OperatorType> = [],
    requiredProperties: object = {}
  ) {
    this.interfacename = interfacename
    this.functionTypes = functionTypes
    this.operatorTypes = operatorTypes
    this.requiredProperties = requiredProperties
  }

  typeApplies(typeToCheck: Type, scope: Scope) {
    // Solve circular dependency issue
    for (const requiredProperty of Object.keys(this.requiredProperties)) {
      if (!typeToCheck.properties.hasOwnProperty(requiredProperty)) return false
    }

    for (const functionType of this.functionTypes) {
      if (!functionType.functionname) continue // Anonymous functions checked at callsite
      const potentialFunctionsBox = scope.deepGet(functionType.functionname)
      if (!potentialFunctionsBox || potentialFunctionsBox.type !== Type.builtinTypes["function"]) {
        console.error(functionType.functionname + " is not the name of a function")
        process.exit(-48)
      }
      const potentialFunctions = potentialFunctionsBox.val
      let functionFound = false
      for (const potentialFunction of potentialFunctions) {
        const argTypes = potentialFunction.getArguments()
        let argsMatch = true
        for (let i = 0; i < argTypes.length; i++) {
          const functionTypeArgType = functionType.args[i]
          if (argTypes[i] === functionTypeArgType) continue
          if (argTypes[i].originalType === functionTypeArgType) continue
          if (argTypes[i] === typeToCheck) continue
          if (
            !!argTypes[i].iface &&
            !!functionTypeArgType.iface &&
            argTypes[i].iface === functionTypeArgType.iface
          ) continue
          argsMatch = false
          break
        }
        if (!argsMatch) continue
        functionFound = true
        break
      }
      if (!functionFound) return false
    }

    for (const operatorType of this.operatorTypes) {
      const potentialOperatorsBox = scope.deepGet(operatorType.operatorname)
      if (!potentialOperatorsBox || potentialOperatorsBox.type !== Type.builtinTypes.operator) {
        console.error(`${operatorType.operatorname} is not an operator`)
        console.error(potentialOperatorsBox)
        process.exit(-52)
      }
      const potentialOperators = potentialOperatorsBox.val
      let operatorFound = false
      for (const potentialOperator of potentialOperators) {
        for (const potentialFunction of potentialOperator.potentialFunctions) {
          const argTypes = potentialFunction.getArguments()
          let argsMatch = true
          for (let i = 0; i < argTypes.length; i++) {
            const operatorTypeArgType = operatorType.args[i]
            if (argTypes[i] === operatorTypeArgType) continue
            if (argTypes[i].originalType === operatorTypeArgType) continue
            if (argTypes[i] === typeToCheck) continue
            if (
              !!argTypes[i].iface &&
              !!operatorTypeArgType.iface &&
              argTypes[i].iface === operatorTypeArgType.iface
            ) continue
            argsMatch = false
            break
          }
          if (!argsMatch) continue
          operatorFound = true
          break
        }
      }
      if (!operatorFound) return false
    }

    return true
  }

  static fromAst(interfaceAst: any, scope: Scope) { // TODO: replace ANTLR
    // Construct the basic interface, the wrapper type, and insert it into the scope
    // This is all necessary so the interface can self-reference when constructing the function and
    // operator types.
    const interfacename = interfaceAst.VARNAME().getText()
    let iface = new Interface(interfacename)
    const ifaceType = new Type(interfacename, false, iface)
    const ifaceTypeBox = new Box(ifaceType, Type.builtinTypes.type)
    scope.put(interfacename, ifaceTypeBox)

    // Now, insert the actual declarations of the interface, if there are any (if there are none,
    // it will provide only as much as a type generic -- you can set it to a variable and return it
    // but nothing else, unlike Go's ridiculous interpretation of a bare interface).
    if (!!interfaceAst.interfaceline()) {
      for (const interfaceline of interfaceAst.interfaceline()) {
        if (!!interfaceline.functiontypeline()) {
          const functiontypeline = interfaceline.functiontypeline()
          let functionname = null
          if (!!functiontypeline.VARNAME()) {
            functionname = functiontypeline.VARNAME().getText()
          }
          const typenames = functiontypeline.functiontype().varn();
          const returnTypeBox = scope.deepGet(typenames[typenames.length - 1].getText())
          if (!returnTypeBox || returnTypeBox.type !== Type.builtinTypes.type) {
            console.error(typenames.get(typenames.size() - 1).getText() + " is not a type")
            process.exit(-48)
          }
          const returnType = returnTypeBox.val
          let args = []
          for (let i = 0; i < typenames.length - 1; i++) {
            const argumentBox = scope.deepGet(typenames[i].getText())
            if (!argumentBox || argumentBox.type !== Type.builtinTypes.type) {
              console.error(typenames.get(i).getText() + " is not a type")
              process.exit(-49)
            }
            args.push(argumentBox.val)
          }
          const functionType = new FunctionType(functionname, args, returnType)
          iface.functionTypes.push(functionType)
        }
        if (!!interfaceline.operatortypeline()) {
          const operatorname = interfaceline.operatortypeline().operators().getText()
          const isPrefix = !interfaceline.operatortypeline().leftarg()
          const argTypenames = []
          if (!isPrefix) {
            argTypenames.push(interfaceline.operatortypeline().leftarg().getText())
          }
          argTypenames.push(interfaceline.operatortypeline().rightarg().getText())
          const returnTypename = interfaceline.operatortypeline().varn().getText()
          const args = argTypenames.map(n => {
            const box = scope.deepGet(n)
            if (!box || box.type !== Type.builtinTypes.type) {
              console.error(`${n} is not a type`)
              process.exit(-50)
            }
            return box.val
          })
          const returnBox = scope.deepGet(returnTypename)
          if (!returnBox || returnBox.type !== Type.builtinTypes.type) {
            console.error(`${returnTypename} is not a type`)
            process.exit(-51)
          }
          const returnType = returnBox.val
          const operatorType = new OperatorType(operatorname, isPrefix, args, returnType)
          iface.operatorTypes.push(operatorType)
        }
        if (!!interfaceline.propertytypeline()) {
          const propertyTypeBox = scope.deepGet(interfaceline.propertytypeline().varn().getText())
          if (!propertyTypeBox || propertyTypeBox.type !== Type.builtinTypes.type) {
            console.error(interfaceline.propertytypeline().varn().getText() + " is not a type")
            process.exit(-52)
          }
          iface.requiredProperties[
            interfaceline.propertytypeline().VARNAME().getText()
          ] = propertyTypeBox.val
        }
      }
    }
    return ifaceTypeBox
  }
}

export class Type {
  typename: string
  builtIn: boolean
  isGenericStandin: boolean
  properties: Properties
  generics: Generics
  originalType: Type | null
  unionTypes: Array<Type> | null
  iface: Interface | null
  alias: Type | null

  constructor(...args: Array<any>) {
    // Simulate multiple dispatch by duck typing the args
    // TODO: Switch this to arguments with default values
    if (args.length === 1) {
      this.typename = args[0]
      this.builtIn = false
      this.isGenericStandin = false
      this.properties = {}
      this.generics = {}
      this.originalType = null
      this.unionTypes = null
      this.iface = null
      this.alias = null
    } else if (args.length === 2) {
      this.typename = args[0]
      this.builtIn = args[1]
      this.isGenericStandin = false
      this.properties = {}
      this.generics = {}
      this.originalType = null
      this.unionTypes = null
      this.iface = null
      this.alias = null
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
        this.alias = null
      } else if (args[2] instanceof Interface) {
        this.typename = args[0]
        this.builtIn = args[1]
        this.isGenericStandin = false
        this.properties = {}
        this.generics = {}
        this.originalType = null
        this.unionTypes = null
        this.iface = args[2]
        this.alias = null
      } else if (args[2] instanceof Array) {
        this.typename = args[0]
        this.builtIn = args[1]
        this.isGenericStandin = false
        this.properties = {}
        this.generics = {}
        this.originalType = null
        this.unionTypes = args[2]
        this.iface = null
        this.alias = null
      } else if (args[2] instanceof Object) {
        this.typename = args[0]
        this.builtIn = args[1]
        this.isGenericStandin = false
        this.properties = args[2]
        this.generics = {}
        this.originalType = null
        this.unionTypes = null
        this.iface = null
        this.alias = null
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
      this.alias = null
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

  static fromAst(typeAst: any, scope: Scope) { // TODO: Migrate away from ANTLR
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
        const property = scope.deepGet(lineAst.varn().getText())
        if (property == null || property.type.typename !== "type") {
          if (type.generics.hasOwnProperty(typeName)) {
            type.properties[propertyName] = new Type(typeName, true, true)
          } else {
            console.error(lineAst.varn().getText() + " is not a type")
            process.exit(-4)
          }
        } else {
          type.properties[propertyName] = property.val
        }
      }
    }
    if (typeAst.othertype() != null && typeAst.othertype().length == 1) {
      const otherTypebox = scope.deepGet(typeAst.othertype(0).typename().getText())

      if (!otherTypebox) {
        console.error("Type " + typeAst.othertype(0).getText() + " not defined")
        process.exit(-38)
      }
      if (otherTypebox.type !== Type.builtinTypes.type) {
        console.error(typeAst.othertype(0).getText() + " is not a valid type")
        process.exit(-39)
      }

      let othertype = otherTypebox.val
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
        if (othertypeBox.type !== Type.builtinTypes.type) {
          console.error(othertype.getText() + " is not a valid type")
          process.exit(-49)
        }

        let othertypeVal = othertypeBox.val
        if (othertypeVal.generics.length > 0 && othertype.typegenerics() != null) {
          let solidTypes = []
          for (const fulltypenameAst of othertype.typegenerics().fulltypename()) {
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

  solidify(genericReplacements: Array<string>, scope: Scope) {
    let replacementTypes = []
    for (const typename of genericReplacements) {
      const typebox = scope.deepGet(typename)
      if (typebox == null || typebox.type.typename !== "type") {
        console.error(typename + " type not found")
        process.exit(-35)
      }
      replacementTypes.push(typebox.val)
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
    scope.put(solidifiedName, new Box(solidified, Type.builtinTypes.type))
    return solidified
  }

  // This is only necessary for the numeric types. TODO: Can we eliminate it?
  castable(otherType: Type) {
    const intTypes = ["int8", "int16", "int32", "int64"]
    const floatTypes = ["float32", "float64"]
    if (intTypes.includes(this.typename) && intTypes.includes(otherType.typename)) return true
    if (floatTypes.includes(this.typename) && floatTypes.includes(otherType.typename)) return true
    if (floatTypes.includes(this.typename) && intTypes.includes(otherType.typename)) return true
    return false
  }

  static builtinTypes = {
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
}

export default Type

