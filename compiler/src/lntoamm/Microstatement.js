const { v4: uuid, } = require('uuid')

const { LnParser, } = require('../ln')
const StatementType = require('./StatementType')
const Box = require('./Box')
const UserFunction = require('./UserFunction')

class Microstatement {
  constructor(...args) {
    if (args.length === 5) {
      // "Normal" microstatement
      this.statementType = args[0]
      this.scope = args[1]
      this.pure = args[2]
      this.outputName = args[3]
      this.alias = ""
      this.outputType = Box.builtinTypes.void
      this.inputNames = []
      this.fns = []
      this.closureStatements = args[4]
    } else if (args.length === 8) {
      // Aliasing microstatement (must be REREF)
      this.statementType = args[0]
      this.scope = args[1]
      this.pure = args[2]
      this.outputName = args[3]
      this.alias = args[4]
      this.outputType = args[5]
      this.inputNames = args[6]
      this.fns = args[7]
      this.closureStatements = []
    } else if (args.length === 7) {
      // Void-returning closure
      this.statementType = args[0]
      this.scope = args[1]
      this.pure = args[2]
      this.outputName = args[3]
      this.alias = ""
      this.outputType = args[4]
      this.inputNames = args[5]
      this.fns = args[6]
      this.closureStatements = []
    } else if (args.length === 6) {
      // Non-void returning closure
      this.statementType = args[0]
      this.scope = args[1]
      this.pure = args[2]
      this.outputName = args[3]
      this.alias = ""
      this.outputType = args[4]
      this.inputNames = []
      this.fns = []
      this.closureStatements = args[5]
    }
  }

  toString() {
    let outString = "";
    switch (this.statementType) {
      case StatementType.CONSTDEC:
        outString = "const " + this.outputName + ": " + this.outputType.typename
        if (this.fns.length > 0) {
          outString += " = " + this.fns[0].getName() + "(" + this.inputNames.join(", ") + ")"
        } else if (this.inputNames.length > 0) {
          outString += " = " + this.inputNames[0] // Doesn't appear the list is ever used here
        }
        break
      case StatementType.LETDEC:
        outString = "let " + this.outputName + ": " + this.outputType.typename
        if (this.fns.length > 0) {
          outString += " = " + this.fns[0].getName() + "(" + this.inputNames.join(", ") + ")"
        } else if (this.inputNames.length > 0) {
          outString += " = " + this.inputNames[0] // Doesn't appear the list is ever used here
        }
        break
      case StatementType.ASSIGNMENT:
        outString = this.outputName
        if (this.fns.length > 0) {
          outString += " = " + this.fns[0].getName() + "(" + this.inputNames.join(", ") + ")"
        } else if (this.inputNames.length > 0) {
          outString += " = " + this.inputNames[0] // Doesn't appear the list is ever used here
        }
        break
      case StatementType.CALL:
        if (this.fns.length > 0) {
          outString += this.fns[0].getName() + "(" + this.inputNames.join(", ") + ")"
        }
        break
      case StatementType.EMIT:
        outString = "emit " + this.outputName + " "
        if (this.fns.length > 0) {
          outString += this.fns[0].getName() + "(" + this.inputNames.join(", ") + ")"
        } else if (this.inputNames.length > 0) {
          outString += this.inputNames[0] // Doesn't appear the list is ever used here
        }
        break
      case StatementType.CLOSURE:
        outString = "const " + this.outputName + ": function = fn (): void {\n"
        for (const m of this.closureStatements) {
          const s = m.toString()
          if (s !== "") {
            outString += "    " + m.toString() + "\n"
          }
        }
        outString += "  }"
        break
      case StatementType.REREF:
      case StatementType.ARG:
        // Intentionally never output anything, this is metadata for the transpiler algo only
        break
    }
    return outString
  }

  static fromVarName(varName, microstatements) {
    let original = null
    for (let i = microstatements.length - 1; i > -1; i--) {
      const microstatement = microstatements[i]
      // TODO: var resolution is complex. Need to revisit this.
      if (microstatement.outputName === varName) {
        original = microstatement
        break
      }
      if (microstatement.alias === varName) {
        original = microstatement
        break
      }
    }
    return original
  }

  static fromVarAst(varAst, scope, microstatements) {
    let original = Microstatement.fromVarName(varAst.getText(), microstatements)
    if (original == null) {
      original = scope.deepGet(varAst.getText())
    }
    if (original == null || !(original instanceof Microstatement)) {
      console.error(varAst.getText() + " cannot be found")
      console.error(
        varAst.getText() +
        " on line " +
        varAst.start.line +
        ":" +
        varAst.start.column
      )
      process.exit(-104)
    }
    // When a variable is reassigned (or was referenced in a function call or operator statement,
    // instead of duplicating its data, add a microstatement to rereference that data (all of the
    // function and operator calls expect their arguments to be the N statements preceding them).
    microstatements.push(new Microstatement(
      StatementType.REREF,
      scope,
      true,
      original.outputName,
      original.outputType,
      [],
      [],
    ))
  }

  static fromConstantsAst(constantsAst, scope, microstatements) {
    const constName = "_" + uuid().replace(/-/g, "_")
    const constBox = Box.fromConstantsAst(
      constantsAst,
      scope,
      null,
      true
    )
    let constVal
    try {
      JSON.parse(constantsAst.getText()) // Will fail on strings with escape chars
      constVal = constantsAst.getText()
    } catch (e) {
      // Hackery to get these strings to work
      constVal = JSON.stringify(constantsAst.getText().replace(/^["']/, '').replace(/["']$/, ''))
    }
    microstatements.push(new Microstatement(
      StatementType.CONSTDEC,
      scope,
      true,
      constName,
      constBox.type,
      [constVal],
      [],
    ))
  }

  static fromBasicAssignablesAst(basicAssignablesAst, returnTypeHint, scope, microstatements) {
    // Functions will be inlined in a second pass over the microstatements whereever it is called.
    // For now we still create the function object and the microstatement to assign it
    if (basicAssignablesAst.functions() != null) {
      const fnToAssign = UserFunction.fromAst(basicAssignablesAst.functions(), scope)
      Microstatement.closureFromUserFunction(fnToAssign, scope, microstatements)
      return
    }
    // Here is where we inline the functions that were defined elsewhere or just above here! Or if
    // it's a built-in function, we just call it as originally expected.
    if (basicAssignablesAst.calls() != null) {
      Microstatement.fromCallsAst(
        basicAssignablesAst.calls(),
        scope,
        microstatements
      )
      return
    }
    // A `var` assignment is simply a renaming of a variable. We need to find the existing
    // microstatement for that `var` name and "tag it" in the scope as an alias that can be looked
    // up later. For now, we'll include a useless reassignment for simplicity's sake.
    if (basicAssignablesAst.varn() != null) {
      Microstatement.fromVarAst(
        basicAssignablesAst.varn(),
        scope,
        microstatements
      )
      return
    }
    // `constants` are relatively simple affair.
    if (basicAssignablesAst.constants() != null) {
      Microstatement.fromConstantsAst(basicAssignablesAst.constants(), scope, microstatements)
      return
    }
    // `groups` are just grouped `withOperators`.
    if (basicAssignablesAst.groups() != null) {
      Microstatement.fromWithOperatorsAst(
        basicAssignablesAst.groups().withoperators(),
        null,
        scope,
        microstatements
      )
      return
    }
    // `typeof` is a special statement to get the type from a variable. This is usually static but
    // can be dynamic in certain cases. That's going to be tough to represent in the bytecode that
    // otherwise strips all of the type data away. (Though with all functions inlined the type
    // "branches" can be serialized in some fashion back to bare types. Only event handlers and
    // event emission on ADTs, or opcodes that return ADTs remain a concern.)
    // TODO: For now, ignore this complexity and assume it can just be serialized.
    if (basicAssignablesAst.typeofn() != null) {
      // First evaluate the type's basicassignables.
      Microstatement.fromBasicAssignablesAst(
        basicAssignablesAst.typeofn().basicassignables(),
        null,
        scope,
        microstatements
      )
      // The last microstatement is the one we want to get the type data from.
      const last = microstatements[microstatements.size() - 1]
      const constName = "_" + uuid().replace(/-/g, "_")
      microstatements.push(new Microstatement(
        StatementType.CONSTDEC,
        scope,
        true,
        constName,
        Box.builtinTypes["string"],
        [last.outputType.typename],
        [],
      ))
      return
    }
    // The conversion of object literals is devolved to alangraphcode when types are erased, at this
    // stage they're just passed through as-is. TODO: This is assuming everything inside of them are
    // constants. That is not a valid assumption and should be revisited.
    if (basicAssignablesAst.objectliterals() != null) {
      const constName = "_" + uuid().replace(/-/g, "_")
      const typeBox = scope.deepGet(basicAssignablesAst.objectliterals().othertype().getText())
      if (typeBox == null) {
        console.error(basicAssignablesAst.objectliterals().othertype().getText() + " is not defined")
        console.error(
          basicAssignablesAst.getText() +
          " on line " +
          basicAssignablesAst.start.line +
          ":" +
          basicAssignablesAst.start.column
        )
        process.exit(-105)
      }
      if (typeBox.typeval == null) {
        console.error(basicAssignablesAst.objectliterals().othertype().getText() + " is not a type")
        console.error(
          basicAssignablesAst.getText() +
          " on line " +
          basicAssignablesAst.start.line +
          ":" +
          basicAssignablesAst.start.column
        )
        process.exit(-106)
      }
      microstatements.push(new Microstatement(
        StatementType.CONSTDEC,
        scope,
        true,
        constName,
        typeBox.typeval,
        [basicAssignablesAst.objectliterals().getText()],
        [],
      ))
      return
    }
  }

  static fromWithOperatorsAst(withOperatorsAst, returnTypeHint, scope, microstatements) {
    // Short circuit on the trivial case
    if (
      withOperatorsAst.operatororassignable().length === 1 &&
      !!withOperatorsAst.operatororassignable(1).basicassignables()
    ) {
      Microstatement.fromBasicAssignablesAst(
        withOperatorsAst.operatororassignable(1).basicassignables(),
        returnTypeHint,
        scope,
        microstatements,
      )
    }
    let withOperatorsList = []
    for (const operatorOrAssignable of withOperatorsAst.operatororassignable()) {
      if (operatorOrAssignable.operators() != null) {
        const operator = operatorOrAssignable.operators()
        const op = scope.deepGet(operator.getText())
        if (op == null || op.operatorval == null) {
          console.error("Operator " + operator.getText() + " is not defined")
          process.exit(-34)
        }
        withOperatorsList.push(op)
      }
      if (operatorOrAssignable.basicassignables() != null) {
        Microstatement.fromBasicAssignablesAst(
          operatorOrAssignable.basicassignables(),
          null,
          scope,
          microstatements
        )
        const last = microstatements[microstatements.length - 1]
        withOperatorsList.push(new Box(last)) // Wrapped in a box to make this work
      }
    }
    // Now to combine these operators and values in the correct order. A compiled language could
    // never do something so inefficient, but I don't care about performance right now, so here's
    // the algorithm: while the list length is greater than 1, perform the two steps:
    // 1. Find the operator with the greatest precedence
    // 2. Apply the underlying function to the values on either side of the operator (or just the
    //    right side if the operator is a prefix operator), then replace the operator with the
    //    returned value in the list and delete the impacted values.
    while (withOperatorsList.length > 1) {
      let maxPrecedence = -1
      let maxOperatorLoc = -1
      let maxOperatorListLoc = -1
      for (let i = 0; i < withOperatorsList.length; i++) {
        if (withOperatorsList[i].operatorval != null) {
          const ops = withOperatorsList[i].operatorval;
          let op = null
          let operatorListLoc = -1
          let operatorPrecedence = -127
          if (ops.length == 1) {
            op = ops[0]
            operatorListLoc = 0
          } else {
            // TODO: We need to identify which particular operator applies in this case.
            // We're just going to short-circuit this process on the first operator that matches
            // but we need to come up with a "best match" behavior (ie, if one argument is an int8
            // it may choose the int64-based operator because it was first and it can cast int8 to
            // int64 and then miss the specialized int8 version of the function).
            let left = null
            if (i != 0) left = withOperatorsList[i - 1]
            let right = null
            if (i != withOperatorsList.length - 1) right = withOperatorsList[i + 1]
            // Skip over any operator that is followed by another operator as it must be a prefix
            // operator (or a syntax error, but we'll catch that later)
            if (right === null || !!right.microstatementval) {
              for (let j = 0; j < ops.length; j++) {
                if (
                  ops[j].precedence > operatorPrecedence &&
                  ops[j].applicableFunction(
                    left === null ? // Left is special, if two operators are in a row, this one
                      null :        // needs to be a prefix operator for this to work at all
                      !!left.microstatementval ?
                        left.microstatementval.outputType :
                        null,
                    right === null ? null : right.microstatementval.outputType,
                    scope
                  ) != null
                ) {
                  op = ops[j]
                  operatorListLoc = j
                  operatorPrecedence = op.precedence
                }
              }
            }
            // During the process of determining the operator ordering, there may be tests that
            // will not match because operator precedence will convert the neighboring types into
            // types that will match. This is complicated and doing this statically will be more
            // difficult, but for now, just skip over these.
            if (op == null) continue
          }
          
          if (op.precedence > maxPrecedence) {
            maxPrecedence = op.precedence
            maxOperatorLoc = i
            maxOperatorListLoc = operatorListLoc
          }
        }
      }
      if (maxPrecedence == -1 || maxOperatorLoc == -1) {
        console.error("Cannot resolve operators with remaining statement")
        console.error(withOperatorsAst.getText())
        let withOperatorsTranslation = []
        for (let i = 0; i < withOperatorsList.length; i++) {
          const node = withOperatorsList[i]
          if (node.operatorval != null) {
            withOperatorsTranslation.push(node.operatorval[0].name)
          } else {
            withOperatorsTranslation.push("<" + node.microstatementval.outputType.typename + ">")
          }
        }
        console.error(withOperatorsTranslation.join(" "))
        process.exit(-34)
      }
      const op = withOperatorsList[maxOperatorLoc].operatorval[maxOperatorListLoc]
      let realArgNames = []
      let realArgTypes = []
      if (!op.isPrefix) {
        const left = withOperatorsList[maxOperatorLoc - 1].microstatementval
        realArgNames.push(left.outputName)
        realArgTypes.push(left.outputType)
      }
      const right = withOperatorsList[maxOperatorLoc + 1].microstatementval
      realArgNames.push(right.outputName)
      realArgTypes.push(right.outputType)
      UserFunction
        .dispatchFn(op.potentialFunctions, realArgTypes, scope)
        .microstatementInlining(realArgNames, scope, microstatements)
      const last = microstatements[microstatements.length - 1]
      withOperatorsList[maxOperatorLoc] = new Box(last)
      withOperatorsList.splice(maxOperatorLoc + 1, 1)
      if (!op.isPrefix) {
        withOperatorsList.splice(maxOperatorLoc - 1, 1)
      }
    }
  }

  static closureFromUserFunction(userFunction, scope, microstatements) {
    // TODO: Add support for closures with arguments
    let len = microstatements.length;
    for (const s of userFunction.statements) {
      if (s.statementOrAssignableAst instanceof LnParser.StatementsContext) {
        Microstatement.fromStatementsAst(s.statementOrAssignableAst, scope, microstatements)
      } else {
        Microstatemnt.fromAssignablesAst(s.statementOrAssignableAst, scope, microstatements)
      }
    }
    let newlen = microstatements.length;
    // There might be off-by-one bugs in the conversion here
    const innerMicrostatements = microstatements.slice(len, newlen)
    microstatements.splice(len, newlen - len)
    const constName = "_" + uuid().replace(/-/g, "_")
    microstatements.push(new Microstatement(
      StatementType.CLOSURE,
      scope,
      true, // TODO: Figure out if this is true or not
      constName,
      Box.builtinTypes['function'],
      innerMicrostatements
    ))
  }

  static closureFromBlocklikesAst(blocklikesAst, scope, microstatements) {
    // There are roughly two paths for closure generation of the blocklike. If it's a var reference
    // to another function, use the scope to grab the function definition directly, run the inlining
    // logic on it, then attach them to a new microstatement declaring the closure. If it's closure
    // that could (probably usually will) reference the outer scope, the inner statements should be
    // converted as normal, but with the current length of the microstatements array tracked so they
    // can be pruned back off of the list to be reattached to a closure microstatement type.
    const constName = "_" + uuid().replace(/-/g, "_")
    if (blocklikesAst.varn() != null) {
      const fnToClose = scope.deepGet(blocklikesAst.varn())
      if (fnToClose == null || fnToClose.functionval == null) {
        console.error(blocklikesAst.varn().getText() + " is not a function")
        process.exit(-111)
      }
      // TODO: Revisit this on resolving the appropriate function if multiple match, right now just
      // take the first one.
      const closureFn = fnToClose.functionval[0]
      let innerMicrostatements = []
      closureFn.microstatementInlining([], scope, innerMicrostatements)
      microstatements.push(new Microstatement(
        StatementType.CLOSURE,
        scope,
        true, // Guaranteed true in this case, it's not really a closure
        constName,
        innerMicrostatements
      ))
    } else {
      let len = microstatements.length;
      if (blocklikesAst.functionbody() != null) {
        for (const s of blocklikesAst.functionbody().statements()) {
          Microstatement.fromStatementsAst(s, scope, microstatements)
        }
      } else {
        if (blocklikesAst.functions().fullfunctionbody().functionbody() != null) {
          for (const s of blocklikesAst.functions().fullfunctionbody().functionbody().statements()) {
            Microstatement.fromStatementsAst(s, scope, microstatements)
          }
        } else {
          Microstatement.fromAssignablesAst(
            blocklikesAst.functions().fullfunctionbody().assignables(),
            scope,
            microstatements
          )
        }
      }
      let newlen = microstatements.length;
      // There might be off-by-one bugs in the conversion here
      const innerMicrostatements = microstatements.slice(len, newlen)
      microstatements.splice(len, newlen - len)
      microstatements.push(new Microstatement(
        StatementType.CLOSURE,
        scope,
        true, // Guaranteed true in this case, it's not really a closure
        constName,
        innerMicrostatements
      ))
    }
  }

  static fromEmitsAst(emitsAst, scope, microstatements) {
    if (emitsAst.assignables() != null) {
      // If there's an assignable value here, add it to the list of microstatements first, then
      // rewrite the final const assignment as the emit statement.
      Microstatement.fromAssignablesAst(emitsAst.assignables(), scope, microstatements)
      const eventBox = scope.deepGet(emitsAst.varn())
      if (eventBox.eventval == null) {
        console.error(emitsAst.varn().getText() + " is not an event!")
        console.error(
          emitsAst.getText() +
          " on line " +
          emitsAst.start.line +
          ":" +
          emitsAst.start.column
        )
        process.exit(-101)
      }
      const last = microstatements[microstatements.length - 1]
      if (
        last.outputType != eventBox.eventval.type &&
        !eventBox.eventval.type.castable(last.outputType)
      ) {
        console.error(
          "Attempting to assign a value of type " +
          last.outputType.typename +
          " to an event of type " +
          eventBox.eventval.type.typename
        )
        console.error(
          emitsAst.getText() +
          " on line " +
          emitsAst.start.line +
          ":" +
          emitsAst.start.column
        )
        process.exit(-103)
      }
      microstatements.push(new Microstatement(
        StatementType.EMIT,
        scope,
        true,
        eventBox.eventval.name,
        eventBox.eventval.type,
        [last.outputName],
        [],
      ))
    } else {
      // Otherwise, create an emit statement with no value
      const eventBox = scope.deepGet(emitsAst.varn());
      if (eventBox.eventval == null) {
        console.error(emitsAst.varn().getText() + " is not an event!")
        console.error(
          emitsAst.getText() +
          " on line " +
          emitsAst.start.line +
          ":" +
          emitsAst.start.column
        )
        process.exit(-102)
      }
      if (eventBox.eventval.type != Box.builtinTypes.void) {
        console.error(emitsAst.varn().getText() + " must have a value emitted to it!")
        console.error(
          emitsAst.getText() +
          " on line " +
          emitsAst.start.line +
          ":" +
          emitsAst.start.column
        )
        process.exit(-103)
      }
      microstatements.push(new Microstatement(
        StatementType.EMIT,
        scope,
        true,
        eventBox.eventval.name,
        Box.builtinTypes.void,
        [],
        [],
      ))
    }
  }

  static fromExitsAst(exitsAst, scope, microstatements) {
    // `alan--` doesn't have the concept of a `return` statement, the functions are all inlined
    // and the last assigned value for the function *is* the return statement
    if (exitsAst.assignables() != null) {
      // If there's an assignable value here, add it to the list of microstatements
      Microstatement.fromAssignablesAst(
        exitsAst.assignables(),
        scope,
        microstatements
      )
    } else {
      // Otherwise, create a microstatement with no value
      microstatements.push(new Microstatement(
        StatementType.CONSTDEC,
        scope,
        true,
        constName,
        scope.deepGet("void").typeval,
        ["void"],
        null
      ))
    }
  }

  static fromCallsAst(callsAst, scope, microstatements) {
    // Function call syntax also supports method chaining syntax, and you can chain off of any
    // assignable value (where they're wrapped in parens for clarity, with a special exception for
    // constants that would be unambiguous). This means there are three classes of function calls:
    // 1. Simple function call `fn(args)`
    // 2. Chained function calls `fn1(args1).fn2(args2)` (equivalent to `fn2(fn1(args1), args2)`)
    // 3. Chained data function calls `arg1.fn1(args2).fn2(args3)` (`fn2(fn1(arg1, args2), args3)`)
    // Four possible paths here:
    // 1. Only function calls potentially chained to each other.
    // 2. A constant is the first value, then function calls chained afterwards
    // 3. Any other value type wrapped in parens, then function calls chained afterwards.
    // 4. A variable name then a `.` then a function name. Both can contain dots, as well
    // These four can be taken care of in the following way: If it's 2 or 3, eval them and
    // generate the relevant microstatements. Take the last microstatement and store in a
    // `firstArg` variable. If it's the first case, `firstArg` is `null`. If it's the final case,
    // disambiguate it by iterating through the potential `varname.methodname` combinations and if
    // one is found, that gets set as the `firstArg`, but this is only done if its the first run
    // through the loops *and* no `firstArg` value already exists. After that, loop through the 
    // `var` and `fncall` entries, evaluating the arguments and adding to the `realArg*` lists
    // (and putting the `firstArg` value in the first entry of each if it exists). Then perform
    // the function `microstatementInlining` and take the last microstatement as the new
    // `firstArg` for the next loop iteration until all method calls have been taken care of.
    let firstArg = null
    if (callsAst.constants() != null) {
      Microstatement.fromConstantsAst(callsAst.constants(), scope, microstatements)
      firstArg = microstatements[microstatements.length - 1]
    }
    if (callsAst.assignables() != null) {
      Microstatement.fromAssignablesAst(callsAst.assignables(), scope, microstatements)
      firstArg = microstatements[microstatements.length - 1]
    }
    for (let i = 0; i < callsAst.varn().length; i++) {
      // First, resolve the function. TODO: Need to add support for closure functions defined in
      // the same function, which would not be in an outer scope passed in.
      let fnBox = scope.deepGet(callsAst.varn(i).getText())
      if (i == 0 && firstArg == null && fnBox == null) {
        // This may be a method-style access on something with nested scoping
        // TODO: Make this more robust in the future. Currently assuming the last ".something" is
        // the method call and everything before it is easily accessible through the scopes.
        const varSegs = callsAst.varn(0).getText().split(".")
        const methodName = varSegs[varSegs.length - 1]
        let scopePath = ""
        for (let j = 0; j < varSegs.length - 1; j++) {
          scopePath += varSegs[j] + "."
        }
        scopePath = scopePath.substring(0, scopePath.length - 1)
        firstArg = Microstatement.fromVarName(scopePath, microstatements)
        if (firstArg == null) { // It wasn't this, either, just return the same error
          console.error("Undefined function called: " + callsAst.varn(0).getText())
          process.exit(-140)
        }
        fnBox = scope.deepGet(methodName)
      }
      // Build up a list of the arguments to be passed into the function, first 'eval'ing them and
      // getting the relevant microstatements defined.
      let realArgNames = []
      let realArgTypes = []
      if (firstArg != null) {
        if (firstArg.alias !== "") {
          for (const m of microstatements) {
            if (m.outputName === firstArg.outputName && m.outputType.iface === null) {
              firstArg = m
              break
            }
          }
        } else if (firstArg.outputType.iface !== null) {
          for (const m of microstatements) {
            if (m.outputName === firstArg.outputName && m.outputType.iface === null) {
              firstArg = m
              break
            }
          }
        }
        realArgNames.push(firstArg.outputName)
        realArgTypes.push(firstArg.outputType)
      }
      if (callsAst.fncall(i).assignablelist() != null) {
        for (const assignablesAst of callsAst.fncall(i).assignablelist().assignables()) {
          Microstatement.fromAssignablesAst(assignablesAst, scope, microstatements)
          let last = microstatements[microstatements.length - 1]
          if (last.alias !== "" || last.outputType.iface !== null) {
            for (const m of microstatements) {
              if (m.outputName === last.outputName && m.outputType.iface === null) {
                last = m
                break
              }
            }
          }
          realArgNames.push(last.outputName)
          realArgTypes.push(last.outputType)
        }
      }
      // Do a scan of the microstatements for an inner defined closure that is being called.
      // TODO: What if they decided to shove this closure into an object but then use it directly?
      if (fnBox === null || !fnBox.functionval) {
        const fnName = callsAst.varn(i).getText()
        let actualFnName
        for (let i = microstatements.length - 1; i >= 0; i--) {
          if (microstatements[i].alias === fnName) {
            actualFnName = microstatements[i].outputName
            continue
          }
          if (
            microstatements[i].outputName === actualFnName &&
            microstatements[i].closureStatements &&
            microstatements[i].closureStatements.length > 0
          ) {
            microstatements.push(...microstatements[i].closureStatements)
            return
          }
        }
      }
      if (fnBox === null || !fnBox.functionval) {
        console.error(callsAst.varn(i).getText() + " is not a function!")
        console.error(
          callsAst.getText() +
          " on line " +
          callsAst.start.line +
          ":" +
          callsAst.start.column
        )
        process.exit(-106)
      }
      // Generate the relevant microstatements for this function. UserFunctions get inlined with the
      // return statement turned into a const assignment as the last statement, while built-in
      // functions are kept as function calls with the correct renaming.
      UserFunction
        .dispatchFn(fnBox.functionval, realArgTypes, scope)
        .microstatementInlining(realArgNames, scope, microstatements)
      firstArg = microstatements[microstatements.length - 1]
    }
  }

  static fromAssignmentsAst(assignmentsAst, scope, microstatements) {
    const letName = assignmentsAst.varn().getText()
    let letType = null
    let actualLetName
    for (let i = microstatements.length - 1; i >= 0; i--) {
      const microstatement = microstatements[i]
      if (microstatement.alias === letName) {
        actualLetName = microstatement.outputName
        continue
      }
      if (microstatement.outputName === actualLetName) {
        if (microstatement.statementType === StatementType.LETDEC) {
          letType = microstatement.outputType
          break
        } else {
          console.error("Attempting to reassign a non-let variable.")
          console.error(
            letName +
            " on line " +
            assignmentsAst.line +
            ":" +
            assignmentsAst.start.column
          )
          process.exit(100)
        }
      }
    }
    // TODO: Clean up the const/let declarations and assignments. That this is possible with the
    // parser is bad here, but necessary for let declarations because of the weird re-use of stuff.
    if (assignmentsAst.assignables() == null) {
      console.error("Let variable re-assignment without a value specified.")
      console.error(
        letName +
        " on line " +
        assignmentsAst.start.line +
        ":" +
        assignmentsAst.start.column
      )
      process.exit(101)
    }
    // An assignable may either be a basic constant or could be broken down into other microstatements
    // The classification with assignables is: if it's a `withoperators` type it *always* becomes
    // multiple microstatements and it should return the variable name it generated to store the data.
    // If it's a `basicassignables` type it could be either a "true constant" or generate multiple
    // microstatements. The types that fall under the "true constant" category are: functions,
    // var, and constants.
    if (assignmentsAst.assignables().withoperators() != null) {
      // Update the microstatements list with the operator serialization
      Microstatement.fromWithOperatorsAst(
        assignmentsAst.assignables().withoperators(),
        letType.typename,
        scope,
        microstatements
      )
      // By definition the last microstatement is the const assignment we care about, so we can just
      // mutate its object to rename the output variable name to the name we need instead.
      microstatements[microstatements.length - 1].outputName = actualLetName
      microstatements[microstatements.length - 1].statementType = StatementType.ASSIGNMENT
      return
    }
    if (assignmentsAst.assignables().basicassignables() != null) {
      Microstatement.fromBasicAssignablesAst(
        assignmentsAst.assignables().basicassignables(),
        letType.typename,
        scope,
        microstatements
      )
      // The same rule as above, the last microstatement is already a const assignment for the value
      // that we care about, so just rename its variable to the one that will be expected by other
      // code.
      microstatements[microstatements.length - 1].outputName = actualLetName
      microstatements[microstatements.length - 1].statementType = StatementType.ASSIGNMENT
      return
    }
  }

  static fromLetdeclarationAst(letdeclarationAst, scope, microstatements) {
    // TODO: Once we figure out how to handle re-assignment to let variables as new variable names
    // with all references to that variable afterwards rewritten, these can just be brought in as
    // constants, too.
    const letName = "_" + uuid().replace(/-/g, "_")
    let letAlias
    let letTypeHint = null
    if (letdeclarationAst.VARNAME() != null) {
      letAlias = letdeclarationAst.VARNAME().getText()
      letTypeHint = letdeclarationAst.assignments().varn().getText()
      if (letdeclarationAst.assignments().typegenerics() != null) {
        letTypeHint += letdeclarationAst.assignments().typegenerics().getText()
      }
    } else {
      letAlias = letdeclarationAst.assignments().varn().getText()
      // We don't know the type ahead of time and will have to rely on inference in this case
    }
    if (letdeclarationAst.assignments().assignables() == null) {
      // This is the situation where a variable is declared but no value is yet assigned. This can
      // be (and the interpreter currently behaves) as if this was a `type | void` situation, but
      // perhaps that should be required to be explicit?
      const blankLet = new Microstatement(
        StatementType.LETDEC,
        scope,
        true,
        letName,
        scope.deepGet("void").typeval,
        ["void"],
        null
      )
      // This is a terminating condition for the microstatements, though
      microstatements.push(blankLet)
      microstatements.push(new Microstatement(
        StatementType.REREF,
        scope,
        true,
        letName,
        letAlias,
        Box.builtinTypes.void,
        [],
        [],
      ))
      return
    }
    // An assignable may either be a basic constant or could be broken down into other microstatements
    // The classification with assignables is: if it's a `withoperators` type it *always* becomes
    // multiple microstatements and it should return the variable name it generated to store the data.
    // If it's a `basicassignables` type it could be either a "true constant" or generate multiple
    // microstatements. The types that fall under the "true constant" category are: functions,
    // var, and constants.
    if (letdeclarationAst.assignments().assignables().withoperators() != null) {
      // Update the microstatements list with the operator serialization
      Microstatement.fromWithOperatorsAst(
        letdeclarationAst.assignments().assignables().withoperators(),
        letTypeHint,
        scope,
        microstatements
      )
      // By definition the last microstatement is the const assignment we care about, so we can just
      // mutate its object to rename the output variable name to the name we need instead.
      microstatements[microstatements.length - 1].statementType = StatementType.LETDEC
      microstatements.push(new Microstatement(
        StatementType.REREF,
        scope,
        true,
        microstatements[microstatements.length - 1].outputName,
        letAlias,
        microstatements[microstatements.length - 1].outputType,
        [],
        [],
      ))
      return
    }
    if (letdeclarationAst.assignments().assignables().basicassignables() != null) {
      Microstatement.fromBasicAssignablesAst(
        letdeclarationAst.assignments().assignables().basicassignables(),
        letTypeHint,
        scope,
        microstatements
      )
      // The same rule as above, the last microstatement is already a const assignment for the value
      // that we care about, so just rename its variable to the one that will be expected by other
      // code.
      microstatements[microstatements.length - 1].statementType = StatementType.LETDEC
      microstatements.push(new Microstatement(
        StatementType.REREF,
        scope,
        true,
        microstatements[microstatements.length - 1].outputName,
        letAlias,
        microstatements[microstatements.length - 1].outputType,
        [],
        [],
      ))
      return
    }
  }

  static fromConstdeclarationAst(constdeclarationAst, scope, microstatements) {
    // TODO: Weirdness in the ANTLR grammar around declarations needs to be cleaned up at some point
    const constName = "_" + uuid().replace(/-/g, "_")
    let constAlias
    let constTypeHint = null
    if (constdeclarationAst.VARNAME() != null) {
      constAlias = constdeclarationAst.VARNAME().getText()
      constTypeHint = constdeclarationAst.assignments().varn().getText()
      if (constdeclarationAst.assignments().typegenerics() != null) {
        constTypeHint += constdeclarationAst.assignments().typegenerics().getText()
      }
    } else {
      constAlias = constdeclarationAst.assignments().varn().getText()
      // We don't know the type ahead of time and will have to refer on inference in this case
    }
    if (constdeclarationAst.assignments().assignables() == null) {
      // This is a weird edge case where a constant with no assignment was declared. Should this
      // even be legal?
      const weirdConst = new Microstatement(
        StatementType.CONSTDEC,
        scope,
        true,
        constName,
        Box.builtinTypes.void,
        ["void"],
        null
      )
      // This is a terminating condition for the microstatements, though
      microstatements.push(weirdConst)
      microstatements.push(new Microstatement(
        StatementType.REREF,
        scope,
        true,
        constName,
        constAlias,
        Box.builtinTypes.void,
        [],
        [],
      ))
      return
    }
    // An assignable may either be a basic constant or could be broken down into other microstatements
    // The classification with assignables is: if it's a `withoperators` type it *always* becomes
    // multiple microstatements and it should return the variable name it generated to store the data.
    // If it's a `basicassignables` type it could be either a "true constant" or generate multiple
    // microstatements. The types that fall under the "true constant" category are: functions,
    // var, and constants.
    if (constdeclarationAst.assignments().assignables().withoperators() != null) {
      // Update the microstatements list with the operator serialization
      Microstatement.fromWithOperatorsAst(
        constdeclarationAst.assignments().assignables().withoperators(),
        constTypeHint,
        scope,
        microstatements
      )
      // By definition the last microstatement is the const assignment we care about, so we can just
      // mutate its object to rename the output variable name to the name we need instead.
      microstatements.push(new Microstatement(
        StatementType.REREF,
        scope,
        true,
        microstatements[microstatements.length - 1].outputName,
        constAlias,
        microstatements[microstatements.length - 1].outputType,
        [],
        [],
      ))
      return
    }
    if (constdeclarationAst.assignments().assignables().basicassignables() != null) {
      Microstatement.fromBasicAssignablesAst(
        constdeclarationAst.assignments().assignables().basicassignables(),
        constTypeHint,
        scope,
        microstatements
      )
      // The same rule as above, the last microstatement is already a const assignment for the value
      // that we care about, so just rename its variable to the one that will be expected by other
      // code.
      microstatements.push(new Microstatement(
        StatementType.REREF,
        scope,
        true,
        microstatements[microstatements.length - 1].outputName,
        constAlias,
        microstatements[microstatements.length - 1].outputType,
        [],
        [],
      ))
      return
    }
  }

  // DFS recursive algo to get the microstatements in a valid ordering
  static fromStatementsAst(statementAst, scope, microstatements) {
    if (statementAst.declarations() != null) {
      if (statementAst.declarations().constdeclaration() != null) {
        Microstatement.fromConstdeclarationAst(
          statementAst.declarations().constdeclaration(),
          scope,
          microstatements
        )
      } else {
        Microstatement.fromLetdeclarationAst(
          statementAst.declarations().letdeclaration(),
          scope,
          microstatements
        )
      }
    }
    if (statementAst.assignments() != null) {
      Microstatement.fromAssignmentsAst(
        statementAst.assignments(),
        scope,
        microstatements
      )
    }
    if (statementAst.calls() != null) {
      Microstatement.fromCallsAst(
        statementAst.calls(),
        scope,
        microstatements
      )
    }
    if (statementAst.exits() != null) {
      Microstatement.fromExitsAst(
        statementAst.exits(),
        scope,
        microstatements
      )
    }
    if (statementAst.emits() != null) {
      Microstatement.fromEmitsAst(
        statementAst.emits(),
        scope,
        microstatements
      )
    }
    
    return microstatements
  }

  static fromAssignablesAst(assignablesAst, scope, microstatements) {
    if (assignablesAst.basicassignables() != null) {
      Microstatement.fromBasicAssignablesAst(
        assignablesAst.basicassignables(),
        null,
        scope,
        microstatements
      )
    } else {
      Microstatement.fromWithOperatorsAst(
        assignablesAst.withoperators(),
        null,
        scope,
        microstatements
      )
    }
  }

  static fromStatement(statement, microstatements) {
    if (statement.statementOrAssignableAst instanceof LnParser.StatementsContext) {
      Microstatement.fromStatementsAst(
        statement.statementOrAssignableAst,
        statement.scope,
        microstatements
      )
    } else {
      // Otherwise it's a one-liner function
      Microstatement.fromAssignablesAst(
        statement.statementOrAssignableAst,
        statement.scope,
        microstatements
      )
    }
  }
}

module.exports = Microstatement
