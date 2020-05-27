const Type = require('./Type')
const FunctionType = require('./FunctionType')

class Interface {
  constructor(...args) {
    if (args.length === 1) {
      this.interfacename = args[0]
      this.functionTypes = []
      this.operatorTypes = []
      this.requiredProperties = {}
    } else if (args.length === 4) {
      this.interfacename = args[0]
      this.functionTypes = args[1]
      this.operatorTypes = args[2]
      this.requiredProperties = args[3]
    }
  }

  typeApplies(typeToCheck, scope) {
    // Solve circular dependency issue
    const Box = require('./Box')
    for (const requiredProperty of Object.keys(this.requiredProperties)) {
      if (!typeToCheck.properties.hasOwnProperty(requiredProperty)) return false
    }

    for (const functionType of this.functionTypes) {
      if (functionType.functionname === null) continue // Anonymous functions checked at callsite
      const potentialFunctionsBox = scope.deepGet(functionType.functionname)
      if (
        potentialFunctionsBox == null ||
        potentialFunctionsBox.type != Box.builtinTypes["function"]
      ) {
        console.error(functionType.functionname + " is not the name of a function")
        process.exit(-48)
      }
      const potentialFunctions = potentialFunctionsBox.functionval
      let functionFound = false;
      for (const potentialFunction of potentialFunctions) {
        const argTypes = potentialFunction.getArguments()
        let argsMatch = true;
        for (let i = 0; i < argTypes.length; i++) {
          const functionTypeArgType = functionType.args[i];
          if (argTypes[i] == functionTypeArgType) continue
          if (argTypes[i].originalType == functionTypeArgType) continue
          if (argTypes[i] == typeToCheck) continue
          if (
            argTypes[i].iface != null &&
            functionTypeArgType.iface != null &&
            argTypes[i].iface == functionTypeArgType.iface
          ) continue
          argsMatch = false
          break
        }
        if (!argsMatch) continue
        functionFound = true
        break
        // TODO: Need to do special work to handle n-ary functions, but users can't define those yet
      }
      if (!functionFound) return false
    }

    for (const operatorType of this.operatorTypes) {
      // TODO: Implement me!
    }

    return true
  }

  static fromAst(interfaceAst, scope) {
    const Box = require('./Box')
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
    if (interfaceAst.interfaceline() != null) {
      for (const interfaceline of interfaceAst.interfaceline()) {
        if (interfaceline.functiontypeline() != null) {
          const functiontypeline = interfaceline.functiontypeline()
          let functionname = null
          if (functiontypeline.VARNAME() != null) {
            functionname = functiontypeline.VARNAME().getText()
          }
          const typenames = functiontypeline.functiontype().varn();
          const returnTypeBox = scope.deepGet(typenames[typenames.length - 1].getText())
          if (returnTypeBox == null || returnTypeBox.typeval == null) {
            console.error(typenames.get(typenames.size() - 1).getText() + " is not a type")
            process.exit(-48)
          }
          const returnType = returnTypeBox.typeval
          let args = []
          for (let i = 0; i < typenames.length - 1; i++) {
            const argumentBox = scope.deepGet(typenames[i].getText())
            if (argumentBox == null || argumentBox.typeval == null) {
              console.error(typenames.get(i).getText() + " is not a type")
              process.exit(-49)
            }
            args.push(argumentBox.typeval)
          }
          const functionType = new FunctionType(functionname, args, returnType)
          iface.functionTypes.push(functionType)
        }
        if (interfaceline.operatortypeline() != null) {
          // TODO: Implement me! 
          console.error("Operator type declarations not yet implemented!")
        }
        if (interfaceline.propertytypeline() != null) {
          const propertyTypeBox = scope.deepGet(interfaceline.propertytypeline().varn().getText())
          if (propertyTypeBox == null || propertyTypeBox.typeval == null) {
            console.error(interfaceline.propertytypeline().varn().getText() + " is not a type")
            process.exit(-50)
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

module.exports = Interface
