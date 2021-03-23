import { v4 as uuid, } from 'uuid'

import * as Ast from './Ast'
import Microstatement from './Microstatement'
import Scope from './Scope'
import Statement from './Statement'
import StatementType from './StatementType'
import Type from './Type'
import { Args, Fn, } from './Function'
import { LPNode, } from '../lp'
import * as Conditional from './Conditional'

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

  isUnwrapReturn: () => false

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

  maybeTransform(interfaceMap: Map<Type, Type>, scope?: Scope) {
    if (this.statements.some(s => s.hasObjectLiteral())) {
    //   // First pass, convert conditionals to `cond` fn calls and wrap assignment statements
      let statementAsts = []
    //   let hasConditionalReturn = false // Flag for potential second pass
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
          let newScope = this.scope
          if (scope !== undefined) {
            newScope = new Scope(scope)
            newScope.secondaryPar = this.scope
          }
          const originaltypestr = `${basetypestr.trim()}<${genericstr.trim()}>`
          let originalType = newScope.deepGet(originaltypestr) as Type
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
            const baseType = newScope.deepGet(baseTypeName) as Type
            if (!baseType || !(baseType instanceof Type)) { // Now we panic
              throw new Error('This should be impossible')
            }
            originalType = baseType.solidify(generics, newScope)
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
          let newScope = this.scope
          if (scope !== undefined) {
            newScope = new Scope(scope)
            newScope.secondaryPar = this.scope
          }
          const originaltypestr = `${basetypestr.trim()}<${genericstr.trim()}>`
          let originalType = newScope.deepGet(originaltypestr) as Type
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
            const baseType = newScope.deepGet(baseTypeName) as Type
            if (!baseType || !(baseType instanceof Type)) { // Now we panic
              throw new Error('This should be impossible')
            }
            originalType = baseType.solidify(generics, newScope)
          }
          const replacementType = originalType.realize(interfaceMap, newScope)
          return `: ${replacementType.typename}${openstr}`
        })
        const correctedAst = Ast.statementAstFromString(secondCorrection)
        s.statementAst = correctedAst
        // statementAsts.push(correctedAst)
        let newScope = this.scope
        if (scope !== undefined) {
          newScope = new Scope(scope)
          newScope.secondaryPar = this.scope
        }
        if (s.statementAst.has('assignments')) {
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
    s: Scope,
    microstatements: Array<Microstatement>,
  ) {
    const dbg = (msg: any) => this.getName() === 'reducePar' && console.log(msg);
    const scope = new Scope(s)
    scope.secondaryPar = this.scope
    // Perform a transform, if necessary, before generating the microstatements
    // Resolve circular dependency issue
    const internalNames = Object.keys(this.args)
    const inputs = realArgNames.map(n => Microstatement.fromVarName(n, scope, microstatements))
    const inputTypes = inputs.map(i => i.outputType)
    const originalTypes = Object.values(this.getArguments())
    const interfaceMap: Map<Type, Type> = new Map()
    originalTypes.forEach((t, i) => t.typeApplies(inputTypes[i], scope, interfaceMap))
    const fn = this.maybeTransform(interfaceMap, scope)
    // First, check that there are no ENTERFNS that contain a similar instance of the
    // transformed function, which would cause an infinite loop in compilation and
    // abort with a useful error message.
    // const recursiveProof = microstatements.findIndex(m => m.statementType === StatementType.ENTERFN && m.fns[0] == fn)
    // if (recursiveProof !== -1) {
    //   let path = [microstatements[recursiveProof].fns[0].getName()];
    //   path.push(fn.getName());
    //   const pathstr = path.join(' -> ');
    //   throw new Error(`Recursive callstack detected: ${pathstr}. Aborting.`);
    // }
    // Get the current statement length for usage in multiple cleanup routines
    // TODO: fix opcodes inserting mstmts and re-enable this
    const originalStatementLength = microstatements.length
    dbg(`original length: ${originalStatementLength}`)
    // add a marker for this function
    const enterfn = new Microstatement(
      StatementType.ENTERFN,
      scope,
      true,
      '',
      Type.builtinTypes.void,
      [],
      [fn],
      undefined,
      undefined,
      undefined,
      undefined,
      fn.getReturnType(),
    )
    microstatements.push(enterfn)
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
    for (const s of fn.statements) {
      // const before = microstatements[originalStatementLength];
      // const snapshot = [...microstatements];
      Microstatement.fromStatement(s, microstatements, scope)
      // TODO: opcodes that accept a closure (that aren't condfn) cause this
      // if statement to get evaluated...
      // if (microstatements[originalStatementLength] !== before) {
      //   console.log('------------------ VIOLATION');
      //   // console.log(s.statementAst.t.trim());
      //   // console.log(before)
      //   // console.log(microstatements[originalStatementLength])
      //   for (let ii = 0; ii < microstatements.length; ii++) {
      //     if (microstatements[ii] !== snapshot[ii]) {
      //       console.log(snapshot[ii])
      //       console.log(microstatements[ii])
      //       break;
      //     }
      //   }
      // }
    }
    // const originalStatementLength = microstatements.findIndex(m => m === enterfn);
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
    if (!fn.getReturnType().typeApplies(last.outputType, scope, new Map()))  {
      const returnTypeAst = Ast.fulltypenameAstFromString(fn.getReturnType().typename)
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
      if (fn.getReturnType().iface) {
        const originalArgTypes = Object.values(fn.args)
        for (let i = 0; i < inputTypes.length; i++) {
          if (fn.getReturnType() === originalArgTypes[i]) {
            microstatements[microstatements.length - 1].outputType = inputTypes[i]
          }
        }
      } else if (returnSubtypes.some((t: Type) => t.hasInterfaceType())) {
        const oldReturnType = fn.getReturnType()
        const originalArgTypes = Object.values(fn.args)
        for (let i = 0; i < inputTypes.length; i++) {
          for (let j = 0; j < returnSubtypes.length; j++) {
            if (returnSubtypes[j] === originalArgTypes[i]) {
              returnSubtypes[j] = inputTypes[i]
            }
          }
        }
        // Try to tackle issue with `main` and `alt` when they are in a function without branching
        if (returnSubtypes.some((t: Type) => !!t.iface)) {
          last.outputType = fn.getReturnType()
        } else {
          // We were able to piece together the right type info, let's use it
          let newReturnType = oldReturnType.originalType.solidify(
            returnSubtypes.map((t: Type) => t.typename),
            scope
          )
          last.outputType = newReturnType
        }
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
        if (lastSubtypes.some((t: Type) => t.hasInterfaceType())) {
          const oldLastType = last.outputType
          const originalArgTypes = Object.values(fn.args)
          for (let i = 0; i < inputTypes.length; i++) {
            for (let j = 0; j < lastSubtypes.length; j++) {
              if (lastSubtypes[j] === originalArgTypes[i]) {
                lastSubtypes[j] = inputTypes[i]
              }
            }
          }
          if (lastSubtypes.some((t: Type) => t.hasInterfaceType())) {
            // Just fall back to the user-provided type for now
            last.outputType = fn.getReturnType()
          } else {
            let newLastType = oldLastType.originalType.solidify(
              lastSubtypes.map((t: Type) => t.typename),
              scope
            )
            last.outputType = newLastType
          }
        }
      }
    }
    // If `last` is a REREF, we also need to potentially update the type on the original record
    for (let i = 0; i < microstatements.length; i++) {
      let m = microstatements[i]
      if (m.outputName === last.outputName && m.statementType !== StatementType.REREF) {
        m.outputType = last.outputType
        break
      }
    }
    // Now that we're done with this, we need to pop out all of the ENTERFN microstatements created
    // after fn one so we don't mark non-recursive calls to a function multiple times as recursive
    // TODO: This is not the most efficient way to do things, come up with a better metadata
    // mechanism to pass around.
    // note: keep the ENTERFN that we inserted, it's necessary for handling conditionals
    for (let i = originalStatementLength + 1; i < microstatements.length; i++) {
      if (microstatements[i].statementType === StatementType.ENTERFN) {
        microstatements.splice(i, 1);
        i--;
      }
    }

    const tailIdx = microstatements.slice(originalStatementLength).findIndex(ms => ms.statementType === StatementType.TAIL);
    if (tailIdx !== -1) {
      const tail = microstatements.splice(tailIdx);
      Conditional.handleTail(
        microstatements,
        tail,
        [],
      )
      // if fn isn't a closure, drop the return that got inserted at the end of handleTail
      if (originalStatementLength !== 0) {
        microstatements.pop();
      }
    }

    microstatements.splice(originalStatementLength, 1);
  }

  static dispatchFn(
    fns: Array<Fn>,
    argumentTypeList: Array<Type>,
    s: Scope
  ) {
    let fn = null
    for (let i = 0; i < fns.length; i++) {
      const scope = new Scope(s)
      scope.secondaryPar = (fns[i] as UserFunction).scope
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
