import { LPNode, NulLP } from '../lp';
import Fn from './Fn';
import Operator from './Operator';
import Scope from './Scope';
import { Equalable, genName, isFnArray, isOpArray, TODO } from './util';

type Fields = {[name: string]: Type | null};
export type FieldIndices = {[name: string]: number};
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
  ast: LPNode | null
  abstract get ammName(): string;

  constructor(
    name: string,
    ast: LPNode = null,
  ) {
    this.name = name;
    this.ast = ast;
  }

  abstract compatibleWithConstraint(ty: Type, scope: Scope): boolean;
  abstract constrain(to: Type, scope: Scope): void;
  abstract eq(that: Equalable): boolean;
  abstract instance(): Type;
  abstract tempConstrain(to: Type, scope: Scope): void;
  abstract resetTemp(): void;
  abstract size(): number;

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
    return new HasField(name, null, ty);
  }

  static hasMethod(name: string, params: Type[], ret: Type): Type {
    return new HasMethod(name, null, params, ret);
  }

  static hasOperator(name: string, params: Type[], ret: Type, isPrefix: boolean): Type {
    return new HasOperator(name, null, params, ret, isPrefix);
  }

  dupIfNotLocalInterface(): Type | null {
    return null;
  }

  fieldIndices(): FieldIndices {
    let name: string;
    try {
      name = this.instance().name;
    } catch (e) {
      name = this.name;
    }
    if (name !== '') {
      name = ` ${name}`;
    }
    throw new Error(`Type${name} does not have fields`);
  }

  isFixed(): boolean {
    // only a handful of builtin types are fixed
    return false;
  }
}

class Builtin extends Type {
  get ammName(): string {
    return this.name;
  }

  constructor(
    name: string,
  ) {
    super(name);
  }

  compatibleWithConstraint(ty: Type, scope: Scope): boolean {
    if (ty instanceof Builtin) {
      return this.eq(ty);
    } else if (ty instanceof OneOf || ty instanceof Generated) {
      return ty.compatibleWithConstraint(this, scope);
    } else if (ty instanceof HasOperator) {
      return Has.operator(ty, scope, this).length !== 0;
    } else if (ty instanceof HasMethod) {
      return Has.method(ty, scope, this).length !== 0;
    } else if (ty instanceof HasField) {
      return TODO('add field support for builtins');
    } else if (ty instanceof Interface) {
      // FIXME: once builtins can have fields
      return ty.fields.length === 0 &&
            ty.methods.every(m => this.compatibleWithConstraint(m, scope)) &&
            ty.operators.every(o => this.compatibleWithConstraint(o, scope));
    } else {
      console.log(ty);
      TODO('other types constraining to builtin types');
    }
  }

  constrain(ty: Type, scope: Scope) {
    if (ty instanceof OneOf || ty instanceof Generated || ty instanceof Interface) {
      ty.constrain(this, scope);
    } else if (ty instanceof HasOperator) {
      if (Has.operator(ty, scope, this).length === 0) {
        throw new Error(`type ${this.name} does not have operator ${ty.name}`);
      }
    } else if (ty instanceof HasMethod && Has.method(ty, scope, this).length === 0) {
      throw new Error(`type ${this.name} does not have method ${ty.name}(${ty.params.map(p => p === null ? this : p).map(ty => ty.name).join(', ')})`);
    } else if (ty instanceof HasField) {
      throw new Error(`type ${this.name} does not have field ${ty.name}`);
    } else if (!this.compatibleWithConstraint(ty, scope)) {
      throw new Error(`type ${this.name} could not be constrained to ${ty.name}`);
    }
  }

  eq(that: Equalable): boolean {
    if (that instanceof Builtin) {
      return this === that;
    } else if (that instanceof Interface) {
      if (that instanceof Generated && that.delegate !== null) {
        return this.eq(that.delegate);
      }
      return that.tempDelegate !== null && this.eq(that.tempDelegate);
    } else {
      return false;
    }
  }

  fieldIndices(): FieldIndices {
    return TODO("determine if it's even worth keeping the Builtin class");
  }

  instance(): Type {
    return this;
  }

  isFixed(): boolean {
    // TODO: this is pretty lazy and we should probably have a better
    // way to do this
    switch (this.name) {
      case 'int64':
      case 'int32':
      case 'int16':
      case 'int8':
      case 'float64':
      case 'float32':
      case 'bool':
      case 'void':
        return true;
      default:
        return false;
    }
  }

  tempConstrain(ty: Type, scope: Scope) {
    this.constrain(ty, scope);
  }

  resetTemp() {
    // do nothing
  }

  size(): number {
    // TODO: should probably figure out non-opaque types but we'll see
    // what happens with this class first (after fn selection fix)
    switch (this.name) {
      case 'void':
        return 0;
      default:
        // yes, strings are only size 1 since they're Pascal string ptrs
        return 1;
    }
  }
}

class Struct extends Type {
  args: GenericArgs
  fields: Fields
  order: FieldIndices

  get ammName(): string {
    return this.name;
  }

  constructor(
    name: string,
    ast: LPNode | null,
    args: GenericArgs,
    fields: Fields,
  ) {
    super(name, ast);
    this.args = args;
    this.fields = fields;
    this.order = {};
    let sizeTracker = 0;
    for (let fieldName in this.fields) {
      this.order[fieldName] = sizeTracker;
      sizeTracker += this.fields[fieldName].size();
    }
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

  compatibleWithConstraint(ty: Type, scope: Scope): boolean {
    if (ty instanceof Struct) {
      return this.eq(ty);
    } else if (ty instanceof HasField) {
      return this.fields.hasOwnProperty(ty.name) && this.fields[ty.name].compatibleWithConstraint(ty.ty, scope);
    } else if (ty instanceof HasMethod) {
      TODO('get methods and operators for types? (probably during fn selection fix?)');
    } else if (ty instanceof Interface || ty instanceof OneOf) {
      return ty.compatibleWithConstraint(this, scope);
    } else {
      return false;
    }
  }

  constrain(to: Type, scope: Scope) {
    if (!this.compatibleWithConstraint(to, scope)) {
      throw new Error(`incompatible types: ${this.name} is not compatible with ${to.name}`);
    }
    if (to instanceof HasField) {
      to.ty.constrain(this.fields[to.name], scope);
    }
  }

  eq(that: Equalable): boolean {
    // TODO: more generic && more complex structs
    return that instanceof Struct && this === that;
  }

  fieldIndices(): FieldIndices {
    return this.order;
  }

  instance(): Type {
    return this; // TODO: this right?
  }

  tempConstrain(to: Type, scope: Scope) {
    // TODO: can structs have temp constraints?
    this.constrain(to, scope);
  }

  resetTemp() {
    // TODO: can structs have temp constraints?
  }

  size(): number {
    // by lazily calculating, should be able to avoid having `OneOf` select
    // issues in ducked types
    return Object.values(this.fields).map(ty => ty.size()).reduce((l, r) => l + r);
  }
}

abstract class Has extends Type {
  get ammName(): string {
    throw new Error('None of the `Has` constraints should have their ammName requested...');
  }

  constructor(
    name: string,
    ast: LPNode | null,
  ) {
    super(name, ast);
  }

  static field(field: HasField, ty: Type): boolean {
    // TODO: structs
    return false;
  }

  static method(method: HasMethod, scope: Scope, ty: Type): Fn[] {
    let fns = scope.get(method.name);
    if (!isFnArray(fns)) {
      return [];
    }
    return Fn.select(fns, method.params.map(p => p === null ? ty : p), scope);
  }

  static operator(operator: HasOperator, scope: Scope, ty: Type): Operator[] {
    let ops: Operator[] = scope.get(operator.name);
    // if there is no op by that name, RIP
    if (!isOpArray(ops)) {
      return [];
    }
    // filter out ops that aren't the same fixity
    ops = ops.filter(op => op.isPrefix === operator.isPrefix);
    if (operator.isPrefix) {
      return ops.filter(op => op.select(scope, operator.params[0] || ty) !== []);
    } else {
      return ops.filter(op => op.select(scope, operator.params[0] || ty, operator.params[1] || ty) !== []);
    }
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

  size(): number {
    throw new Error(`Has constraints do not have a size (this error should never be thrown)`);
  }
}

class HasField extends Has {
  ty: Type

  constructor(
    name: string,
    ast: LPNode | null,
    ty: Type,
  ) {
    super(name, ast);
    this.ty = ty;
  }

  static fromPropertyTypeLine(ast: LPNode, scope: Scope): HasField {
    const name = ast.get('variable').t.trim();
    const ty = Type.getFromTypename(ast.get('fulltypename'), scope);
    return new HasField(
      name,
      ast,
      ty,
    );
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
    ast: LPNode | null,
    params: (Type | null)[],
    ret: Type | null,
  ) {
    super(name, ast);
    this.params = params;
    this.ret = ret;
  }

  static fromFunctionTypeLine(ast: LPNode, scope: Scope, ifaceName: string): HasMethod {
    const name = ast.get('variable').t.trim();
    let work = ast.get('functiontype');
    let params: (Type | null)[] = [
      work.get('fulltypename'),
      ...work.get('cdr').getAll().map(cdr => cdr.get('fulltypename')),
    ].map(tyNameAst => tyNameAst.t.trim() === ifaceName ? null : Type.getFromTypename(tyNameAst, scope));
    let ret = work.get('returntype').t.trim() === ifaceName ? null : Type.getFromTypename(work.get('returntype'), scope);
    return new HasMethod(name, ast, params, ret);
  }

  eq(that: Equalable): boolean {
    return super.eq(that) &&
      that instanceof HasMethod &&
      this.params.reduce((eq, param, ii) => eq && (param === null ? that.params[ii] === null : param.eq(that.params[ii])), true) &&
      this.ret.eq(that.ret);
  }
}

class HasOperator extends HasMethod {
  isPrefix: boolean

  constructor(
    name: string,
    ast: LPNode | null,
    params: (Type | null)[],
    ret: Type | null,
    isPrefix: boolean,
  ) {
    super(name, ast, params, ret);
    this.isPrefix = isPrefix;
  }

  static fromOperatorTypeLine(ast: LPNode, scope: Scope, ifaceName: string): HasOperator {
    let isPrefix = true;
    let params: (Type | null)[] = [];
    if (ast.get('optleftarg').has()) {
      let leftTypename = ast.get('optleftarg').get('leftarg');
      let leftTy = leftTypename.t.trim() === ifaceName ? null : Type.getFromTypename(leftTypename, scope);
      params.push(leftTy);
      isPrefix = false;
    }
    const op = ast.get('operators').t.trim();
    let rightTypename = ast.get('rightarg');
    let rightTy = rightTypename.t.trim() === ifaceName ? null : Type.getFromTypename(rightTypename, scope);
    params.push(rightTy);
    let retTypename = ast.get('fulltypename');
    let retTy = retTypename.t.trim() === ifaceName ? null : Type.getFromTypename(retTypename, scope);
    return new HasOperator(
      op,
      ast,
      params,
      retTy,
      isPrefix,
    );
  }

  eq(that: Equalable): boolean {
    return super.eq(that) && that instanceof HasOperator && this.isPrefix === that.isPrefix;
  }
}

class Interface extends Type {
  // TODO: it's more optimal to have fields, methods, and operators in
  // maps so we can cut down searching and such.
  fields: HasField[]
  methods: HasMethod[]
  operators: HasOperator[]
  tempDelegate: Type | null
  private isDuped: boolean

  get ammName(): string {
    if (this.tempDelegate) {
      return this.tempDelegate.ammName;
    } else {
      throw new Error(`Interfaces should not have their ammName requested`);
    }
  }

  constructor(
    name: string,
    ast: LPNode | null,
    fields: HasField[],
    methods: HasMethod[],
    operators: HasOperator[],
  ) {
    super(name, ast);
    this.fields = fields;
    this.methods = methods;
    this.operators = operators;
    this.tempDelegate = null;
    this.isDuped = false;
  }

  static fromAst(ast: LPNode, scope: Scope): Interface {
    const name = ast.get('variable').t.trim();
    let work = ast.get('interfacedef');
    if (work.has('interfacebody')) {
      work = work.get('interfacebody').get('interfacelist');
      const lines = [
        work.get('interfaceline'),
        ...work.get('cdr').getAll().map(cdr => cdr.get('interfaceline')),
      ];
      let fields: HasField[] = [];
      let methods: HasMethod[] = [];
      let operators: HasOperator[] = [];
      lines.forEach(line => {
        if (line.has('propertytypeline')) {
          fields.push(HasField.fromPropertyTypeLine(line.get('propertytypeline'), scope));
        } else if (line.has('functiontypeline')) {
          methods.push(HasMethod.fromFunctionTypeLine(line.get('functiontypeline'), scope, name));
        } else if (line.has('operatortypeline')) {
          operators.push(HasOperator.fromOperatorTypeLine(line.get('operatortypeline'), scope, name));
        } else {
          throw new Error(`invalid ast: ${work}`);
        }
      });
      return new Interface(
        name,
        ast,
        fields,
        methods,
        operators,
      );
    } else if (work.has('interfacealias')) {
      TODO('interface aliases')
    } else {
      throw new Error(`invalid ast: ${work}`);
    }
  }

  compatibleWithConstraint(ty: Type, scope: Scope): boolean {
    if (this.tempDelegate !== null) {
      return this.tempDelegate.compatibleWithConstraint(ty, scope);
    }
    if (ty instanceof Builtin || ty instanceof Struct) {
      if (ty instanceof Builtin && this.fields.length !== 0) {
        // if ty is a Builtin and there are field requirements,
        // then this interface doesn't apply.
        // TODO: this might change depending on the opaque types
        // we introduce
        return false;
      } else if (ty instanceof Struct) {
        // ty is a Struct, ensure it has the same fields
        // if (!this.fields.every(field => ty.fields[field.name] && ty.fields[field.name].eq(field.ty))) {
        //   return false;
        // }
        if (!this.fields.every(field => Has.field(field, ty))) {
          return false;
        }
      }

      // check methods
      return this.methods.every(m => Has.method(m, scope, ty)) &&
        this.operators.every(o => Has.operator(o, scope, ty));
      // check operators
    } else if (ty instanceof Interface) {
      // ensure `ty ⊆ this`
      return ty.fields.every(field => this.fields.find(f => f.eq(field))) &&
        ty.methods.every(method => this.methods.find(m => m.eq(method))) &&
        ty.operators.every(operator => this.operators.find(o => o.eq(operator)));
    } else if (ty instanceof HasField) {
      return this.fields.some(field => field.eq(ty));
    } else if (ty instanceof HasOperator) {
      return this.operators.some(operator => operator.eq(ty));
    } else if (ty instanceof HasMethod) {
      return this.methods.some(method => method.eq(ty));
    } else if (ty instanceof Generated || ty instanceof OneOf) {
      return ty.compatibleWithConstraint(this, scope);
    } else {
      throw new Error(`unsure of what type the constraint is - this error should never be thrown!`);
    }
  }

  constrain(ty: Type, scope: Scope) {
    const baseErrorString = `type ${ty.name} was constrained to interface ${this.name} but doesn't have`;
    this.fields.forEach(f => {
      if (!ty.compatibleWithConstraint(f, scope)) {
        throw new Error(`${baseErrorString} field ${f.name} with type ${f.ty}`);
      }
    });
    this.methods.forEach(m => {
      if (!Has.method(m, scope, ty)) {
        throw new Error(`${baseErrorString} method ${m.name}(${m.params.map(p => p === null ? ty : p).map(t => t.name).join(', ')})`);
      }
    });
    this.operators.forEach(o => {
      if (Has.operator(o, scope, ty)) return;
      if (o.isPrefix) {
        throw new Error(`${baseErrorString} prefix operator \`${o.name} ${ty.name}\``);
      } else {
        throw new Error(`${baseErrorString} infix operator \`${o.params[0] || ty.name} ${o.name} ${o.params[1] || ty.name}\``);
      }
    });
  }

  eq(that: Equalable): boolean {
    if (that instanceof Generated) {
      return that.eq(this);
    } else if (that instanceof Interface) {
      // FIXME: this is technically wrong, but there's no other way
      // to get the current generic params working without depending
      // on `eq` returning this. Ideally, we would be checking to
      // make sure all of the constraints match
      return this === that;
    } else if (this.tempDelegate) {
      return this.tempDelegate.eq(that);
    } else {
      return false;
    }
  }

  instance(): Type {
    if (this.tempDelegate !== null) {
      return this.tempDelegate.instance();
    } else {
      throw new Error(`Could not resolve interface type`);
    }
  }

  tempConstrain(to: Type, _scope: Scope) {
    if (this === to) {
      throw new Error('huh?')
    }
    if (this.tempDelegate !== null && !this.tempDelegate.eq(to)) {
      throw new Error('interface type is already constrained');
    }
    this.tempDelegate = to;
  }

  resetTemp() {
    this.tempDelegate = null;
  }

  dupIfNotLocalInterface(): Type | null {
    if (this.isDuped) return null;
    let dup = new Interface(
      this.name,
      this.ast,
      [...this.fields],
      [...this.methods],
      [...this.operators],
    );
    dup.isDuped = true;
    return dup;
  }

  size(): number {
    if (this.tempDelegate) {
      return this.tempDelegate.size();
    } else {
      TODO(`figure out how Interface should return from size() if there's not tempDelegate`);
    }
  }
}

// technically, generated types are a kind of interface - we just get to build up
// the interface through type constraints instead of through explicit requirements.
// this'll make untyped fn parameters easier once they're implemented.
class Generated extends Interface {
  delegate: Type | null

  get ammName(): string {
    if (this.delegate) {
      return this.delegate.ammName;
    } else if (this.tempDelegate) {
      return this.tempDelegate.ammName;
    } else {
      throw new Error(`Could not determine ammName for Generated type`);
    }
  }

  constructor() {
    super(genName(), new NulLP(), [], [], []);
    this.delegate = null;
  }

  // TODO: ok so i have to do a couple of things
  // 1. move tempDelegate to the interface type
  // 2. delegate to super if delegate isn't set
  // 3. ???
  compatibleWithConstraint(ty: Type, scope: Scope): boolean {
    if (this.delegate && super.tempDelegate) {
      throw new Error('ugh')
    }
    if (this.delegate !== null) {
      return this.delegate.compatibleWithConstraint(ty, scope);
    }
    if (this.tempDelegate !== null) {
      return this.tempDelegate.compatibleWithConstraint(ty, scope);
    }
    return true;
  }

  constrain(to: Type, scope: Scope) {
    // if `super.tempDelegate` is set, something is *very* wrong because
    // all permanent constraints should already be processed...
    // if we need to allow `tempConstrain`s to get processed `constrain`s,
    // then this check should be at the end of this method and pass the
    // removed `tempDelegate` to the new permanent delegate's `tempConstrain`
    if (this.tempDelegate) {
      throw new Error(`cannot process temporary type constraints before permanent type constraints`);
    }

    if (this.delegate !== null) {
      this.delegate.constrain(to, scope);
    } else if (to instanceof HasOperator) {
      this.operators.push(to);
    } else if (to instanceof HasMethod) {
      this.methods.push(to);
    } else if (to instanceof HasField) {
      this.fields.push(to);
    } else if (to instanceof Interface) {
      this.fields.push(...to.fields);
      this.methods.push(...to.methods);
      this.operators.push(...to.operators);
      if (to instanceof Generated && to.delegate !== null) {
        this.delegate = to.delegate;
      }
    } else {
      this.delegate = to;
      this.constrain(this.delegate, scope);
    }
  }

  eq(that: Equalable): boolean {
    if (that instanceof Generated) {
      return this.delegate !== null && that.delegate !== null && this.delegate.eq(that.delegate);
    } else if (that instanceof Interface) {
      if (that.tempDelegate !== null) {
        if (this.delegate !== null) {
          return this.delegate.eq(that.tempDelegate);
        } else if (this.tempDelegate !== null) {
          return this.tempDelegate.eq(that.tempDelegate);
        } else {
          return false;
        }
      } else {
        return this.delegate === null && this.tempDelegate === null;
      }
    } else {
      return this.delegate !== null && this.delegate.eq(that);
    }
  }

  instance(): Type {
    if (this.delegate !== null) {
      return this.delegate.instance();
    } else if (this.tempDelegate !== null) {
      return this.tempDelegate.instance();
    } else {
      throw new Error(`could not resolve Generated type`);
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
    if (super.tempDelegate !== null) {
      this.tempDelegate = null;
    } else if (this.delegate !== null) {
      this.tempDelegate.resetTemp();
    }
  }

  size(): number {
    if (this.delegate) {
      return this.delegate.size();
    } else if (this.tempDelegate) {
      return this.tempDelegate.size();
    } else {
      TODO(`figure out how Generated should return from size() if there's not tempDelegate`);
    }
  }
}

class OneOf extends Type {
  selection: Type[]
  tempSelect: Type[] | null
  private selected: Type | null;

  get ammName(): string {
    return this.select().ammName;
  }

  constructor(
    selection: Type[],
    tempSelect: Type[] = null,
  ) {
    super(genName());
    this.selection = selection;
    this.tempSelect = tempSelect;
    this.selected = null;
  }

  private select(): Type {
    let selected: Type;
    if (this.tempSelect !== null) {
      if (this.tempSelect.length === 0) {
        throw new Error();
      }
      selected = this.tempSelect[this.tempSelect.length - 1];
    } else if (this.selection.length > 0) {
      selected = this.selection[this.selection.length - 1];
    } else {
      throw new Error(`type selection impossible - no possible types left`);
    }
    if (this.selected === null) {
      this.selected = selected;
    } else if (this.selected !== selected) {
      // this should never happen, but let's make sure of that :)
      TODO('uh somehow selected different types - check on this');
    }
    return selected;
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
    const selected =  this.select();
    if (selected === undefined) {
      throw new Error(`uh whaaaaat`);
    }
    return selected.instance();
  }

  tempConstrain(to: Type, scope: Scope) {
    this.tempSelect = this.selection.filter(ty => ty.compatibleWithConstraint(to, scope));
  }

  resetTemp() {
    this.tempSelect = [];
  }

  size(): number {
    return this.select().size();
  }
}
