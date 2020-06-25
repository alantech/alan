const { v4: uuid, } = require('uuid')

const Ast = require('./Ast')
const Statement = require('./Statement')
const StatementType = require('./StatementType')
const Type = require('./Type')
const { LnParser, } = require('../ln')

// This only implements the parts required for the compiler
class UserFunction {
  constructor(name, args, returnType, closureScope, statements, pure) {
    this.name = name
    this.args = args
    this.returnType = returnType
    this.closureScope = closureScope
    for (let i = 0; i < statements.length - 1; i++) {
      if (statements[i].isReturnStatement()) {
        // There are unreachable statements after this line, abort
        console.error(`Unreachable code in function '${name}' after:`)
        console.error(
          statements[i].statementOrAssignableAst.getText().trim() +
          " on line " +
          statements[i].statementOrAssignableAst.start.line +
          ":" +
          statements[i].statementOrAssignableAst.start.column
        )
        process.exit(-201)
      }
    }
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
    const returnType = Type.builtinTypes.void
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
              if (getArgType.type !== Type.builtinTypes["type"]) {
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
                  if (othertypeBox.type != Type.builtinTypes["type"]) {
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
        if (getArgType.type != Type.builtinTypes["type"]) {
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
        if (getReturnType == null || getReturnType.type != Type.builtinTypes["type"]) {
          if (functionAst.argtype().othertype(0).typegenerics() != null) {
            getReturnType = closureScope.deepGet(functionAst.argtype().othertype(0).typename().getText())
            if (getReturnType == null) {
              console.error("Could not find type " + functionAst.argtype().getText() + " for function " + functionAst.VARNAME().getText())
              process.exit(-59)
            }
            if (getReturnType.type !== Type.builtinTypes["type"]) {
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
              if (othertypeBox.type !== Type.builtinTypes["type"]) {
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
      returnType = Type.builtinTypes["void"]
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
    let hasConditionalReturn = false // Flag for potential second pass
    const condName = "_" + uuid().replace(/-/g, "_")
    const condStatement = Ast.statementAstFromString(`
      const ${condName}: bool = ${cond.withoperators().getText()}
    `.trim() + '\n')
    const condBlockFn = (cond.blocklikes(0).functionbody() ?
      UserFunction.fromFunctionbodyAst(cond.blocklikes(0).functionbody(), scope) :
      cond.blocklikes(0).varn() ?
        scope.deepGet(cond.blocklikes(0).varn()).functionval[0] :
        UserFunction.fromFunctionsAst(cond.blocklikes(0).functions(), scope)
    ).maybeTransform()
    if (condBlockFn.statements[condBlockFn.statements.length - 1].isReturnStatement()) {
      hasConditionalReturn = true
    }
    const condBlock = condBlockFn.toFnStr()
    const condCall = Ast.statementAstFromString(`
      cond(${condName}, ${condBlock})
    `.trim() + '\n') // TODO: If the blocklike is a reference, grab it and inline it
    newStatements.push(condStatement, condCall)
    if (!!cond.ELSE()) {
      if (!!cond.blocklikes(1)) {
        const elseBlockFn = (cond.blocklikes(1).functionbody() ?
          UserFunction.fromFunctionbodyAst(cond.blocklikes(1).functionbody(), scope) :
          cond.blocklikes(1).varn() ?
            scope.deepGet(cond.blocklikes(1).varn()).functionval[0] :
            UserFunction.fromFunctionsAst(cond.blocklikes(1).functions(), scope)
        ).maybeTransform()
        if (elseBlockFn.statements[elseBlockFn.statements.length - 1].isReturnStatement()) {
          hasConditionalReturn = true
        }
        const elseBlock = elseBlockFn.toFnStr()
        const elseStatement = Ast.statementAstFromString(`
          cond(!${condName}, ${elseBlock})
        `.trim() + '\n')
        newStatements.push(elseStatement)
      } else {
        const res = UserFunction.conditionalToCond(cond.conditionals(), scope)
        const innerCondStatements = res[0]
        if (res[1]) hasConditionalReturn = true
        const elseStatement = Ast.statementAstFromString(`
          cond(!${condName}, fn {
            ${innerCondStatements.map(s => s.getText()).join('\n')}
          })
        `.trim() + '\n')
        newStatements.push(elseStatement)
      }
    }
    return [newStatements, hasConditionalReturn]
  }

  static earlyReturnRewrite(retVal, retNotSet, statements, scope) {
    let replacementStatements = []
    while (statements.length > 0) {
      const s = statements.shift()
      if (s.calls() && s.calls().varn(0).getText().trim() === 'cond') {
        // Potentially need to rewrite
        const args = s.calls().fncall(0).assignablelist()
        if (args && args.assignables().length == 2) {
          const block = args.assignables(1).basicassignables().functions()
          const blockFn = UserFunction.fromAst(block, scope)
          if (blockFn.statements[blockFn.statements.length - 1].isReturnStatement()) {
            const innerStatements = blockFn.statements.map(s => s.statementOrAssignableAst)
            const newBlockStatements = UserFunction.earlyReturnRewrite(
              retVal, retNotSet, innerStatements, scope
            )
            const cond = args.assignables(0).getText().trim()
            const newBlock = Ast.statementAstFromString(`
              cond(${cond}, fn {
                ${newBlockStatements.map(s => s.getText()).join('\n')}
              })
            `.trim() + '\n')
            replacementStatements.push(newBlock)
            if (statements.length > 0) {
              const remainingStatements = UserFunction.earlyReturnRewrite(
                retVal, retNotSet, statements, scope
              )
              const remainingBlock = Ast.statementAstFromString(`
                cond(${retNotSet}, fn {
                  ${remainingStatements.map(s => s.getText()).join('\n')}
                })
              `.trim() + '\n')
              replacementStatements.push(remainingBlock)
            }
          }
        }
      } else {
        replacementStatements.push(s)
      }
    }
    // If no inner conditional was found in this branch, check if there's a final return
    if (replacementStatements[replacementStatements.length - 1].exits()) {
      const retStatement = replacementStatements.pop()
      if (retStatement.exits().assignables()) {
        const newAssign = Ast.statementAstFromString(`
          ${retVal} = assign(${retStatement.exits().assignables().getText()})
        `.trim() + '\n')
        replacementStatements.push(newAssign)
      }
      replacementStatements.push(Ast.statementAstFromString(`
        ${retNotSet} = assign(false)
      `.trim() + '\n'))
    }
    return replacementStatements
  }

  maybeTransform() {
    if (this.statements.some(s => s.isConditionalStatement())) {
      // First pass, convert conditionals to `cond` fn calls and wrap assignment statements
      let statementAsts = []
      let hasConditionalReturn = false // Flag for potential second pass
      for (let i = 0; i < this.statements.length; i++) {
        const s = this.statements[i]
        if (s.isConditionalStatement()) {
          const cond = s.statementOrAssignableAst.conditionals()
          const res  = UserFunction.conditionalToCond(cond, this.closureScope)
          const newStatements = res[0]
          if (res[1]) hasConditionalReturn = true
          statementAsts.push(...newStatements)
        } else if (s.statementOrAssignableAst instanceof LnParser.AssignmentsContext) {
          // TODO: Clean up the const/let/assignment grammar mistakes.
          const a = s.statementOrAssignableAst
          if (a.assignables()) {
            const wrappedAst = Ast.statementAstFromString(`
              ${a.varn().getText()} = assign(${a.assignables().getText()})
            `.trim() + '\n')
            statementAsts.push(wrappedAst)
          } else {
            statementAsts.push(s.statementOrAssignableAst)
          }
        } else if (s.statementOrAssignableAst instanceof LnParser.LetdeclarationContext) {
          const l = s.statementOrAssignableAst
          // TODO: More cleanup of const/let/assignment here, too
          let name = ""
          let type = undefined
          if (l.VARNAME()) {
            name = l.VARNAME().getText()
            type = l.assignments().varn().getText()
            if (l.assignments().typegenerics()) {
              type += l.assignments().typegenerics().getText()
            }
          } else {
            name = l.assignments().varn().getText()
          }
          if (l.assignments().assignables()) {
            const v = l.assignments().assignables().getText()
            const wrappedAst = Ast.statementAstFromString(`
              let ${name}${type ? `: ${type}` : ''} = assign(${v})
            `.trim() + '\n')
            statementAsts.push(wrappedAst)
          } else {
            statementAsts.push(s.statementOrAssignableAst)
          }
        } else {
          statementAsts.push(s.statementOrAssignableAst)
        }
      }
      // Second pass, there was a conditional return, mutate everything *again* so the return is
      // instead hoisted into writing a closure variable
      if (hasConditionalReturn) {
        // Need the UUID to make sure this is unique if there's multiple layers of nested returns
        const retNamePostfix = "_" + uuid().replace(/-/g, "_")
        const retVal = "retVal" + retNamePostfix
        const retNotSet = "retNotSet" + retNamePostfix
        const retValStatement = Ast.statementAstFromString(`
          let ${retVal}: ${this.returnType.typename}
        `.trim() + '\n')
        const retNotSetStatement = Ast.statementAstFromString(`
          let ${retNotSet}: bool = assign(true)
        `.trim() + '\n')
        let replacementStatements = [retValStatement, retNotSetStatement]
        replacementStatements.push(...UserFunction.earlyReturnRewrite(
          retVal, retNotSet, statementAsts, this.closureScope
        ))
        replacementStatements.push(Ast.statementAstFromString(`
          return ${retVal}
        `.trim() + '\n'))
        statementAsts = replacementStatements
      }

      const fnStr = `
        fn ${this.name || ''} (${Object.keys(this.args).map(argName => `${argName}: ${this.args[argName].typename}`)}): ${this.returnType.typename} {
          ${statementAsts.map(s => s.getText()).join('\n')}
        }
      `.trim()
      const fn = UserFunction.fromAst(Ast.functionAstFromString(fnStr), this.closureScope)
      return fn
    }
    return this
  }

  microstatementInlining(realArgNames, scope, microstatements) {
    // Perform a transform, if necessary, before generating the microstatements
    const fn = this.maybeTransform()
    // Resolve circular dependency issue
    const Microstatement = require('./Microstatement')
    const internalNames = Object.keys(fn.args)
    const originalStatementLength = microstatements.length
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
    // Delete `REREF`s except a `return` statement's `REREF` to make sure it doesn't interfere with
    // the outer scope (if it has the same variable name defined, for instance)
    for (let i = originalStatementLength; i < microstatements.length - 1; i++) {
      if (microstatements[i].statementType == StatementType.REREF) {
        microstatements.splice(i, 1)
        i--
      }
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
