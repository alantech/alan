import { Interface } from "readline";
import { LPNode } from "../lp";
import Event from "./Event";
import Fn from "./Fn";
import Scope from "./Scope";
import { Type } from "./Types";

type StatementKind = Assignment | Assignable | Conditional | Declaration | Emit | Exit

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
  content: StatementKind

  constructor(
    node: LPNode,
    content: StatementKind,
  ) {
    this.node = node;
    this.content = content;
  }

  static fromAst(ast: LPNode, scope: Scope, metadata: StatementMetaData): Statement {
    let content: StatementKind = null;
    if (ast.has('assignments')) {
      content = Assignment.fromAst(ast.get('assignments'), scope, metadata);
    } else if (ast.has('assignables')) {
      content = Assignable.fromAst(ast.get('assignables'), scope, metadata);
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
}

class Assignment {
  upstream: Declaration
  assignable: Assignable

  constructor(
    upstream: Declaration,
    assignable: Assignable,
  ) {
    this.upstream = upstream;
    this.assignable = assignable;
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
    return new Assignment(upstream, assignable);
  }
}

class Assignable {
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
}

class Conditional {
  branches: Array<[Assignable | true, Fn]>

  constructor(
    branches: Array<[Assignable | true, Fn]>,
  ) {
    this.branches = branches;
  }

  static fromAst(ast: LPNode, scope: Scope, metadata: StatementMetaData): Conditional {
    let branches = [];
    do {
      let cond: Assignable | true = true;
      if (ast.has('assignables')) {
        let condAst = ast.get('assignables');
        cond = Assignable.fromAst(condAst, scope, metadata);
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
      branches.push([cond, then]);
    } while (ast.has('blocklike'));
    return new Conditional(branches);
  }
}

class Declaration {
  mutable: boolean
  name: string
  ty: Type | Interface | null
  assignable: Assignable

  constructor(
    mutable: boolean,
    name: string,
    ty: Type | Interface | null,
    assignable: Assignable,
  ) {
    this.mutable = mutable;
    this.name = name;
    this.ty = ty;
    this.assignable = assignable;
  }

  static fromAst(ast: LPNode, scope: Scope, metadata: StatementMetaData): Declaration {
    let mutable: boolean;
    if (ast.has('constdeclaration')) {
      ast = ast.get('constdeclaration');
      mutable = false;
    } else {
      ast = ast.get('letdeclaration');
      mutable = true;
    }
    const name = ast.get('variable').t;
    const assignables = Assignable.fromAst(ast.get('assignables'), scope, metadata);
    let ty = null;
    if (ast.has('typedec')) {
      ast = ast.get('typdec');
      ty = Type.getFromTypename(ast.get('fulltypename'), scope);
    }
    const dec = new Declaration(
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
}

class Emit {
  event: Event
  assignable: Assignable | null

  constructor(
    event: Event,
    assignable: Assignable | null,
  ) {
    this.event = event;
    this.assignable = assignable;
  }

  static fromAst(ast: LPNode, scope: Scope, metadata: StatementMetaData): Emit {
    const eventName = ast.get('eventname').t.trim();
    const event = scope.get(eventName);
    if (event === null) {
      throw new Error(`event not defined: ${eventName}`);
    } else if (!(event instanceof Event)) {
      throw new Error(`cannot emit to non-events (${eventName} is not an event)`);
    }

    ast = ast.get('retval');
    let assignables = null;
    if (ast.has()) {
      assignables = ast.get().get('assignables');
    }

    return new Emit(event, Assignable.fromAst(assignables, scope, metadata));
  }
}

class Exit {
  assignable: Assignable | null

  constructor(
    assignable: Assignable | null,
  ) {
    this.assignable = assignable;
  }

  static fromAst(ast: LPNode, scope: Scope, metadata: StatementMetaData): Exit {
    ast = ast.get('retval');
    let assignables = null;
    if (ast.has()) {
      assignables = ast.get('assignables');
    }
    return new Exit(Assignable.fromAst(assignables, scope, metadata));
  }
}
