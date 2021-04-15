import { LPNode, NamedAnd, NamedOr } from "../lp";
import Output from "./Amm";
import Event from './Event'
import Fn from "./Fn";
import opcodes from "./opcodes";
import Scope from "./Scope";
import Type, { Builtin, FunctionType } from "./Types";
import { genName, TODO } from "./util";

export class MetaData {
  scope: Scope
  variables: { [name: string]: Dec }
  retConstraints: Type[]

  constructor(
    scope: Scope,
    variables: { [name: string]: Dec } = null,
    retConstraints: Type[] = null,
  ) {
    this.scope = scope;
    this.variables = variables !== null ? variables : {};
    this.retConstraints = retConstraints !== null ? retConstraints : [];
  }

  var(name: string): Dec {
    if (this.variables[name] == null) {
      return null;
    }
    return this.variables[name];
  }

  define(dec: Dec) {
    if (this.var(dec.name) !== null) {
      throw new Error(`Can't redefine value ${dec.name}`);
    }
    this.variables[dec.name] = dec;
  }
}

export default abstract class Stmt {
  ast: LPNode

  constructor(
    ast: LPNode,
  ) {
    this.ast = ast;
  }

  // interface fns
  abstract exprTy(): Type;
  abstract inline(amm: Output): void;

  // =================
  // factory functions
  static fromAst(ast: LPNode, metadata: MetaData): Stmt[] {
    let stmts = [];
    if (ast.has('assignables')) {
      stmts.push(...Stmt.fromAssignables(ast.get('assignables'), metadata));
    } else if (ast.has('assignments')) {
      stmts.push(...Assign.fromAssignmentsAst(ast.get('assignments'), metadata));
    } else if (ast.has('conditionals')) {
      stmts.push(...Cond.fromConditionalsAst(ast.get('conditionals'), metadata));
    } else if (ast.has('declarations')) {
      stmts.push(...Dec.fromAst(ast.get('declarations'), metadata));
    } else if (ast.has('emits')) {
      stmts.push(...Emit.fromAst(ast.get('emits'), metadata));
    } else if (ast.has('exits')) {
      stmts.push(...Exit.fromAst(ast.get('exits'), metadata));
    } else {
      throw new Error(`unrecognized statement ast: ${ast}`);
    }
    return stmts;
  }

  static fromAssignables(ast: LPNode, metadata: MetaData): Stmt[] {
    let stmts = [];

    let asts: LPNode[] = ast.getAll().map(a => a.get('withoperators'));
    if (asts.length > 1) TODO('operators');
    asts = asts.length === 1 ? asts.pop().get('baseassignablelist').getAll().map(a => a.get('baseassignable')) : [];
    for (let ii = 0; ii < asts.length; ii++) {
      let work = asts[ii];
      if (work.has('objectliterals')) {
        TODO('object literals');
      } else if (work.has('functions')) {
        TODO('functions in functions');
      } else if (work.has('variable')) {
        const varName = work.get('variable').t;
        if (ii === asts.length - 1) {
          let dec = metadata.var(varName);
          if (dec === null) {
            throw new Error(`${varName} not defined`);
          }
          stmts.push(dec.ref());
          break;
        }
        const next = asts[ii + 1];
        if (next.has('fncall')) {
          // make things nice and pretty :)
          let callAst = new NamedAnd(
            work.get('variable').t + next.get('fncall').t,
            {
              fnname: work.get('variable'),
              fncall: next.get('fncall'),
            },
            (work as NamedOr).filename,
            work.line,
            work.char,
          );
          stmts.push(...Call.fromAsts(
            callAst,
            // null,
            varName,
            next.get('fncall'),
            metadata,
          ));
          ii += 1;
        } else if (next.has('methodsep')) {
          TODO('accesses/methods');
        } else {
          throw new Error(`unexpected token: expected dot or call, found ${next}`);
        }
      } else if (work.has('constants')) {
        work = work.get('constants');
        let dec = Dec.generate(Lit.fromConstantsAst(work, metadata));
        metadata.define(dec);
        stmts.push(dec);
      } else {
        // TODO: don't lump in HOF and chains
        throw new Error(`unexpected token: expected variable or value, found ${work.t.trim()}`);
      }
    }

    return stmts;
  }
}

export class Assign extends Stmt {
  upstream: Dec
  val: Stmt

  constructor(
    ast: LPNode,
    upstream: Dec,
    assignTo: Stmt,
  ) {
    super(ast);
    this.upstream = upstream;
    this.val = assignTo;
  }

  static fromAssignmentsAst(ast: LPNode, metadata: MetaData): Stmt[] {
    let stmts: Stmt[] = [];
    const name = ast.get('varn').t;
    const upstream = metadata.var(name);
    if (upstream === null) {
      throw new Error(`can't assign to ${name}: not found`);
    }
    stmts.push(...Stmt.fromAssignables(ast.get('assignables'), metadata));
    const expr = stmts.pop();
    let assign: Assign;
    if (expr instanceof Dec) {
      assign = new Assign(ast, upstream, expr.val);
    } else if (expr instanceof Ref) {
      TODO('figure out how to resolve this???');
    } else {
      throw new Error(`invalid assignment state: not a declaration`);
    }
    upstream.ty.constrain(assign.val.exprTy());
    stmts.push(assign);
    return stmts;
  }

  exprTy(): Type {
    throw new Error(`assignments aren't expressions`);
  }

  inline(amm: Output) {
    const name = this.upstream.ammName;
    const ty = this.upstream.ty.breakdown(); // always use the declaration's type, since it's been reduced.
    if (this.val instanceof Call) {
      this.val.inline(amm, name, ty, '');
    } else if (this.val instanceof Lit) {
      amm.assign('', name, ty, this.val.val);
    } else {
      throw new Error(`Unexpected assignment expression: ${this.val}`);
    }
  }
}

export class Call extends Stmt {
  fn: Fn
  args: Ref[]

  constructor(
    ast: LPNode,
    fn: Fn,
    args: Ref[],
  ) {
    super(ast)
    this.fn = fn;
    this.args = args;
  }

  static fromAsts(
    wholeAst: LPNode,
    fnName: string,
    fnCallAst: LPNode,
    metadata: MetaData,
  ): Stmt[] {
    let stmts = [];

    fnCallAst = fnCallAst.get('assignablelist');
    const argAsts: LPNode[] = [
      fnCallAst.get('assignables'),
      ...fnCallAst.get('cdr').getAll().map(a => a.get('assignables')),
    ];
    const args: Ref[] = argAsts.map(a => {
      const snap = stmts.length;
      stmts.push(...Stmt.fromAssignables(a, metadata));
      if (stmts.length <= snap) {
        throw new Error(`didn't generate any statements for arg ${a.t.trim()}`);
      }
      let dec: Stmt = stmts[stmts.length - 1];
      if (dec instanceof Dec) {
        return dec.ref();
      } else if (dec instanceof Ref) {
        return stmts.pop() as Ref;
      } else {
        console.log(dec);
        throw new Error(`got unexpected statement for arg ${a.t.trim()}: ${dec}`);
      }
    });

    let fns = metadata.scope.deepGet(fnName);
    if (fns === null) {
      fns = [];
    }
    if (!(fns instanceof Array) || !(fns.length > 0 && fns[0] instanceof Fn)) {
      throw new Error(`not a function: ${fnName}`);
    }
    // TODO: try and get it from `metadata.var` here as well
    if (fns.length === 0) {
      throw new Error(`no function found for ${fnName}`);
    } else if (fns.length > 1) {
      TODO('fn selection')
    }
    let fn = fns[0];
    if (Object.keys(fn.args).length !== args.length) {
      throw new Error(`argument mismatch: ${fnName} expects ${Object.keys(fn.args).length} args but ${args.length} were provided`);
    }
    const call = new Call(
      wholeAst,
      fn,
      args,
    );
    let dec = Dec.generate(call);
    // just in case the return value of the function has to mutate the value
    dec.mutable = true;
    // TODO: will probably change with fn selection?
    dec.ty.constrain(fn.retTy);
    metadata.define(dec);
    stmts.push(dec);

    return stmts;
  }

  exprTy(): Type {
    return this.fn.retTy;
  }

  inline(amm: Output, assign?: string, ty?: Type, kind?: 'const' | 'let' | ''): void {
    // TODO: determine if/when to inline vs just call once that syntax is supported in AMM
    // TODO: also fix this so that functions can be called multiple times
    if (!assign || !ty || !kind) {
      throw new Error(`bad call`);
    }
    ty.constrain(this.fn.retTy);
    this.fn.inline(amm, this.args, assign, kind);
  }
}
// // TODO: try and revive fn selection using ideas from this class:
// export class Call extends Stmt {
//   // TODO: Fn[]
//   fns: Fn
//   args: Ref[]
//   retTy: Type
//   callTy: FunctionType

//   constructor(
//     ast: LPNode,
//     fns: Fn,
//     args: Ref[],
//     retTy: Type = null,
//     callTy: FunctionType = null,
//   ) {
//     super(ast);
//     // TODO: selection logic is pretty off :/
//     // fns = fns.filter(fn => Object.keys(fn.args).length === args.length)
//     // // fns = fns.filter(fn => callTy.compatibleWithConstraint(fn.fnType));
//     // if (fns.length === 0) {
//     //   throw new Error(`could not find function for call site \`${ast}\``)
//     // }
//     // let fnRet = fns[0].retTy;
//     // for (let ii = 1; ii < fns.length; ii++) {
//     //   if (fnRet !== fns[ii].retTy) {
//     //     fnRet = null;
//     //   }
//     // }
//     // if (fnRet !== null) {
//     // }
//     if (retTy === null) {
//       // TODO: retTy = Type.generate();
//       retTy = fns.retTy;
//     }
//     // if (callTy === null) {
//     //   callTy = new FunctionType('CALL', args.map(r => r.ty), retTy);
//     // }
//     // if (callTy.retTy !== retTy) {
//     //   throw new Error('errr');
//     // }
//     // const fnTypes = fns.map(fn => fn.fnType);
//     // this.fns = fns;
//     this.args = args;
//     this.retTy = retTy;
//     this.callTy = callTy;
//     // TODO: i have a feeling this isn't the right way to go...
//     // this.callTy.callSelect = Type.oneOf(fnTypes);
//   }

//   static fromAsts(
//     wholeAst: LPNode,
//     accessed: Dec | Ref | null,
//     fnName: string,
//     fnCallAst: LPNode,
//     metadata: MetaData,
//   ): Stmt[] {
//     let stmts: Stmt[] = [];

//     fnCallAst = fnCallAst.get('assignablelist');
//     let args: Ref[] = [
//       fnCallAst.get('assignables'),
//       ...fnCallAst.get('cdr').getAll().map(n => n.get('assignables')),
//     ].map(ast => {
//       stmts.push(...Stmt.fromAssignables(ast, metadata));
//       let dec: Stmt = stmts[stmts.length - 1];
//       if (!(dec instanceof Dec) && !(dec instanceof Ref)) {
//         throw new Error(`declaration not generated for arg ${ast.t.trim()}`);
//       }
//       if (dec instanceof Ref) {
//         return stmts.pop() as Ref;
//       } else {
//         return dec.ref();
//       }
//     });
//     if (accessed !== null) {
//       args.unshift(accessed.ref());
//     }

//     // TODO: function selection
//     // let fns: Fn[] = [];
//     // let fromScope = metadata.scope.get(fnName);
//     // if (Array.isArray(fromScope) && fromScope.length > 0 && fromScope[0] instanceof Fn) {
//     //   fns.push(...fromScope);
//     // }
//     // if (metadata.var(fnName) !== null) {
//     //   TODO('closure calling')
//     // }
//     let fns = metadata.scope.get(fnName) || metadata.var(fnName);

//     const call = new Call(
//       wholeAst,
//       fns,
//       args,
//     );
//     // if (call.fns.length === 0) {
//     //   throw new Error('sanity check failed :(');
//     // } else if (call.fns.length > 1) {
//     //   TODO('type-constraining for function selection');
//     // } else { // call.fns.length === 1
//     //   // TODO: will probably have to change this once fn selection is done.
//     //   let fnTy = call.fns[0];
//     //   if (Object.keys(fnTy.args).length !== args.length) {
//     //     throw new Error('~~ Minecraft Villager sad noise :( ~~');
//     //   }
//     //   for (let ii = 0; ii < args.length; ii++) {
//     //     let argName = args[ii].dec.name;
//     //     let argMeta = metadata.var(argName);
//     //     if (argMeta.dec !== args[ii].dec) {
//     //       throw new Error('invalid call state: arg ref and var def mismatch');
//     //     }
//     //     let paramTy = Object.values(fnTy.args)[ii].ty;
//     //     argMeta.constraints.push(paramTy);
//     //   }
//     // }
//     stmts.push(call);

//     return stmts;
//   }

//   inline(amm: Output, assignName?: string, assignTy?: Type) {
//     TODO('inline fns?');
//   }

//   exprTy(): Type {
//     return this.retTy;
//   }
// }

export class Closure extends Stmt {
  exprTy(): Type {
    return TODO('closures')
  }

  inline(amm: Output) {
    TODO('closure inlining');
  }
}

export class Cond extends Stmt {
  static fromConditionalsAst(ast: LPNode, metadata: MetaData): Stmt[] {
    return TODO('build conditionals');
  }

  exprTy(): Type {
    throw new Error(`conditionals can't be used as expressions`);
  }

  inline(amm: Output) {
    TODO('conditional inlining');
  }
}

export class Dec extends Stmt {
  private __ammName: string
  mutable: boolean
  name: string
  ty: Type
  val: Stmt

  get ammName(): string {
    return this.__ammName;
  }

  set ammName(to: string) {
    this.__ammName = to;
  }

  constructor(
    ast: LPNode,
    mutable: boolean,
    name: string,
    ty: Type | null = null,
    val: Stmt,
  ) {
    super(ast);
    this.mutable = mutable;
    this.name = name;
    this.ty = ty !== null ? ty : Type.generate();
    this.val = val;
    this.ammName = genName();
  }

  static fromAst(ast: LPNode, metadata: MetaData): Stmt[] {
    let stmts: Stmt[] = [];
    let work: LPNode = null;
    let mutable: boolean;
    if (ast.has('constdeclaration')) {
      work = ast.get('constdeclaration');
      mutable = false;
    } else {
      work = ast.get('letdeclaration');
      mutable = true;
    }
    const name = work.get('variable').t;
    let ty: Type = null;
    if (work.has('typedec')) {
      const tyName = work.get('typedec').get('fulltypename');
      ty = Type.getFromTypename(tyName, metadata.scope);
    }
    stmts.push(...Stmt.fromAssignables(work.get('assignables'), metadata));
    let dec = stmts.pop();
    if (dec instanceof Dec) {
      metadata.variables[name] = metadata.var(dec.name);
      metadata.variables[dec.name] = null;
      dec.ty.constrain(ty);
      dec.mutable = mutable;
      dec.name = name;
    } else if (dec instanceof Ref) {
      const upTy = dec.ty;
      dec = new Dec(
        ast,
        mutable,
        name,
        ty,
        dec,
      );
      (dec as Dec).ty.constrain(upTy);
      metadata.define(dec as Dec);
    } else {
      throw new Error(`Can't get declaration value from most recent node: ${dec}`);
    }
    stmts.push(dec);
    return stmts;
  }

  static generate(stmt: Stmt) {
    return new Dec(
      stmt.ast,
      false,
      genName(),
      null,
      stmt,
    );
  }

  ref(): Ref {
    return new Ref(this.ast, this);
  }

  exprTy(): Type {
    throw new Error(`declarations can't be used as expressions`);
  }

  inline(amm: Output) {
    const name = this.ammName;
    let ty: Builtin;
    try {
      ty = this.ty.breakdown();
    } catch (e) {
      console.log('~~~', this);
      throw e;
    }
    if (this.val instanceof Call) {
      const kind = this.mutable ? 'let' : 'const';
      this.val.inline(amm, name, ty, kind);
    } else if (this.val instanceof Lit) {
      // don't copy the global value, just use it whenever this declaration is used
      this.__ammName = amm.global('const', this.val.ty.breakdown(), this.val.val);
    } else {
      throw new Error(`unexpected expression: ${this.val}`);
    }
  }
}

export class FnArg extends Dec {
  get ammName(): string {
    if (super.val != null) {
      if (!(super.val instanceof Ref)) {
        throw new Error(`expected fn arg to be set to a reference (here it is: ${super.val})`);
      }
      return super.val.ammName;
    } else {
      return this.name;
    }
  }

  set ammName(to: string) {
    if (super.val === null) {
      throw new Error(`cannot rename variables that aren't being inlined`);
    } else {
      super.ammName = to;
    }
  }

  constructor(
    ast: LPNode,
    name: string,
    ty: Type,
  ) {
    super(ast, true, name, ty, null);
    if (ty === null) {
      TODO('function params without a type specified');
    }
  }

  static fromArgAst(ast: LPNode, metadata: MetaData): FnArg {
    let name = ast.get('variable').t;
    let typename = ast.get('fulltypename');
    let argTy = Type.getFromTypename(typename, metadata.scope);
    if (argTy === null) {
      TODO('args with implicit types');
    } else if (!(argTy instanceof Type)) {
      throw new Error(`Function argument is not a valid type: ${typename.t}`);
    }
    const arg = new FnArg(ast, name, argTy);
    metadata.define(arg);
    const dec = metadata.var(arg.name);
    if (dec !== arg) {
      throw new Error('ugggghhhhh');
    }
    return arg;
  }

  ammOut(): [string, Builtin] {
    return TODO('TODO:')
  }
}

export class Emit extends Stmt {
  event: Event
  emitVal: Ref | null

  constructor(
    ast: LPNode,
    event: Event,
    emitVal: Ref | null,
  ) {
    super(ast);
    this.event = event;
    this.emitVal = emitVal;
  }

  static fromAst(ast: LPNode, metadata: MetaData): Stmt[] {
    let stmts: Stmt[] = [];
    if (ast.get('retval').has()) {
      let emitVal = ast.get('retval').get('assignables');
      let emitValSplit = Stmt.fromAssignables(emitVal, metadata);
      stmts.push(...emitValSplit);
    }
    const eventName = ast.get('eventname').t;
    const event = metadata.scope.deepGet(eventName);
    if (event === null) {
      throw new Error(`event ${eventName} not defined`);
    } else if (!(event instanceof Event)) {
      throw new Error(`cannot emit to non-events (${eventName} is not an event)`);
    }
    const emitVal = stmts[stmts.length - 1];
    if (!(emitVal instanceof Dec) && !(emitVal instanceof Ref)) {
      throw new Error('no declaration or reference created for emit value');
    }
    const emitRef = emitVal.ref();
    const emit = new Emit(ast, event, emitRef)
    emit.emitVal.dec.ty.constrain(event.eventTy);
    stmts.push(emit);
    return stmts;
  }

  pushAMM(indent: string, output: string) {
    output.concat(
      indent,
      'emit ',
      this.event.ammName,
      ' ',
      this.emitVal.ammName,
    );
  }

  exprTy(): Type {
    throw new Error(`emits can't be used as expressions`);
  }

  inline(amm: Output) {
    amm.emit(this.event.ammName, this.emitVal.ammName);
  }
}

export class Exit extends Stmt {
  exitVal: Ref | null

  constructor(
    ast: LPNode,
    exitVal: Ref | null,
  ) {
    super(ast);
    this.exitVal = exitVal;
  }

  static fromAst(ast: LPNode, metadata: MetaData): Stmt[] {
    let stmts: Stmt[] = [];
    if (ast.get('retval').has()) {
      let exitValAst = ast.get('retval').get('assignables');
      let exitValSplit = Stmt.fromAssignables(exitValAst, metadata);
      stmts.push(...exitValSplit);
      const exitVal = stmts[stmts.length - 1];
      if (!(exitVal instanceof Dec) && !(exitVal instanceof Ref)) {
        throw new Error('no declaration or reference created for emit value');
      }
      const exitRef = exitVal.ref();
      stmts.push(new Exit(ast, exitRef));
      metadata.retConstraints.push(exitRef.ty);
    } else {
      stmts.push(new Exit(ast, null));
      metadata.retConstraints.push(opcodes().get('void'));
    }
    return stmts;
  }

  exprTy(): Type {
    throw new Error(`returns can't be used as expressions`);
  }

  inline(amm: Output) {
    if (!this.exitVal.ty.compatibleWithConstraint(opcodes().get('void'))) {
      amm.exit(this.exitVal.ammName);
    } else {
      amm.exit();
    }
  }
}

export class Lit extends Stmt {
  val: string
  ty: Type

  constructor(
    ast: LPNode,
    val: string,
    ty: Type | null,
  ) {
    super(ast);
    this.val = val;
    this.ty = ty !== null ? ty : Type.generate();
  }

  static fromConstantsAst(ast: LPNode, _metadata: MetaData): Lit {
    let val = ast.t.trim();
    let ty = null;
    if (ast.has('bool')) {
      ty = opcodes().get('bool');
    } else if (ast.has('str')) {
      ty = opcodes().get('string');
      val = val.replace(/^'|'$/g, '"');
    } else if (ast.has('num')) {
      if (val.indexOf('.') !== -1) {
        ty = Type.oneOf([
          'float32',
          'float64',
        ].map(t => opcodes().get(t)));
      } else {
        ty = Type.oneOf([
          'float32',
          'float64',
          'int8',
          'int16',
          'int32',
          'int64',
        ].map(t => opcodes().get(t)));
      }
    } else {
      throw new Error(`unrecognized constants node: ${ast}`);
    }
    return new Lit(ast, val, ty);
  }

  exprTy(): Type {
    return this.ty;
  }

  inline(_amm: Output) {
    throw new Error('literals cannot be statements in AMM');
  }
}

export class Ref extends Stmt {
  dec: Dec

  get ammName(): string {
    return this.dec.ammName;
  }

  get ty(): Type {
    return this.dec.ty;
  }

  constructor(
    ast: LPNode,
    dec: Dec,
  ) {
    super(ast);
    this.dec = dec;
  }

  ref(): Ref {
    return this;
  }

  exprTy(): Type {
    return this.dec.ty;
  }

  inline(_amm: Output) {
    throw new Error('references cannot be statements in AMM');
  }
}
