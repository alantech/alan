import { LPNode } from "../lp";
import Scope from "./Scope";
import { genName, TODO } from "./util";

type Fields = {[name: string]: Type | null};
export type GenericArgs = {[name: string]: Type | null};
type TypeName = [string, TypeName[]];

const parseFulltypename = (node: LPNode): TypeName => {
  const name = node.get('typename').t.trim();
  let genericTys: TypeName[] = [];
  if (node.get('opttypegenerics').has()) {
    const generics = node.get('opttypegenerics').get();
    genericTys.push(parseFulltypename(generics.get('fulltypename')));
    generics.get('cdr').getAll().map(n => n.get('fulltypename'));
  }
  return [name, genericTys];
};

export default abstract class Type {
  name: string
  alias: Type | null

  constructor(
    name: string,
    alias: Type | null,
  ) {
    this.name = name;
    this.alias = alias;
  }

  static fromInterfacesAst(ast: LPNode, scope: Scope): Type {
    return Interface.fromAst(ast, scope);
  }

  static fromTypesAst(ast: LPNode, scope: Scope): Type {
    return Struct.fromAst(ast, scope);
  }

  static generate(): Type {
    return new Generated();
  }

  static oneOf(types: Type[]): Type {
    return new OneOf(types);
  }

  static newBuiltin(name: string, generics: string[]): Type {
    return new Builtin(name);
    // let genericArgs: GenericArgs = {};
    // generics.forEach(arg => genericArgs[arg] = null);
    // return new Struct(
    //   name,
    //   null,
    //   genericArgs,
    //   {},
    // );
  }

  static getFromTypename(name: LPNode | string, scope: Scope): Type {
    return scope.get(name.toString().trim());
  }

  abstract breakdown(): Builtin;
  abstract compatibleWithConstraint(that: Type): boolean;
}

export class Builtin extends Type {
  get ammName(): string {
    return this.name;
  }

  constructor(
    name: string,
  ) {
    super(name, null);
  }

  breakdown(): Builtin {
    return this;
  }

  compatibleWithConstraint(that: Type): boolean {
    // TODO: generics, other checks that aren't lazy
    return this === that;
  }
}

export class FunctionType extends Type {
  argTys: Type[]
  retTy: Type
  callSelect: Type

  constructor(
    name: string,
    argTys: Type[],
    retTy: Type,
  ) {
    super(name, null);
    this.argTys = argTys;
    this.retTy = retTy;
  }

  breakdown(): Builtin {
    return TODO('function types???');
  }

  compatibleWithConstraint(that: Type): boolean {
    return TODO('?')
  }
}

class Struct extends Type {
  ast: LPNode
  generics: GenericArgs
  properties: Fields

  constructor(
    name: string,
    ast: LPNode,
    genericArgs: GenericArgs,
    properties: Fields,
  ) {
    super(name, null);
    this.ast = ast;
    this.generics = genericArgs;
    this.properties = properties;
  }

  static fromAst(ast: LPNode, scope: Scope): Struct {
    let work = ast;
    const names = parseFulltypename(work.get('fulltypename'));
    if (names[1].some(ty => ty[1].length !== 0)) {
      throw new Error(`Generic type variables can't have generic type arguments`);
    }
    const typeName = names[0];
    let genericArgs: GenericArgs = {};
    names[1].forEach(n => genericArgs[n[0]] = null);

    work = ast.get('typedef');
    if (work.has('typebody')) {
      work = work.get('typebody').get('typelist');
      const lines = [
        work.get('typeline'),
        ...work.get('cdr').getAll().map(n => n.get('typeline')),
      ];
      let fields: Fields = {};
      lines.forEach(line => {
        const fieldName = line.get('variable').t;
        const fieldTy = Type.getFromTypename(line.get('fulltypename'), scope);
        if (fieldTy instanceof Interface) {
          throw new Error(`type fields can't be interfaces (I think)`);
        }
        fields[fieldName] = fieldTy;
      });
      return new Struct(typeName, ast, genericArgs, fields);
    } else {
      ast = ast.get('typealias');
      TODO('type aliases');
    }
  }

  breakdown(): Builtin {
    return TODO('breakdown structs');
  }

  compatibleWithConstraint(that: Type): boolean {
    if (that instanceof Struct) {
      // TODO: generics
      return this === that;
    } else {
      TODO()
    }
  }
}

class Interface extends Type {
  constructor(
    name: string,
  ) {
    super(name, null);
  }

  static fromAst(ast: LPNode, scope: Scope): Interface {
    return null;
  }

  breakdown(): Builtin {
    return TODO('can interfaces even be broken down? i think not...');
  }

  compatibleWithConstraint(that: Type): boolean {
    return TODO()
  }
}

// Types not available in Alan, but we must use anyways
class Generated extends Type {
  delegate: Type | null

  constructor() {
    super(genName(), null);
    this.delegate = null;
  }

  breakdown(): Builtin {
    if (this.delegate !== null) {
      return this.delegate.breakdown();
    } else {
      throw new Error(`Couldn't resolve generated type`);
    }
  }

  compatibleWithConstraint(that: Type): boolean {
    if (this.delegate === null) {
      this.delegate = that;
      return true;
    } else {
      return this.delegate.compatibleWithConstraint(that);
    }
  }
}

class OneOf extends Type {
  possibilities: Type[]

  constructor(
    possibilities: Type[],
  ) {
    super(genName(), null);
    this.possibilities = possibilities;
  }

  breakdown(): Builtin {
    let delegate = this.possibilities[this.possibilities.length - 1];
    if (!delegate) {
      throw new Error(`none of the types worked`);
    }
    return delegate.breakdown();
  }

  compatibleWithConstraint(that: Type): boolean {
    this.possibilities = this.possibilities.filter(ty => ty.compatibleWithConstraint(that));
    return this.possibilities.length > 0;
  }
}
