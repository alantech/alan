import { v4 as uuid, } from 'uuid'

import * as Ast from './Ast'
import Microstatement from './Microstatement'
import Scope from './Scope'
import Statement from './Statement'
import StatementType from './StatementType'
import Type from './Type'
import { Args, Fn, } from './Function'
import { LPNode, } from '../lp'

class UserFunction implements Fn {
  name: string
  args: Args
  returnType: Type | LPNode
  scope: Scope
  statements: Array<Statement>
  pure: boolean

  constructor(
    name: string,
    args: Args,
    returnType: Type | LPNode,
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
${statements[i].statementAst.t.trim()} on line ${statements[i].statementAst.line}:${statements[i].statementAst.char}`)
      }
    }
    this.statements = statements
    this.pure = pure
  }

  static fromAst(functionishAst: LPNode, scope: Scope) {
    if (
      functionishAst.has('fnname') ||
      functionishAst.has('functions') ||
      functionishAst.has('functionbody')
    ) { // It's a `blocklike` node
      if (functionishAst.has('functions')) {
        return UserFunction.fromFunctionsAst(functionishAst.get('functions'), scope)
      }
      if (functionishAst.has('functionbody')) {
        return UserFunction.fromFunctionbodyAst(functionishAst.get('functionbody'), scope)
      }
      if (functionishAst.has('fnname')) {
        // TODO: We didn't cover this path before?
      }
    }
    if (functionishAst.has('fn')) { // It's a `functions` node
      return UserFunction.fromFunctionsAst(functionishAst, scope)
    }
    if (functionishAst.has('openCurly')) { // It's a `functionbody` node
      return UserFunction.fromFunctionbodyAst(functionishAst, scope)
    }
    return null
  }

  static fromFunctionbodyAst(functionbodyAst: LPNode, scope: Scope) {
    let args = {}
    const returnType = Type.builtinTypes.void
    let pure = true // Assume purity and then downgrade if needed
    const statementsAst = functionbodyAst.get('statements')
    const statements = statementsAst.getAll().map(r => {
      const statement = Statement.create(r.get('statement'), scope)
      if (!statement.pure) pure = false
      return statement
    })
    return new UserFunction(null, args, returnType, scope, statements, pure)
  }

  static fromFunctionsAst(functionAst: LPNode, scope: Scope) {
    const name = functionAst.has('optname') ? functionAst.get('optname').t : null
    let args = {}
    if (functionAst.get('optargs').has('arglist')) {
      const argsAst = functionAst.get('optargs').get('arglist')
      const argsArr = []
      argsArr.push({
        variable: argsAst.get('variable').t,
        fulltypename: argsAst.get('fulltypename'),
      })
      argsAst.get('cdr').getAll().forEach(r => {
        argsArr.push({
          variable: r.get('variable').t,
          fulltypename: r.get('fulltypename'),
        })
      })
      for (let i = 0; i < argsArr.length; i++) {
        const argName = argsArr[i].variable
        let getArgType = scope.deepGet(argsArr[i].fulltypename.t)
        if (!getArgType) {
          if (argsArr[i].fulltypename.has('opttypegenerics')) {
            getArgType =
              scope.deepGet(argsArr[i].fulltypename.get('typename').t) as Type
            if (!getArgType) {
              throw new Error(
                "Could not find type " + argsArr[i].fulltypename.t + " for argument " + argName
              )
            }
            if (!(getArgType instanceof Type)) {
              throw new Error("Function argument is not a valid type: " + argsArr[i].fulltypename.t)
            }
            const genericTypes = []
            const genericAst = argsArr[i].fulltypename.get('opttypegenerics').get('generics')
            genericTypes.push(genericAst.get('fulltypename').t)
            genericAst.get('cdr').getAll().forEach(r => {
              genericTypes.push(r.get('fulltypename').t)
            })
            getArgType = getArgType.solidify(genericTypes, scope)
          } else {
            throw new Error(
              "Could not find type " + argsArr[i].fulltypename.t + " for argument " + argName
            )
          }
        }
        if (!(getArgType instanceof Type)) {
          throw new Error("Function argument is not a valid type: " + argsArr[i].fulltypename.t)
        }
        args[argName] = getArgType
      }
    }
    let pure = true
    let statements = []
    if (functionAst.get('fullfunctionbody').has('functionbody')) {
      const functionbody = functionAst.get('fullfunctionbody').get('functionbody')
      statements = functionbody.get('statements').getAll().map(r => {
        let statement = Statement.create(r.get('statement'), scope)
        if (!statement.pure) pure = false
        return statement
      })
    } else {
      const assignablesAst = functionAst
        .get('fullfunctionbody')
        .get('assignfunction')
        .get('assignables')
      const statementAst = Ast.statementAstFromString(`return ${assignablesAst.t};`)
      const statement = Statement.create(statementAst, scope)
      if (!statement.pure) pure = false
      statements.push(statement)
    }
    return new UserFunction(name, args, functionAst, scope, statements, pure)
  }

  getName() {
    return this.name
  }
  getArguments() {
    return this.args
  }
  generateReturnType() {
    const functionAst = this.returnType as LPNode // Abusing field to lazily load the return type
    let returnType = null
    let scope = this.scope
    let args = this.args
    if (functionAst.has('optreturntype')) {
      const fulltypename = functionAst.get('optreturntype').get('fulltypename')
      let getReturnType = scope.deepGet(fulltypename.t)
      if (getReturnType == null || !(getReturnType instanceof Type)) {
        if (fulltypename.has('opttypegenerics')) {
          getReturnType = scope.deepGet(fulltypename.get('typename').t)
          if (getReturnType == null) {
            throw new Error(
              "Could not find type " +
              fulltypename.t +
              " for function " +
              functionAst.get('optname').t
            )
          }
          if (!(getReturnType instanceof Type)) {
            throw new Error("Function return is not a valid type: " + fulltypename.t)
          }
          let genericTypes = []
          genericTypes.push(fulltypename.get('opttypegenerics').get('generics').get('fulltypename').t)
          fulltypename.get('opttypegenerics').get('generics').get('cdr').getAll().forEach(r => {
            genericTypes.push(r.get('fulltypename').t)
          })
          getReturnType = getReturnType.solidify(genericTypes, scope)
        } else {
          throw new Error(
            "Could not find type " +
            fulltypename.t +
            " for function " +
            functionAst.get('optname').t
          )
        }
      }
      returnType = getReturnType
    }
    if (functionAst.get('fullfunctionbody').has('functionbody')) {
      if (returnType === null) returnType = Type.builtinTypes['void']
    } else {
      const assignablesAst = functionAst
        .get('fullfunctionbody')
        .get('assignfunction')
        .get('assignables')
      if (!returnType && Object.keys(args).every(arg => args[arg].typename !== 'function')) {
        // We're going to use the Microstatement logic here
        const microstatements = []
        // First lets add all microstatements from the provided scope into the list
        // TODO: If this pattern is ever used more than once, add a new method to the Scope type
        Object.keys(scope.vals).forEach(val => {
          if (scope.vals[val] instanceof Microstatement) {
            microstatements.push(scope.vals[val])
          }
        })
        Object.keys(args).forEach(arg => {
          microstatements.push(new Microstatement(
            StatementType.REREF,
            scope,
            true,
            arg,
            args[arg],
            [],
            [],
            arg,
          ))
        })
        Microstatement.fromAssignablesAst(assignablesAst, scope, microstatements)
        const last = microstatements[microstatements.length - 1]
        if (last.statementType !== StatementType.EMIT) {
          // TODO: Come up with a better solution than this hackery for void function calls as the
          // only value for a one-liner function
          returnType = last.outputType
        } else {
          returnType = Type.builtinTypes.void
        }
      } else if (!returnType) {
        // TODO: Generalize this hackery for opcodes that take closure functions
        const opcodeName = assignablesAst.t.split('(')[0]
        const opcode = scope.deepGet(opcodeName) as Array<Fn>
        returnType = opcode ? opcode[0].getReturnType() : Type.builtinTypes['void']
      }
    }
    return returnType
  }

  getReturnType() {
    if (!(this.returnType instanceof Type)) {
      this.returnType = this.generateReturnType()
    }
    return this.returnType as Type
  }

  isPure() {
    return this.pure
  }

  toFnStr() {
    return `
      fn ${this.name || ''} (${Object.keys(this.args).map(argName => `${argName}: ${this.args[argName].typename}`).join(', ')}): ${this.getReturnType().typename} {
        ${this.statements.map(s => s.statementAst.t).join('\n')}
      }
    `.trim()
  }

  static conditionalToCond(cond: LPNode, scope: Scope) {
    let newStatements: Array<LPNode> = []
    let hasConditionalReturn = false // Flag for potential second pass
    const condName = "_" + uuid().replace(/-/g, "_")
    const condStatement = Ast.statementAstFromString(`
      const ${condName}: bool = ${cond.get('assignables').t}
    `.trim() + ';')
    const condBlockFn = (cond.get('blocklike').has('functionbody') ?
      UserFunction.fromFunctionbodyAst(cond.get('blocklike').get('functionbody'), scope) :
      cond.get('blocklike').has('fnname') ?
        // TODO: If more than one function matches, need to run multiple dispatch logic
        scope.deepGet(cond.get('blocklike').get('fnname').t)[0] :
        UserFunction.fromFunctionsAst(cond.get('blocklike').get('functions'), scope)
    ).maybeTransform(new Map())
    if (condBlockFn.statements[condBlockFn.statements.length - 1].isReturnStatement()) {
      hasConditionalReturn = true
    }
    const condBlock = condBlockFn.toFnStr()
    const condCall = Ast.statementAstFromString(`
      cond(${condName}, ${condBlock})
    `.trim() + ';') // TODO: If the blocklike is a reference, grab it and inline it
    newStatements.push(condStatement, condCall)
    if (cond.has('elsebranch')) {
      const notcond = cond.get('elsebranch')
      if (notcond.get('condorblock').has('blocklike')) {
        const notblock = notcond.get('condorblock').get('blocklike')
        const elseBlockFn = (notblock.has('functionbody') ?
          UserFunction.fromFunctionbodyAst(notblock.get('functionbody'), scope) :
          notblock.has('fnname') ?
            // TODO: If more than one function matches, need to run multiple dispatch logic
            scope.deepGet(notblock.get('fnname').t)[0] :
            UserFunction.fromFunctionsAst(notblock.get('functions'), scope)
        ).maybeTransform(new Map())
        if (elseBlockFn.statements[elseBlockFn.statements.length - 1].isReturnStatement()) {
          hasConditionalReturn = true
        }
        const elseBlock = elseBlockFn.toFnStr()
        const elseStatement = Ast.statementAstFromString(`
          cond(not(${condName}), ${elseBlock})
        `.trim() + ';')
        newStatements.push(elseStatement)
      } else {
        const res = UserFunction.conditionalToCond(
          notcond.get('condorblock').get('conditionals'),
          scope
        )
        const innerCondStatements = res[0] as Array<LPNode>
        if (res[1]) hasConditionalReturn = true
        const elseStatement = Ast.statementAstFromString(`
          cond(!${condName}, fn {
            ${innerCondStatements.map(s => s.t).join('\n')}
          })
        `.trim() + ';')
        newStatements.push(elseStatement)
      }
    }
    return [newStatements, hasConditionalReturn]
  }

  static earlyReturnRewrite(
    retVal: string,
    retNotSet: string,
    statements: Array<LPNode>,
    scope: Scope
  ) {
    let replacementStatements = []
    while (statements.length > 0) {
      const s = statements.shift()
      // TODO: This doesn't work for actual direct-usage of `cond` in some sort of method chaining
      // if that's even possible. Probably lots of other weirdness to deal with here.
      if (
        s.has('assignables') &&
        s
          .get('assignables')
          .get('assignables')
          .getAll()[0]
          .get('withoperators')
          .get('baseassignablelist')
          .getAll()
          .length >= 2 &&
        s
          .get('assignables')
          .get('assignables')
          .getAll()[0]
          .get('withoperators')
          .get('baseassignablelist')
          .getAll()[0]
          .t
          .trim() === 'cond' &&
        s
          .get('assignables')
          .get('assignables')
          .getAll()[0]
          .get('withoperators')
          .get('baseassignablelist')
          .getAll()[1]
          .get('baseassignable')
          .has('fncall')
      ) {
        // TODO: Really need to rewrite
        const argsAst = s
            .get('assignables')
            .get('assignables')
            .getAll()[0]
            .get('withoperators')
            .get('baseassignablelist')
            .getAll()[1]
            .get('baseassignable')
            .get('fncall')
            .get('assignablelist')
        const args = []
        if (argsAst.has('assignables')) {
          args.push(argsAst.get('assignables'))
          argsAst.get('cdr').getAll().forEach(r => {
            args.push(r.get('assignables'))
          })
        }
        if (args.length == 2) {
          const block = args[1]
            .getAll()[0]
            .get('withoperators')
            .has('baseassignablelist') ?
            args[1]
              .getAll()[0]
              .get('withoperators')
              .get('baseassignablelist')
              .getAll()[0]
              .get('baseassignable') :
            null
          if (block) {
            const blockFn = UserFunction.fromAst(block, scope)
            if (blockFn.statements[blockFn.statements.length - 1].isReturnStatement()) {
              const innerStatements = blockFn.statements.map(s => s.statementAst)
              const newBlockStatements = UserFunction.earlyReturnRewrite(
                retVal, retNotSet, innerStatements, scope
              )
              const cond = args[0].t.trim()
              const newBlock = Ast.statementAstFromString(`
                cond(${cond}, fn {
                  ${newBlockStatements.map(s => s.t).join('\n')}
                })
              `.trim() + ';')
              replacementStatements.push(newBlock)
              if (statements.length > 0) {
                const remainingStatements = UserFunction.earlyReturnRewrite(
                  retVal, retNotSet, statements, scope
                )
                const remainingBlock = Ast.statementAstFromString(`
                  cond(${retNotSet}, fn {
                    ${remainingStatements.map(s => s.t).join('\n')}
                  })
                `.trim() + ';')
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
      } else {
        replacementStatements.push(s)
      }
    }
    // If no inner conditional was found in this branch, check if there's a final return
    if (replacementStatements[replacementStatements.length - 1].has('exits')) {
      const retStatement = replacementStatements.pop()
      if (retStatement.get('exits').get('retval').has('assignables')) {
        const newAssign = Ast.statementAstFromString(`
          ${retVal} = ref(${retStatement.get('exits').get('retval').get('assignables').t})
        `.trim() + ';')
        replacementStatements.push(newAssign)
      }
      replacementStatements.push(Ast.statementAstFromString(`
        ${retNotSet} = clone(false)
      `.trim() + ';'))
    }
    return replacementStatements
  }

  maybeTransform(interfaceMap: Map<Type, Type>, scope?: Scope) {
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
        const str = s.statementAst.t
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
            const baseTypeName = typeAst.get('typename').t
            const generics = []
            if (typeAst.has('opttypegenerics')) {
              const genericsAst = typeAst.get('opttypegenerics').get('generics')
              generics.push(genericsAst.get('fulltypename').t)
              genericsAst.get('cdr').getAll().forEach(r => {
                generics.push(r.get('fulltypename').t)
              })
            }
            const baseType = this.scope.deepGet(baseTypeName) as Type
            if (!baseType || !(baseType instanceof Type)) { // Now we panic
              throw new Error('This should be impossible')
            }
            originalType = baseType.solidify(generics, this.scope)
          }
          let newScope = this.scope
          if (scope !== undefined) {
            newScope = new Scope(scope)
            newScope.secondaryPar = this.scope
          }
          const replacementType = originalType.realize(interfaceMap, newScope)
          return `new ${replacementType.typename} ${openstr}`
        })
        // TODO: Get rid of these regex-based type corrections
        const secondCorrection = corrected.replace(/: (?!new )([^:<,]+)<([^{\)]+)>( *[,{\)])/g, (
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
            const baseTypeName = typeAst.get('typename').t
            const generics = []
            if (typeAst.has('opttypegenerics')) {
              const genericsAst = typeAst.get('opttypegenerics').get('generics')
              generics.push(genericsAst.get('fulltypename').t)
              genericsAst.get('cdr').getAll().forEach(r => {
                generics.push(r.get('fulltypename').t)
              })
            }
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
          const cond = s.statementAst.get('conditionals')
          const res = UserFunction.conditionalToCond(cond, this.scope)
          const newStatements = res[0] as Array<LPNode>
          if (res[1]) hasConditionalReturn = true
          statementAsts.push(...newStatements)
        } else if (s.statementAst.has('assignments')) {
          const a = s.statementAst.get('assignments')
          const wrappedAst = Ast.statementAstFromString(`
            ${a.get('varn').t} = ref(${a.get('assignables').t})
          `.trim() + ';')
          statementAsts.push(wrappedAst)
        } else if (
          s.statementAst.has('declarations') &&
          s.statementAst.get('declarations').has('letdeclaration')
        ) {
          const l = s.statementAst.get('declarations').get('letdeclaration')
          const name = l.get('variable').t
          const type = l.has('typedec') ? l.get('typedec').get('fulltypename').t : undefined
          const v = l.get('assignables').t
          const wrappedAst = Ast.statementAstFromString(`
            let ${name}${type ? `: ${type}` : ''} = ref(${v})
          `.trim() + ';')
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
          let ${retVal}: ${this.getReturnType().typename} = clone()
        `.trim() + ';')
        const retNotSetStatement = Ast.statementAstFromString(`
          let ${retNotSet}: bool = clone(true)
        `.trim() + ';')
        let replacementStatements = [retValStatement, retNotSetStatement]
        replacementStatements.push(...UserFunction.earlyReturnRewrite(
          retVal, retNotSet, statementAsts, this.scope
        ))
        replacementStatements.push(Ast.statementAstFromString(`
          return ${retVal}
        `.trim() + ';'))
        statementAsts = replacementStatements
      }

      // TODO: Should these be attached to the scope or should callers provide a merged scope?
      const newArgs = {}
      for (const argName in this.args) {
        const a = this.args[argName]
        newArgs[argName] = interfaceMap.has(a) ? interfaceMap.get(a) : a
        this.scope.put(newArgs[argName].typename, newArgs[argName])
      }
      const newRet = interfaceMap.has(this.getReturnType()) ?
        interfaceMap.get(this.getReturnType()) : this.getReturnType()
      this.scope.put(newRet.typename, newRet)

      const fnStr = `
        fn ${this.name || ''} (${Object.keys(newArgs).map(argName => `${argName}: ${newArgs[argName].typename}`).join(', ')}): ${newRet.typename} {
          ${statementAsts.map(s => s.t).join('\n')}
        }
      `.trim()
      const fn = UserFunction.fromAst(Ast.functionAstFromString(fnStr), this.scope)
      return fn
    } else {
      let hasNewType = false
      const newArgs = {}
      for (const argName in this.args) {
        const a = this.args[argName]
        newArgs[argName] = interfaceMap.has(a) ? interfaceMap.get(a) : a
        if (newArgs[argName] !== this.args[argName]) {
          this.scope.put(newArgs[argName].typename, newArgs[argName])
          hasNewType = true
        }
      }
      const newRet = interfaceMap.has(this.getReturnType()) ?
        interfaceMap.get(this.getReturnType()) : this.getReturnType()
      if (newRet !== this.getReturnType()) {
        this.scope.put(newRet.typename, newRet)
        hasNewType = true
      }
      if (hasNewType) {
        const statementAsts = this.statements.map(s => s.statementAst)
        const fnStr = `
          fn ${this.name || ''} (${Object.keys(newArgs).map(argName => `${argName}: ${newArgs[argName].typename}`).join(', ')}): ${newRet.typename} {
            ${statementAsts.map(s => s.t).join('\n')}
          }
        `.trim()
        const fn = UserFunction.fromAst(Ast.functionAstFromString(fnStr), this.scope)
        return fn
      } else {
        return this
      }
    }
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
    const fn = this.maybeTransform(interfaceMap, scope)
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
    if (!this.getReturnType().typeApplies(last.outputType, scope, new Map()))  {
      const returnTypeAst = Ast.fulltypenameAstFromString(this.getReturnType().typename)
      let returnSubtypes = []
      if (returnTypeAst.has('opttypegenerics')) {
        const generics = returnTypeAst.get('opttypegenerics').get('generics')
        const returnSubtypeAsts = []
        returnSubtypeAsts.push(generics.get('fulltypename'))
        generics.get('cdr').getAll().forEach(r => {
          returnSubtypeAsts.push(r.get('fulltypename'))
        })
        returnSubtypes = returnSubtypeAsts.map(r => {
          let t = scope.deepGet(r.t)
          if (!t) {
            const innerGenerics = []
            if (r.has('opttypegenerics')) {
              innerGenerics.push(r.get('opttypegenerics').get('generics').get('fulltypename').t)
              r.get('opttypegenerics').get('generics').get('cdr').getAll().forEach(r2 => {
                innerGenerics.push(r2.t)
              })
            }
            t = (scope.deepGet(r.get('typename').t) as Type).solidify(innerGenerics, scope)
          }
          return t
        })
      }
      if (this.getReturnType().iface) {
        const originalArgTypes = Object.values(this.args)
        for (let i = 0; i < inputTypes.length; i++) {
          if (this.getReturnType() === originalArgTypes[i]) {
            microstatements[microstatements.length - 1].outputType = inputTypes[i]
          }
        }
      } else if (returnSubtypes.some((t: Type) => !!t.iface)) {
        const oldReturnType = this.getReturnType()
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
        const lastSubtypes = []
        if (lastTypeAst.has('opttypegenerics')) {
          const generics = lastTypeAst.get('opttypegenerics').get('generics')
          lastSubtypes.push(scope.deepGet(generics.get('fulltypename').t))
          generics.get('cdr').getAll().forEach(r => {
            lastSubtypes.push(scope.deepGet(r.get('fulltypename').t))
          })
        }
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
      errMsg += '\n' + fns[0].getName() + "(" + argTypes.join(", ") + ")\n"
      errMsg += 'Candidate functions considered:\n'
      for (let i = 0; i < fns.length; i++) {
        const fn = fns[i]
        if (fn instanceof UserFunction) {
          const fnStr = fn.toFnStr().split('{')[0]
          errMsg += `${fnStr}\n`
        } else {
          // TODO: Add this to the opcode definition, too?
          errMsg += `fn ${fn.getName()}(${Object.entries(fn.getArguments()).map(kv => `${kv[0]}: ${kv[1].typename}`)}): ${fn.getReturnType().typename}\n`
        }
      }
      throw new Error(errMsg)
    }
    return fn
  }
}

export default UserFunction
