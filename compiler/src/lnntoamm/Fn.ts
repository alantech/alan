import { LPNode, NulLP } from '../lp';
import Output from './Amm';
import opcodes from './opcodes';
import Scope from './Scope';
import Stmt, { Assign, Dec, Emit, Exit, FnArg, MetaData, Ref } from './Statement';
import Type, { FunctionType } from './Types';
import { TODO } from './util';

export type Args = {[name: string]: FnArg};

export default class Fn {
  // null if it's an anonymous fn
  name: string | null
  ast: LPNode
  // the scope this function is defined in is the `par`
  scope: Scope
  args: Args
  retTy: Type
  body: Stmt[]
  // not used by this class, but used by Statements
  metadata: MetaData
  fnType: FunctionType

  get argNames(): string[] {
    return Object.keys(this.args);
  }

  constructor(
    ast: LPNode,
    scope: Scope,
    name: string | null,
    args: Args,
    retTy: Type | null,
    body: Stmt[],
    metadata: MetaData = null,
  ) {
    this.ast = ast;
    this.scope = scope;
    this.name = name;
    this.args = args;
    this.retTy = retTy !== null ? retTy : Type.generate();
    this.body = body;
    this.metadata = metadata !== null ? metadata : new MetaData(scope);
    this.fnType = new FunctionType(
      this.name,
      Object.values(this.args).map(a => a.ty),
      this.retTy
    );
  }

  static fromFunctionsAst(
    ast: LPNode,
    scope: Scope,
  ): Fn {
    // TODO: inheritance
    let metadata = new MetaData(scope);

    const name = ast.get('optname').has() ? ast.get('optname').get().t : null;
    let args: Args = {};
    if (ast.get('optargs').has('arglist')) {
      let argAsts = [
        ast.get('optargs').get('arglist'),
        ...ast.get('optargs').get('arglist').get('cdr').getAll(),
      ];
      argAsts.forEach(argAst => {
        const arg = FnArg.fromArgAst(argAst, metadata)
        args[arg.name] = arg;
      });
    }

    let retTy: Type;
    if (ast.get('optreturntype').has()) {
      const name = ast.get('optreturntype').get().get('fulltypename');
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

    let body = [];
    let bodyAsts: LPNode | LPNode[] = ast.get('fullfunctionbody');
    if (bodyAsts.has('functionbody')) {
      bodyAsts = bodyAsts.get('functionbody').get('statements').getAll().map(s => s.get('statement'));
      bodyAsts.forEach(ast => body.push(...Stmt.fromAst(ast, metadata)));
    } else {
      bodyAsts = bodyAsts.get('assignfunction');
      body = Stmt.fromAst(bodyAsts, metadata);
      const retVal = body[body.length - 1];
      if (!(retVal instanceof Dec)) {
        throw new Error(`illegal function body: ${bodyAsts}`);
      }
      body.push(new Exit(bodyAsts, retVal.ref()));
    }

    return new Fn(
      ast,
      new Scope(scope),
      name,
      args,
      retTy,
      body,
    );
  }

  static fromFunctionbody(
    ast: LPNode,
    scope: Scope,
  ): Fn {
    let body = [];
    let metadata = new MetaData(scope);
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

  getType(): FunctionType {
    return this.fnType;
  }

  asHandler(amm: Output, event: string) {
    let handlerArgs = [];
    for (let arg of Object.keys(this.args)) {
      handlerArgs.push([arg, this.args[arg].ty.breakdown()]);
    }
    amm.addHandler(event, handlerArgs, this.retTy.breakdown());
    let isReturned = false;
    for (let ii = 0; ii < this.body.length; ii++) {
      const stmt = this.body[ii];
      if (stmt instanceof Dec || stmt instanceof Assign || stmt instanceof Emit) {
        stmt.inline(amm);
      } else if (stmt instanceof Exit) {
        stmt.inline(amm);
        isReturned = true;
        if (ii !== this.body.length - 1) {
          throw new Error(`hmmmm... unreachable statements probably should've been caught earlier?`);
        }
        break;
      } else {
        throw new Error(`did not expect to inline stmt: ${stmt}`);
      }
    }
    if (!isReturned) {
      if (this.retTy.breakdown() !== opcodes().get('void')) {
        throw new Error(`no return value for function`);
      }
      amm.exit();
    }
  }

  // TODO: this will have to change in order to call fns multiple times - maybe deep cloning?
  inline(amm: Output, args: Ref[], assign: string, kind: 'const' | 'let' | '') {
    let argNames = Object.keys(this.args);
    if (argNames.length !== args.length) {
      // this should be caught by Call, it's just a sanity check
      throw new Error(`number of arguments off`);
    }
    for (let ii = 0; ii < argNames.length; ii++) {
      this.args[argNames[ii]].val = args[ii];
    }
    const last = this.body[this.body.length - 1];
    if (last instanceof Exit && last.exitVal !== null) {
      if (kind === 'const' && last.exitVal.dec.mutable) {
        kind = 'let';
      } else if (kind === 'let' && !last.exitVal.dec.mutable) {
        last.exitVal.dec.mutable = true;
      } else if (kind === '') {
        TODO('figure out how to do return value rewrites pls');
      }
      last.exitVal.dec.ammName = assign;
    }
    for (let ii = 0; ii < this.body.length; ii++) {
      const stmt = this.body[ii];
      if (stmt instanceof Dec || stmt instanceof Assign || stmt instanceof Emit) {
        stmt.inline(amm);
      } else if (stmt instanceof Exit) {
        amm.assign(
          kind,
          assign,
          stmt.exitVal.ty.breakdown(),
          stmt.exitVal.ammName,
        );
      } else {
        throw new Error(`did not expect to inline stmt: ${stmt}`);
      }
    }
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
        args[argName] = new FnArg(new NulLP(), argName, ty);
      }
    }
    let retTy = __opcodes.get(retTyName);
    if (retTy === null || !(retTy instanceof Type)) {
      throw new Error()
    }
    super(new NulLP(), __opcodes, name, args, retTy, []);
    __opcodes.put(name, [this]);
  }

  inline(amm: Output, args: Ref[], assign: string, kind: 'const' | 'let' | '') {
    amm.assign(
      kind,
      assign,
      this.retTy.breakdown(),
      this,
      args.map(ref => ref.ammName),
    );
  }
}