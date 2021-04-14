import { LPNode, NamedAnd, NamedOr, NulLP } from '../lp';
import Event from './Event';
import opcodes from './opcodes';
import Scope from './Scope';
import Type, { FunctionType } from './Types';
import { genName, TODO } from './util';

// the value is null if the type is to be inferred
export type Args = {[name: string]: FnArg};

export default class Fn {
  // null if it's an anonymous fn
  name: string | null
  ast: LPNode
  // the scope this function is defined in is the `par`
  scope: Scope
  args: Args
  // null if the type is to be inferred
  retTy: Type
  body: Stmt[]
  // not used by this class, but used by Statements
  metadata: MetaData
  fnType: FunctionType

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
    for (let argName of Object.keys(this.args)) {
      if (this.args[argName].ty === null) {
        this.args[argName].ty = Type.generate();
      }
    }
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
    // metadata: MetaData = null,
  ): Fn {
    // TODO: inheritance
    let metadata = new MetaData(scope);

    let work = ast;
    const name = work.get('optname').has() ? work.get('optname').get().t : null;
    let args: Args = {};
    if (work.get('optargs').has('arglist')) {
      let argAsts = [
        work.get('optargs').get('arglist'),
        ...work.get('optargs').get('arglist').get('cdr').getAll(),
      ];
      argAsts.forEach(argAst => FnArg.fromArgAst(argAst, metadata));
    }

    const retTy = work.get('optreturntype').has() ? work.get('optreturntype').get().get('fulltypename') : 'void';

    let body = [];
    let bodyAsts: LPNode | LPNode[] = work.get('fullfunctionbody');
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
      Type.getFromTypename(retTy, scope),
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
      opcodes().get('void'),
      body,
      metadata,
    );
  }

  getType(): FunctionType {
    return this.fnType;
  }

  constraints(): [VarMD[], Type[]] {
    return [Object.values(this.metadata.variables), this.metadata.retConstraints];
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
}

class VarMD {
  dec: Dec
  constraints: Type[]

  constructor(
    dec: Dec,
    constraints: Type[] = [],
  ) {
    this.dec = dec;
    this.constraints = constraints;
  }
}

class MetaData {
  scope: Scope
  variables: { [name: string]: VarMD }
  retConstraints: Type[]

  constructor(
    scope: Scope,
    variables: { [name: string]: VarMD } = null,
    retConstraints: Type[] = null,
  ) {
    this.scope = scope;
    this.variables = variables !== null ? variables : {};
    this.retConstraints = retConstraints !== null ? retConstraints : [];
  }

  var(name: string): VarMD {
    if (this.variables[name] == null) {
      return null;
    }
    return this.variables[name];
  }

  define(dec: Dec) {
    if (this.var(dec.name) !== null) {
      throw new Error(`Can't redefine value ${dec.name}`);
    }
    this.variables[dec.name] = new VarMD(dec);
  }
}

abstract class Stmt {
  ast: LPNode

  constructor(
    ast: LPNode,
  ) {
    this.ast = ast;
  }

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

    let asts: LPNode[] = ast.getAll();
    if (asts.length > 1) TODO('operators');
    asts = asts.length === 1 ? asts.pop().get('withoperators').get('baseassignablelist').getAll().map(a => a.get('baseassignable')) : [];
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
          stmts.push(Dec.generate(dec.dec.ref()));
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
            null,
            varName,
            next.get('fncall'),
            metadata,
          ));
          const call = stmts.pop();
          stmts.push(Dec.generate(call));
        } else if (next.has('methodsep')) {
          TODO('accesses/methods');
        } else {
          throw new Error(`unexpected token: expected dot or call, found ${next}`);
        }
      } else if (work.has('constants')) {
        work = work.get('constants');
        let dec = Dec.generate(Lit.fromConstantsAst(ast, metadata));
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

class Assign extends Stmt {
  upstream: Dec
  assignTo: Stmt

  constructor(
    ast: LPNode,
    upstream: Dec,
    assignTo: Stmt,
  ) {
    super(ast);
    this.upstream = upstream;
    this.assignTo = assignTo;
  }

  static fromAssignmentsAst(ast: LPNode, metadata: MetaData): Stmt[] {
    let stmts: Stmt[] = [];
    const name = ast.get('varn').t;
    if (metadata.var[name] === null) {}
    return stmts;
  }
}

class Call extends Stmt {
  fns: Fn[]
  args: Ref[]
  retTy: Type
  callTy: FunctionType

  constructor(
    ast: LPNode,
    fns: Fn[],
    args: Ref[],
    retTy: Type = null,
    callTy: FunctionType = null,
  ) {
    super(ast);
    fns = fns.filter(fn => Object.keys(fn.args).length === args.length)
    fns = fns.filter(fn => callTy.compatibleWithConstraint(fn.fnType));
    if (fns.length === 0) {
      throw new Error(`could not find function for call site \`${ast}\``)
    }
    this.fns = fns;
    this.args = args;
    if (retTy === null) {
      retTy = Type.generate();
    }
    if (callTy === null) {
      callTy = new FunctionType('CALL', args.map(r => r.ty), retTy);
    }
    if (callTy.retTy !== retTy) {
      throw new Error('errr');
    }
    this.retTy = retTy;
    this.callTy = callTy;
    const fnTypes = this.fns.map(fn => fn.fnType);
    this.callTy.callSelect = Type.oneOf(fnTypes);
  }

  static fromAsts(
    wholeAst: LPNode,
    accessed: Dec | Ref | null,
    fnName: string,
    fnCallAst: LPNode,
    metadata: MetaData,
  ): Stmt[] {
    let stmts: Stmt[] = [];

    fnCallAst = fnCallAst.get('assignablelist');
    let args: Ref[] = [
      fnCallAst.get('assignables'),
      ...fnCallAst.get('cdr').getAll().map(n => n.get('assignables')),
    ].map(ast => {
      stmts.push(...Stmt.fromAssignables(ast, metadata));
      let dec: Stmt = stmts[stmts.length - 1];
      if (!(dec instanceof Dec)) {
        throw new Error(`declaration not generated for arg ${ast.t.trim()}`);
      }
      return dec.ref();
    });
    if (accessed !== null) {
      args.unshift(accessed.ref());
    }

    let fns: Fn[] = [];
    let fromScope = metadata.scope.get(fnName);
    if (Array.isArray(fromScope) && fromScope.length > 0 && fromScope[0] instanceof Fn) {
      fns.push(...fromScope);
    }
    if (metadata.var(fnName) !== null) {
      TODO('closure calling')
    }

    stmts.push(new Call(
      wholeAst,
      fns,
      args,
    ))

    return stmts;
  }
}

class Closure extends Stmt {
}

class Cond extends Stmt {
  static fromConditionalsAst(ast: LPNode, metadata: MetaData): Stmt[] {
    return TODO('build conditionals');
  }
}

class Dec extends Stmt {
  mutable: boolean
  name: string
  ty: Type
  val: Stmt

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
    const last = stmts.pop();
    let val: Stmt;
    if (last instanceof Dec) {
      val = last.val;
    } else {
      throw new Error(`Can't get declaration value from most recent node (${last})`);
    }
    stmts.push(new Dec(ast, mutable, name, ty, val));
    return stmts;
  }

  static generate(stmt: Stmt) {
    return new Dec(
      stmt.ast,
      false,
      genName(),
      null, // TODO: getOutputName
      stmt,
    );
  }

  ref(): Ref {
    return new Ref(this.ast, this);
  }
}

class FnArg extends Dec {
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
    return arg;
  }
}

class Emit extends Stmt {
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
    let emitVal = stmts[stmts.length - 1];
    if (!(emitVal instanceof Dec) && !(emitVal instanceof Ref)) {
      throw new Error('no declaration or reference created for emit value');
    }
    stmts.push(new Emit(ast, event, emitVal.ref()))
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
}

class Exit extends Stmt {
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
      let exitVal = stmts[stmts.length - 1];
      if (!(exitVal instanceof Dec) && !(exitVal instanceof Ref)) {
        throw new Error('no declaration or reference created for emit value');
      }
      stmts.push(new Exit(ast, exitVal.ref()));
    } else {
      stmts.push(new Exit(ast, null));
    }
    return stmts;
  }

  pushAMM(indent: string, output: string) {
    output.concat(
      indent,
      'return',
      ...(this.exitVal !== null ? [' ', this.exitVal.ammName] : []),
    );
  }
}

class Lit extends Stmt {
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
    const val = ast.t.trim();
    let ty = null;
    if (ast.has('bool')) {
      ty = opcodes().get('bool');
    } else if (ast.has('str')) {
      ty = opcodes().get('string');
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
}

class Ref extends Stmt {
  dec: Dec

  get ammName(): string {
    return TODO('ref amm name');
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
}
