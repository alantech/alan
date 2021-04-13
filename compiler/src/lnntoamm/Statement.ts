import { LPNode } from "../lp";
import Event from "./Event";
import Fn from "./Fn";
import opcodes from "./opcodes";
import Scope from "./Scope";
import Type, { FunctionType } from "./Types";
import { genName, TODO } from "./util";

interface Stmt {
  ast: LPNode | LPNode[];
  split(): Statement[];
  constrain(metadata: StatementMetaData): void;
  getOutputType(): Type;
}

export class VarMetadata {
  dec: Declaration
  constraints: Type[]

  constructor(
    dec: Declaration,
    constraints: Type[] = [],
  ) {
    this.dec = dec;
    this.constraints = constraints;
  }
}

// very similar to a Scope, but there's no exports and might
// contain more than just declarations? idk yet
export class StatementMetaData {
  outputConstraints: Type[]
  vars: {[name: string]: VarMetadata}

  constructor(
    private __upstream: StatementMetaData = null,
    vars: {[name: string]: VarMetadata} = {},
  ) {
    this.outputConstraints = [];
    this.vars = vars;
  }

  var(name: string): VarMetadata {
    const local = this.vars[name];
    if (local == undefined) {
      if (this.__upstream !== null) {
        return this.__upstream.var(name);
      } else {
        return null;
      }
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
    } else {
      throw new Error(`invalid statement ast`);
    }
    return new Statement(ast, content);
  }

  transform(): Statement[] {
    return this.content.split();
  }

  constrain(metadata: StatementMetaData) {
    this.content.constrain(metadata);
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

    for (let assignable of this.ast.getAll().map(a => a.get('withoperators'))) {
      if (assignable.has('baseassignablelist')) {
        this.splitBaseAssignableList(assignable.get('baseassignablelist'), split);
      } else {
        TODO('operator support in splitting');
      }
    }
    return split;
  }

  constrain(_metadata: StatementMetaData) {
    throw new Error(`Assignable can't be type-checked!`);
  }

  getOutputType(): Type {
    throw new Error(`Can't determine type of Assignable`);
  }

  private splitBaseAssignableList(list: LPNode, split: Statement[]) {
    let assignables = list.getAll().map(a => a.get('baseassignable'));
    for (let ii = 0; ii < assignables.length; ii++) {
      let assignable = assignables[ii];
      if (assignable.has('objectliterals')) {
        TODO('objectliterals');
      } else if (assignable.has('functions')) {
        TODO('functions');
      } else if (assignable.has('fncall')) {
        TODO('fncall');
      } else if (assignable.has('variable')) {
        if (ii === assignables.length - 1) {
          TODO('is it just a varref?')
        }
        const next = assignables[ii + 1];
        if (next.has('fncall')) {
          this.splitCall(assignable.get('variable'), next.get('fncall'), split);
          ii += 1;
        } else if (next.has('methodsep')) {
          TODO()
        } else {
          console.log(assignables);
          TODO(`unsure of how to handle the above LPNodes (stopped at index ${ii})`);
        }
      } else if (assignable.has('constants')) {
        assignable = assignable.get('constants');
        let dec = Declaration.generate(new Literal(assignable));
        this.metadata.vars[dec.name] = new VarMetadata(dec);
        split.push(new Statement(assignable, dec));
      } else if (assignable.has('methodsep')) {
        TODO('methodsep');
      } else {
        console.log(assignable, assignables)
        throw new Error('huh');
      }
    }
  }

  private splitCall(
    fnName: LPNode,
    call: LPNode,
    split: Statement[],
    withMethod: Declaration | VarRef | null = null
  ) {
    let argDecs = call.get('assignablelist')
                      .getAll()
                      .map(node => Assignable.fromAst(node, this.scope, this.metadata))
                      .map(assignable => Declaration.generateOrVarRef(assignable));
    if (withMethod !== null) {
      argDecs.unshift(withMethod);
    }
    argDecs.forEach(dec => {
      split.push(...dec.split());
      if (dec instanceof Declaration) {
        this.metadata.vars[dec.name] = new VarMetadata(dec);
      } else {
        this.metadata.vars[dec.dec.name] = new VarMetadata(dec.dec);
      }
    });
    let fns = this.scope.get(fnName.t);
    let fnCall = new FnCall([fnName, call], fns, argDecs);
    let resDec = Declaration.generate(fnCall);
    split.push(...resDec.split());
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
    if (metadata.var(assignName) === null) {
      throw new Error(`${assignName} is not defined`);
    }
    const upstream = metadata.var(assignName);
    if (!upstream.dec.mutable) {
      throw new Error(`can't reassign to ${assignName} (not a let variable)`);
    }
    const assignable = Assignable.fromAst(ast.get('assignables'), scope, metadata);
    return new Assignment(ast, upstream.dec, assignable);
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

  constrain(metadata: StatementMetaData) {
    if (this.final === null) {
      throw new Error(`Assignment isn't prepared`);
    }
    this.final.constrain(metadata);
    let metavar = metadata.var(this.upstream.name);
    if (metavar === null) {
      throw new Error(`${this.upstream.name} doesn't exist in metadata?`);
    }
    metavar.constraints.push(this.final.getOutputType());
  }

  getOutputType(): Type {
    throw new Error(`assignments aren't expressions`);
  }
}

class Conditional implements Stmt {
  ast: LPNode
  branches: Array<[[Assignable | true, LPNode], [Fn, LPNode]]>

  constructor(
    ast: LPNode,
    branches: Array<[[Assignable | true, LPNode], [Fn, LPNode]]>,
  ) {
    this.ast = ast;
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
        // TODO: prefer most-recently-defined
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
    return new Conditional(ast, branches);
  }

  split(): Statement[] {
    let split = [];

    for (let [[cond, condAst], [then, thenAst]] of this.branches) {
      TODO('conditionals');
    }

    return split;
  }

  constrain(metadata: StatementMetaData) {
    TODO('conditionals');
  }

  getOutputType(): Type {
    return TODO('conditionals');
  }
}

export class Declaration implements Stmt {
  ast: LPNode | LPNode[]
  mutable: boolean
  name: string
  ty: Type
  assignable: Assignable
  final: Stmt | null

  constructor(
    ast: LPNode | LPNode[],
    mutable: boolean,
    name: string,
    ty: Type | null,
    assignable: Assignable,
    final: Stmt | null = null,
  ) {
    this.ast = ast;
    this.mutable = mutable;
    this.name = name;
    this.ty = ty !== null ? ty : Type.generate();
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
    if (metadata.var(dec.name) !== null) {
      throw new Error(`cannot shadow variable names`);
    }
    metadata.vars[this.name] = new VarMetadata(dec);
    return dec;
  }

  static generate(final: Stmt): Declaration {
    // const _abc123 = final;
    return new Declaration(
      final.ast,
      false,
      genName(),
      null,
      null,
      final,
    );
  }

  static generateOrVarRef(final: Stmt): Declaration | VarRef {
    if (final instanceof VarRef) {
      return final;
    } else {
      return this.generate(final);
    }
  }

  ref(): VarRef {
    return new VarRef(this.ast instanceof Array ? this.ast[0] : this.ast, this);
  }

  split(): Statement[] {
    if (this.assignable === null) {
      return [new Statement(this.ast instanceof Array ? this.ast[0] : this.ast, this)];
    }
    let split = this.assignable.split();
    let last = split.pop();
    if (last instanceof Declaration) {
      // we only inherit the final value, we don't care about anything else
      this.final = last.final;
    } else {
      throw new Error('Invalid declaration state');
    }
    split.push(new Statement(this.ast instanceof Array ? this.ast[0] : this.ast, this));
    return split;
  }

  constrain(metadata: StatementMetaData) {
    if (this.final === null) {
      throw new Error(`Declaration isn't prepared`);
    }
    this.final.constrain(metadata);
    let metavar = metadata.var(this.name);
    if (metavar === null || metavar.dec !== this) {
      // if metavar is null, that means that this struct
      // didn't get inserted, but doesn't necessarily mean
      // there's a scope error (should be handled by whoever)
      // created this instance
      throw new Error(`invalid declaration state ${metavar === null}`);
    }
    metavar.constraints.push(this.final.getOutputType());
  }

  getOutputType(): Type {
    throw new Error(`declarations aren't expressions`);
  }

  acceptConstraints(constraints: Type[]) {
    for (let constraint of constraints) {
    }
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
    const event = scope.deepGet(eventName);
    if (event === null) {
      throw new Error(`event not defined: ${eventName}`);
    } else if (!(event instanceof Event)) {
      throw new Error(`cannot emit to non-events (${eventName} is not an event)`);
    }

    let assignables = null;
    if (ast.get('retval').has()) {
      assignables = Assignable.fromAst(
        ast.get('retval').get('assignables'),
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
    if (assignables.length === 0) {
      console.log(this.assignable)
      throw new Error('er')
    }
    let emitValDec = assignables[assignables.length - 1];
    if (emitValDec.content instanceof Declaration) {
      this.final = emitValDec.content.ref();
    } else {
      throw new Error(`Invalid emit state`);
    }
    assignables.push(new Statement(this.ast, this));
    return assignables;
  }

  constrain(metadata: StatementMetaData) {
    if (this.final === null) {
      throw new Error(`Emit isn't prepared`);
    }
    this.final.constrain(metadata);
    let metavar = metadata.var(this.final.dec.name);
    if (metavar === null) {
      throw new Error(`${this.final.dec.name} doesn't exist in metadata?`);
    }
    metavar.constraints.push(this.event.eventTy);
  }

  getOutputType(): Type {
    throw new Error(`emits aren't expressions`);
  }
  // constrain(constraints: Constraint[]) {
  //   if (this.assignable === null) {
  //     if (this.event.eventTy !== opcodes().get('void')) {
  //       throw new Error(`Must emit a value to non-void events`);
  //     }
  //   } else if (this.final !== null) {
  //     // constrain the variable we're emitting to the type of the event
  //     if (!(this.final instanceof VarRef)) {
  //       throw new Error('not emitting a varref?');
  //     }
  //     this.final.constrain(constraints, this.event.eventTy);
  //   } else {
  //     throw new Error(`Something's not quite right - emit statement with no varref`)
  //   }
  // }
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

  constrain(metadata: StatementMetaData) {
    let retTy = this.final !== null ? this.final.dec.ty : opcodes().get('void');
    metadata.outputConstraints.push(retTy);
  }

  getOutputType(): Type {
    return TODO('')
  }
  // constrain(constraints: Constraint[]) {
  //   if (this.assignable === null) {
  //     // it's a void value (naked return)
  //     constraints.push([null, opcodes().get('void')]);
  //   } else if (this.final !== null) {
  //     this.final.constrain(constraints, null);
  //   } else {
  //     throw new Error(`Something's not quite right - return statement with no varref`);
  //   }
  // }
}

// ===================================================
// types that Assignables boil down to (shouldn't be used directly in a Statement built from LPNodes)
class Literal implements Stmt {
  ast: LPNode

  constructor(
    ast: LPNode,
  ) {
    this.ast = ast;
  }

  split(): Statement[] {
    return [new Statement(this.ast, this)];
  }

  constrain(_metadata: StatementMetaData) {
    // do nothing
  }

  getOutputType(): Type {
    if (this.ast.has('bool')) {
      return opcodes().get('bool');
    } else if (this.ast.has('str')) {
      return opcodes().get('string');
    } else if (this.ast.get('num')) {
      let num = this.ast.get('num').t;
      if (num.indexOf('.') !== -1) {
        return Type.oneOf([
          'float64',
          'float32',
        ].map(t => opcodes().get(t)));
      } else {
        return Type.oneOf([
          'int64',
          'int32',
          'int16',
          'int8',
          'float64',
          'float32',
        ].map(t => opcodes().get(t)));
      }
    } else {
      throw new Error('invalid literal node');
    }
  }
  // constrain(constraints: Constraint[], name?: string) {
  //   if (name === undefined) {
  //     throw new Error(`attempting to constrain a type to a non-existent value`);
  //   }
  //   if (this.ast.has('bool')) {
  //     constraints.push([name, opcodes().get('bool')]);
  //   } else if (this.ast.has('num')) {
  //     let num = this.ast.get('num');
  //     if (num.has('integer')) {
  //       constraints.push([name, opcodes().get('int64')]);
  //     } else if (num.has('real')) {
  //       constraints.push([name, opcodes().get('float64')]);
  //     }
  //   } else if (this.ast.has('str')) {
  //     constraints.push([name, opcodes().get('string')]);
  //   } else {
  //     throw new Error('unrecognized constant literal type');
  //   }
  // }
}

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

  constrain(metadata: StatementMetaData) {
    TODO('')
  }

  getOutputType(): Type {
    return TODO('')
  }
  // constrain(constraints: Constraint[], name?: string) {
  //   TODO('fn types')
  // }
}

export class VarRef implements Stmt {
  ast: LPNode
  dec: Declaration

  get ty(): Type {
    return this.dec.ty;
  }

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

  constrain(_metadata: StatementMetaData) {
    // do nothing
  }

  getOutputType(): Type {
    let outTy = this.dec.final.getOutputType();
    if (outTy === null) {
      console.log(this.dec);
      throw new Error('ah');
    }
    return outTy;
  }
  // constrain(constraints: Constraint[], to?: string | Ty) {
  //   if (to === undefined) {
  //     throw new Error('varref has nothing to constrain to')
  //   }
  //   if (this.dec.ty === null) {
  //     throw new Error(`${this.dec.name} doesn't have a type!`);
  //   }
  //   if (typeof to === 'string') {
  //     constraints.push([to, this.dec.ty]);
  //   } else {
  //     constraints.push([this.dec.name, to]);
  //   }
  // }
}

class FnCall implements Stmt {
  ast: LPNode[]
  fns: Fn[]
  args: (Declaration | VarRef)[]
  retTy: Type
  callTy: FunctionType | null

  constructor(
    ast: LPNode[],
    fns: Fn[],
    args: (Declaration | VarRef)[],
    retTy: Type = null,
    callTy: FunctionType = null,
  ) {
    this.ast = ast;
    this.fns = fns;
    this.args = args;
    this.retTy = Type.generate();
    this.retTy = retTy;
    this.callTy = callTy;
  }

  split(): Statement[] {
    this.fns = this.fns.filter(fn => Object.keys(fn.args).length !== this.args.length);
    return [new Statement(this.ast[0], this)];
  }

  // TODO: this won't work for 1x-checking functions with args that are interface types.
  // see how to fix that if necessary/possible...
  constrain(_metadata: StatementMetaData) {
    if (this.callTy === null) {
      let fnCallText = this.ast.map(a => a.t.trim()).join('');
      let argTys: Type[] = this.args.map(arg => arg.ty);
      this.callTy = new FunctionType(fnCallText, argTys, this.retTy);
    }
    let possConstraints = this.fns.map(fn => fn.getType());
    this.callTy.callConstraints.push(Type.oneOf(possConstraints));
    return this.callTy;
  }

  getOutputType(): Type {
    return this.retTy;
  }
  // constrain(constraints: Constraint[], name?: string) {
  //   const params = Object.keys(this.fn.args);
  //   // This sanity check isn't necessary - gets caught when generating this class
  //   if (this.args.length < params.length) {
  //     throw new Error(`Not enough arguments passed to function ${this.fn.name}`);
  //   } else if (this.args.length > params.length) {
  //     throw new Error(`Too many arguments passed to function ${this.fn.name}`);
  //   }
  //   TODO('have fn generate constraints');
  //   for (let i = 0; i < params.length; i++) {
  //     constraints.push([this.args[i].name, this.fn.args[params[i]]]);
  //   }
  //   if (name !== undefined) {
  //     constraints.push([name, this.fn.getReturnType()]);
  //   }
  // }
}
