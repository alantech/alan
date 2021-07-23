import { LPNode, NamedAnd, NulLP, Token } from '../lp';
import Output, { AssignKind } from './Amm';
import Expr, { Ref } from './Expr';
import opcodes from './opcodes';
import Scope from './Scope';
import Stmt, { Dec, Exit, FnParam, MetaData } from './Stmt';
import Type, { FunctionType } from './Types';
import { DBG, TODO } from './util';

export default class Fn {
  // null if it's an anonymous fn
  name: string | null;
  ast: LPNode;
  // the scope this function is defined in is the `par`
  scope: Scope;
  params: FnParam[];
  retTy: Type;
  body: Stmt[];
  exprFn: Expr;
  // not used by this class, but used by Statements
  metadata: MetaData;
  ty: FunctionType;

  get argNames(): string[] {
    return Object.keys(this.params);
  }

  constructor(
    ast: LPNode,
    scope: Scope,
    name: string | null,
    params: FnParam[],
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
    this.metadata =
      metadata !== null ? metadata : new MetaData(scope, this.retTy);
    while (
      this.body.reduce(
        (carry, stmt) => stmt.cleanup(this.scope) || carry,
        false,
      )
    );
    const tyAst = ((fnAst: LPNode) => {
      if (fnAst instanceof NulLP) {
        // assume it's for an opcode
        const compilerDefinition = '<compiler definition>';
        const makeToken = (tok: string) =>
          new Token(tok, compilerDefinition, -1, -1);
        return new NamedAnd(
          `opcode ${this.name}`,
          {
            opcode: makeToken('opcode'),
            _whitespace: makeToken(' '),
            opcodeName: makeToken(this.name),
          },
          compilerDefinition,
          -1,
          -1,
        );
      }
      return null;
    })(this.ast);
    this.ty = new FunctionType(
      tyAst,
      this.params.map((param) => param.ty),
      this.retTy,
    );
  }

  static fromFunctionsAst(ast: LPNode, scope: Scope): Fn {
    scope = new Scope(scope);

    const retTy = Type.generate();
    // use the new scope for now so that the fn params can insert interfaces
    // as needed
    const metadata = new MetaData(scope, retTy);

    const name = ast.get('optname').has() ? ast.get('optname').get().t : null;
    const p: LPNode[] = [];
    const arglist = ast.get('optargs').get('arglist');
    if (arglist.has()) {
      p.push(arglist);
      if (arglist.get('cdr').has()) {
        p.push(...arglist.get('cdr').getAll());
      }
    }
    const params = p.map((paramAst) => FnParam.fromArgAst(paramAst, metadata));

    let actualRetTy: Type;
    if (ast.get('optreturntype').has()) {
      const name = ast.get('optreturntype').get('fulltypename');
      actualRetTy = Type.getFromTypename(name, scope);
      if (actualRetTy === null) {
        throw new Error(`Type not in scope: ${name.t.trim()}`);
      }
      if (actualRetTy.dupIfNotLocalInterface() !== null) {
        throw new Error(`type erasure is illegal`);
      }
    } else {
      actualRetTy = Type.oneOf([Type.generate(), opcodes().get('void')]);
    }
    // use the new scope still since it also contains the duplicated types
    retTy.constrain(actualRetTy, scope);

    // use scope.par since it contains interface implementation fns
    metadata.scope = scope.par;

    let body = [];
    let bodyAsts: LPNode | LPNode[] = ast.get('fullfunctionbody');
    if (bodyAsts.has('functionbody')) {
      bodyAsts = bodyAsts
        .get('functionbody')
        .get('statements')
        .getAll()
        .map((s) => s.get('statement'));
      bodyAsts.forEach((ast) => body.push(...Stmt.fromAst(ast, metadata)));
    } else {
      bodyAsts = bodyAsts.get('assignfunction').get('assignables');
      let exitVal: Expr;
      try {
        [body, exitVal] = Expr.fromAssignablesAst(bodyAsts, metadata);
      } catch (e) {
        console.log('body is', bodyAsts);
        throw e;
      }
      if (exitVal instanceof Ref) {
        body.push(new Exit(bodyAsts, exitVal, retTy));
      } else {
        const retVal = Dec.gen(exitVal, metadata);
        body.push(retVal);
        body.push(new Exit(bodyAsts, retVal.ref(), retTy));
      }
    }

    return new Fn(ast, new Scope(scope), name, params, retTy, body);
  }

  static fromFunctionbody(ast: LPNode, scope: Scope): Fn {
    const body = [];
    // use scope.par since it contains interface implementation fns
    const metadata = new MetaData(scope, opcodes().get('void'));
    ast
      .get('statements')
      .getAll()
      .map((s) => s.get('statement'))
      .forEach((ast) => body.push(...Stmt.fromAst(ast, metadata)));
    return new Fn(
      ast,
      scope,
      null,
      [],
      // TODO: if expressions will mean that it's not necessarily void...
      opcodes().get('void'),
      body,
      metadata,
    );
  }

  asHandler(amm: Output, event: string) {
    const handlerParams = [];
    for (const param of this.params) {
      handlerParams.push([param.ammName, param.ty]);
    }
    amm.addHandler(event, handlerParams, this.retTy);
    let isReturned = false;
    for (let ii = 0; ii < this.body.length; ii++) {
      const stmt = this.body[ii];
      stmt.inline(amm);
      if (stmt instanceof Exit) {
        isReturned = true;
        if (ii !== this.body.length - 1) {
          throw new Error(
            `hmmmm... unreachable statements probably should've been caught earlier?`,
          );
        }
      }
    }
    if (!isReturned) {
      if (
        !this.retTy.compatibleWithConstraint(
          opcodes().get('void'),
          this.metadata.scope,
        )
      ) {
        throw new Error(`event handlers should not return values`);
      }
      amm.exit();
    }
  }

  // FIXME: it'll take a bit more work to do better inlining, but it *should* be possible
  // to have `inline` load all of the amm code into a new `Stmt[]` which is then iterated
  // at the handler level to do optimizations and such, similar to the `Microstatement[]`
  // that was loaded but using the same JS objects(?) and the optimizations should only
  // be further inlining...
  // FIXME: another option is to convert to SSA form (talked a bit about in Amm.ts) and then
  // perform optimizations from there. This *might* require the `Stmt[]` array from above
  // *or* we can do it in Amm.ts using only strings (although that might be harder)
  // FIXME: a 3rd option is to make amm itself only SSA and perform the the "register
  // selection" in the ammtox stage. This might be the best solution, since it's the most
  // flexible regardless of the backend, and amm is where that diverges.
  inline(amm: Output, args: Ref[], kind: AssignKind, name: string, ty: Type) {
    if (args.length !== this.params.length) {
      throw new Error(`function call argument mismatch`);
    }
    this.params.forEach((param, ii) => param.assign(args[ii], this.scope));
    for (let ii = 0; ii < this.body.length; ii++) {
      const stmt = this.body[ii];
      if (stmt instanceof Exit) {
        if (ii !== this.body.length - 1) {
          throw new Error(
            `got a return at a bad time (should've been caught already?)`,
          );
        }
        if (ty.eq(opcodes().get('void'))) {
          break;
        }
        const refCall = ty.isFixed() ? 'reff' : 'refv';
        amm.assign(kind, name, ty, refCall, [stmt.ret.ammName]);
        break;
      }
      stmt.inline(amm);
    }
    this.params.forEach((param) => param.unassign());
  }

  resultTyFor(argTys: Type[], scope: Scope): Type | null {
    let res: Type | null = null;
    try {
      this.params.forEach((param, ii) =>
        param.ty.tempConstrain(argTys[ii], scope),
      );
      res = this.retTy.instance();
    } catch (_e) {
      // do nothing: the args aren't applicable to the params so
      // we return null (`res` is already `null`) and we need to
      // ensure the param tys have `resetTemp` called on them.
    }
    this.params.forEach((param) => param.ty.resetTemp());
    return res;
  }
}

// circular dependency issue when this is defined in opcodes.ts :(
export class OpcodeFn extends Fn {
  constructor(
    name: string,
    argDecs: { [name: string]: string },
    retTyName: string,
    __opcodes: Scope,
  ) {
    const scopeForOpcode = new Scope(__opcodes);
    const params = Object.entries(argDecs).map(([name, tyName]) => {
      return new FnParam(new NulLP(), name, Type.getFromTypename(tyName, scopeForOpcode));
    });
    const retTy = Type.getFromTypename(retTyName, scopeForOpcode);
    if (retTy === null) {
      throw new Error('a');
    } else if (retTy.dupIfNotLocalInterface()) {
      throw new Error(`please don't make an opcode with type erasure...`);
    }
    // don't use scopeForOpcode so it gets GC'd, it's not necessary
    super(new NulLP(), __opcodes, name, params, retTy, []);
    __opcodes.put(name, [this]);
  }

  asHandler(_amm: Output, _event: string) {
    TODO('opcodes as event listener???');
  }

  inline(amm: Output, args: Ref[], kind: AssignKind, assign: string, ty: Type) {
    amm.assign(
      kind,
      assign,
      ty,
      this.name,
      args.map((ref) => ref.ammName),
    );
  }
}
