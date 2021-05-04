import { LPNode } from '../lp';
import Scope from './Scope';
import { Equalable, genName, TODO } from './util';

type Fields = {[name: string]: Type | null};
type GenericArgs = {[name: string]: Type | null};
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
  // abstract resetTempConstraints(): void;
  // abstract tempConstrain(to: Type): void;

  static getFromTypename(name: LPNode | string, scope: Scope): Type {
    // TODO: change this to use parseFulltypename
    return scope.get(name.toString().trim());
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

  static oneOf(tys: Type[]): OneOf {
    return new OneOf(tys);
  }

  static newBuiltin(name: string, generics: string[]): Type {
    return new Builtin(name);
    // TODO: this maybe?
    // let genericArgs: GenericArgs = {};
    // generics.forEach(arg => genericArgs[arg] = null);
    // return new Struct(
    //   name,
    //   null,
    //   genericArgs,
    //   {},
    // );
  }
}

export class Builtin extends Type {
  get ammName(): string {
    return this.name;
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
    if (ty instanceof Builtin) {
      return this.eq(ty);
    } if (ty instanceof OneOf || ty instanceof Generated) {
      return ty.compatibleWithConstraint(this);
    } else {
      TODO('other types constraining to builtin types');
    }
  }

  constrain(ty: Type) {
    if (ty instanceof OneOf || ty instanceof Generated) {
      ty.constrain(this);
    } else if (!this.compatibleWithConstraint(ty)) {
      throw new Error(`type ${this.name} could not be constrained to ${ty.name}`);
    }
  }

  eq(that: Equalable): boolean {
    return that instanceof Builtin && this.ammName === that.ammName;
  }

  instance(): Type {
    return this;
  }

  resetTempConstraints() {
    // do nothing
  }

  tempConstrain(to: Type) {
    // builtins don't actually mutate any state when constrained
    this.constrain(to);
  }
}

class Struct extends Type {
  ast: LPNode
  args: GenericArgs
  fields: Fields

  constructor(
    name: string,
    ast: LPNode,
    args: GenericArgs,
    fields: Fields,
  ) {
    super(name);
    this.ast = ast;
    this.args = args;
    this.fields = fields;
  }

  static fromAst(ast: LPNode, scope: Scope): Type {
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

  compatibleWithConstraint(ty: Type): boolean {
    if (ty instanceof Struct) {
      return this.eq(ty);
    } else {
      TODO('constraints with other types for structs');
    }
  }

  constrain(to: Type) {
    if (!this.compatibleWithConstraint(to)) {
      throw new Error(`incompatible types: ${this.name} is not compatible with ${to.name}`);
    }
  }

  eq(that: Equalable): boolean {
    // TODO: more generic && more complex structs
    return that instanceof Struct && this === that;
  }

  instance(): Type {
    return this; // TODO: this right?
  }

  resetTempConstraints() {
    // do nothing, no temporary constrains are accepted
  }

  tempConstrain(to: Type) {
    if (!this.compatibleWithConstraint(to)) {
      throw new Error(`incompatible types: ${this.name} is not compatible with ${to.name}`);
    }
  }
}

class Interface extends Type {
  static fromAst(ast: LPNode, scope: Scope): Interface {
    return null;
  }

  breakdown(): Builtin {
    throw new Error(`interfaces cannot be broken down, and no concrete type was specified`);
  }

  compatibleWithConstraint(ty: Type): boolean {
    return TODO();
  }

  constrain(to: Type) {
    return TODO();
  }

  eq(that: Equalable): boolean {
    // TODO: gets more complicated later
    return that instanceof Interface && this === that;
  }

  instance(): Type {
    return TODO();
  }

  resetTempConstraints() {
    TODO();
  }

  tempConstrain(to: Type) {
    TODO();
  }
}

class Generated extends Type {
  private delegate: Type | null

  constructor() {
    super(genName());
    this.delegate = null;
  }

  breakdown(): Builtin {
    if (this.delegate !== null) {
      return this.delegate.breakdown();
    } else {
      throw new Error(`Couldn't resolve generated type`);
    }
  }

  compatibleWithConstraint(ty: Type): boolean {
    if (this.delegate === null) {
      return true;
    } else {
      return this.delegate.compatibleWithConstraint(ty);
    }
  }

  constrain(to: Type) {
    if (this.delegate === null) {
      this.delegate = to;
    } else {
      this.delegate.constrain(to);
    }
  }

  eq(that: Equalable): boolean {
    if (that instanceof Generated) {
      return this.delegate !== null && that.delegate !== null && this.delegate.eq(that.delegate);
    } else {
      return this.delegate !== null && this.delegate.eq(that);
    }
  }

  instance(): Type {
    if (this.delegate !== null) {
      return this.delegate.instance();
    } else {
      return new Generated();
    }
  }

  // resetTempConstraints() {
  //   if (this.delegate === null) {
  //     throw new Error('bad sign - generated type should not have its temporary constraints reset because they cannot have temporary constraints');
  //   }
  //   this.delegate.resetTempConstraints();
  // }

  // tempConstrain(to: Type) {
  //   if (this.delegate === null) {
  //     throw new Error('generated types without constraints cannot have temporary constraints');
  //   }
  //   this.delegate.tempConstrain(to);
  // }
}

class OneOf extends Type {
  selection: Type[]
  tempSelect: Type | null

  constructor(
    selection: Type[],
  ) {
    super(genName());
    this.selection = selection;
    this.tempSelect = null;
  }

  private select(): Type {
    if (this.tempSelect !== null) {
      return this.tempSelect;
    } else if (this.selection.length > 0) {
      return this.selection[this.selection.length - 1];
    } else {
      throw new Error(`type selection impossible - no possible types left`);
    }
  }

  breakdown(): Builtin {
    return this.select().breakdown();
  }

  compatibleWithConstraint(constraint: Type): boolean {
    return this.selection.some(ty => ty.compatibleWithConstraint(constraint));
  }

  constrain(constraint: Type) {
    this.selection = this.selection.filter(ty => ty.compatibleWithConstraint(constraint));
  }

  eq(that: Equalable): boolean {
    return that instanceof OneOf && this.selection.length === that.selection.length && this.selection.every((ty, ii) => ty.eq(that.selection[ii]));
  }

  instance(): Type {
    return this.select().instance();
  }

  resetTempConstraints() {
    this.tempSelect = null;
  }

  tempConstrain(to: Type) {
    let selected = this.selection.reverse().find(ty => ty.compatibleWithConstraint(to)) || null;
    if (selected === null) {
      throw new Error(`couldn't select a type - none of the possibilities were compatible`);
    }
    this.tempSelect = selected;
  }
}

// export class Types extends Type {
//   breakdown(): Builtin {
//     throw new Error('Method not implemented.');
//   }
//   compatibleWithConstraint(ty: Type): boolean {
//     throw new Error('Method not implemented.');
//   }
//   constrain(to: Type): void {
//     throw new Error('Method not implemented.');
//   }
//   eq(that: Equalable): boolean {
//     throw new Error('Method not implemented.');
//   }
//   instance(): Type {
//     throw new Error('Method not implemented.');
//   }
// }
