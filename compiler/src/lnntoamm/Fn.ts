import { stdout } from 'process';
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
    const fnSigScope = new Scope(scope);

    let retTy: Type;
    if (ast.get('optreturntype').has()) {
      const name = ast.get('optreturntype').get('fulltypename');
      retTy = Type.getFromTypename(name, fnSigScope, { isTyVar: true });
      if (retTy === null) {
        throw new Error(`Type not in scope: ${name.t.trim()}`);
      }
      // if (retTy.dup() !== null) {
      //   // TODO: figure out how to prevent type erasure while allowing
      //   // eg the generic identity function. Or just wait until generic
      //   // fn type parameters.
      //   throw new Error(`type erasure is illegal`);
      // }
    } else {
      retTy = Type.oneOf([Type.generate(), opcodes().get('void')]);
    }

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
    const params = p.map((paramAst) => FnParam.fromArgAst(paramAst, metadata, fnSigScope));

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
      [body, exitVal] = Expr.fromAssignablesAst(bodyAsts, metadata);
      if (exitVal instanceof Ref) {
        body.push(new Exit(bodyAsts, exitVal, retTy));
        retTy.constrain(exitVal.ty, scope);
      } else {
        const retVal = Dec.gen(exitVal, metadata);
        body.push(retVal);
        body.push(new Exit(bodyAsts, retVal.ref(), retTy));
        retTy.constrain(retVal.ty, scope);
      }
    }

    // if (name === 'none') {
    //   console.log('~~~~~~~~ created', name);
    //   stdout.write('params: ');
    //   console.dir(params, { depth: 4 });
    //   stdout.write('retTy: ');
    //   console.dir(retTy, { depth: 4 });
    //   stdout.write('body: ');
    //   console.dir(body, { depth: 4 });
    // }

    return new Fn(ast, new Scope(scope), name, params, retTy, body);
  }

  static fromFunctionbody(ast: LPNode, scope: Scope): Fn {
    scope = new Scope(scope);
    const body = [];
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
    // console.dir(this.body, { depth: 6 });
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
  inline(amm: Output, args: Ref[], kind: AssignKind, name: string, ty: Type, callScope: Scope) {
    if (args.length !== this.params.length) {
      throw new Error(`function call argument mismatch`);
    }
    this.params.forEach((param, ii) => param.assign(args[ii], this.scope));
    this.retTy.tempConstrain(ty, callScope);
    // console.log('----- inlining', this.name);
    // stdout.write('rty: ');
    // console.dir(this.retTy, { depth: 4 });
    // console.dir(this.body, { depth: 8 });
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
    this.retTy.resetTemp();
    this.params.forEach((param) => param.unassign());
  }

  resultTyFor(argTys: Type[], expectResTy: Type, scope: Scope): [Type[], Type] | null {
    let res: [Type[], Type] | null = null;
    try {
      this.params.forEach((param, ii) => {
        return param.ty.tempConstrain(argTys[ii], scope);
      });
      this.retTy.tempConstrain(expectResTy, scope);
      res = [
        this.params.map((param) => param.ty.instance({ interfaceOk: true })),
        this.retTy.instance({ interfaceOk: true }),
      ];
    } catch (e) {
      // do nothing: the args aren't applicable to the params so
      // we return null (`res` is already `null`) and we need to
      // ensure the param tys have `resetTemp` called on them.
      // However, if the Error message starts with `TODO`, then
      // print the error since it's for debugging purposes
      let msg = e.message as string;
      if (msg.startsWith('TODO')) {
        console.log('warning: came across TODO from Types.ts when getting result ty for a function:');
        console.group();
        console.log(msg);
        console.groupEnd();
      }
    }
    this.params.forEach((param) => param.ty.resetTemp());
    this.retTy.resetTemp();
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
    const tyScope = new Scope(__opcodes);
    const params = Object.entries(argDecs).map(([name, tyName]) => {
      return new FnParam(new NulLP(), name, Type.getFromTypename(tyName, tyScope, { isTyVar: true }));
    });
    const retTy = Type.getFromTypename(retTyName, tyScope, { isTyVar: true });
    if (retTy === null || !(retTy instanceof Type)) {
      throw new Error('not a type');
    }
    super(new NulLP(), __opcodes, name, params, retTy, []);
    __opcodes.put(name, [this]);
  }

  asHandler(_amm: Output, _event: string) {
    TODO('opcodes as event listener???');
  }

  inline(amm: Output, args: Ref[], kind: AssignKind, assign: string, ty: Type, callScope: Scope) {
    this.retTy.tempConstrain(ty, callScope);
    amm.assign(
      kind,
      assign,
      ty,
      this.name,
      args.map((ref) => ref.ammName),
    );
    this.retTy.resetTemp();
  }
}
