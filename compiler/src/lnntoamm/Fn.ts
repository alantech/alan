import { LPNode } from "../lp";
import Scope from "./Scope";
import Statement, { StatementMetaData } from "./Statement";
import { Interface, Type } from "./Types";

// the value is null if the type is to be inferred
export type Args = {[name: string]: Type | Interface | null};

export default class Fn {
  ast: LPNode
  // the scope this function is defined in is the `par`
  scope: Scope
  // null if it's an anonymous fn
  name: string | null
  args: Args
  // null if the type is to be inferred
  retTy: LPNode | Type | null
  // later on, we can also add `| Microstatement[]` as an optimization
  body: LPNode | LPNode[] | Statement | Statement[]
  // not used by this class, but used by Statements
  stmtMeta: StatementMetaData

  constructor(
    ast: LPNode,
    scope: Scope,
    name: string | null,
    args: Args,
    retTy: LPNode | Type | null,
    body: LPNode | LPNode[] | Statement | Statement[],
    stmtMeta: StatementMetaData = null,
  ) {
    this.ast = ast;
    this.scope = scope;
    this.name = name;
    this.args = args;
    this.retTy = retTy;
    this.body = body;
    this.stmtMeta = stmtMeta !== null ? stmtMeta : new StatementMetaData();
  }

  static fromFunctionsAst(
    ast: LPNode,
    scope: Scope,
    stmtMeta: StatementMetaData = null,
  ): Fn {
    const name = ast.get('optname').has() ? ast.get('optname').get().t.trim() : null;
    let args: Args = {};
    if (ast.get('optargs').has('arglist')) {
      // RIP DRY :(
      let argsAst = ast.get('optargs').get('arglist');
      let argName = argsAst.get('variable').t.trim();
      let typename = argsAst.get('fulltypename').t.trim();
      let argTy = scope.get(typename);
      if (argTy === null) {
        throw new Error(`Could not find type ${typename} for argument ${argName}`);
      } else if (!(argTy instanceof Type)) {
        throw new Error(`Function argument is not a valid type: ${typename}`);
      }
      args[argName] = argTy;
      for (let argAst of argsAst.get('cdr').getAll()) {
        argName = argAst.get('variable').t.trim();
        typename = argAst.get('fulltypename').t.trim();
        argTy = scope.get(typename);
        if (argTy === null) {
          throw new Error(`Could not find type ${typename} for argument ${argName}`);
        } else if (!(argTy instanceof Type)) {
          throw new Error(`Function argument is not a valid type: ${typename}`);
        }
        args[argName] = argTy;
      }
    }
    const retTy = ast.get('optreturntype').has() ? ast.get('optreturntype').get().get('fulltypename') : null;
    let body = ast.get('fullfunctionbody');
    if (body.has('functionbody')) {
      body = body.get('functionbody').get('statements');
    } else if (body.has('assignfunction')) {
      body = body.get('assignfunction').get('assignables');
    }

    return new Fn(
      ast,
      new Scope(scope),
      name,
      args,
      retTy,
      body,
      stmtMeta,
    );
  }

  static fromFunctionbody(
    ast: LPNode,
    scope: Scope,
    stmtMeta: StatementMetaData = null,
  ): Fn {
    return new Fn(
      ast,
      new Scope(scope),
      null,
      {},
      // TODO: this should be `void`
      null,
      ast.get('statements'),
      stmtMeta,
    );
  }
}
