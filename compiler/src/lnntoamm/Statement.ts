import { LPNode, NulLP } from "../lp";
import Event from "./Event";
import Fn from "./Fn";
import opcodes from "./opcodes";
import Scope from "./Scope";
import { Constraint } from "./typecheck";
import { Interface, Type } from "./Types";
import { TODO } from "./util";

interface Stmt {
  split(): Statement[];
  constrain(constraints: Constraint[]): void;
}

// very similar to a Scope, but there's no exports and might
// contain more than just declarations? idk yet
export class StatementMetaData {
  constructor(
    private __upstream: StatementMetaData = null,
    private __declarations: {[name: string]: Declaration} = {},
  ) {}

  getDec(name: string): Declaration {
    const local = this.__declarations[name];
    if (local === undefined && this.__upstream !== null) {
      return this.__upstream.getDec(name);
    } else {
      return local;
    }
  }
}

export default class Statement {
  node: LPNode
  content: Stmt

  constructor(
    node: LPNode,
    content: Stmt,
  ) {
    this.node = node;
    this.content = content;
  }

  static fromAst(ast: LPNode, scope: Scope, metadata: StatementMetaData): Statement {
    console.log('parsing statement:\n', ast)
    let content: Stmt = null;
    if (ast.has('assignables')) {
      content = Assignable.fromAst(ast.get('assignables'), scope, metadata);
    } else if (ast.has('assignments')) {
      content = Assignment.fromAst(ast.get('assignments'), scope, metadata);
    } else if (ast.has('conditionals')) {
      content = Conditional.fromAst(ast.get('conditionals'), scope, metadata);
    } else if (ast.has('declarations')) {
      content = Declaration.fromAst(ast.get('declarations'), scope, metadata);
    } else if (ast.has('emits')) {
      content = Emit.fromAst(ast.get('emits'), scope, metadata);
    } else if (ast.has('exits')) {
      content = Exit.fromAst(ast.get('exits'), scope, metadata);
    }
    return new Statement(ast, content);
  }

  transform(): Statement[] {
    return this.content.split();
  }

  constrain(constraints: Constraint[]) {
    this.content.constrain(constraints);
  }
}

class Assignable implements Stmt {
  ast: LPNode
  scope: Scope
  metadata: StatementMetaData

  constructor(
    ast: LPNode,
    scope: Scope,
    metadata: StatementMetaData,
  ) {
    this.ast = ast;
    this.scope = scope;
    this.metadata = metadata;
  }

  static fromAst(ast: LPNode, scope: Scope, metadata: StatementMetaData): Assignable {
    return new Assignable(ast, scope, metadata);
  }

  split(): Statement[] {
    let split = [];
    console.log(this.ast);
    for (let assignable of this.ast.getAll()) {
      if (assignable.has('baseassignablelist')) {
        Assignable.splitBaseAssignableList(assignable.get('baseassignablelist'), split);
      } else {
        TODO('operator support in splitting');
      }
    }
    return split;
  }

  constrain(constraints: Constraint[]) {
  }

  static splitBaseAssignableList(list: LPNode, split: Statement[]) {
    for (let assignable of list.getAll()) {
      assignable = assignable.get('baseassignable');
      if (assignable.has('objectliterals')) {
        TODO('objectliterals');
      } else if (assignable.has('functions')) {
        TODO('functions');
      } else if (assignable.has('fncall')) {
        TODO('fncall');
      } else if (assignable.has('variable')) {
        TODO('variable');
      } else if (assignable.has('constants')) {
        TODO('constants');
      } else if (assignable.has('methodsep')) {
        TODO('methodsep');
      }
    }
  }
}

class Assignment implements Stmt {
  ast: LPNode
  upstream: Declaration
  assignable: Assignable
  final: Stmt | null

  constructor(
    ast: LPNode,
    upstream: Declaration,
    assignable: Assignable,
    final: Stmt | null = null,
  ) {
    this.ast = ast;
    this.upstream = upstream;
    this.assignable = assignable;
    this.final = final;
  }

  static fromAst(ast: LPNode, scope: Scope, metadata: StatementMetaData): Assignment {
    const assignName = ast.get('varn').t;
    // can only assign to declarations, nothing in Scope.
    if (metadata.getDec(assignName) === undefined) {
      throw new Error(`${assignName} is not defined`);
    }
    const upstream = metadata.getDec(assignName);
    if (!upstream.mutable) {
      throw new Error(`can't reassign to ${assignName} (not a let variable)`);
    }
    const assignable = Assignable.fromAst(ast.get('assignables'), scope, metadata);
    return new Assignment(ast, upstream, assignable);
  }

  split(): Statement[] {
    let split = this.assignable.split();
    let last = split.pop();
    if (last instanceof Declaration) {
      this.final = last.final;
    } else {
      throw new Error('Invalid assignment state');
    }
    split.push(new Statement(this.ast, this));
    return split;
  }

  constrain(constraints: Constraint[]) {
  }
}

class Conditional implements Stmt {
  branches: Array<[[Assignable | true, LPNode], [Fn, LPNode]]>

  constructor(
    branches: Array<[[Assignable | true, LPNode], [Fn, LPNode]]>,
  ) {
    this.branches = branches;
  }

  static fromAst(ast: LPNode, scope: Scope, metadata: StatementMetaData): Conditional {
    let branches: Array<[[Assignable | true, LPNode], [Fn, LPNode]]> = [];
    do {
      let condAst: LPNode;
      let cond: Assignable | true;
      if (ast.has('assignables')) {
        condAst = ast.get('assignables');
        cond = Assignable.fromAst(condAst, scope, metadata);
      } else {
        condAst = ast.get('elsen');
        cond = true;
      }

      let thenAst = ast.get('blocklike');
      let then: Fn;
      if (thenAst.has('functions')) {
        then = Fn.fromFunctionsAst(thenAst.get('functions'), scope, new StatementMetaData(metadata));
      } else if (thenAst.has('functionbody')) {
        then = Fn.fromFunctionbody(thenAst.get('functionbody'), scope, new StatementMetaData(metadata));
      } else {
        // note: do not pass in the metadata, since functions shouldn't be able to
        // reference variables that aren't in their defined scope.
        thenAst = thenAst.get('fnname');
        const thenFnName = thenAst.t.trim();
        let inScope = scope.get(thenFnName)
        if (inScope === null) {
          throw new Error(`${thenFnName} is not defined`);
        } else if (!(inScope instanceof Fn)) {
          throw new Error(`${thenFnName} is not a function`);
        }
        then = inScope;
      }
      if (Object.keys(then.args).length !== 0) {
        throw new Error(`functions conditionally called cannot require arguments`);
      }
      // don't check the return type yet - we'll do that when type-checking
      branches.push([[cond, condAst], [then, thenAst]]);
    } while (ast.has('blocklike'));
    return new Conditional(branches);
  }

  split(): Statement[] {
    let split = [];

    for (let [[cond, condAst], [then, thenAst]] of this.branches) {
      TODO('conditionals');
    }

    return split;
  }

  constrain(constraints: Constraint[]) {
  }
}

class Declaration implements Stmt {
  ast: LPNode
  mutable: boolean
  name: string
  ty: Type | Interface | null
  assignable: Assignable
  final: Stmt | null

  constructor(
    ast: LPNode,
    mutable: boolean,
    name: string,
    ty: Type | Interface | null,
    assignable: Assignable,
    final: Stmt | null = null,
  ) {
    this.ast = ast;
    this.mutable = mutable;
    this.name = name;
    this.ty = ty;
    this.assignable = assignable;
    this.final = final;
  }

  static fromAst(ast: LPNode, scope: Scope, metadata: StatementMetaData): Declaration {
    let work: LPNode;
    let mutable: boolean;
    if (ast.has('constdeclaration')) {
      work = ast.get('constdeclaration');
      mutable = false;
    } else {
      work = ast.get('letdeclaration');
      mutable = true;
    }
    const name = work.get('variable').t;
    const assignables = Assignable.fromAst(work.get('assignables'), scope, metadata);
    let ty = null;
    if (work.has('typedec')) {
      work = work.get('typdec');
      ty = Type.getFromTypename(work.get('fulltypename'), scope);
    }
    const dec = new Declaration(
      ast,
      mutable,
      name,
      ty,
      assignables,
    );
    if (metadata.getDec(dec.name) !== undefined) {
      throw new Error(`cannot shadow variable names`);
    }
    return dec;
  }

  ref(): VarRef {
    return new VarRef(this.ast, this);
  }

  split(): Statement[] {
    let split = this.assignable.split();
    let last = split.pop();
    if (last instanceof Declaration) {
      // we only inherit the final value, we don't care about anything else
      this.final = last.final;
    } else {
      throw new Error('Invalid declaration state');
    }
    split.push(new Statement(this.ast, this));
    return split;
  }

  constrain(constraints: Constraint[]) {
  }
}

class Emit implements Stmt {
  ast: LPNode
  event: Event
  assignable: Assignable | null
  final: VarRef | null

  constructor(
    ast: LPNode,
    event: Event,
    assignable: Assignable | null,
    final: VarRef | null = null,
  ) {
    this.ast = ast;
    this.event = event;
    this.assignable = assignable;
    this.final = final;
  }

  static fromAst(ast: LPNode, scope: Scope, metadata: StatementMetaData): Emit {
    const eventName = ast.get('eventname').t.trim();
    const event = scope.get(eventName);
    if (event === null) {
      throw new Error(`event not defined: ${eventName}`);
    } else if (!(event instanceof Event)) {
      throw new Error(`cannot emit to non-events (${eventName} is not an event)`);
    }

    let assignables = null;
    if (ast.get('retval').has()) {
      assignables = Assignable.fromAst(
        ast.get('retval').get().get('assignables'),
        scope,
        metadata,
      );
    }

    return new Emit(
      ast,
      event,
      assignables,
    );
  }

  split(): Statement[] {
    let assignables = this.assignable.split();
    let emitValDec = assignables[assignables.length - 1];
    if (emitValDec.content instanceof Declaration) {
      this.final = emitValDec.content.ref();
    } else {
      throw new Error(`Invalid emit state`);
    }
    assignables.push(new Statement(this.ast, this));
    return assignables;
  }

  constrain(constraints: Constraint[]) {
    if (this.assignable === null) {
      if (this.event.eventTy !== opcodes().get('void')) {
        throw new Error(`Must emit a value to non-void events`);
      }
    } else if (this.final !== null) {
      // constrain the variable we're emitting to the type of the event
    } else {
      throw new Error(`Something's not quite right - emit statement with no varref`)
    }
  }
}

export class Exit implements Stmt {
  ast: LPNode
  assignable: Assignable | null
  final: VarRef | null

  constructor(
    ast: LPNode,
    assignable: Assignable | null,
    final: VarRef | null = null,
  ) {
    this.ast = ast;
    this.assignable = assignable;
    this.final = final;
  }

  static fromAst(ast: LPNode, scope: Scope, metadata: StatementMetaData): Exit {
    ast = ast.get('retval');
    let assignables = null;
    if (ast.has()) {
      assignables = ast.get('assignables');
    }
    return new Exit(
      ast,
      Assignable.fromAst(assignables, scope, metadata),
    );
  }

  split(): Statement[] {
    let assignables = this.assignable.split();
    let retValDec = assignables[assignables.length - 1];
    if (retValDec.content instanceof Declaration) {
      this.final = retValDec.content.ref();
    } else {
      throw new Error(`Invalid return state`);
    }
    assignables.push(new Statement(this.ast, this));
    return assignables;
  }

  constrain(constraints: Constraint[]) {
    if (this.assignable === null) {
      // it's a void value (naked return)
      constraints.push([null, opcodes().get('void')]);
    } else if (this.final !== null) {
      this.final.constrain(constraints, null);
    } else {
      throw new Error(`Something's not quite right - return statement with no varref`);
    }
  }
}

// ===================================================
// types that Assignables boil down to (shouldn't be used directly in a Statement)
class Closure implements Stmt {
  ast: LPNode
  fn: Fn

  constructor(
    ast: LPNode,
    fn: Fn,
  ) {
    this.ast = ast;
    this.fn = fn;
  }

  split(): Statement[] {
    return [new Statement(this.ast, this)];
  }

  constrain(constraints: Constraint[], name?: string) {
    TODO('fn types')
  }
}

class VarRef implements Stmt {
  ast: LPNode
  dec: Declaration

  constructor(
    ast: LPNode,
    dec: Declaration,
  ) {
    this.ast = ast;
    this.dec = dec;
  }

  split(): Statement[] {
    return [new Statement(this.ast, this)];
  }

  constrain(constraints: Constraint[], to?: string | Type | Interface) {
    if (to !== undefined) {
      if (this.dec.ty === null) {
        throw new Error(`${this.dec.name} doesn't have a type!`);
      }
      TODO('here')
    }
  }
}

class FnCall implements Stmt {
  ast: LPNode
  fn: Fn
  args: Declaration[]

  constructor(
    ast: LPNode,
    fn: Fn,
    args: Declaration[],
  ) {
    this.ast = ast;
    this.fn = fn;
    this.args = args;
  }

  split(): Statement[] {
    return [new Statement(this.ast, this)];
  }

  constrain(constraints: Constraint[], name?: string) {
    const params = Object.keys(this.fn.args);
    if (this.args.length < params.length) {
      throw new Error(`Not enough arguments passed to function ${this.fn.name}`);
    } else if (this.args.length > params.length) {
      throw new Error(`Too many arguments passed to function ${this.fn.name}`);
    }
    for (let i = 0; i < params.length; i++) {
      constraints.push([this.args[i].name, this.fn.args[params[i]]]);
    }
    if (name !== undefined) {
      constraints.push([name, this.fn.getReturnType()]);
    }
  }
}
