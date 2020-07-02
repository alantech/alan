import Box from './Box'
import FunctionType from './FunctionType'
import OperatorType from './OperatorType'
import Scope from './Scope'
import Type from './Type'

class Interface {
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
      const potentialFunctions = potentialFunctionsBox.functionval
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
        process.exit(-52)
      }
      const potentialOperators = potentialOperatorsBox.operatorval
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
    const ifaceTypeBox = new Box(ifaceType)
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
          if (!returnTypeBox || !returnTypeBox.typeval) {
            console.error(typenames.get(typenames.size() - 1).getText() + " is not a type")
            process.exit(-48)
          }
          const returnType = returnTypeBox.typeval
          let args = []
          for (let i = 0; i < typenames.length - 1; i++) {
            const argumentBox = scope.deepGet(typenames[i].getText())
            if (!argumentBox || !argumentBox.typeval) {
              console.error(typenames.get(i).getText() + " is not a type")
              process.exit(-49)
            }
            args.push(argumentBox.typeval)
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
            if (!box || !box.typeval) {
              console.error(`${n} is not a type`)
              process.exit(-50)
            }
            return box.typeval
          })
          const returnBox = scope.deepGet(returnTypename)
          if (!returnBox || !returnBox.typeval) {
            console.error(`${returnTypename} is not a type`)
            process.exit(-51)
          }
          const returnType = returnBox.typeval
          const operatorType = new OperatorType(operatorname, isPrefix, args, returnType)
          iface.operatorTypes.push(operatorType)
        }
        if (!!interfaceline.propertytypeline()) {
          const propertyTypeBox = scope.deepGet(interfaceline.propertytypeline().varn().getText())
          if (!propertyTypeBox || !propertyTypeBox.typeval) {
            console.error(interfaceline.propertytypeline().varn().getText() + " is not a type")
            process.exit(-52)
          }
          iface.requiredProperties[
            interfaceline.propertytypeline().VARNAME().getText()
          ] = propertyTypeBox.typeval
        }
      }
    }
    return ifaceTypeBox
  }
}

export default Interface
