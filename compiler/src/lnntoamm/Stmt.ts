import { LPNode } from '../lp';
import Output from './Amm';
import Event from './Event';
import Expr, { Ref } from './Expr';
import opcodes from './opcodes';
import Scope from './Scope';
import Type from './Types';
import { genName, TODO } from './util';

export class MetaData {
  scope: Scope;
  variables: { [name: string]: VarDef };
  retTy: Type;

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
  ast: LPNode;

  constructor(ast: LPNode) {
    this.ast = ast;
  }

  /**
   * @returns true if more cleanup might be required
   */
  abstract cleanup(scope: Scope): boolean;
  abstract inline(amm: Output): void;

  static fromAst(ast: LPNode, metadata: MetaData): Stmt[] {
    const stmts = [];
    if (ast.has('assignables')) {
      const [generatedStmts, expr] = Expr.fromAssignablesAst(
        ast.get('assignables').get('assignables'),
        metadata,
      );
      stmts.push(...generatedStmts, Dec.gen(expr, metadata));
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

export class Assign extends Stmt {
  upstream: VarDef;
  expr: Expr;

  constructor(ast: LPNode, upstream: VarDef, expr: Expr) {
    super(ast);
    this.upstream = upstream;
    this.expr = expr;
    if (this.upstream.immutable) {
      throw new Error(
        `cannot reassign to ${this.upstream.name} since it was declared as a const`,
      );
    }
  }

  static fromAssignments(ast: LPNode, metadata: MetaData): Stmt[] {
    const stmts: Stmt[] = [];
    const name = ast.get('varn').t;
    const upstream = metadata.get(name);
    const [generated, expr] = Expr.fromAssignablesAst(ast, metadata);
    upstream.ty.constrain(expr.ty, metadata.scope);
    stmts.push(...generated, new Assign(ast, upstream, expr));
    return stmts;
  }

  cleanup(scope: Scope): boolean {
    const didWork = this.expr.cleanup();
    this.upstream.ty.constrain(this.expr.ty, scope);
    return didWork;
  }

  inline(amm: Output) {
    this.expr.inline(
      amm,
      '',
      this.upstream.ammName,
      this.upstream.ty.instance(),
    );
  }
}

class Cond extends Stmt {
  static fromConditionals(_ast: LPNode, _metadata: MetaData): Stmt[] {
    return TODO('conditionals');
  }

  cleanup(): boolean {
    return TODO('conditionals');
  }

  inline(_amm: Output) {
    TODO('conditionals');
  }
}

export abstract class VarDef extends Stmt {
  immutable: boolean;
  name: string;
  ty: Type;

  abstract get ammName(): string;

  constructor(ast: LPNode, immutable: boolean, name: string, ty: Type) {
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
  private __ammName = '<UNSET>';
  expr: Expr;

  get ammName(): string {
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
    const stmts: Stmt[] = [];
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
    const [generated, expr] = Expr.fromAssignablesAst(
      work.get('assignables'),
      metadata,
    );
    let ty: Type = expr.ty;
    if (work.has('typedec')) {
      const tyName = work.get('typedec').get('fulltypename');
      const found = Type.getFromTypename(tyName, metadata.scope);
      // if the type hint is an interface, then all we have to do
      // is ensure that the expr's ty matches the interface
      const duped = found.dupIfNotLocalInterface();
      if (duped === null) {
        ty = found;
        ty.constrain(expr.ty, metadata.scope);
      } else {
        ty.constrain(duped, metadata.scope);
      }
    }
    const dec = new Dec(ast, immutable, name, ty, expr);
    stmts.push(...generated, dec);
    metadata.define(dec);
    return stmts;
  }

  static gen(expr: Expr, metadata: MetaData): Dec {
    const ty = Type.generate();
    ty.constrain(expr.ty, metadata.scope);
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

  cleanup(scope: Scope): boolean {
    const didWork = this.expr.cleanup();
    this.ty.constrain(this.expr.ty, scope);
    return didWork;
  }

  inline(amm: Output) {
    // refs don't escape the current scope and this only happens 1x/scope,
    // so this is fine
    this.__ammName = genName();
    this.expr.inline(
      amm,
      this.immutable ? 'const' : 'let',
      this.ammName,
      this.ty.instance(),
    );
  }
}

export class FnParam extends VarDef {
  private __assigned: Ref | null;

  get ammName(): string {
    if (this.__assigned === null) {
      return this.name;
    } else {
      return this.__assigned.ammName;
    }
  }

  constructor(ast: LPNode, name: string, ty: Type) {
    super(
      ast,
      false, // default to mutable for fn params
      name,
      ty,
    );
    this.__assigned = null;
  }

  static fromArgAst(ast: LPNode, metadata: MetaData): FnParam {
    const name = ast.get('variable').t;
    const typename = ast.get('fulltypename');
    let paramTy = Type.getFromTypename(typename, metadata.scope);
    if (paramTy === null) {
      paramTy = Type.generate();
      TODO('args with implicit types are not supported yet');
    } else if (!(paramTy instanceof Type)) {
      throw new Error(`Function parameter is not a valid type: ${typename.t}`);
    }
    const duped = paramTy.dupIfNotLocalInterface();
    if (duped !== null) {
      metadata.scope.put(duped.name, duped);
      paramTy = duped;
    }
    const param = new FnParam(ast, name, paramTy);
    metadata.define(param);
    return param;
  }

  assign(to: Ref, scope: Scope) {
    this.__assigned = to;
    this.ty.tempConstrain(to.ty, scope);
  }

  cleanup(): boolean {
    return false;
  }

  inline(_amm: Output) {
    throw new Error(`function parameters shouldn't be inlined`);
  }

  unassign() {
    this.__assigned = null;
    this.ty.resetTemp();
  }
}

class Emit extends Stmt {
  event: Event;
  emitVal: Ref | null;

  constructor(ast: LPNode, event: Event, emitVal: Ref | null) {
    super(ast);
    this.event = event;
    this.emitVal = emitVal;
  }

  static fromEmits(ast: LPNode, metadata: MetaData): Stmt[] {
    const stmts: Stmt[] = [];
    let emitRef: Ref | null = null;
    if (ast.get('retval').has()) {
      const emitVal = ast.get('retval').get('assignables');
      const [generated, expr] = Expr.fromAssignablesAst(emitVal, metadata);
      stmts.push(...generated);
      if (expr instanceof Ref) {
        emitRef = expr;
      } else {
        const emitDec = Dec.gen(expr, metadata);
        stmts.push(emitDec);
        emitRef = emitDec.ref();
      }
    }
    const eventName = ast.get('eventname').t;
    const event = metadata.scope.deepGet(eventName);
    if (event === null) {
      throw new Error(`event ${eventName} not defined`);
    } else if (!(event instanceof Event)) {
      throw new Error(
        `cannot emit to non-events (${eventName} is not an event)`,
      );
    } else if (emitRef !== null) {
      emitRef.ty.constrain(event.eventTy, metadata.scope);
    }
    stmts.push(new Emit(ast, event, emitRef));
    if (!event.eventTy.compatibleWithConstraint(emitRef.ty, metadata.scope)) {
      throw new Error(
        `cannot emit value of type ${emitRef.ty.name} to event ${event.name} because it requires ${event.eventTy.name}`,
      );
    }
    return stmts;
  }

  cleanup(scope: Scope): boolean {
    if (!this.event.eventTy.compatibleWithConstraint(this.emitVal.ty, scope)) {
      throw new Error(
        `cannot emit value of type ${this.emitVal.ty.name} to event ${this.event.name} because it requires ${this.event.eventTy.name}`,
      );
    }
    return false;
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
  ret: Ref | null;
  fnRetTy: Type;

  constructor(ast: LPNode, ret: Ref | null, fnRetTy: Type) {
    super(ast);
    this.ret = ret;
    this.fnRetTy = fnRetTy;
  }

  static fromExits(ast: LPNode, metadata: MetaData): Stmt[] {
    const stmts: Stmt[] = [];
    if (ast.get('retval').has()) {
      const exitValAst = ast.get('retval').get('assignables');
      const [generated, expr] = Expr.fromAssignablesAst(exitValAst, metadata);
      const retVal = Dec.gen(expr, metadata);
      stmts.push(
        ...generated,
        retVal,
        new Exit(ast, retVal.ref(), metadata.retTy),
      );
      metadata.retTy.constrain(expr.ty, metadata.scope);
    } else {
      stmts.push(new Exit(ast, null, opcodes().get('void')));
      metadata.retTy.constrain(opcodes().get('void'), metadata.scope);
    }
    return stmts;
  }

  cleanup(scope: Scope): boolean {
    this.fnRetTy.constrain(this.ret.ty, scope);
    return false;
  }

  inline(amm: Output) {
    if (
      !this.ret.ty.compatibleWithConstraint(opcodes().get('void'), opcodes())
    ) {
      amm.exit(this.ret.ammName);
    } else {
      amm.exit();
    }
  }
}
