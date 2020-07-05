import Scope from './Scope'
import { Fn, } from './Function'
import Operator from './Operator'

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

  constructor(functionname: string | null = null, args: Array<Type> = [], returnType: Type) {
    this.functionname = functionname
    this.args = args
    this.returnType = returnType
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
      const potentialFunctions = scope.deepGet(functionType.functionname) as Array<Fn>
      if (
        !potentialFunctions ||
        !(
          potentialFunctions instanceof Array &&
          potentialFunctions[0].microstatementInlining instanceof Function
        )
      ) {
        console.error(functionType.functionname + " is not the name of a function")
        process.exit(-48)
      }
      let functionFound = false
      for (const potentialFunction of potentialFunctions) {
        const argTypes = potentialFunction.getArguments()
        let argsMatch = true
        let typeNames = Object.keys(argTypes)
        for (let i = 0; i < typeNames.length; i++) {
          const functionTypeArgType = functionType.args[i]
          if (argTypes[typeNames[i]] === functionTypeArgType) continue
          if (argTypes[typeNames[i]].originalType === functionTypeArgType) continue
          if (argTypes[typeNames[i]] === typeToCheck) continue
          if (
            !!argTypes[typeNames[i]].iface &&
            !!functionTypeArgType.iface &&
            argTypes[typeNames[i]].iface === functionTypeArgType.iface
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
      const potentialOperators = scope.deepGet(operatorType.operatorname) as Array<Operator>
      if (
        !potentialOperators ||
        !(
          potentialOperators instanceof Array &&
          potentialOperators[0] instanceof Operator
        )
      ) {
        console.error(`${operatorType.operatorname} is not an operator`)
        process.exit(-52)
      }
      let operatorFound = false
      for (const potentialOperator of potentialOperators) {
        for (const potentialFunction of potentialOperator.potentialFunctions) {
          const argTypes = potentialFunction.getArguments()
          let argsMatch = true
          let typeNames = Object.keys(argTypes)
          for (let i = 0; i < typeNames.length; i++) {
            const operatorTypeArgType = operatorType.args[i]
            if (argTypes[typeNames[i]] === operatorTypeArgType) continue
            if (argTypes[typeNames[i]].originalType === operatorTypeArgType) continue
            if (argTypes[typeNames[i]] === typeToCheck) continue
            if (
              !!argTypes[typeNames[i]].iface &&
              !!operatorTypeArgType.iface &&
              argTypes[typeNames[i]].iface === operatorTypeArgType.iface
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
    const ifaceType = new Type(interfacename, false, false, {}, {}, null, null, iface)
    scope.put(interfacename, ifaceType)

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
          const returnType = scope.deepGet(typenames[typenames.length - 1].getText()) as Type
          if (!returnType || !(returnType instanceof Type)) {
            console.error(typenames.get(typenames.size() - 1).getText() + " is not a type")
            process.exit(-48)
          }
          let args = []
          for (let i = 0; i < typenames.length - 1; i++) {
            const argument = scope.deepGet(typenames[i].getText()) as Type
            if (!argument || !(argument instanceof Type)) {
              console.error(typenames.get(i).getText() + " is not a type")
              process.exit(-49)
            }
            args.push(argument)
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
            if (!box || !(box instanceof Type)) {
              console.error(`${n} is not a type`)
              process.exit(-50)
            }
            return box
          })
          const returnType = scope.deepGet(returnTypename) as Type
          if (!returnType || !(returnType instanceof Type)) {
            console.error(`${returnTypename} is not a type`)
            process.exit(-51)
          }
          const operatorType = new OperatorType(operatorname, isPrefix, args, returnType)
          iface.operatorTypes.push(operatorType)
        }
        if (!!interfaceline.propertytypeline()) {
          const propertyType =
            scope.deepGet(interfaceline.propertytypeline().varn().getText()) as Type
          if (!propertyType || !(propertyType instanceof Type)) {
            console.error(interfaceline.propertytypeline().varn().getText() + " is not a type")
            process.exit(-52)
          }
          iface.requiredProperties[
            interfaceline.propertytypeline().VARNAME().getText()
          ] = propertyType
        }
      }
    }
    return ifaceType
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

  constructor(
    typename: string,
    builtIn: boolean = false,
    isGenericStandin: boolean = false,
    properties: Properties = {},
    generics: Generics = {},
    originalType: Type | null = null,
    unionTypes: Array<Type> | null = null,
    iface: Interface | null = null,
    alias: Type | null = null,
  ) {
    this.typename = typename
    this.builtIn = builtIn
    this.isGenericStandin = isGenericStandin
    this.properties = properties
    this.generics = generics
    this.originalType = originalType
    this.unionTypes = unionTypes
    this.iface = iface
    this.alias = alias
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
        const property = scope.deepGet(lineAst.varn().getText()) as Type
        if (!property || !(property instanceof Type)) {
          if (type.generics.hasOwnProperty(typeName)) {
            type.properties[propertyName] = new Type(typeName, true, true)
          } else {
            console.error(lineAst.varn().getText() + " is not a type")
            process.exit(-4)
          }
        } else {
          type.properties[propertyName] = property
        }
      }
    }
    if (typeAst.othertype() != null && typeAst.othertype().length == 1) {
      const otherTypebox = scope.deepGet(typeAst.othertype(0).typename().getText()) as Type

      if (!otherTypebox) {
        console.error("Type " + typeAst.othertype(0).getText() + " not defined")
        process.exit(-38)
      }
      if (!(otherTypebox instanceof Type)) {
        console.error(typeAst.othertype(0).getText() + " is not a valid type")
        process.exit(-39)
      }

      let othertype = otherTypebox
      if (Object.keys(othertype.generics).length > 0 && !!typeAst.othertype(0).typegenerics()) {
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
        const othertypeBox = scope.deepGet(othertype.typename().getText()) as Type

        if (!othertypeBox) {
          console.error("Type " + othertype.getText() + " not defined")
          process.exit(-48)
        }
        if (!(othertypeBox instanceof Type)) {
          console.error(othertype.getText() + " is not a valid type")
          process.exit(-49)
        }

        let othertypeVal = othertypeBox
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
      const typebox = scope.deepGet(typename) as Type
      if (!typebox || !(typebox instanceof Type)) {
        console.error(typename + " type not found")
        process.exit(-35)
      }
      replacementTypes.push(typebox)
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
    scope.put(solidifiedName, solidified)
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
    Error: new Type("Error", true, false, {
      message: new Type("string", true, true),
      code: new Type("int64", true, true),
    }),
    "Array": new Type("Array", true, false, {
      records: new Type("V", true, true),
    }, {
      V: 0,
    }),
    Map: new Type("Map", true, false, {
      key: new Type("K", true, true),
      value: new Type("V", true, true),
    }, {
      K: 0,
      V: 1,
    }),
    KeyVal: new Type("KeyVal", true, false, {
      key: new Type("K", true, true),
      value: new Type("V", true, true),
    }, {
      K: 0,
      V: 1,
    }),
    "function": new Type("function", true),
    operator: new Type("operator", true),
    Event: new Type("Event", true, false, {
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

