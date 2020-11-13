import { v4 as uuid, } from 'uuid'

import * as Ast from './Ast'
import Microstatement from './Microstatement'
import Scope from './Scope'
import Statement from './Statement'
import StatementType from './StatementType'
import Type from './Type'
import { Args, Fn, } from './Function'
import { LnParser, } from '../ln'

class UserFunction implements Fn {
  name: string
  args: Args
  returnType: Type
  scope: Scope
  statements: Array<Statement>
  pure: boolean

  constructor(
    name: string,
    args: Args,
    returnType: Type,
    scope: Scope,
    statements: Array<Statement>,
    pure: boolean
  ) {
    this.name = name
    this.args = args
    this.returnType = returnType
    this.scope = scope
    for (let i = 0; i < statements.length - 1; i++) {
      if (statements[i].isReturnStatement()) {
        // There are unreachable statements after this line, abort
        throw new Error(`Unreachable code in function '${name}' after:
${statements[i].statementAst.getText().trim()} on line ${statements[i].statementAst.start.line}:${statements[i].statementAst.start.column}`)
      }
    }
    this.statements = statements
    this.pure = pure
  }

  static fromAst(functionishAst: any, scope: Scope) { // TODO: Eliminate ANTLR
    if (functionishAst instanceof LnParser.BlocklikesContext) {
      if (functionishAst.functions() != null) {
        return UserFunction.fromFunctionsAst(functionishAst.functions(), scope)
      }
      if (functionishAst.functionbody() != null) {
        return UserFunction.fromFunctionbodyAst(functionishAst.functionbody(), scope)
      }
    }
    if (functionishAst instanceof LnParser.FunctionsContext) {
      return UserFunction.fromFunctionsAst(functionishAst, scope)
    }
    if (functionishAst instanceof LnParser.FunctionbodyContext) {
      return UserFunction.fromFunctionbodyAst(functionishAst, scope)
    }
    return null
  }

  static fromFunctionbodyAst(functionbodyAst: any, scope: Scope) { // TODO: Eliminate ANTLR
    let args = {}
    const returnType = Type.builtinTypes.void
    let pure = true // Assume purity and then downgrade if needed
    let statements = []
    const statementsAst = functionbodyAst.statements()
    for (const statementAst of statementsAst) {
      const statement = Statement.create(statementAst, scope)
      if (!statement.pure) pure = false
      statements.push(statement)
    }
    return new UserFunction(null, args, returnType, scope, statements, pure)
  }

  static fromFunctionsAst(functionAst: any, scope: Scope) { // TODO: Eliminate ANTLR
    const name = functionAst.VARNAME() == null ? null : functionAst.VARNAME().getText()
    let args = {}
    const argsAst = functionAst.arglist()
    if (argsAst !== null) {
      const arglen = argsAst.VARNAME().length
      for (let i = 0; i < arglen; i++) {
        const argName = argsAst.VARNAME(i).getText()
        let getArgType = scope.deepGet(argsAst.argtype(i).getText())
        if (!getArgType) {
          if (argsAst.argtype(i).othertype().length === 1) {
            if (argsAst.argtype(i).othertype(0).typegenerics() !== null) {
              getArgType =
                scope.deepGet(argsAst.argtype(i).othertype(0).typename().getText()) as Type
              if (!getArgType) {
                throw new Error("Could not find type " + argsAst.argtype(i).getText() + " for argument " + argName)
              }
              if (!(getArgType instanceof Type)) {
                throw new Error("Function argument is not a valid type: " + argsAst.argtype(i).getText())
              }
              let genericTypes = []
              for (const fulltypename of argsAst.argtype(i).othertype(0).typegenerics().fulltypename()) {
                genericTypes.push(fulltypename.getText())
              }
              getArgType = getArgType.solidify(genericTypes, scope)
            } else {
              throw new Error("Could not find type " + argsAst.argtype(i).getText() + " for argument " + argName)
            }
          }
        }
        if (!(getArgType instanceof Type)) {
          throw new Error("Function argument is not a valid type: " + argsAst.argtype(i).getText())
        }
        args[argName] = getArgType
      }
    }
    let returnType = null
    if (functionAst.argtype() !== null) {
      if (functionAst.argtype().othertype().length === 1) {
        let getReturnType = scope.deepGet(functionAst.argtype().getText())
        if (getReturnType == null || !(getReturnType instanceof Type)) {
          if (functionAst.argtype().othertype(0).typegenerics() != null) {
            getReturnType = scope.deepGet(functionAst.argtype().othertype(0).typename().getText())
            if (getReturnType == null) {
              throw new Error("Could not find type " + functionAst.argtype().getText() + " for function " + functionAst.VARNAME().getText())
            }
            if (!(getReturnType instanceof Type)) {
              throw new Error("Function return is not a valid type: " + functionAst.argtype().getText())
            }
            let genericTypes = []
            for (const fulltypename of functionAst.argtype().othertype(0).typegenerics().fulltypename()) {
              genericTypes.push(fulltypename.getText())
            }
            getReturnType = getReturnType.solidify(genericTypes, scope)
          } else {
            throw new Error("Could not find type " + functionAst.argtype().getText() + " for function " + functionAst.VARNAME().getText())
          }
        }
        returnType = getReturnType
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
        let statement = Statement.create(statementAst, scope)
        if (!statement.pure) pure = false
        statements.push(statement)
      }
    } else {
      const assignablesAst = functionAst.fullfunctionbody().assignables()
      const statementAst = Ast.statementAstFromString(`return ${assignablesAst.getText()}\n`)
      const statement = Statement.create(statementAst, scope)
      if (!statement.pure) pure = false
      statements.push(statement)
      // TODO: Infer the return type for anything other than calls or object literals
      if (assignablesAst.basicassignables() && assignablesAst.basicassignables().calls()) {
        const fnCall =
          scope.deepGet(assignablesAst.basicassignables().calls().varn(0).getText()) as Array<Fn>
        if (
          fnCall &&
          fnCall instanceof Array &&
          fnCall[0].microstatementInlining instanceof Function
        ) {
          // TODO: For now, also take the first matching function name, in the future
          // figure out the argument types provided recursively to select appropriately
          // similar to how the Microstatements piece works
          returnType = (fnCall[0] as Fn).getReturnType()
        }
      } else if (
        assignablesAst.basicassignables() &&
        assignablesAst.basicassignables().objectliterals()
      ) {
        if (assignablesAst.basicassignables().objectliterals().typeliteral()) {
          returnType = scope.deepGet(
            assignablesAst.basicassignables().objectliterals().typeliteral().othertype().getText().trim()
          )
          if (!returnType) {
            const fulltypeAst = Ast.fulltypenameAstFromString(
              assignablesAst.basicassignables().objectliterals().typeliteral().othertype().getText()
            )
            const baseType = scope.deepGet(fulltypeAst.varn().getText()) as Type
            if (!baseType) {
              throw new Error(`Return type ${baseType} not defined`)
            }
            returnType = baseType.solidify(
              fulltypeAst.typegenerics().fulltypename().map((f: any) => f.getText()),
              scope
            )
          }
        } else if (assignablesAst.basicassignables().objectliterals().mapliteral()) {
          returnType = scope.deepGet(
            assignablesAst.basicassignables().objectliterals().mapliteral().othertype().getText().trim()
          )
          if (!returnType) {
            const fulltypeAst = Ast.fulltypenameAstFromString(
              assignablesAst.basicassignables().objectliterals().mapliteral().othertype().getText()
            )
            const baseType = scope.deepGet(fulltypeAst.varn().getText()) as Type
            if (!baseType) {
              throw new Error(`Return type ${baseType} not defined`)
            }
            returnType = baseType.solidify(
              fulltypeAst.typegenerics().fulltypename().map((f: any) => f.getText()),
              scope
            )
          }
        } else if (functionAst.argtype()) {
          returnType = scope.deepGet(functionAst.argType().getText().trim())
        } else {
          if (assignablesAst.basicassignables().objectliterals().arrayliteral().othertype()) {
            returnType = scope.deepGet(
              assignablesAst.basicassignables().objectliterals().arrayliteral().othertype().getText().trim()
            )
            if (!returnType) {
              const fulltypeAst = Ast.fulltypenameAstFromString(
                assignablesAst.basicassignables().objectliterals().mapliteral().othertype().getText()
              )
              const baseType = scope.deepGet(fulltypeAst.varn().getText()) as Type
              if (!baseType) {
                throw new Error(`Return type ${baseType} not defined`)
              }
              returnType = baseType.solidify(
                fulltypeAst.typegenerics().fulltypename().map((f: any) => f.getText()),
                scope
              )
            }
          } else {
            // We're going to use the Microstatement logic here
            const microstatements = []
            Microstatement.fromAssignablesAst(
              assignablesAst.basicassignables().objectliterals().arrayliteral().assignableslist(0),
              scope,
              microstatements,
            )
            returnType = microstatements[microstatements.length - 1].outputType
          }
        }
      }
    }
    return new UserFunction(name, args, returnType, scope, statements, pure)
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
  isPure() {
    return this.pure
  }

  toFnStr() {
    return `
      fn ${this.name || ''} (${Object.keys(this.args).map(argName => `${argName}: ${this.args[argName].typename}`).join(', ')}): ${this.returnType.typename} {
        ${this.statements.map(s => s.statementAst.getText()).join('\n')}
      }
    `.trim()
  }

  static conditionalToCond(cond: any, scope: Scope) { // TODO: Eliminate ANTLR
    let newStatements: Array<any> = []
    let hasConditionalReturn = false // Flag for potential second pass
    const condName = "_" + uuid().replace(/-/g, "_")
    const condStatement = Ast.statementAstFromString(`
      const ${condName}: bool = ${cond.withoperators().getText()}
    `.trim() + '\n')
    const condBlockFn = (cond.blocklikes(0).functionbody() ?
      UserFunction.fromFunctionbodyAst(cond.blocklikes(0).functionbody(), scope) :
      cond.blocklikes(0).varn() ?
        scope.deepGet(cond.blocklikes(0).varn().getText())[0] :
        UserFunction.fromFunctionsAst(cond.blocklikes(0).functions(), scope)
    ).maybeTransform(new Map())
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
            scope.deepGet(cond.blocklikes(1).varn().getText())[0] :
            UserFunction.fromFunctionsAst(cond.blocklikes(1).functions(), scope)
        ).maybeTransform(new Map())
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
        const innerCondStatements = res[0] as Array<any>
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

  static earlyReturnRewrite(
    retVal: string,
    retNotSet: string,
    statements: Array<any>, // TODO: Eliminate ANTLR
    scope: Scope
  ) {
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
            const innerStatements = blockFn.statements.map(s => s.statementAst)
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
          } else {
            replacementStatements.push(s)
          }
        } else {
          replacementStatements.push(s)
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
          ${retVal} = ref(${retStatement.exits().assignables().getText()})
        `.trim() + '\n')
        replacementStatements.push(newAssign)
      }
      replacementStatements.push(Ast.statementAstFromString(`
        ${retNotSet} = clone(false)
      `.trim() + '\n'))
    }
    return replacementStatements
  }

  maybeTransform(interfaceMap: Map<Type, Type>) {
    if (
      this.statements.some(s => s.isConditionalStatement()) ||
      this.statements.some(s => s.hasObjectLiteral())
    ) {
      // First pass, convert conditionals to `cond` fn calls and wrap assignment statements
      let statementAsts = []
      let hasConditionalReturn = false // Flag for potential second pass
      for (let i = 0; i < this.statements.length; i++) {
        let s = new Statement(
          this.statements[i].statementAst,
          this.statements[i].scope,
          this.statements[i].pure,
        )
        // Potentially rewrite the type for the object literal to match the interface type used by
        // a specific call
        const str = s.statementAst.getText()
        const corrected = str.replace(/new ([^<]+)<([^{\[]+)> *([{\[])/g, (
          _: any,
          basetypestr: string,
          genericstr: string,
          openstr: string,
        ) => {
          const originaltypestr = `${basetypestr.trim()}<${genericstr.trim()}>`
          let originalType = this.scope.deepGet(originaltypestr) as Type
          if (!originalType || !(originalType instanceof Type)) {
            // It may be the first time this particular type has shown up, let's build it
            const typeAst = Ast.fulltypenameAstFromString(originaltypestr)
            const baseTypeName = typeAst.varn().getText()
            const generics = typeAst.typegenerics().fulltypename().map((g: any) => g.getText())
            const baseType = this.scope.deepGet(baseTypeName) as Type
            if (!baseType || !(baseType instanceof Type)) { // Now we panic
              throw new Error('This should be impossible')
            }
            originalType = baseType.solidify(generics, this.scope)
          }
          const replacementType = originalType.realize(interfaceMap, this.scope)
          return `new ${replacementType.typename} ${openstr}`
        })
        const secondCorrection = corrected.replace(/: ([^:<]+)<([^{\)]+)>( *[,{\)])/g, (
          _: any,
          basetypestr: string,
          genericstr: string,
          openstr: string,
        ) => {
          const originaltypestr = `${basetypestr.trim()}<${genericstr.trim()}>`
          let originalType = this.scope.deepGet(originaltypestr) as Type
          if (!originalType || !(originalType instanceof Type)) {
            // It may be the first time this particular type has shown up, let's build it
            const typeAst = Ast.fulltypenameAstFromString(originaltypestr)
            const baseTypeName = typeAst.varn().getText()
            const generics = typeAst.typegenerics().fulltypename().map((g: any) => g.getText())
            const baseType = this.scope.deepGet(baseTypeName) as Type
            if (!baseType || !(baseType instanceof Type)) { // Now we panic
              throw new Error('This should be impossible')
            }
            originalType = baseType.solidify(generics, this.scope)
          }
          const replacementType = originalType.realize(interfaceMap, this.scope)
          return `: ${replacementType.typename}${openstr}`
        })
        const correctedAst = Ast.statementAstFromString(secondCorrection)
        s.statementAst = correctedAst
        // statementAsts.push(correctedAst)
        if (s.isConditionalStatement()) {
          const cond = s.statementAst.conditionals()
          const res  = UserFunction.conditionalToCond(cond, this.scope)
          const newStatements = res[0] as Array<any>
          if (res[1]) hasConditionalReturn = true
          statementAsts.push(...newStatements)
        } else if (s.statementAst instanceof LnParser.AssignmentsContext) {
          const a = s.statementAst
          const wrappedAst = Ast.statementAstFromString(`
            ${a.varn().getText()} = ref(${a.assignables().getText()})
          `.trim() + '\n')
          statementAsts.push(wrappedAst)
        } else if (s.statementAst instanceof LnParser.LetdeclarationContext) {
          const l = s.statementAst
          const name = l.VARNAME().getText()
          const type = l.othertype() ? l.othertype().getText() : undefined
          const v = l.assignables().getText()
          const wrappedAst = Ast.statementAstFromString(`
            let ${name}${type ? `: ${type}` : ''} = ref(${v})
          `.trim() + '\n')
          statementAsts.push(wrappedAst)
        } else {
          statementAsts.push(s.statementAst)
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
          let ${retVal}: ${this.returnType.typename} = clone()
        `.trim() + '\n')
        const retNotSetStatement = Ast.statementAstFromString(`
          let ${retNotSet}: bool = clone(true)
        `.trim() + '\n')
        let replacementStatements = [retValStatement, retNotSetStatement]
        replacementStatements.push(...UserFunction.earlyReturnRewrite(
          retVal, retNotSet, statementAsts, this.scope
        ))
        replacementStatements.push(Ast.statementAstFromString(`
          return ${retVal}
        `.trim() + '\n'))
        statementAsts = replacementStatements
      }

      const fnStr = `
        fn ${this.name || ''} (${Object.keys(this.args).map(argName => `${argName}: ${this.args[argName].typename}`).join(', ')}): ${this.returnType.typename} {
          ${statementAsts.map(s => s.getText()).join('\n')}
        }
      `.trim()
      const fn = UserFunction.fromAst(Ast.functionAstFromString(fnStr), this.scope)
      return fn
    }
    return this
  }

  microstatementInlining(
    realArgNames: Array<string>,
    scope: Scope,
    microstatements: Array<Microstatement>,
  ) {
    // Get the current statement length for usage in multiple cleanup routines
    const originalStatementLength = microstatements.length
    // First, check if there are any ENTERFN microstatements indicating a nested inlining, then
    // check that list for self-containment, which would cause an infinite loop in compilation and
    // abort with a useful error message.
    const enterfns = microstatements.filter(m => m.statementType === StatementType.ENTERFN)
    const isRecursive = enterfns.some(m => m.fns[0] === this)
    if (isRecursive) {
      let path = enterfns
        .slice(enterfns.findIndex(m => m.fns[0] === this))
        .map(m => m.fns[0].getName())
      path.push(this.getName())
      let pathstr = path.join(' -> ')
      throw new Error(`Recursive callstack detected: ${pathstr}. Aborting.`)
    } else {
      // Otherwise, add a marker for this
      microstatements.push(new Microstatement(
        StatementType.ENTERFN,
        scope,
        true,
        '',
        Type.builtinTypes.void,
        [],
        [this],
      ))
    }
    // Perform a transform, if necessary, before generating the microstatements
    // Resolve circular dependency issue
    const internalNames = Object.keys(this.args)
    const inputs = realArgNames.map(n => Microstatement.fromVarName(n, scope, microstatements))
    const inputTypes = inputs.map(i => i.outputType)
    const originalTypes = Object.values(this.getArguments())
    const interfaceMap: Map<Type, Type> = new Map()
    originalTypes.forEach((t, i) => t.typeApplies(inputTypes[i], scope, interfaceMap))
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
        inputTypes[i],
        [],
        [],
        internalNames[i],
      ))
    }
    const fn = this.maybeTransform(interfaceMap)
    for (const s of fn.statements) {
      Microstatement.fromStatement(s, microstatements, scope)
    }
    // Delete `REREF`s except a `return` statement's `REREF` to make sure it doesn't interfere with
    // the outer scope (if it has the same variable name defined, for instance)
    for (let i = originalStatementLength; i < microstatements.length - 1; i++) {
      if (microstatements[i].statementType == StatementType.REREF) {
        microstatements.splice(i, 1)
        i--
      }
    }
    // If the output return type is an interface or is a realized generic with an inner interface
    // type, figure out what its actual type is. This is assuming that any input type of the same
    // interface's real type is the same as the output type, which is a valid assumption as long as
    // all inputs of that particular interface are the same type. TODO: If this is not true, it must
    // be a compile-time error earlier on.
    const last = microstatements[microstatements.length - 1]
    if (!this.returnType.typeApplies(last.outputType, scope, new Map()))  {
      const returnTypeAst = Ast.fulltypenameAstFromString(this.returnType.typename)
      const returnTypeGenerics = returnTypeAst.typegenerics()
      const returnSubtypes = returnTypeGenerics ? returnTypeGenerics.fulltypename().map(
        (t: any) => scope.deepGet(t.getText())
      ) : []
      if (this.returnType.iface) {
        const originalArgTypes = Object.values(this.args)
        for (let i = 0; i < inputTypes.length; i++) {
          if (this.returnType === originalArgTypes[i]) {
            microstatements[microstatements.length - 1].outputType = inputTypes[i]
          }
        }
      } else if (returnSubtypes.some((t: Type) => !!t.iface)) {
        const oldReturnType = this.returnType
        const originalArgTypes = Object.values(this.args)
        for (let i = 0; i < inputTypes.length; i++) {
          for (let j = 0; j < returnSubtypes.length; j++) {
            if (returnSubtypes[j] === originalArgTypes[i]) {
              returnSubtypes[j] = inputTypes[i]
            }
          }
        }
        let newReturnType = oldReturnType.originalType.solidify(
          returnSubtypes.map((t: Type) => t.typename),
          scope
        )
        last.outputType = newReturnType
      } else {
        const lastTypeAst = Ast.fulltypenameAstFromString(last.outputType.typename)
        const lastTypeGenerics = lastTypeAst.typegenerics()
        const lastSubtypes = lastTypeGenerics ? lastTypeGenerics.fulltypename().map(
          (t: any) => scope.deepGet(t.getText()) || (scope.deepGet(t.varn().getText()) as Type).solidify(
            t.typegenerics().fulltypename().map((t: any) => t.getText()),
            scope
          )
        ) : []
        if (lastSubtypes.some((t: Type) => !!t.iface)) {
          const oldLastType = last.outputType
          const originalArgTypes = Object.values(this.args)
          for (let i = 0; i < inputTypes.length; i++) {
            for (let j = 0; j < lastSubtypes.length; j++) {
              if (lastSubtypes[j] === originalArgTypes[i]) {
                lastSubtypes[j] = inputTypes[i]
              }
            }
          }
          let newLastType = oldLastType.originalType.solidify(
            lastSubtypes.map((t: Type) => t.typename),
            scope
          )
          last.outputType = newLastType
        }
      }
    }
    // Now that we're done with this, we need to pop out all of the ENTERFN microstatements created
    // after this one so we don't mark non-recursive calls to a function multiple times as recursive
    // TODO: This is not the most efficient way to do things, come up with a better metadata
    // mechanism to pass around.
    for (let i = originalStatementLength; i < microstatements.length; i++) {
      if (microstatements[i].statementType === StatementType.ENTERFN) {
        microstatements.splice(i, 1)
        i--
      }
    }
  }

  static dispatchFn(
    fns: Array<Fn>,
    argumentTypeList: Array<Type>,
    scope: Scope
  ) {
    let fn = null
    for (let i = 0; i < fns.length; i++) {
      const args = fns[i].getArguments()
      const argList = Object.values(args)
      if (argList.length !== argumentTypeList.length) continue
      let skip = false
      for (let j = 0; j < argList.length; j++) {
        if (argList[j].typeApplies(argumentTypeList[j], scope)) continue
        skip = true
      }
      if (skip) continue
      fn = fns[i]
    }
    if (fn == null) {
      let errMsg = "Unable to find matching function for name and argument type set"
      let argTypes = []
      for (let i = 0; i < argumentTypeList.length; i++) {
        argTypes.push("<" + argumentTypeList[i].typename + ">")
      }
      errMsg += '\n' + fns[0].getName() + "(" + argTypes.join(", ") + ")"
      throw new Error(errMsg)
    }
    return fn
  }
}

export default UserFunction
