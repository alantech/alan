import Scope from './Scope'
import { Fn, } from './Function'
import Operator from './Operator'
import { fulltypenameAstFromString, } from './Ast'

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
        throw new Error(functionType.functionname + " is not the name of a function")
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
          if (
            argTypes[typeNames[i]].originalType === functionTypeArgType.originalType &&
            Object.values(functionTypeArgType.properties).every((prop, j) => {
              const comparable = Object.values(argTypes[typeNames[i]].properties)[j]
              if (prop === comparable) return true
              if (prop.iface && prop.iface.typeApplies(comparable, scope)) return true
              return false
            })
          ) continue
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
        throw new Error(`${operatorType.operatorname} is not an operator`)
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
    const interfacename = interfaceAst.VARNAME(0).getText()
    let iface = new Interface(interfacename)
    const ifaceType = new Type(interfacename, false, false, {}, {}, null, iface)
    scope.put(interfacename, ifaceType)

    // Now, insert the actual declarations of the interface, if there are any (if there are none,
    // it will provide only as much as a type generic -- you can set it to a variable and return it
    // but nothing else, unlike Go's ridiculous interpretation of a bare interface).
    if (!!interfaceAst.interfacebody() && !!interfaceAst.interfacebody().interfacelist()) {
      for (const interfaceline of interfaceAst.interfacebody().interfacelist().interfaceline()) {
        if (!!interfaceline.functiontypeline()) {
          const functiontypeline = interfaceline.functiontypeline()
          let functionname = null
          if (!!functiontypeline.VARNAME()) {
            functionname = functiontypeline.VARNAME().getText()
          }
          const typenames = functiontypeline.functiontype().fulltypename();
          const returnType = scope.deepGet(typenames[typenames.length - 1].getText()) as Type
          if (!returnType || !(returnType instanceof Type)) {
            throw new Error(typenames.get(typenames.size() - 1).getText() + " is not a type")
          }
          let args = []
          for (let i = 0; i < typenames.length - 1; i++) {
            const argument = scope.deepGet(typenames[i].getText()) as Type
            if (!argument || !(argument instanceof Type)) {
              throw new Error(typenames.get(i).getText() + " is not a type")
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
          const returnTypename = interfaceline.operatortypeline().fulltypename().getText()
          const args = argTypenames.map(n => {
            const box = scope.deepGet(n)
            if (!box || !(box instanceof Type)) {
              throw new Error(`${n} is not a type`)
            }
            return box
          })
          const returnType = scope.deepGet(returnTypename) as Type
          if (!returnType || !(returnType instanceof Type)) {
            throw new Error(`${returnTypename} is not a type`)
          }
          const operatorType = new OperatorType(operatorname, isPrefix, args, returnType)
          iface.operatorTypes.push(operatorType)
        }
        if (!!interfaceline.propertytypeline()) {
          const propertyType =
            scope.deepGet(interfaceline.propertytypeline().varn().getText()) as Type
          if (!propertyType || !(propertyType instanceof Type)) {
            throw new Error(interfaceline.propertytypeline().varn().getText() + " is not a type")
          }
          iface.requiredProperties[
            interfaceline.propertytypeline().VARNAME().getText()
          ] = propertyType
        }
      }
    } else if (!!interfaceAst.VARNAME(1)) {
      // It's an alias, so grab it and give it the new name
      const otherInterface = scope.deepGet(interfaceAst.VARNAME(1).getText()) as Type
      if (!(otherInterface instanceof Type) || !otherInterface.iface) {
        throw new Error(`${interfaceAst.varn().getText()} is not an interface`)
      }
      // Replace the interface with the other one
      ifaceType.iface = otherInterface.iface
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
  iface: Interface | null
  alias: Type | null

  constructor(
    typename: string,
    builtIn: boolean = false,
    isGenericStandin: boolean = false,
    properties: Properties = {},
    generics: Generics = {},
    originalType: Type | null = null,
    iface: Interface | null = null,
    alias: Type | null = null,
  ) {
    this.typename = typename
    this.builtIn = builtIn
    this.isGenericStandin = isGenericStandin
    this.properties = properties
    this.generics = generics
    this.originalType = originalType
    this.iface = iface
    this.alias = alias
  }

  toString() {
    if (this.iface != null) return "// Interfaces TBD"
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
    const genScope = new Scope()
    const typeScope = new Scope(scope)
    typeScope.secondaryPar = genScope
    if (typeAst.typegenerics() != null) {
      const generics = typeAst.typegenerics().fulltypename()
      for (let i = 0; i < generics.length; i++) {
        type.generics[generics[i].getText()] = i
        genScope.put(generics[i].getText(), new Type(generics[i].getText(), true, true))
      }
    }
    if (typeAst.typebody() != null) {
      const lines = typeAst.typebody().typelist().typeline()
      for (const lineAst of lines) {
        const propertyName = lineAst.VARNAME().getText()
        const typeName = lineAst.fulltypename().getText().trim()
        const property = typeScope.deepGet(typeName) as Type
        if (!property || !(property instanceof Type)) {
          // Potentially a type that depends on the type generics of this type
          const baseTypeName = lineAst.fulltypename().typename().getText()
          const innerGenerics = lineAst.fulltypename().typegenerics().fulltypename()
          const genericsList = []
          const genericsQueue = []
          for (const generic of innerGenerics) {
            genericsList.push(generic)
          }
          while (genericsList.length > 0) {
            const generic = genericsList.shift()
            genericsQueue.push(generic)
            if (generic.typegenerics()) {
              genericsList.push(...generic.typegenerics().fulltypename())
            }
          }
          while (genericsQueue.length > 0) {
            const generic = genericsQueue.pop()
            const innerType = typeScope.deepGet(generic.getText()) as Type
            if (!innerType) {
              const innerBaseTypeName = generic.typename().getText()
              const innerBaseType = typeScope.deepGet(innerBaseTypeName) as Type
              if (!innerBaseType) {
                throw new Error('wut')
              }
              innerBaseType.solidify(
                generic.typegenerics().fulltypename().map((t: any) => t.getText()),
                typeScope,
              )
            }
          }
          const baseType = scope.deepGet(baseTypeName) as Type
          if (!baseType || !(baseType instanceof Type)) {
            throw new Error(lineAst.fulltypename().getText() + " is not a type")
          }
          type.properties[propertyName] = baseType.solidify(
            innerGenerics.map((t: any) => t.getText()),
            typeScope,
          )
        } else {
          type.properties[propertyName] = property
        }
      }
    }
    if (typeAst.fulltypename() != null) {
      const otherTypebox = scope.deepGet(typeAst.fulltypename().typename().getText()) as Type
      if (!otherTypebox) {
        throw new Error("Type " + typeAst.fulltypename().getText() + " not defined")
      }
      if (!(otherTypebox instanceof Type)) {
        throw new Error(typeAst.fulltypename().getText() + " is not a valid type")
      }

      let fulltypename = otherTypebox
      if (Object.keys(fulltypename.generics).length > 0 && !!typeAst.fulltypename().typegenerics()) {
        let solidTypes = []
        for (const fulltypenameAst of typeAst.fulltypename().typegenerics().fulltypename()) {
          solidTypes.push(fulltypenameAst.getText())
        }
        fulltypename = fulltypename.solidify(solidTypes, scope)
      }

      // For simplification of the type aliasing functionality, the other type is attached as
      // an alias. The module construction will, if present, perfer the alias over the actual
      // type, to make sure built-in types that are aliased continue to work. This means that
      // `type varA == type varB` will work if `varA` is assigned to an alias and `varB` to the
      // orignal type. I can see the argument either way on this, but the simplicity of this
      // approach is why I will go with this for now.
      type.alias = fulltypename
    }
    scope.put(type.typename, type)
    return type
  }

  solidify(genericReplacements: Array<string>, scope: Scope) {
    let genericTypes = Object.keys(this.generics).map(t => new Type(t, true, true))
    let replacementTypes = []
    for (const typename of genericReplacements) {
      const typebox = scope.deepGet(typename) as Type
      if (!typebox || !(typebox instanceof Type)) {
        const fulltypename = fulltypenameAstFromString(typename)
        if (fulltypename.typegenerics()) {
          const basename = fulltypename.typename().getText()
          const generics = fulltypename.typegenerics().fulltypename().map((t: any) => t.getText())
          const baseType = scope.deepGet(basename) as Type
          if (!baseType || !(baseType instanceof Type)) {
            throw new Error(basename + " type not found")
          } else {
            const newtype = baseType.solidify(generics, scope)
            replacementTypes.push(newtype)
          }
        } else {
          throw new Error(typename + " type not found")
        }
      } else {
        replacementTypes.push(typebox)
      }
    }
    const genericMap = new Map()
    genericTypes.forEach((g, i) => genericMap.set(g, replacementTypes[i]))
    const solidifiedName = this.typename + "<" + genericReplacements.join(", ") + ">"
    let solidified = new Type(solidifiedName, this.builtIn)
    solidified.originalType = this
    for (const propKey of Object.keys(this.properties)) {
      const propValue = this.properties[propKey]
      const newPropValue = propValue.realize(genericMap, scope)
      solidified.properties[propKey] = newPropValue
    }
    scope.put(solidifiedName, solidified)
    return solidified
  }

  typeApplies(otherType: Type, scope: Scope, interfaceMap: Map<Type, Type> = new Map()) {
    if (this.typename === otherType.typename) return true
    if (!!this.iface) {
      const applies = this.iface.typeApplies(otherType, scope)
      if (applies) {
        interfaceMap.set(this, otherType)
      }
      return applies
    }
    if (
      !this.originalType ||
      !otherType.originalType ||
      this.originalType.typename !== otherType.originalType.typename
    ) return false
    const typeAst = fulltypenameAstFromString(this.typename) as any
    const otherTypeAst = fulltypenameAstFromString(otherType.typename) as any
    const generics = typeAst.typegenerics().fulltypename().map((g: any) => (
      scope.deepGet(g.getText()) ||
      Type.fromStringWithMap(g.getText(), interfaceMap, scope)) as Type
    )
    const otherGenerics = otherTypeAst.typegenerics().fulltypename().map((g: any) => (
      scope.deepGet(g.getText()) ||
      Type.fromStringWithMap(g.getText(), interfaceMap, scope)) as Type
    )
    return generics.every((t: Type, i: any) => t.typeApplies(otherGenerics[i], scope, interfaceMap))
  }

  // There has to be a more elegant way to tackle this
  static fromStringWithMap(typestr: string, interfaceMap: Map<Type, Type>, scope: Scope) {
    const typeAst = fulltypenameAstFromString(typestr)
    const baseName = typeAst.typename().getText()
    const baseType = scope.deepGet(baseName) as Type
    if (typeAst.typegenerics()) {
      const genericNames = typeAst.typegenerics().fulltypename().map((t: any) => t.getText())
      const generics = genericNames.map((t: string) => {
        const interfaceMapping = [
          ...interfaceMap.entries()
        ].find(e => e[0].typename === t.trim())
        if (interfaceMapping) return interfaceMapping[1]
        const innerType = Type.fromStringWithMap(t, interfaceMap, scope)
        return innerType
      })
      return baseType.solidify(
        generics.map((g: Type) => interfaceMap.get(g) || g).map((t: Type) => t.typename),
        scope
      )
    } else {
      return interfaceMap.get(baseType) || baseType
    }
  }

  realize(interfaceMap: Map<Type, Type>, scope: Scope) {
    if (!!this.isGenericStandin) return [
      ...interfaceMap.entries()
    ].find(e => e[0].typename === this.typename)[1]
    if (!this.iface && !this.originalType) return this
    if (!!this.iface) return interfaceMap.get(this) || this
    const self = new Type(
      this.typename,
      this.builtIn,
      this.isGenericStandin,
      { ...this.properties, },
      { ...this.generics, },
      this.originalType,
      this.iface,
      this.alias,
    )
    const newProps = Object.values(self.properties).map(t => t.realize(interfaceMap, scope))
    Object.keys(self.properties).forEach((k, i) => {
      self.properties[k] = newProps[i]
    })
    const newType = Type.fromStringWithMap(self.typename, interfaceMap, scope)
    return newType
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
    "Error": new Type("Error", true, false, {
      msg: new Type("string", true, true),
    }),
    "Maybe": new Type("Maybe", true, false, {
      value: new Type("T", true, true),
    }, {
      T: 0,
    }),
    "Result": new Type("Result", true, false, {
      value: new Type("T", true, true),
      error: new Type("Error", true, false, {
        msg: new Type("string", true, true),
      }),
    }, {
      T: 0,
    }),
    "Either": new Type("Either", true, false, {
      main: new Type("T", true, true),
      alt: new Type("U", true, true),
    }, {
      T: 0,
      U: 1,
    }),
    "Array": new Type("Array", true, false, {
      records: new Type("V", true, true),
    }, {
      V: 0,
    }),
    ExecRes: new Type("ExecRes", false, false, {
      exitCode: new Type("int64", true),
      stdout: new Type("string", true),
      stderr: new Type("string", true),
    }),
    InitialReduce: new Type("InitialReduce", false, false, {
      arr: new Type("Array<T>", true, false, {
        records: new Type("T", true, true),
      }, {
        T: 0,
      }),
      initial: new Type("U", true, true),
    }, {
      T: 0,
      U: 1,
    }),
    // HTTP server opcode-related builtin Types, also defined in std/http.ln
    InternalRequest: new Type("InternalRequest", true, false, {
      url: new Type("string", true),
      headers: new Type("Array<KeyVal<string, string>>", true, false, {
        records: new Type('KeyVal<string, string>>', true, false, {
          key: new Type("string", true),
          val: new Type("string", true),
        }),
      }),
      body: new Type('string', true),
      connId: new Type('int64', true),
    }),
    InternalResponse: new Type("InternalResponse", true, false, {
      status: new Type("int64", true),
      headers: new Type("Array<KeyVal<string, string>>", true, false, {
        records: new Type('KeyVal<string, string>>', true, false, {
          key: new Type("string", true),
          val: new Type("string", true),
        }),
      }),
      body: new Type('string', true),
      connId: new Type('int64', true),
    }),
    Seq: new Type("Seq", true, false, {
      counter: new Type("int64", true, true),
      limit: new Type("int64", true, true),
    }),
    Self: new Type("Self", true, false, {
      seq: new Type("Seq", true, false, {
        counter: new Type("int64", true, true),
        limit: new Type("int64", true, true),
      }),
      recurseFn: new Type("function", true),
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

