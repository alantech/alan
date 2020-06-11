const { v4: uuid, } = require('uuid')

const Ast = require('./Ast')
const Box = require('./Box')
const Statement = require('./Statement')
const StatementType = require('./StatementType')
const { LnParser, } = require('../ln')

// This only implements the parts required for the compiler
class UserFunction {
  constructor(name, args, returnType, closureScope, statements, pure) {
    this.name = name
    this.args = args
    this.returnType = returnType
    this.closureScope = closureScope
    this.statements = statements
    this.pure = pure
  }

  static fromAst(functionishAst, closureScope) {
    if (functionishAst instanceof LnParser.BlocklikesContext) {
      if (functionishAst.functions() != null) {
        return UserFunction.fromFunctionsAst(functionishAst.functions(), closureScope)
      }
      if (functionishAst.functionbody() != null) {
        return UserFunction.fromFunctionbodyAst(functionishAst.functionbody(), closureScope)
      }
    }
    if (functionishAst instanceof LnParser.FunctionsContext) {
      return UserFunction.fromFunctionsAst(functionishAst, closureScope)
    }
    if (functionishAst instanceof LnParser.FunctionbodyContext) {
      return UserFunction.fromFunctionbodyAst(functionishAst, closureScope)
    }
    return null
  }

  static fromFunctionbodyAst(functionbodyAst, closureScope) {
    let args = {}
    const returnType = Box.builtinTypes.void
    let pure = true // Assume purity and then downgrade if needed
    let statements = []
    const statementsAst = functionbodyAst.statements()
    for (const statementAst of statementsAst) {
      const statement = Statement.create(statementAst, closureScope)
      if (!statement.pure) pure = false
      statements.push(statement)
    }
    return new UserFunction(null, args, returnType, closureScope, statements, pure)
  }

  static fromFunctionsAst(functionAst, closureScope) {
    const name = functionAst.VARNAME() == null ? null : functionAst.VARNAME().getText()
    let args = {}
    const argsAst = functionAst.arglist()
    if (argsAst !== null) {
      const arglen = argsAst.VARNAME().length
      for (let i = 0; i < arglen; i++) {
        const argName = argsAst.VARNAME(i).getText()
        let getArgType = closureScope.deepGet(argsAst.argtype(i).getText())
        if (getArgType === null) {
          if (argsAst.argtype(i).othertype().length === 1) {
            if (argsAst.argtype(i).othertype(0).typegenerics() !== null) {
              getArgType = closureScope.deepGet(argsAst.argtype(i).othertype(0).typename().getText())
              if (getArgType == null) {
                console.error("Could not find type " + argsAst.argtype(i).getText() + " for argument " + argName)
                process.exit(-39)
              }
              if (getArgType.type !== Box.builtinTypes["type"]) {
                console.error("Function argument is not a valid type: " + argsAst.argtype(i).getText())
                process.exit(-50);
              }
              let genericTypes = []
              for (const fulltypename of argsAst.argtype(i).othertype(0).typegenerics().fulltypename()) {
                genericTypes.push(fulltypename.getText())
              }
              getArgType = new Box(getArgType.typeval.solidify(genericTypes, closureScope))
            } else {
              console.error("Could not find type " + argsAst.argtype(i).getText() + " for argument " + argName)
              process.exit(-51)
            }
          } else { // It's an inline-declared union type
            const othertypes = argsAst.argtype(i).othertype()
            let unionTypes = []
            for (const othertype of othertypes) {
              let othertypeBox = closureScope.deepGet(othertype.getText())
              if (othertypeBox == null) {
                if (othertype.typegenerics() != null) {
                  othertypeBox = closureScope.deepGet(othertype.typename().getText())
                  if (othertypeBox == null) {
                    console.error("Could not find type " + othertype.getText() + " for argument " + argName)
                    process.exit(-59)
                  }
                  if (othertypeBox.type != Box.builtinTypes["type"]) {
                    console.error("Function argument is not a valid type: " + othertype.getText())
                    process.exit(-60)
                  }
                  let genericTypes = []
                  for (const fulltypename of othertype.typegenerics().fulltypename()) {
                    genericTypes.push(fulltypename.getText())
                  }
                  othertypeBox = new Box(othertypeBox.typeval.solidify(genericTypes, closureScope))
                } else {
                  console.error("Could not find type " + othertype.getText() + " for argument " + argName)
                  process.exit(-51)
                }
              }
              unionTypes.push(othertypeBox.typeval)
            }
            const union = new Type(argsAst.argtype(i).getText(), false, unionTypes)
            getArgType = new Box(union)
          }
        }
        if (getArgType.type != Box.builtinTypes["type"]) {
          console.error("Function argument is not a valid type: " + argsAst.argtype(i).getText())
          process.exit(-13)
        }
        args[argName] = getArgType.typeval
      }
    }
    let returnType = null
    if (functionAst.argtype() !== null) {
      if (functionAst.argtype().othertype().length === 1) {
        let getReturnType = closureScope.deepGet(functionAst.argtype().getText())
        if (getReturnType == null || getReturnType.type != Box.builtinTypes["type"]) {
          if (functionAst.argtype().othertype(0).typegenerics() != null) {
            getReturnType = closureScope.deepGet(functionAst.argtype().othertype(0).typename().getText())
            if (getReturnType == null) {
              console.error("Could not find type " + functionAst.argtype().getText() + " for function " + functionAst.VARNAME().getText())
              process.exit(-59)
            }
            if (getReturnType.type !== Box.builtinTypes["type"]) {
              console.error("Function return is not a valid type: " + functionAst.argtype().getText())
              process.exit(-60)
            }
            let genericTypes = []
            for (const fulltypename of functionAst.argType().othertype(0).typegenerics().fulltypename()) {
              genericTypes.push(fulltypename.getText())
            }
            getReturnType = new Box(getReturnType.typeval.solidify(genericTypes, closureScope))
          } else {
            console.error("Could not find type " + functionAst.argtype().getText() + " for function " + functionAst.VARNAME().getText())
            process.exit(-61)
          }
        }
        returnType = getReturnType.typeval
      } else {
        const othertypes = functionAst.argtype().othertype()
        let unionTypes = []
        for (const othertype of othertypes) {
          let othertypeBox = closureScope.deepGet(othertype.getText())
          if (othertypeBox === null) {
            if (othertype.typegenerics() !== null) {
              othertypeBox = closureScope.deepGet(othertype.typename().getText())
              if (othertypeBox === null) {
                console.error("Could not find return type " + othertype.getText() + " for function " + functionAst.VARNAME().getText())
                process.exit(-59)
              }
              if (othertypeBox.type !== Box.builtinTypes["type"]) {
                console.error("Function argument is not a valid type: " + othertype.getText())
                process.exit(-60)
              }
              let genericTypes = []
              for (const fulltypename of othertype.typegenerics().fulltypename()) {
                genericTypes.push(fulltypename.getText())
              }
              othertypeBox = new Box(othertypeBox.typeval.solidify(genericTypes, closureScope))
            } else {
              console.error("Could not find return type " + othertype.getText() + " for function " + functionAst.VARNAME().getText())
              process.exit(-51)
            }
          }
          unionTypes.push(othertypeBox.typeval)
        }
        returnType = new Type(functionAst.argtype().getText(), false, unionTypes)
      }
    } else {
      // TODO: Infer the return type by finding the return value and tracing backwards
      returnType = Box.builtinTypes["void"]
    }
    let pure = true
    let statements = []
    const functionbody = functionAst.fullfunctionbody().functionbody()
    if (functionbody !== null) {
      const statementsAst = functionbody.statements()
      for (const statementAst of statementsAst) {
        let statement = Statement.create(statementAst, closureScope)
        if (!statement.pure) pure = false
        statements.push(statement)
      }
    } else {
      const assignablesAst = functionAst.fullfunctionbody().assignables()
      let statement = Statement.create(assignablesAst, closureScope)
      if (!statement.pure) pure = false
      statements.push(statement)
      // TODO: Infer the return type for anything other than calls of other functions
      if (assignablesAst.basicassignables() && assignablesAst.basicassignables().calls()) {
        const fnCall = closureScope.deepGet(assignablesAst.basicassignables().calls().varn(0))
        if (fnCall && fnCall.functionval) {
          // TODO: For now, also take the first matching function name, in the future
          // figure out the argument types provided recursively to select appropriately
          // similar to how the Microstatements piece works
          returnType = fnCall.functionval[0].getReturnType()
        }
      }
    }
    return new UserFunction(name, args, returnType, closureScope, statements, pure)
  }

  getName() {
    return this.name
  }
  getArguments() {
    return this.args
  }
  getReturnType() {
    return this.returnType
  }
  isNary() {
    return false // TODO: support `...rest` in the future
  }
  isPure() {
    return this.pure
  }

  toFnStr() {
    if (
      this.statements.length === 1 &&
      this.statements[0].statementOrAssignableAst instanceof LnParser.AssignablesContext
    ) {
      return `
        fn ${this.name || ''} (${Object.keys(this.args).map(argName => `${argName}: ${this.args[argName].typename}`).join(', ')}): ${this.returnType.typename} = ${this.statements[0].statementOrAssignableAst.getText()}
      `.trim()
    }
    return `
      fn ${this.name || ''} (${Object.keys(this.args).map(argName => `${argName}: ${this.args[argName].typename}`).join(', ')}): ${this.returnType.typename} {
        ${this.statements.map(s => s.statementOrAssignableAst.getText()).join('\n')}
      }
    `.trim()
  }

  static conditionalToCond(cond, scope) {
    let newStatements = []
    const condName = "_" + uuid().replace(/-/g, "_")
    const condStatement = Ast.statementAstFromString(`
      const ${condName}: bool = ${cond.withoperators().getText()}
    `.trim() + '\n')
    const condBlock = (cond.blocklikes(0).functionbody() ?
      UserFunction.fromFunctionbodyAst(cond.blocklikes(0).functionbody(), scope) :
      cond.blocklikes(0).varn() ?
        scope.deepGet(cond.blocklikes(0).varn()).functionval[0] :
        UserFunction.fromFunctionsAst(cond.blocklikes(0).functions(), scope)
    ).maybeTransform().toFnStr()
    const condCall = Ast.statementAstFromString(`
      cond(${condName}, ${condBlock})
    `.trim() + '\n') // TODO: If the blocklike is a reference, grab it and inline it
    newStatements.push(condStatement, condCall)
    if (!!cond.ELSE()) {
      if (!!cond.blocklikes(1)) {
        const elseBlock = (cond.blocklikes(1).functionbody() ?
          UserFunction.fromFunctionbodyAst(cond.blocklikes(1).functionbody(), scope) :
          cond.blocklikes(1).varn() ?
            scope.deepGet(cond.blocklikes(1).varn()).functionval[0] :
            UserFunction.fromFunctionsAst(cond.blocklikes(1).functions(), scope)
        ).maybeTransform().toFnStr()
        const elseStatement = Ast.statementAstFromString(`
          cond(!${condName}, ${elseBlock})
        `.trim() + '\n')
        newStatements.push(elseStatement)
      } else {
        const innerCondStatements = UserFunction.conditionalToCond(cond.conditionals(), scope)
        const elseStatement = Ast.statementAstFromString(`
          cond(!${condName}, fn {
            ${innerCondStatements.map(s => s.getText()).join('\n')}
          })
        `.trim() + '\n')
        newStatements.push(elseStatement)
      }
    }
    return newStatements
  }

  maybeTransform() {
    if (this.statements.some(s => s.isConditionalStatement())) {
      // First pass, convert conditionals to `cond` fn calls
      let statementAsts = []
      for (let i = 0; i < this.statements.length; i++) {
        const s = this.statements[i]
        if (s.isConditionalStatement()) {
          const cond = s.statementOrAssignableAst.conditionals()
          const newStatements = UserFunction.conditionalToCond(cond, this.closureScope)
          statementAsts.push(...newStatements)
        } else {
          statementAsts.push(s.statementOrAssignableAst)
        }
      }

      const fnStr = `
        fn ${this.name || ''} (${Object.keys(this.args).map(argName => `${argName}: ${this.args[argName].typename}`)}): ${this.returnType.typename} {
          ${statementAsts.map(s => s.getText()).join('\n')}
        }
      `.trim()
      return UserFunction.fromAst(Ast.functionAstFromString(fnStr), this.closureScope)
    }
    return this
  }

  microstatementInlining(realArgNames, scope, microstatements) {
    // Perform a transform, if necessary, before generating the microstatements
    const fn = this.maybeTransform()
    // Resolve circular dependency issue
    const Microstatement = require('./Microstatement')
    const internalNames = Object.keys(fn.args)
    for (let i = 0; i < internalNames.length; i++) {
      const realArgName = realArgNames[i]
      // Instead of copying the relevant data, define a reference to where the data is located with
      // an alias for the function's expected variable name so statements referencing the argument
      // can be rewritten to use the new variable name.
      microstatements.push(new Microstatement(
        StatementType.REREF,
        scope,
        true,
        realArgName,
        internalNames[i],
        fn.args[internalNames[i]],
        [],
        [],
      ))
    }
    for (const s of fn.statements) {
      Microstatement.fromStatement(s, microstatements)
    }
  }

  static dispatchFn(fns, argumentTypeList, scope) {
    let fn = null;
    for (let i = 0; i < fns.length; i++) {
      const isNary = fns[i].isNary()
      const args = fns[i].getArguments()
      const argList = Object.values(args)
      if (!isNary && argList.length != argumentTypeList.length) continue
      if (isNary && argList.length > argumentTypeList.length) continue
      let skip = false
      for (let j = 0; j < argList.length; j++) {
        if (argList[j].typename === argumentTypeList[j].typename) continue
        if (
          argList[j].iface != null &&
          argList[j].iface.typeApplies(argumentTypeList[j], scope)
        ) continue
        if (argList[j].generics.length > 0 && argumentTypeList[j].originalType == argList[j]) {
          continue
        }
        if (
          argList[j].originalType != null &&
          argumentTypeList[j].originalType == argList[j].originalType
        ) {
          for (const propKey in argList[j].properties) {
            const propVal = argList[j].properties[propKey]
            if (
              propVal ==
              argumentTypeList[j].properties[propKey]
            ) continue
            if (
              propVal.iface != null &&
              propVal.iface.typeApplies(
                argumentTypeList[j].properties[propKey],
                scope
              )
            ) continue
            skip = true
          }
          continue
        }
        if (argList[j].unionTypes != null) {
          let unionSkip = true
          for (const unionType of argList[j].unionTypes) {
            // TODO: support other union types
            if (unionType.typename === argumentTypeList[j].typename) {
              unionSkip = false
              break
            }
          }
          if (!unionSkip) continue
        }
        skip = true
      }
      if (skip) continue
      fn = fns[i]
    }
    if (fn == null) {
      console.error("Unable to find matching function for name and argument type set")
      let argTypes = []
      for (let i = 0; i < argumentTypeList.length; i++) {
        argTypes.push("<" + argumentTypeList[i].typename + ">")
      }
      console.error(fns[0].getName() + "(" + argTypes.join(", ") + ")")
      process.exit(-40)
    }
    return fn
  }
}

module.exports = UserFunction
