import Operator from './Operator'
import Scope from './Scope'
import { Fn, } from './Function'
import { LnParser, } from '../ln'

// Only implements the pieces necessary for the first stage compiler
class Statement {
  statementAst: any // TODO: Migrate off ANTLR for better typing here
  scope: Scope
  pure: boolean

  constructor(statementAst: any, scope: Scope, pure: boolean) {
    this.statementAst = statementAst,
    this.scope = scope
    this.pure = pure
  }

  isConditionalStatement() {
    return this.statementAst.conditionals() !== null
  }

  isReturnStatement() {
    return this.statementAst.exits() !== null
  }

  static baseAssignableHasObjectLiteral(baseAssignableAst: any) { // TODO: Remove ANTLR
    if (baseAssignableAst.objectliterals()) return true
    return false
  }

  static assignablesHasObjectLiteral(assignablesAst: any) { // TODO: Remove ANTLR
    for (const ba of assignablesAst.baseassignables()) {
      if (Statement.baseAssignableHasObjectLiteral(ba)) return true
      if (!!ba.fncall() && !!ba.fncall().assignablelist()) {
        const innerAssignables = ba.fncall().assignablelist().assignables()
        for (const ia of innerAssignables) {
          if (Statement.assignablesHasObjectLiteral(ia)) return true
        }
      }
    }
    return false
  }

  static assignmentsHasObjectLiteral(assignmentsAst: any) { // TODO: Remove ANTLR
    return Statement.assignablesHasObjectLiteral(assignmentsAst.assignables())
  }

  hasObjectLiteral() {
    const s = this.statementAst
    if (s.declarations()) {
      const d = s.declarations().constdeclaration() || s.declarations().letdeclaration()
      return Statement.assignablesHasObjectLiteral(d.assignables())
    }
    if (s.assignments()) return Statement.assignmentsHasObjectLiteral(s.assignments())
    if (s.assignables()) return Statement.assignablesHasObjectLiteral(s.assignables())
    if (s.exits() && s.exits().assignables()) return Statement.assignablesHasObjectLiteral(
      s.exits().assignables()
    )
    if (s.emits() && s.emits().assignables()) return Statement.assignablesHasObjectLiteral(
      s.emits().assignables()
    )
    // TODO: Cover conditionals
    return false
  }

  static isCallPure(callAst: any, scope: Scope) { // TODO: Migrate off ANTLR
    // TODO: Add purity checking for chained method-style calls
    const fn = scope.deepGet(callAst.callbase(0).varn(0).getText()) as Array<Fn>
    if (!fn) {
      // TODO: This function may be defined in the execution scope, we won't know until runtime
      // right now, but it should be determinable at "compile time". Need to fix this to check
      // if prior statements defined it, for now, just assume it exists and is not pure
      return false
    }
    if (!(fn instanceof Array && fn[0].microstatementInlining instanceof Function)) {
      throw new Error(callAst.callbase(0).varn(0).getText() + " is not a function")
    }
    // TODO: Add all of the logic to determine which function to use in here, too. For now,
    // let's just assume they all have the same purity state, which is a terrible assumption, but
    // easier.
    if (!fn[0].isPure()) return false
    const assignableListAst = callAst.callbase(0).fncall(0).assignablelist()
    if (assignableListAst == null) { // No arguments to this function call
      return true
    }
    for (const assignable of assignableListAst.assignables()) {
      if (Statement.isAssignablePure(assignable, scope) === false) return false
    }
    return true
  }

  static isAssignablePure(assignableAst: any, scope: Scope) { // TODO: Migrate off ANTLR
    // TODO: Redo this
    return true
  }

  static create(statementAst: any, scope: Scope) { // TODO: Migrate off ANTLR
    if (!!statementAst.exception) {
      throw statementAst.exception
    }
    let pure = true
    if (statementAst.declarations() != null) {
      if (statementAst.declarations().constdeclaration() != null) {
        pure = Statement.isAssignablePure(
          statementAst.declarations().constdeclaration().assignables(),
          scope
        )
      } else if (statementAst.declarations().letdeclaration() != null) {
        if (statementAst.declarations().letdeclaration().assignables() == null) {
          pure = true
        } else {
          pure = Statement.isAssignablePure(
            statementAst.declarations().letdeclaration().assignables(),
            scope
          )
        }
      } else {
        throw new Error("Bad assignment somehow reached")
      }
    }
    if (statementAst.assignments() != null) {
      if (statementAst.assignments().assignables() != null) {
        pure = Statement.isAssignablePure(statementAst.assignments().assignables(), scope)
      }
    }
    if (statementAst.assignables() != null) {
      pure = Statement.isAssignablePure(statementAst.assignables(), scope)
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
  }

  toString() {
    return this.statementAst.getText()
  }
}

export default Statement
