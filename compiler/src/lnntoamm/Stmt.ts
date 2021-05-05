import { LPNode } from '../lp';
import Output from './Amm';
import Event from './Event';
import Expr, { Ref } from './Expr';
import opcodes from './opcodes';
import Scope from './Scope';
import Type from './Types';
import { genName, TODO } from './util';

export class MetaData {
  scope: Scope
  variables: { [name: string]: VarDef }
  retTy: Type

  constructor(
    scope: Scope,
    retTy: Type,
    variables: { [name: string]: VarDef } = null,
  ) {
    this.scope = scope;
    this.retTy = retTy;
    this.variables = variables !== null ? variables : {};
  }

  get(name: string): VarDef | null {
    if (!this.variables.hasOwnProperty(name)) {
      return null;
    }
    return this.variables[name];
  }

  define(dec: VarDef) {
    if (this.get(dec.name) !== null) {
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

  abstract inline(amm: Output): void;

  static fromAst(ast: LPNode, metadata: MetaData): Stmt[] {
    let stmts = [];
    if (ast.has('assignables')) {
      let [ss, expr] = Expr.fromAssignablesAst(ast.get('assignments'), metadata);
      stmts.push(...ss, Dec.gen(expr, metadata));
    } else if (ast.has('assignments')) {
      stmts.push(...Assign.fromAssignments(ast.get('assignments'), metadata));
    } else if (ast.has('conditionals')) {
      stmts.push(...Cond.fromConditionals(ast.get('conditionals'), metadata));
    } else if (ast.has('declarations')) {
      stmts.push(...Dec.fromDeclarations(ast.get('declarations'), metadata));
    } else if (ast.has('emits')) {
      stmts.push(...Emit.fromEmits(ast.get('emits'), metadata));
    } else if (ast.has('exits')) {
      stmts.push(...Exit.fromExits(ast.get('exits'), metadata));
    } else {
      throw new Error(`unrecognized statement ast: ${ast}`);
    }
    return stmts;
  }
}

class Assign extends Stmt {
  upstream: VarDef
  expr: Expr

  constructor(
    ast: LPNode,
    upstream: VarDef,
    expr: Expr,
  ) {
    super(ast);
    this.upstream = upstream;
    this.expr = expr;
    if (this.upstream.immutable) {
      throw new Error(`cannot reassign to ${this.upstream.name} since it was declared as a const`);
    }
  }

  static fromAssignments(ast: LPNode, metadata: MetaData): Stmt[] {
    let stmts: Stmt[] = [];
    const name = ast.get('varn').t;
    const upstream = metadata.get(name);
    let [generated, expr] = Expr.fromAssignablesAst(ast, metadata);
    upstream.ty.constrain(expr.ty);
    stmts.push(...generated, new Assign(ast, upstream, expr));
    return stmts;
  }

  inline(amm: Output) {
    this.expr.inline(amm, '', this.upstream.ammName, this.upstream.ty.breakdown());
  }
}

class Cond extends Stmt {
  static fromConditionals(_ast: LPNode, _metadata: MetaData): Stmt[] {
    return TODO('conditionals');
  }

  inline(_amm: Output) {
    TODO('conditionals');
  }
}

export abstract class VarDef extends Stmt {
  immutable: boolean
  name: string
  ty: Type

  abstract get ammName(): string;

  constructor(
    ast: LPNode,
    immutable: boolean,
    name: string,
    ty: Type,
  ) {
    super(ast);
    this.immutable = immutable;
    this.name = name;
    this.ty = ty;
  }

  ref(): Ref {
    return new Ref(this);
  }
}

export class Dec extends VarDef {
  private __ammName: string = '';
  expr: Expr

  get ammName(): string {
    if (this.__ammName === '') {
      this.__ammName = genName();
    }
    return this.__ammName;
  }

  constructor(
    ast: LPNode,
    immutable: boolean,
    name: string,
    defTy: Type,
    expr: Expr,
  ) {
    super(ast, immutable, name, defTy);
    this.expr = expr;
  }

  static fromDeclarations(ast: LPNode, metadata: MetaData): Stmt[] {
    let stmts: Stmt[] = [];
    let work: LPNode;
    let immutable: boolean;
    if (ast.has('constdeclaration')) {
      work = ast.get('constdeclaration');
      immutable = true;
    } else {
      work = ast.get('letdeclaration');
      immutable = false;
    }
    const name = work.get('variable').t;
    let [generated, expr] = Expr.fromAssignablesAst(work.get('assignables'), metadata);
    let ty: Type = expr.ty;
    if (work.has('typedec')) {
      const tyName = work.get('typedec').get('fulltypename');
      ty = Type.getFromTypename(tyName, metadata.scope);
    }
    ty.constrain(expr.ty);
    let dec = new Dec(
      ast,
      immutable,
      name,
      ty,
      expr,
    );
    stmts.push(...generated, dec);
    metadata.define(dec);
    return stmts;
  }

  static gen(expr: Expr, metadata: MetaData): Dec {
    let ty = Type.generate();
    ty.constrain(expr.ty);
    const dec = new Dec(
      expr.ast,
      false, // default to mutable in case of eg builder pattern
      genName(),
      ty,
      expr,
    );
    metadata.define(dec);
    return dec;
  }

  inline(amm: Output) {
    this.expr.inline(
      amm,
      this.immutable ? 'const' : 'let',
      this.ammName,
      this.ty.breakdown(),
    );
  }
}

export class FnParam extends VarDef {
  private __assigned: Ref | null

  get ammName(): string {
    if (this.__assigned === null) {
      return this.name;
    } else {
      return this.__assigned.ammName;
    }
  }

  constructor(
    ast: LPNode,
    name: string,
    ty: Type,
  ) {
    super(
      ast,
      false, // default to mutable for fn params
      name,
      ty,
    );
    this.__assigned = null;
  }

  static fromArgAst(ast: LPNode, metadata: MetaData): FnParam {
    let name = ast.get('variable').t;
    let typename = ast.get('fulltypename');
    let paramTy = Type.getFromTypename(typename, metadata.scope);
    if (paramTy === null) {
      paramTy = Type.generate();
      TODO('args with implicit types are not supported yet');
    } else if (!(paramTy instanceof Type)) {
      throw new Error(`Function parameter is not a valid type: ${typename.t}`);
    }
    const param = new FnParam(ast, name, paramTy);
    metadata.define(param);
    return param;
  }

  assign(to: Ref) {
    this.__assigned = to;
  }

  inline(_amm: Output) {
    throw new Error(`function parameters shouldn't be inlined`);
  }

  unassign() {
    this.__assigned = null;
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

  static fromEmits(ast: LPNode, metadata: MetaData): Stmt[] {
    let stmts: Stmt[] = [];
    let emitRef: Ref | null = null;
    if (ast.get('retval').has()) {
      let emitVal = ast.get('retval').get('assignables');
      let [generated, expr] = Expr.fromAssignablesAst(emitVal, metadata);
      let emitDec = Dec.gen(expr, metadata);
      stmts.push(...generated, emitDec);
      emitRef = emitDec.ref();
    }
    const eventName = ast.get('eventname').t;
    const event = metadata.scope.deepGet(eventName);
    if (event === null) {
      throw new Error(`event ${eventName} not defined`);
    } else if (!(event instanceof Event)) {
      throw new Error(`cannot emit to non-events (${eventName} is not an event)`);
    } else if (emitRef !== null) {
      emitRef.ty.constrain(event.eventTy);
    }
    stmts.push(new Emit(ast, event, emitRef));
    if (!event.eventTy.compatibleWithConstraint(emitRef.ty)) {
      throw new Error(``)
    }
    return stmts;
  }

  inline(amm: Output) {
    if (!this.event.eventTy.eq(opcodes().get('void'))) {
      amm.emit(this.event.ammName, this.emitVal.ammName);
    } else {
      amm.emit(this.event.ammName);
    }
  }
}

export class Exit extends Stmt {
  ret: Ref | null

  constructor(
    ast: LPNode,
    ret: Ref | null,
  ) {
    super(ast);
    this.ret = ret;
  }

  static fromExits(ast: LPNode, metadata: MetaData): Stmt[] {
    let stmts: Stmt[] = [];
    if (ast.get('retval').has()) {
      let exitValAst = ast.get('retval').get('assignables');
      let [generated, expr] = Expr.fromAssignablesAst(exitValAst, metadata);
      let retVal = Dec.gen(expr, metadata);
      stmts.push(...generated, retVal, new Exit(ast, retVal.ref()));
      metadata.retTy.constrain(expr.ty);
    } else {
      stmts.push(new Exit(ast, null));
      metadata.retTy.constrain(opcodes().get('void'));
    }
    return stmts;
  }

  inline(amm: Output) {
    if (!this.ret.ty.compatibleWithConstraint(opcodes().get('void'))) {
      amm.exit(this.ret.ammName);
    } else {
      amm.exit();
    }
  }
}
