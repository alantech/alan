import { LPNode } from '../lp';
import Output from './Amm';
import Event from './Event';
import Expr, { Ref } from './Expr';
import opcodes from './opcodes';
import Scope from './Scope';
import Type from './Types';
import { genName, TODO } from './util';

/**
 * Metadata for the current statement's context. This allows it to
 * get references to other Stmts that are defined in the same scope
 * or in outer scopes, the containing scope, and the return type of
 * the containing function. This makes accessing other Stmts an O(1)
 * operation instead of O(n).
 */
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

/*
Reassignment. Honestly, this can all be moved into `Dec` - I thought there
would be some utility in keeping them separate but I don't see it. Just make
sure that if it's a reassignment then it needs *the same Type reference* as the
upstream `Dec`'s (ie `this.type = upstream.type`)

I recommend keeping the `Assign` class name
*/
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
    const [generated, expr] = Expr.fromAssignablesAst(
      ast.get('assignables'),
      metadata,
    );
    upstream.ty.constrain(expr.ty, metadata.scope);
    stmts.push(...generated, new Assign(ast, upstream, expr));
    return stmts;
  }

  cleanup(scope: Scope): boolean {
    const upTy = this.upstream.ty;
    const didWork = this.expr.cleanup(upTy);
    upTy.constrain(this.expr.ty, scope);
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

/*
A thought, before going into it: this would be a lot easier if conditionals
were expressions instead of statements. The design I made is inspired by how
they're done in FP langs, but it's difficult to achieve perfectly and has
resulted in some hacky behavior that should be investigated before attempting
to use in any AVM-based FP langs.

TODO: the implementation of this will be much less hacky if amm generation
was converted to the visitor pattern

Conditionals require a bit of work. Here, a few things need to happen:

1. Create a table that matches conditions to functions/closures to execute.
```
if foo { closure 1 }
else if bar { closure 2 }
else { closure 3 }
// above gets transformed into:
[
  [foo, closure 1],
  [bar, closure 2],
  [true, closure 3],
]
```

2. The functions for each branch must have 0 parameters and all return the
same type, which is equal to their internal type wrapped in a `Maybe`.
```
fn foo() {
  if bar() {
    // closure 1
    return 0;
  } else if baz() {
    // closure 2
    return 1;
  } else {
    // closure 3
    'hello world'.print();
  }
  return 2;
}
// closure 1 gets transformed into:
fn { return some(0); }
// closure 2 gets transformed into:
fn { return some(1); }
// closure 3 gets transformed into:
fn {
  'hello world'.print();
  return none();
}
```

3. Create an instance of an Opaque `CondTable` type (I created the
`condtable` opcode while working on this, which I recommend doing).

4. For each branch, insert the opcode `condfn` with the first argument
being the CondTable. The second argument should be the condition being
used to determine if the branch gets executed. The third argument is
the closure to execute.

5. Insert the opcode `execcond` after all branches have been passed,
with the only argument being the CondTable. The return type is the
return type of all the branches.

6. If the surrounding function guarantees a returned value, then unwrap
the returned Maybe.

---
On the opcode implementation side, `condfn` should insert the closure
into the CondTable if the condition is true and the CondTable is empty.
*/
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
  // this name results in double negatives :)
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

/*
Variable declaration. In order to do the intended mutation tracking,
we might actually have to associate it with the `Dec`'s type like in
Rust. This can be done, but might be fairly complex to achieve.

This class has to do a lot of constraining to make sure that everything
agrees on types for various variables.
*/
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
      const duped = found.dup();
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
    const dec = new Dec(
      expr.ast,
      false, // default to mutable in case of eg builder pattern
      genName(),
      expr.ty,
      expr,
    );
    metadata.define(dec);
    return dec;
  }

  cleanup(scope: Scope): boolean {
    const didWork = this.expr.cleanup(this.ty);
    this.ty.constrain(this.expr.ty, scope);
    return didWork;
  }

  inline(amm: Output) {
    // refs don't escape the current scope and this only happens 1x/scope,
    // so this is fine
    this.__ammName = genName();
    try {
      this.expr.inline(
        amm,
        this.immutable ? 'const' : 'let',
        this.ammName,
        this.ty.instance(),
      );
    } catch (e) {
      console.dir(this, { depth: 6 });
      throw e;
    }
  }
}

/*
FnParams aren't syntactic statements, so it's kinda weird to think
of this class as being a subclass of Stmt. Despite not actually being
included in amm output (with a couple exceptions, of course), there
are enough similarities that it's convenient to keep it here.
*/
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

  static fromArgAst(
    ast: LPNode,
    metadata: MetaData,
    fnSigScope: Scope,
  ): FnParam {
    const name = ast.get('variable').t;
    const typename = ast.get('fulltypename');
    let paramTy = Type.getFromTypename(typename, fnSigScope, { isTyVar: true });
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

/*
This class just has to make sure that a variable's type matches
the emitted event's type. If the event is `void`, then there won't
be a variable to emit.
*/
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

/*
This statement ensures that the given variable's type matches the type
of the containing function.
*/
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
