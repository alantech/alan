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
  abstract compatibleWithConstraint(ty: Type, scope: Scope): boolean;
  abstract constrain(to: Type, scope: Scope): void;
  abstract eq(that: Equalable): boolean;
  abstract instance(): Type;
  abstract tempConstrain(to: Type, scope: Scope): void;
  abstract resetTemp(): void;

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

  static hasField(name: string, ty: Type): Type {
    return new HasField(name, ty);
  }

  static hasMethod(name: string, params: Type[], ret: Type): Type {
    return new HasMethod(name, params, ret);
  }

  static hasOperator(name: string, params: Type[], ret: Type, isPrefix: boolean): Type {
    return new HasOperator(name, params, ret, isPrefix);
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

  compatibleWithConstraint(ty: Type, scope: Scope): boolean {
    if (ty instanceof Builtin) {
      return this.eq(ty);
    } else if (ty instanceof OneOf || ty instanceof Generated) {
      return ty.compatibleWithConstraint(this, scope);
    } else if (ty instanceof HasMethod) {
      TODO('get methods and operators for types');
    } else if (ty instanceof HasField) {
      return false;
    } else {
      TODO('other types constraining to builtin types');
    }
  }

  constrain(ty: Type, scope: Scope) {
    if (ty instanceof OneOf || ty instanceof Generated) {
      ty.constrain(this, scope);
    } else if (ty instanceof HasMethod) {
      TODO('get methods and operators for types');
    } else if (!this.compatibleWithConstraint(ty, scope)) {
      throw new Error(`type ${this.name} could not be constrained to ${ty.name}`);
    }
  }

  eq(that: Equalable): boolean {
    return that instanceof Builtin && this.ammName === that.ammName;
  }

  instance(): Type {
    return this;
  }

  tempConstrain(ty: Type, scope: Scope) {
    this.constrain(ty, scope);
  }

  resetTemp() {
    // do nothing
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
    } else if (ty instanceof HasField) {
      // TODO:
    } else if (ty instanceof HasMethod) {
      TODO('get methods and operators for types');
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

  tempConstrain(to: Type) {
    // TODO: can structs have temp constraints?
    this.constrain(to);
  }

  resetTemp() {
    // TODO: can structs have temp constraints?
  }
}

abstract class Has extends Type {
  constructor(
    name: string,
  ) {
    super(name);
  }

  breakdown(): Builtin {
    throw new Error(`cannot breakdown a Has constraint (this error should never be thrown)`);
  }

  // convenience for `Type.hasX(...).compatibleWithConstraint(ty)`
  compatibleWithConstraint(ty: Type, scope: Scope): boolean {
    return ty.compatibleWithConstraint(this, scope);
  }

  // convenience for `Type.hasX(...).constrain(ty)`
  constrain(to: Type, scope: Scope) {
    to.constrain(this, scope);
  }

  eq(that: Equalable): boolean {
    return that instanceof Has && that.name === this.name;
  }

  instance(): Type {
    throw new Error(`cannot get instance of Has constraint (this error should never be thrown)`);
  }

  // there should never be a case where `Type.hasX(...).tempConstrain(...)`
  tempConstrain(_t: Type) {
    throw new Error(`cannot temporarily constrain a Has constraint (this error should never be thrown)`);
  }

  // there can never be temp constraints
  resetTemp() {
    throw new Error(`Has constraints cannot have temporary constraints (this error should never be thrown)`);
  }
}

class HasField extends Has {
  ty: Type

  constructor(
    name: string,
    ty: Type,
  ) {
    super(name);
    this.ty = ty;
  }

  eq(that: Equalable): boolean {
    return super.eq(that) && that instanceof HasField && that.ty.eq(this.ty);
  }
}

class HasMethod extends Has {
  // null if it refers to the implementor's type. Only used when
  // working on interfaces
  params: (Type | null)[]
  ret: Type | null

  constructor(
    name: string,
    params: (Type | null)[],
    ret: Type | null,
  ) {
    super(name);
    this.params = params;
    this.ret = ret;
  }

  eq(that: Equalable): boolean {
    return super.eq(that) &&
      that instanceof HasMethod &&
      this.params.reduce((eq, param, ii) => eq && param.eq(that.params[ii]), true) &&
      this.ret.eq(that.ret);
  }
}

class HasOperator extends HasMethod {
  isPrefix: boolean

  constructor(
    name: string,
    params: (Type | null)[],
    ret: Type | null,
    isPrefix: boolean,
  ) {
    super(name, params, ret);
    this.isPrefix = isPrefix;
  }

  eq(that: Equalable): boolean {
    return super.eq(that) && that instanceof HasOperator && this.isPrefix === that.isPrefix;
  }
}

class Interface extends Type {
  fields: HasField[]
  methods: HasMethod[]
  operators: HasOperator[]

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

  tempConstrain(to: Type) {
    TODO();
  }

  resetTemp() {
    TODO();
  }
}

class Generated extends Type {
  private delegate: Type | null
  private tempDelegate: Type | null

  constructor() {
    super(genName());
    this.delegate = null;
    this.tempDelegate = null;
  }

  breakdown(): Builtin {
    if (this.tempDelegate !== null) {
      return this.tempDelegate.breakdown();
    } else if (this.delegate !== null) {
      return this.delegate.breakdown();
    } else {
      throw new Error(`Couldn't resolve generated type`);
    }
  }

  compatibleWithConstraint(ty: Type, scope: Scope): boolean {
    if (this.tempDelegate !== null) {
      return this.tempDelegate.compatibleWithConstraint(ty, scope);
    } else if (this.delegate !== null) {
      return this.delegate.compatibleWithConstraint(ty, scope);
    } else {
      return true;
    }
  }

  constrain(to: Type, scope: Scope) {
    // if `this.tempDelegate` is set, something is *very* wrong because
    // all permanent constraints should already be processed...
    // if we need to allow `tempConstrain`s to get processed `constrain`s,
    // then this check should be at the end of this method and pass the
    // removed `tempDelegate` to the new permanent delegate's `tempConstrain`
    if (this.tempDelegate) {
      throw new Error(`cannot process temporary type constraints after permanent type constraints`);
    }

    if (this.delegate !== null) {
      this.delegate.constrain(to, scope);
    } else if (to instanceof Has) {
      TODO('generate interface and delegate');
    } else {
      this.delegate = to;
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

  tempConstrain(to: Type, scope: Scope) {
    if (this.delegate !== null) {
      this.delegate.tempConstrain(to, scope);
    } else if (this.tempDelegate !== null) {
      TODO('temp constraints to a temporary constraint???');
    } else if (to instanceof Has) {
      TODO("i'm not sure");
    } else {
      this.tempDelegate = to;
    }
  }

  resetTemp() {
    if (this.tempDelegate !== null) {
      this.tempDelegate = null;
    } else if (this.delegate !== null) {
      this.tempDelegate.resetTemp();
    }
  }
}

class OneOf extends Type {
  selection: Type[]
  tempSelect: Type[] | null

  constructor(
    selection: Type[],
    tempSelect: Type[] = null,
  ) {
    super(genName());
    this.selection = selection;
    this.tempSelect = tempSelect;
  }

  private select(): Type {
    if (this.tempSelect !== null) {
      if (this.tempSelect.length === 0) {
        throw new Error();
      }
      return this.tempSelect[this.tempSelect.length - 1];
    } else if (this.selection.length > 0) {
      return this.selection[this.selection.length - 1];
    } else {
      throw new Error(`type selection impossible - no possible types left`);
    }
  }

  breakdown(): Builtin {
    return this.select().breakdown();
  }

  compatibleWithConstraint(constraint: Type, scope: Scope): boolean {
    return this.selection.some(ty => ty.compatibleWithConstraint(constraint, scope));
  }

  constrain(constraint: Type, scope: Scope) {
    this.selection = this.selection.filter(ty => ty.compatibleWithConstraint(constraint, scope));
  }

  eq(that: Equalable): boolean {
    return that instanceof OneOf && this.selection.length === that.selection.length && this.selection.every((ty, ii) => ty.eq(that.selection[ii]));
  }

  instance(): Type {
    return this.select().instance();
  }

  tempConstrain(to: Type, scope: Scope) {
    this.tempSelect = this.selection.filter(ty => ty.compatibleWithConstraint(to, scope));
  }

  resetTemp() {
    this.tempSelect = [];
  }
}
