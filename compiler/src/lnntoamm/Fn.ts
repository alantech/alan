import { LPNode, NulLP } from '../lp';
import Output, { AssignKind } from './Amm';
import Expr, { Ref } from './Expr';
import opcodes from './opcodes';
import Scope from './Scope';
import Stmt, { Dec, Exit, FnParam, MetaData } from './Stmt';
import Type, { Builtin } from './Types';
import { TODO } from './util';

export type Params = {[name: string]: FnParam};

export default class Fn {
  // null if it's an anonymous fn
  name: string | null
  ast: LPNode
  // the scope this function is defined in is the `par`
  scope: Scope
  params: Params
  retTy: Type
  body: Stmt[]
  // not used by this class, but used by Statements
  metadata: MetaData

  get argNames(): string[] {
    return Object.keys(this.params);
  }

  constructor(
    ast: LPNode,
    scope: Scope,
    name: string | null,
    params: Params,
    retTy: Type | null,
    body: Stmt[],
    metadata: MetaData = null,
  ) {
    this.ast = ast;
    this.scope = scope;
    this.name = name;
    this.params = params;
    this.retTy = retTy !== null ? retTy : Type.generate();
    this.body = body;
    this.metadata = metadata !== null ? metadata : new MetaData(scope, this.retTy);
  }

  static fromFunctionsAst(
    ast: LPNode,
    scope: Scope,
  ): Fn {
    let retTy: Type;
    if (ast.get('optreturntype').has()) {
      const name = ast.get('optreturntype').get('fulltypename');
      retTy = Type.getFromTypename(name, scope);
      if (retTy === null) {
        throw new Error(`Type not in scope: ${name.t.trim()}`);
      }
    } else {
      retTy = Type.oneOf([
        Type.generate(),
        opcodes().get('void'),
      ])
    }

    // TODO: inheritance
    let metadata = new MetaData(scope, retTy);

    const name = ast.get('optname').has() ? ast.get('optname').get().t : null;
    let params: Params = {};
    if (ast.get('optargs').has('arglist')) {
      let paramAsts = [
        ast.get('optargs').get('arglist'),
        ...ast.get('optargs').get('arglist').get('cdr').getAll(),
      ];
      paramAsts.forEach(paramAst => {
        const arg = FnParam.fromArgAst(paramAst, metadata)
        params[arg.name] = arg;
      });
    }

    let body = [];
    let bodyAsts: LPNode | LPNode[] = ast.get('fullfunctionbody');
    if (bodyAsts.has('functionbody')) {
      bodyAsts = bodyAsts.get('functionbody').get('statements').getAll().map(s => s.get('statement'));
      bodyAsts.forEach(ast => body.push(...Stmt.fromAst(ast, metadata)));
    } else {
      bodyAsts = bodyAsts.get('assignfunction').get('assignables');
      let exitVal: Expr;
      [body, exitVal] = Expr.fromAssignablesAst(bodyAsts, metadata);
      let retVal = Dec.gen(exitVal, metadata);
      body.push(retVal);
      body.push(new Exit(bodyAsts, retVal.ref()));
    }

    return new Fn(
      ast,
      new Scope(scope),
      name,
      params,
      retTy,
      body,
    );
  }

  static fromFunctionbody(
    ast: LPNode,
    scope: Scope,
  ): Fn {
    let body = [];
    let metadata = new MetaData(scope, opcodes().get('void'));
    ast.get('statements').getAll().map(s => s.get('statement')).forEach(ast => body.push(...Stmt.fromAst(ast, metadata)));
    return new Fn(
      ast,
      new Scope(scope),
      null,
      {},
      // TODO: if expressions will mean that it's not necessarily void...
      opcodes().get('void'),
      body,
      metadata,
    );
  }

  acceptsTypes(tys: Type[]): boolean {
    let params = Object.values(this.params);
    if (params.length !== tys.length) {
      return false;
    }
    for (let ii = 0; ii < params.length; ii++) {
      if (!params[ii].ty.compatibleWithConstraint(tys[ii])) {
        return false;
      }
    }
    return true;
  }

  asHandler(amm: Output, event: string) {
    let handlerParams = [];
    for (let param of Object.keys(this.params)) {
      handlerParams.push([param, this.params[param].ty.breakdown()]);
    }
    amm.addHandler(event, handlerParams, this.retTy.breakdown());
    let isReturned = false;
    for (let ii = 0; ii < this.body.length; ii++) {
      const stmt = this.body[ii];
      stmt.inline(amm);
      if (stmt instanceof Exit) {
        isReturned = true;
        if (ii !== this.body.length - 1) {
          throw new Error(`hmmmm... unreachable statements probably should've been caught earlier?`);
        }
      }
    }
    if (!isReturned) {
      if (!this.retTy.compatibleWithConstraint(opcodes().get('void'))) {
        throw new Error(`event handlers should not return values`);
      }
      amm.exit();
    }
  }

  inline(amm: Output, args: Ref[], kind: AssignKind, name: string, ty: Builtin) {
    let paramDefs = Object.values(this.params);
    if (args.length !== paramDefs.length) {
      throw new Error(`function call argument mismatch`);
    }
    for (let ii = 0; ii < paramDefs.length; ii++) {
      paramDefs[ii].assign(args[ii]);
    }
    for (let ii = 0; ii < this.body.length; ii++) {
      const stmt = this.body[ii];
      if (stmt instanceof Exit) {
        if (ii !== this.body.length - 1) {
          throw new Error(`got a return at a bad time (should've been caught already?)`);
        }
        amm.assign(kind, name, ty, 'reff', [stmt.ret.ammName]);
        break;
      }
      stmt.inline(amm);
    }
    paramDefs.forEach(def => def.unassign());
  }
}

// circular dependency issue when this is defined in opcodes.ts :(
export class OpcodeFn extends Fn {
  constructor(
    name: string,
    argDecs: {[name: string]: string},
    retTyName: string,
    __opcodes: Scope,
  ) {
    let args = {};
    for (let argName of Object.keys(argDecs)) {
      let argTy = argDecs[argName];
      let ty = __opcodes.get(argTy);
      if (ty === null) {
        throw new Error(`opcode ${name} arg ${argName} uses a type that's not defined`);
      } else if (!(ty instanceof Type)) {
        throw new Error(`opcode ${name} arg ${argName} doesn't have a valid type`);
      } else {
        args[argName] = new FnParam(new NulLP(), argName, ty);
      }
    }
    let retTy = __opcodes.get(retTyName);
    if (retTy === null || !(retTy instanceof Type)) {
      throw new Error()
    }
    super(new NulLP(), __opcodes, name, args, retTy, []);
    __opcodes.put(name, [this]);
  }

  asHandler(_amm: Output, _event: string) {
    TODO('opcodes as event listener???');
  }

  inline(amm: Output, args: Ref[], kind: AssignKind, assign: string, ty: Builtin) {
    amm.assign(
      kind,
      assign,
      ty,
      this,
      args.map(ref => ref.ammName),
    );
  }
}