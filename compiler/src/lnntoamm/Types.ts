import { LPNode } from '../lp';
import Scope from './Scope';
import { Equalable, genName, TODO } from './util';

type TypeName = [string, TypeName[]];

const parseFulltypename = (node: LPNode): TypeName => {
  const name = node.get('typename').t.trim();
  let genericTys: TypeName[] = [];
  if (node.has('opttypegenerics')) {
    const generics = node.get('opttypegenerics');
    genericTys.push(parseFulltypename(generics.get('fulltypename')));
    genericTys.push(...generics.get('cdr').getAll().map(n => n.get('fulltypename')).map(parseFulltypename));
  }
  return [name, genericTys];
};

// TODO: figure out type aliases (i think it actually makes sense to make a new type?)
export default abstract class Type implements Equalable {
  name: string

  constructor(
    name: string,
  ) {
    this.name = name;
  }

  abstract breakdown(): Builtin;
  abstract compatibleWithConstraint(ty: Type): boolean;
  abstract constrain(to: Type): void;
  abstract eq(that: Equalable): boolean;
  abstract instance(): Type;

  static getFromTypename(name: LPNode | string, scope: Scope): Type {
    // TODO: change this to use parseFulltypename
    return scope.get(name.toString().trim());
  }

  static fromInterfacesAst(ast: LPNode, scope: Scope): Type {
    return null;
  }

  static fromTypesAst(ast: LPNode, scope: Scope): Type {
    return null;
  }

  static generate(): Generated {
    return new Generated();
  }

  static oneOf(tys: Type[]): OneOf {
    return null;
  }

  static newBuiltin(name: string, generics: string[]): Type {
    return null;
  }
}

export class Builtin extends Type {
  get ammName(): string {
    return '';
  }

  constructor(
    name: string,
  ) {
    super(name);
  }

  breakdown(): Builtin {
    return this;
  }

  compatibleWithConstraint(ty: Type): boolean {
    return null;
  }

  constrain(to: Type) {
    if (to instanceof Builtin) {
      TODO()
    } else {
      TODO('other types');
    }
  }

  eq(that: Equalable): boolean {
    return null;
  }

  instance(): Type {
    return null;
  }
}

class Struct extends Type {
  breakdown(): Builtin {
    return null;
  }

  compatibleWithConstraint(ty: Type): boolean {
    return null;
  }

  constrain(to: Type) {
    TODO()
  }

  eq(that: Equalable): boolean {
    return null;
  }

  instance(): Type {
    return null;
  }
}

class Interface extends Type {
  breakdown(): Builtin {
    return null;
  }

  compatibleWithConstraint(ty: Type): boolean {
    return null;
  }

  constrain(to: Type) {
  }

  eq(that: Equalable): boolean {
    return null;
  }

  instance(): Type {
    return null;
  }
}

class Generated extends Type {
  constructor() {
    super(genName());
  }

  breakdown(): Builtin {
    return null;
  }

  compatibleWithConstraint(ty: Type): boolean {
    return null;
  }

  constrain(to: Type) {
    TODO()
  }

  eq(that: Equalable): boolean {
    return null;
  }

  instance(): Type {
    return null;
  }
}

class OneOf extends Type {
  selection: Type[]

  constructor(
    selection: Type[],
  ) {
    super(genName());
    this.selection = selection;
  }

  breakdown(): Builtin {
    return null;
  }

  compatibleWithConstraint(ty: Type): boolean {
    return null;
  }

  constrain(to: Type) {
    TODO()
  }

  eq(that: Equalable): boolean {
    return null;
  }

  instance(): Type {
    return null;
  }
}

export class Types extends Type {
  breakdown(): Builtin {
    throw new Error('Method not implemented.');
  }
  compatibleWithConstraint(ty: Type): boolean {
    throw new Error('Method not implemented.');
  }
  constrain(to: Type): void {
    throw new Error('Method not implemented.');
  }
  eq(that: Equalable): boolean {
    throw new Error('Method not implemented.');
  }
  instance(): Type {
    throw new Error('Method not implemented.');
  }
}
