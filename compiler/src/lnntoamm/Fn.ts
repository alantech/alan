import { LPNode } from "../lp";
import Scope from "./Scope";
import Statement, { StatementMetaData } from "./Statement";
import { Interface, Type } from "./Types";
import { TODO } from "./util";

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
  // TODO: get rid of singular Statement type
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
    let work = ast;
    const name = work.get('optname').has() ? work.get('optname').get().t : null;
    let args: Args = {};
    if (work.get('optargs').has('arglist')) {
      // RIP DRY :(
      let argsAst = work.get('optargs').get('arglist');
      let argName = argsAst.get('variable').t;
      let typename = argsAst.get('fulltypename').t;
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
    const retTy = work.get('optreturntype').has() ? work.get('optreturntype').get().get('fulltypename') : null;
    let body: LPNode | LPNode[] = work.get('fullfunctionbody');
    if (body.has('functionbody')) {
      body = body.get('functionbody').get('statements').getAll();
    } else {
      body = body.get('assignfunction');
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
      ast.get('statements').getAll(),
      stmtMeta,
    );
  }

  transform() {
    if (isLPNode(this.body)) {
      // it's an LPNode
      this.body = Statement.fromAst(this.body, this.scope, this.stmtMeta);
    } else if (isLPNodeArr(this.body)) {
      // it's a list of LPNodes
      this.body.map(node => Statement.fromAst(node, this.scope, this.stmtMeta));
    }

    if (this.body instanceof Statement) {
      this.body = this.body.transform();
    } else if (isStatementArr(this.body)) {
      const body = this.body;
      this.body = [];
      for (let stmt of body) {
        (this.body as Statement[]).push(...stmt.transform());
      }
    }
  }

  getReturnType(): Type | Interface {
    TODO('generate return type of functions');
    return null;
  }
}

const isLPNode = (obj: LPNode | LPNode[] | Statement | Statement[]): obj is LPNode => {
  return !Array.isArray(obj) && !(obj instanceof Statement);
}

const isLPNodeArr = (obj: LPNode | LPNode[] | Statement | Statement[]): obj is LPNode[] => {
  return Array.isArray(obj) && (obj.length === 0 || obj[0] instanceof Statement);
}

const isStatementArr = (obj: LPNode | LPNode[] | Statement | Statement[]): obj is Statement[] => {
  return Array.isArray(obj) && (obj.length === 0 || obj[0] instanceof Fn);
}
