import { LPNode } from "../lp";
import Scope from "./Scope";
import Statement, { Declaration, StatementMetaData, VarMetadata } from "./Statement";
import Type, { FunctionType } from "./Types";

// the value is null if the type is to be inferred
export type Args = {[name: string]: Type | null};

export default class Fn {
  // null if it's an anonymous fn
  name: string | null
  ast: LPNode
  // the scope this function is defined in is the `par`
  scope: Scope
  args: Args
  // null if the type is to be inferred
  retTy: Type
  // later on, we can also add `| Microstatement[]` as an optimization
  // TODO: get rid of singular Statement type
  body: LPNode | LPNode[] | Statement | Statement[]
  // not used by this class, but used by Statements
  stmtMeta: StatementMetaData
  fnType: FunctionType

  constructor(
    ast: LPNode,
    scope: Scope,
    name: string | null,
    args: Args,
    retTy: Type | null,
    body: LPNode | LPNode[] | Statement | Statement[],
    stmtMeta: StatementMetaData = null,
  ) {
    this.ast = ast;
    this.scope = scope;
    this.name = name;
    this.args = args;
    for (let argName of Object.keys(this.args)) {
      if (this.args[argName] === null) {
        this.args[argName] = Type.generate();
      }
    }
    this.retTy = retTy !== null ? retTy : Type.generate();
    this.body = body;
    this.stmtMeta = stmtMeta !== null ? stmtMeta : new StatementMetaData();
    this.fnType = new FunctionType(this.name, Object.values(this.args), this.retTy);
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
      let argAsts = [
        work.get('optargs').get('arglist'),
        ...work.get('optargs').get('arglist').get('cdr').getAll(),
      ];
      for (let argAst of argAsts) {
        let argName = argAst.get('variable').t;
        let typename = argAst.get('fulltypename');
        let argTy = Type.getFromTypename(typename, scope);
        if (argTy === null) {
          throw new Error(`Could not find type ${typename.t.trim()} for argument ${argName}`);
        } else if (!(argTy instanceof Type)) {
          throw new Error(`Function argument is not a valid type: ${typename.t.trim()}`);
        }
        args[argName] = argTy;
      }
    }
    const retTy = work.get('optreturntype').has() ? work.get('optreturntype').get().get('fulltypename') : null;
    let body: LPNode | LPNode[] = work.get('fullfunctionbody');
    if (body.has('functionbody')) {
      body = body.get('functionbody').get('statements').getAll().map(s => s.get('statement'));
    } else {
      body = body.get('assignfunction');
    }

    return new Fn(
      ast,
      new Scope(scope),
      name,
      args,
      Type.getFromTypename(retTy, scope),
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
      ast.get('statements').getAll().map(s => s.get('statement')),
      stmtMeta,
    );
  }

  transform() {
    if (isLPNode(this.body)) {
      // it's an LPNode
      this.body = Statement.fromAst(this.body, this.scope, this.stmtMeta);
    } else if (isLPNodeArr(this.body)) {
      // it's a list of LPNodes
      this.body = this.body.map(node => Statement.fromAst(node, this.scope, this.stmtMeta));
    } else {
      console.log(this.body);
      throw new Error('uhhhhhhhh?');
    }

    if (this.body instanceof Statement) {
      this.body = this.body.transform();
    } else if (isStatementArr(this.body)) {
      const body = this.body;
      this.body = [];
      for (let stmt of body) {
        (this.body as Statement[]).push(...stmt.transform());
      }
    } else {
      console.log(this.body)
      throw new Error('not transforming...?');
    }
  }

  getType(): FunctionType {
    return null;
  }

  // TODO: pretty sure this is just gonna be Function types :)
  constraints(): [{dec: Declaration, constraints: Type[]}[], Type[]] {
    this.transform();
    if (!isStatementArr(this.body)) {
      throw new Error(`Constraints can't be generated without full function body being generated`);
    }
    console.log(this.body);
    this.body.forEach(stmt => stmt.constrain(this.stmtMeta));
    const varConstraints = Object.values(this.stmtMeta.vars);
    console.log(varConstraints);
    return [varConstraints, this.stmtMeta.outputConstraints];
  }
}

const isLPNode = (obj: LPNode | LPNode[] | Statement | Statement[]): obj is LPNode => {
  return !Array.isArray(obj) && !(obj instanceof Statement);
}

const isLPNodeArr = (obj: LPNode | LPNode[] | Statement | Statement[]): obj is LPNode[] => {
  return Array.isArray(obj) && (obj.length === 0 || isLPNode(obj[0]));
}

const isStatementArr = (obj: LPNode | LPNode[] | Statement | Statement[]): obj is Statement[] => {
  return Array.isArray(obj) && (obj.length === 0 || !isLPNode(obj[0]));
}
