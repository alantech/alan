const Box = require('./Box')
const { LnParser, } = require('../ln')

// Only implements the pieces necessary for the first stage compiler
class Statement {
  constructor(statementOrAssignableAst, scope, pure) {
    this.statementOrAssignableAst = statementOrAssignableAst,
    this.scope = scope
    this.pure = pure
  }

  isConditionalStatement() {
    return this.statementOrAssignableAst instanceof LnParser.StatementsContext &&
      this.statementOrAssignableAst.conditionals() !== null
  }

  isReturnStatement() {
    return this.statementOrAssignableAst instanceof LnParser.AssignablesContext ||
      this.statementOrAssignableAst.exits() !== null
  }

  static isCallPure(callAst, scope) {
    // TODO: Add purity checking for chained method-style calls
    const functionBox = scope.deepGet(callAst.varn(0))
    if (functionBox == null) {
      // TODO: This function may be defined in the execution scope, we won't know until runtime
      // right now, but it should be determinable at "compile time". Need to fix this to check
      // if prior statements defined it, for now, just assume it exists and is not pure
      return false
    }
    if (functionBox.type !== Box.builtinTypes["function"]) {
      console.error(callAst.varn(0).getText() + " is not a function")
      process.exit(-17)
    }
    // TODO: Add all of the logic to determine which function to use in here, too. For now,
    // let's just assume they all have the same purity state, which is a terrible assumption, but
    // easier.
    if (!functionBox.functionval[0].isPure()) return false
    const assignableListAst = callAst.fncall(0).assignablelist()
    if (assignableListAst == null) { // No arguments to this function call
      return true
    }
    for (const assignable of assignableListAst.assignables()) {
      if (Statement.isAssignablePure(assignable, scope) === false) return false
    }
    return true
  }

  static isWithOperatorsPure(withOperatorsAst, scope) {
    for (const operatorOrAssignable of withOperatorsAst.operatororassignable()) {
      if (operatorOrAssignable.operators() != null) {
        const operator = operatorOrAssignable.operators()
        const op = scope.deepGet(operator.getText())
        if (op == null || op.operatorval == null) {
          console.error("Operator " + operator.getText() + " is not defined")
          process.exit(-33)
        }
        // TODO: Similar to the above, need to figure out logic to determine which particular function
        // will be the one called. For now, just assume the first one and fix this later.
        if (!op.operatorval[0].potentialFunctions[0].isPure()) return false
      }
      if (operatorOrAssignable.basicassignables() != null) {
        if (!Statement.isBasicAssignablePure(operatorOrAssignable.basicassignables(), scope)) {
          return false
        }
      }
    }
    
    return true
  }

  static isBasicAssignablePure(basicAssignable, scope) {
    if (basicAssignable.functions() != null) {
      // Defining a function in itself is a pure situation
      return true
    }
    if (basicAssignable.calls() != null) {
      return Statement.isCallPure(basicAssignable.calls(), scope)
    }
    if (basicAssignable.varn() != null) {
      // This would be a read-only operation to pull a value into local scope
      return true
    }
    if (basicAssignable.constants() != null) {
      // This is an explicit constant that cannot impact any outer scope
      return true
    }
    if (basicAssignable.groups() != null) {
      // This is a "group" (parens surrounding one or more operators and operands)
      return Statement.isWithOperatorsPure(basicAssignable.groups().withoperators(), scope)
    }
    // Shouldn't be reached
    return false
  }

  static isAssignablePure(assignableAst, scope) {
    if (assignableAst.basicassignables() != null) {
      return Statement.isBasicAssignablePure(assignableAst.basicassignables(), scope)
    }
    if (assignableAst.withoperators() != null) {
      return Statement.isWithOperatorsPure(assignableAst.withoperators(), scope)
    }
    // This should never be reached
    console.error("Impossible assignment situation")
    process.exit(-14)
  }

  static create(statementOrAssignableAst, scope) {
    if (statementOrAssignableAst instanceof LnParser.AssignablesContext) {
      const pure = Statement.isAssignablePure(statementOrAssignableAst, scope)
      return new Statement(statementOrAssignableAst, scope, pure)
    } else if (statementOrAssignableAst instanceof LnParser.StatementsContext) {
      const statementAst = statementOrAssignableAst
      let pure = true
      if (statementAst.declarations() != null) {
        if (statementAst.declarations().constdeclaration() != null) {
          pure = Statement.isAssignablePure(
            statementAst.declarations().constdeclaration().assignments().assignables(),
            scope
          )
        } else if (statementAst.declarations().letdeclaration() != null) {
          if (statementAst.declarations().letdeclaration().assignments() != null) {
            if (statementAst.declarations().letdeclaration().assignments().assignables() == null) {
              pure = true
            } else {
              pure = Statement.isAssignablePure(
                statementAst.declarations().letdeclaration().assignments().assignables(),
                scope
              )
            }
          }
        } else {
          console.error("Bad assignment somehow reached")
          process.exit(-18)
        }
      }
      if (statementAst.assignments() != null) {
        if (statementAst.assignments().assignables() != null) {
          pure = Statement.isAssignablePure(statementAst.assignments().assignables(), scope)
        }
      }
      if (statementAst.calls() != null) {
        pure = Statement.isCallPure(statementAst.calls(), scope)
      }
      if (statementAst.exits() != null) {
        if (statementAst.exits().assignables() != null) {
          pure = Statement.isAssignablePure(statementAst.exits().assignables(), scope)
        }
      }
      if (statementAst.emits() != null) {
        if (statementAst.emits().assignables() != null) {
          pure = Statement.isAssignablePure(statementAst.emits().assignables(), scope)
        }
      }
      return new Statement(statementAst, scope, pure)
    } else {
      // What?
      console.error("This should not be possible")
      process.exit(-19)
    }
  }

  toString() {
    return statementOrAssignableAst.getText()
  }
}

module.exports = Statement
