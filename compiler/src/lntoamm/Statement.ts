import Operator from './Operator';
import Scope from './Scope';
import { Fn } from './Function';
import { LPNode } from '../lp';

// Only implements the pieces necessary for the first stage compiler
class Statement {
  statementAst: LPNode;
  scope: Scope;
  pure: boolean;

  constructor(statementAst: LPNode, scope: Scope, pure: boolean) {
    (this.statementAst = statementAst), (this.scope = scope);
    this.pure = pure;
  }

  isConditionalStatement() {
    return this.statementAst.has('conditionals');
  }

  isReturnStatement() {
    return this.statementAst.has('exits');
  }

  static baseAssignableHasObjectLiteral(baseAssignableAst: LPNode) {
    return baseAssignableAst.has('objectliterals');
  }

  static assignablesHasObjectLiteral(assignablesAst: LPNode) {
    for (const w of assignablesAst.getAll()) {
      const wo = w.get('withoperators');
      if (wo.has('operators')) continue;
      for (const b of wo.get('baseassignablelist').getAll()) {
        const ba = b.get('baseassignable');
        if (Statement.baseAssignableHasObjectLiteral(ba)) return true;
        if (ba.has('fncall') && ba.get('fncall').has('assignablelist')) {
          const innerAssignables = [];
          innerAssignables.push(
            ba.get('fncall').get('assignablelist').get('assignables'),
          );
          ba.get('fncall')
            .get('assignablelist')
            .get('cdr')
            .getAll()
            .map((a) => {
              innerAssignables.push(a.get('assignables'));
            });
          for (const ia of innerAssignables) {
            if (Statement.assignablesHasObjectLiteral(ia)) return true;
          }
        }
      }
    }
    return false;
  }

  static assignmentsHasObjectLiteral(assignmentsAst: LPNode) {
    return Statement.assignablesHasObjectLiteral(
      assignmentsAst.get('assignables'),
    );
  }

  hasObjectLiteral() {
    const s = this.statementAst;
    if (s.has('declarations')) {
      const d = s.get('declarations').has('constdeclaration')
        ? s.get('declarations').get('constdeclaration')
        : s.get('declarations').get('letdeclaration');
      return Statement.assignablesHasObjectLiteral(d.get('assignables'));
    }
    if (s.has('assignments'))
      return Statement.assignmentsHasObjectLiteral(s.get('assignments'));
    if (s.has('assignables'))
      return Statement.assignablesHasObjectLiteral(s.get('assignables'));
    if (s.has('exits') && s.get('exits').get('retval').has('assignables')) {
      return Statement.assignablesHasObjectLiteral(
        s.get('exits').get('retval').get('assignables'),
      );
    }
    if (s.has('emits') && s.get('emits').get('retval').has('assignables')) {
      return Statement.assignablesHasObjectLiteral(
        s.get('emits').get('retval').get('assignables'),
      );
    }
    // TODO: Cover conditionals
    return false;
  }

  static isAssignablePure(assignableAst: LPNode, scope: Scope) {
    // TODO: Redo this
    return true;
  }

  static create(statementAst: LPNode | Error, scope: Scope) {
    if (statementAst instanceof Error) throw statementAst;
    let pure = true;
    if (statementAst.has('declarations')) {
      if (statementAst.get('declarations').has('constdeclaration')) {
        pure = Statement.isAssignablePure(
          statementAst
            .get('declarations')
            .get('constdeclaration')
            .get('assignables'),
          scope,
        );
      } else if (statementAst.get('declarations').has('letdeclaration')) {
        pure = Statement.isAssignablePure(
          statementAst
            .get('declarations')
            .get('letdeclaration')
            .get('assignables'),
          scope,
        );
      } else {
        throw new Error(
          'Malformed AST. Invalid const/let declaration structure',
        );
      }
    }
    if (statementAst.has('assignments')) {
      if (statementAst.get('assignments').has('assignables')) {
        pure = Statement.isAssignablePure(
          statementAst.get('assignments').get('assignables'),
          scope,
        );
      }
    }
    if (statementAst.has('assignables')) {
      pure = Statement.isAssignablePure(
        statementAst.get('assignables').get('assignables'),
        scope,
      );
    }
    if (statementAst.has('exits')) {
      if (statementAst.get('exits').has('assignables')) {
        pure = Statement.isAssignablePure(
          statementAst.get('exits').get('assignables'),
          scope,
        );
      }
    }
    if (statementAst.has('emits')) {
      if (statementAst.get('emits').has('assignables')) {
        pure = Statement.isAssignablePure(
          statementAst.get('emits').get('assignables'),
          scope,
        );
      }
    }
    return new Statement(statementAst, scope, pure);
  }

  toString() {
    return this.statementAst.t;
  }
}

export default Statement;
