import { LPNode, NulLP } from '../lp';
import { fulltypenameAstFromString } from './Ast';
import Fn from './Fn';
import Operator from './Operator';
import Scope from './Scope';
import { Equalable, genName, isFnArray, isOpArray, TODO } from './util';

type Fields = { [name: string]: Type | null };
export type FieldIndices = { [name: string]: number };
type GenericArgs = { [name: string]: Type | null };
type TypeName = [string, TypeName[]];

interface Generalizable {
  generics: GenericArgs;
  solidify(types: Type[]): Type;
}

const generalizable = (val: Type): val is Type & Generalizable => {
  return 'generics' in val;
}

const parseFulltypename = (node: LPNode): TypeName => {
  const name = node.get('typename').t.trim();
  const genericTys: TypeName[] = [];
  if (node.has('opttypegenerics')) {
    const generics = node.get('opttypegenerics').get('generics');
    genericTys.push(parseFulltypename(generics.get('fulltypename')));
    if (generics.has('cdr')) {
      genericTys.push(
        ...generics
          .get('cdr')
          .getAll()
          .map((n) => n.get('fulltypename'))
          .map(parseFulltypename),
      );
    }
  }
  return [name, genericTys];
};

// TODO: figure out type aliases (i think it actually makes sense to make a new type?)
export default abstract class Type implements Equalable {
  name: string;
  ast: LPNode | null;
  abstract get ammName(): string;

  constructor(name: string, ast: LPNode = null) {
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
    if (typeof name === 'string') {
      name = fulltypenameAstFromString(name);
    }
    const parsed = parseFulltypename(name);
    const solidify = ([name, generics]: TypeName): Type => {
      const ty = scope.get(name);
      if (ty === null) {
        throw new Error(`Could not find type ${name}`);
      } else if (!(ty instanceof Type)) {
        throw new Error(`${name} is not a type`);
      }
      if (generalizable(ty)) {
        const genericArgLen = Object.keys(ty.generics).length;
        if (genericArgLen !== generics.length) {
          console.log([name, generics]);
          throw new Error(`Bad typename: type ${name} expects ${genericArgLen} type arguments, but ${generics.length} were provided`);
        }
        let solidifiedTypeArgs = generics.map(solidify);
        // interfaces can't have generic type params so no need to call
        // dupIfNotLocalInterface
        return ty.solidify(solidifiedTypeArgs);
      } else if (generics.length !== 0) {
        throw new Error(`Bad typename: type ${name} doesn't expect any type arguments, but ${generics.length} were provided`);
      } else {
        const duped = ty.dupIfNotLocalInterface();
        if (duped === null) {
          return ty;
        } else {
          // note: if scope isn't *only* for the function's arguments,
          // this'll override the module scope and that would be bad.
          scope.put(name, duped);
          return duped;
        }
      }
    }
    return solidify(parsed);
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

  static interface(name: string): Type {
    return new Interface(name, new NulLP(), [], [], []);
  }

  static oneOf(tys: Type[]): OneOf {
    return new OneOf(tys);
  }

  static opaque(name: string, generics: string[]): Type {
    return new Opaque(name, generics);
  }

  static hasField(name: string, ty: Type): Type {
    return new HasField(name, null, ty);
  }

  static hasMethod(name: string, params: Type[], ret: Type): Type {
    return new HasMethod(name, null, params, ret);
  }

  static hasOperator(
    name: string,
    params: Type[],
    ret: Type,
    isPrefix: boolean,
  ): Type {
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

  fnselectOptions(): Type[] {
    return [this];
  }
}

class Opaque extends Type {
  generics: GenericArgs

  get ammName(): string {
    let generics = '';
    if (Object.keys(this.generics).length !== 0) {
      let genNames: string[] = [];
      for (let [tyVar, ty] of Object.entries(this.generics)) {
        if (ty === null) {
          genNames.push(tyVar);
        } else {
          genNames.push(ty.ammName);
        }
      }
      generics = '<' + genNames.join(', ') + '>';
    }
    return this.name + generics;
  }

  constructor(
    name: string,
    generics: string[],
  ) {
    super(name);
    this.generics = {};
    generics.forEach((g) => this.generics[g] = null);
  }

  compatibleWithConstraint(that: Type, scope: Scope): boolean {
    if (that instanceof Opaque) {
      const thisGenerics = Object.values(this.generics);
      const thatGenerics = Object.values(that.generics);
      if (this.name !== that.name || thisGenerics.length !== thatGenerics.length) {
        return false;
      }
      for (let ii = 0; ii < thisGenerics.length; ii++) {
        const thisTy = thisGenerics[ii];
        const thatTy = thatGenerics[ii];
        if (thisTy === null || thatTy === null) {
          continue;
        } else if (!thisTy.compatibleWithConstraint(thatTy, scope)) {
          return false;
        }
      }
      return true;
    } else if (
      that instanceof Generated ||
      that instanceof Interface ||
      that instanceof OneOf
    ) {
      return that.compatibleWithConstraint(this, scope);
    } else if (that instanceof HasField) {
      return false;
    } else if (that instanceof HasOperator) {
      return Has.operator(that, scope, this).length !== 0;
    } else if (that instanceof HasMethod) {
      return Has.method(that, scope, this).length !== 0;
    } else {
      TODO('Opaque constraint compatibility with other types');
    }
  }

  constrain(that: Type, scope: Scope): void {
    if (!this.compatibleWithConstraint(that, scope)) {
      throw new Error(`Cannot constraint type ${this.ammName} to ${that.ammName}`);
    }
    if (that instanceof Opaque) {
      const thisGens = Object.keys(this.generics);
      const thatGens = Object.keys(that.generics);
      for (let ii = 0; ii < thisGens.length; ii++) {
        if (this.generics[thisGens[ii]] === null) {
          this.generics[thisGens[ii]] = that.generics[thatGens[ii]];
        } else {
          this.generics[thisGens[ii]].constrain(that.generics[thatGens[ii]], scope);
        }
      }
    } else if (that instanceof OneOf) {
      that.constrain(this, scope);
    } else if (that instanceof Interface) {
      // do nothing
    } else {
      console.log(this);
      console.log(that);
      throw 'uh';
    }
  }

  eq(that: Equalable): boolean {
    if (!(that instanceof Opaque) || this.name !== that.name) {
      return false;
    }
    const thisGenerics = Object.values(this.generics);
    const thatGenerics = Object.values(that.generics);
    return thisGenerics.length === thatGenerics.length &&
      thisGenerics.every((thisGen, ii) => {
        const thatGen = thatGenerics[ii];
        if (thisGen === null || thatGen === null) {
          return false;
        } else {
          return thisGen.eq(thatGen);
        }
      });
  }

  instance(): Type {
    const genNames = Object.keys(this.generics);
    if (genNames.length === 0) {
      // minor optimization: if there's no generics then we
      // keep the same JS object to reduce the number of allocs
      return this;
    }
    const instance = new Opaque(this.name, genNames);
    for (let name of genNames) {
      instance.generics[name] = this.generics[name].instance();
    }
    return instance;
  }

  isFixed(): boolean {
    switch (this.name) {
      case 'bool':
      case 'int8':
      case 'int16':
      case 'int32':
      case 'int64':
      case 'float32':
      case 'float64':
      case 'void':
        return true;
      default:
        return false;
    }
  }

  tempConstrain(that: Type, scope: Scope): void {
    if (!this.compatibleWithConstraint(that, scope)) {
      throw new Error(`Cannot temporarily constrain type ${this.ammName} to ${that.ammName}`);
    }
    if (that instanceof Opaque) {
      const thisGens = Object.keys(this.generics);
      const thatGens = Object.keys(that.generics);
      for (let ii = 0; ii < thisGens.length; ii++) {
        if (this.generics[thisGens[ii]] === null) {
          throw new Error(`tempConstrain to generic that isn't assigned to anything?`);
        } else {
          this.generics[thisGens[ii]].tempConstrain(that.generics[thatGens[ii]], scope);
        }
      }
    } else if (that instanceof Interface) {
      if (!this.compatibleWithConstraint(that, scope)) {
        throw new Error(`type ${this.ammName} doesn't `);
      }
    } else if (that instanceof OneOf) {
      that.tempConstrain(this, scope);
    } else {
      console.log('this:', this);
      console.log('that:', that);
      TODO('tempConstraining Opaque type');
    }
  }

  resetTemp(): void {
    for (let generic in this.generics) {
      this.generics[generic].resetTemp();
    }
  }

  size(): number {
    switch (this.name) {
      case 'void':
        return 0;
      case 'Result':
        const containedTypes = Object.values(this.generics);
        return containedTypes
          .map(t => {
            if (t === null) {
              throw new Error(`cannot compute size of ${this.ammName}`);
            } else {
              return t.size();
            }
          })
          .reduce((s1, s2) => s1 + s2, 1);
      default:
        return 1;
    }
  }

  solidify(types: Type[]): Type {
    if (Object.values(this.generics).some((g) => g !== null)) {
      throw new Error(`Trying to solidify already-solidified type`);
    }
    const genNames = Object.keys(this.generics);
    if (genNames.length === 0) {
      return this;
    } else if (genNames.length !== types.length) {
      let multiplesAware: string;
      if (types.length === 0) {
        multiplesAware = 'none were';
      } else if (types.length === 1) {
        multiplesAware = '1 was';
      } else {
        multiplesAware = `${types.length} were`;
      }
      throw new Error(`${this.ammName} expected ${genNames.length} type argument${genNames.length > 1 ? 's' : ''}, but ${multiplesAware} provided`);
    }
    const cloned = new Opaque(this.name, genNames);
    for (let ii = 0; ii < genNames.length; ii++) {
      cloned.generics[genNames[ii]] = types[ii];
    }
    return cloned;
  }
}

export class FunctionType extends Type {
  params: Type[];
  retTy: Type;

  get ammName(): string {
    throw new Error('Method not implemented.');
  }

  constructor(ast: LPNode, params: Type[], retTy: Type) {
    super('<function>', ast);
    this.params = params;
    this.retTy = retTy;
  }

  static matrixSelect(fns: Fn[], args: Type[], scope: Scope): [Fn, [Type[], Type]][] {
    const originalLength = fns.length;
    // remove any fns that shouldn't apply
    const callTy = new FunctionType(new NulLP(), args, Type.generate());
    fns = fns.filter((fn) => fn.ty.compatibleWithConstraint(callTy, scope));
    // if it's 0-arity then all we have to do is grab the retTy of the fn
    if (args.length === 0) {
      return fns.reduce(
        (fns, fn) => [
          ...fns,
          [fn, [
            fn.params.map(p => p.ty.instance()),
            fn.retTy.instance(),
          ]],
        ],
        new Array<[Fn, [Type[], Type]]>(),
      );
    }
    // and now to generate the matrix
    // every argument is a dimension within the matrix, but we're
    // representing each dimension _d_ as an index in the matrix
    const matrix: Array<Type[]> = args.map((arg) => {
      return arg.fnselectOptions();
    });
    // TODO: this weight system feels like it can be inaccurate
    // the weight of a particular function is computed by the sum
    // of the indices in each dimension, with the highest sum
    // having the greatest preference
    const fnsByWeight = new Map<number, [Fn, [Type[], Type]][]>();
    const indices = matrix.map(() => 0);
    // keep it as for instead of while for debugging reasons
    for (let i = 0; ; i++) {
      const weight = indices.reduce((w, c) => w + c);
      const argTys = matrix.map((options, ii) => options[indices[ii]]);
      const fnsForWeight = fnsByWeight.get(weight) || [];
      fnsForWeight.push(
        ...fns.reduce((fns, fn) => {
          const retTy = fn.signatureFor(argTys, scope);
          if (retTy === null) {
            return fns;
          } else {
            return [...fns, [fn, retTy] as [Fn, [Type[], Type]]];
          }
        }, new Array<[Fn, [Type[], Type]]>()),
      );
      fnsByWeight.set(weight, fnsForWeight);
      if (
        indices.every(
          (idxInDim, dimIdx) => idxInDim === matrix[dimIdx].length - 1,
        )
      ) {
        break;
      }
      // now change the indices. This mostly works like binary addition,
      // except each digit `i` is in base `j` where `j = matrix[i].length`
      // so if `matrix[1].length === 1` then the carry for `indices[1]`
      // is just passed to `matrix[0]`
      indices.reduceRight((carry, _matIdx, idx) => {
        indices[idx] += carry;
        if (indices[idx] === matrix[idx].length) {
          indices[idx] = 0;
          return 1;
        } else {
          return 0;
        }
      }, 1);
    }
    const weights = Array.from(fnsByWeight.keys()).sort();
    // weights is ordered lowest->highest so it's just a matter of
    // appending the tuple at each weight to a list
    const ret = weights.reduce((fns, weight) => {
      let weightFns = fnsByWeight.get(weight);
      weightFns = weightFns.filter(
        ([weightedFn, _retTy]) =>
          fns.findIndex(([fn, _retTy]) => fn === weightedFn) === -1,
      );
      return [...fns, ...weightFns];
    }, new Array<[Fn, [Type[], Type]]>());
    if (ret.length > originalLength || ret.length === 0) {
      console.log('~~~ ERROR');
      console.log('original: ', originalLength);
      console.log('retLength:', ret.length);
      console.log('args:     ', args);
      console.log('matrix:   ', matrix);
      console.log('byweight: ', fnsByWeight);
      console.log('indices:  ', indices);
      throw new Error('somehow got more options when fn selecting');
    }
    return ret;
  }

  compatibleWithConstraint(ty: Type, scope: Scope): boolean {
    if (ty instanceof FunctionType) {
      return (
        this.params.length === ty.params.length &&
        this.params.every((param, ii) =>
          param.compatibleWithConstraint(ty.params[ii], scope),
        ) &&
        this.retTy.compatibleWithConstraint(ty.retTy, scope)
      );
    } else if (ty instanceof OneOf || ty instanceof Generated) {
      return ty.compatibleWithConstraint(this, scope);
    } else {
      return false;
    }
  }

  constrain(to: Type, scope: Scope): void {
    console.log(to);
    TODO('figure out what it means to constrain a function type');
  }

  eq(that: Equalable): boolean {
    if (that instanceof FunctionType) {
      return (
        this.params.length === that.params.length &&
        this.params.every((param, ii) => param.eq(that.params[ii])) &&
        this.retTy.eq(that.retTy)
      );
    } else if (that instanceof Generated || that instanceof OneOf) {
      return that.eq(this);
    } else {
      return false;
    }
  }

  instance(): Type {
    return new FunctionType(
      this.ast,
      this.params.map((param) => param.instance()),
      this.retTy.instance(),
    );
  }

  tempConstrain(to: Type, scope: Scope): void {
    console.log(to);
    TODO('temp constraints on a function type?');
  }

  resetTemp(): void {
    TODO('temp constraints on a function type?');
  }

  size(): number {
    throw new Error('Size should not be requested for function types...');
  }
}

class Struct extends Type {
  args: GenericArgs;
  fields: Fields;
  order: FieldIndices;

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
    for (const fieldName in this.fields) {
      this.order[fieldName] = sizeTracker;
      sizeTracker += this.fields[fieldName].size();
    }
  }

  static fromAst(ast: LPNode, scope: Scope): Type {
    let work = ast;
    const names = parseFulltypename(work.get('fulltypename'));
    if (names[1].some((ty) => ty[1].length !== 0)) {
      throw new Error(
        `Generic type variables can't have generic type arguments`,
      );
    }
    const typeName = names[0];
    const genericArgs: GenericArgs = {};
    names[1].forEach((n) => (genericArgs[n[0]] = null));

    work = ast.get('typedef');
    if (work.has('typebody')) {
      work = work.get('typebody').get('typelist');
      const lines = [
        work.get('typeline'),
        ...work
          .get('cdr')
          .getAll()
          .map((n) => n.get('typeline')),
      ];
      const fields: Fields = {};
      lines.forEach((line) => {
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
      return (
        this.fields.hasOwnProperty(ty.name) &&
        this.fields[ty.name].compatibleWithConstraint(ty.ty, scope)
      );
    } else if (ty instanceof HasMethod) {
      TODO(
        'get methods and operators for types? (probably during fn selection fix?)',
      );
    } else if (ty instanceof Interface || ty instanceof OneOf) {
      return ty.compatibleWithConstraint(this, scope);
    } else {
      return false;
    }
  }

  constrain(to: Type, scope: Scope) {
    if (!this.compatibleWithConstraint(to, scope)) {
      throw new Error(
        `incompatible types: ${this.name} is not compatible with ${to.name}`,
      );
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
    return Object.values(this.fields)
      .map((ty) => ty.size())
      .reduce((l, r) => l + r);
  }
}

abstract class Has extends Type {
  get ammName(): string {
    throw new Error(
      'None of the `Has` constraints should have their ammName requested...',
    );
  }

  constructor(name: string, ast: LPNode | null) {
    super(name, ast);
  }

  static field(field: HasField, ty: Type): boolean {
    // TODO: structs
    return false;
  }

  static method(method: HasMethod, scope: Scope, ty: Type): [Fn, [Type[], Type]][] {
    const fns = scope.get(method.name);
    if (!isFnArray(fns)) {
      return [];
    }
    return FunctionType.matrixSelect(
      fns,
      method.params.map((p) => (p === null ? ty : p)),
      scope,
    );
  }

  static operator(operator: HasOperator, scope: Scope, ty: Type): Operator[] {
    let ops: Operator[] = scope.get(operator.name);
    // if there is no op by that name, RIP
    if (!isOpArray(ops)) {
      return [];
    }
    // filter out ops that aren't the same fixity
    ops = ops.filter((op) => op.isPrefix === operator.isPrefix);
    if (operator.isPrefix) {
      return ops.filter(
        (op) => op.select(scope, operator.params[0] || ty) !== [],
      );
    } else {
      return ops.filter(
        (op) =>
          op.select(
            scope,
            operator.params[0] || ty,
            operator.params[1] || ty,
          ) !== [],
      );
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

  // it returns `any` to make the type system happy
  private nope(msg: string): any {
    throw new Error(
      `Has constraints ${msg} (this error should never be thrown)`,
    );
  }

  instance(): Type {
    return this.nope('cannot represent a compilable type');
  }

  // there should never be a case where `Type.hasX(...).tempConstrain(...)`
  tempConstrain(_t: Type) {
    this.nope('cannot be temporarily constrained');
  }

  // there can never be temp constraints
  resetTemp() {
    this.nope('cannot have temporary constraints');
  }

  size(): number {
    return this.nope('do not have a size');
  }

  fnselectOptions(): Type[] {
    return this.nope(
      'should not be used as a Type when computing function selection',
    );
  }
}

class HasField extends Has {
  ty: Type;

  constructor(name: string, ast: LPNode | null, ty: Type) {
    super(name, ast);
    this.ty = ty;
  }

  static fromPropertyTypeLine(ast: LPNode, scope: Scope): HasField {
    const name = ast.get('variable').t.trim();
    const ty = Type.getFromTypename(ast.get('fulltypename'), scope);
    return new HasField(name, ast, ty);
  }

  eq(that: Equalable): boolean {
    return super.eq(that) && that instanceof HasField && that.ty.eq(this.ty);
  }
}

class HasMethod extends Has {
  // null if it refers to the implementor's type. Only used when
  // working on interfaces
  params: (Type | null)[];
  ret: Type | null;

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

  static fromFunctionTypeLine(
    ast: LPNode,
    scope: Scope,
    ifaceName: string,
  ): HasMethod {
    const name = ast.get('variable').t.trim();
    const work = ast.get('functiontype');
    const params: (Type | null)[] = [
      work.get('fulltypename'),
      ...work
        .get('cdr')
        .getAll()
        .map((cdr) => cdr.get('fulltypename')),
    ].map((tyNameAst) =>
      tyNameAst.t.trim() === ifaceName
        ? null
        : Type.getFromTypename(tyNameAst, scope),
    );
    const ret =
      work.get('returntype').t.trim() === ifaceName
        ? null
        : Type.getFromTypename(work.get('returntype'), scope);
    return new HasMethod(name, ast, params, ret);
  }

  eq(that: Equalable): boolean {
    return (
      super.eq(that) &&
      that instanceof HasMethod &&
      this.params.reduce(
        (eq, param, ii) =>
          eq &&
          (param === null
            ? that.params[ii] === null
            : param.eq(that.params[ii])),
        true,
      ) &&
      this.ret.eq(that.ret)
    );
  }
}

class HasOperator extends HasMethod {
  isPrefix: boolean;

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

  static fromOperatorTypeLine(
    ast: LPNode,
    scope: Scope,
    ifaceName: string,
  ): HasOperator {
    let isPrefix = true;
    const params: (Type | null)[] = [];
    if (ast.get('optleftarg').has()) {
      const leftTypename = ast.get('optleftarg').get('leftarg');
      const leftTy =
        leftTypename.t.trim() === ifaceName
          ? null
          : Type.getFromTypename(leftTypename, scope);
      params.push(leftTy);
      isPrefix = false;
    }
    const op = ast.get('operators').t.trim();
    const rightTypename = ast.get('rightarg');
    const rightTy =
      rightTypename.t.trim() === ifaceName
        ? null
        : Type.getFromTypename(rightTypename, scope);
    params.push(rightTy);
    const retTypename = ast.get('fulltypename');
    const retTy =
      retTypename.t.trim() === ifaceName
        ? null
        : Type.getFromTypename(retTypename, scope);
    return new HasOperator(op, ast, params, retTy, isPrefix);
  }

  eq(that: Equalable): boolean {
    return (
      super.eq(that) &&
      that instanceof HasOperator &&
      this.isPrefix === that.isPrefix
    );
  }
}

class Interface extends Type {
  // TODO: it's more optimal to have fields, methods, and operators in
  // maps so we can cut down searching and such.
  fields: HasField[];
  methods: HasMethod[];
  operators: HasOperator[];
  tempDelegate: Type | null;
  isDuped: boolean;

  get ammName(): string {
    if (this.tempDelegate) {
      return this.tempDelegate.ammName;
    } else {
      return this.name;
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
        ...work
          .get('cdr')
          .getAll()
          .map((cdr) => cdr.get('interfaceline')),
      ];
      const fields: HasField[] = [];
      const methods: HasMethod[] = [];
      const operators: HasOperator[] = [];
      lines.forEach((line) => {
        if (line.has('propertytypeline')) {
          fields.push(
            HasField.fromPropertyTypeLine(line.get('propertytypeline'), scope),
          );
        } else if (line.has('functiontypeline')) {
          methods.push(
            HasMethod.fromFunctionTypeLine(
              line.get('functiontypeline'),
              scope,
              name,
            ),
          );
        } else if (line.has('operatortypeline')) {
          operators.push(
            HasOperator.fromOperatorTypeLine(
              line.get('operatortypeline'),
              scope,
              name,
            ),
          );
        } else {
          throw new Error(`invalid ast: ${work}`);
        }
      });
      return new Interface(name, ast, fields, methods, operators);
    } else if (work.has('interfacealias')) {
      TODO('interface aliases');
    } else {
      throw new Error(`invalid ast: ${work}`);
    }
  }

  compatibleWithConstraint(ty: Type, scope: Scope): boolean {
    if (this.tempDelegate !== null) {
      return this.tempDelegate.compatibleWithConstraint(ty, scope);
    }
    if (ty instanceof Opaque || ty instanceof Struct) {
      if (ty instanceof Opaque && this.fields.length !== 0) {
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
        if (!this.fields.every((field) => Has.field(field, ty))) {
          return false;
        }
      }

      // check methods
      return (
        this.methods.every((m) => Has.method(m, scope, ty)) &&
        this.operators.every((o) => Has.operator(o, scope, ty))
      );
      // check operators
    } else if (ty instanceof Interface) {
      // ensure `ty ⊆ this`
      return (
        ty.fields.every((field) => this.fields.find((f) => f.eq(field))) &&
        ty.methods.every((method) => this.methods.find((m) => m.eq(method))) &&
        ty.operators.every((operator) =>
          this.operators.find((o) => o.eq(operator)),
        )
      );
    } else if (ty instanceof HasField) {
      return this.fields.some((field) => field.eq(ty));
    } else if (ty instanceof HasOperator) {
      return this.operators.some((operator) => operator.eq(ty));
    } else if (ty instanceof HasMethod) {
      return this.methods.some((method) => method.eq(ty));
    } else if (ty instanceof Generated || ty instanceof OneOf) {
      return ty.compatibleWithConstraint(this, scope);
    } else {
      throw new Error(
        `unsure of what type the constraint is - this error should never be thrown!`,
      );
    }
  }

  constrain(ty: Type, scope: Scope) {
    const baseErrorString = `type ${ty.name} was constrained to interface ${this.name} but doesn't have`;
    this.fields.forEach((f) => {
      if (!ty.compatibleWithConstraint(f, scope)) {
        throw new Error(`${baseErrorString} field ${f.name} with type ${f.ty}`);
      }
    });
    this.methods.forEach((m) => {
      if (!Has.method(m, scope, ty)) {
        throw new Error(
          `${baseErrorString} method ${m.name}(${m.params
            .map((p) => (p === null ? ty : p))
            .map((t) => t.name)
            .join(', ')})`,
        );
      }
    });
    this.operators.forEach((o) => {
      if (Has.operator(o, scope, ty)) return;
      if (o.isPrefix) {
        throw new Error(
          `${baseErrorString} prefix operator \`${o.name} ${ty.name}\``,
        );
      } else {
        throw new Error(
          `${baseErrorString} infix operator \`${o.params[0] || ty.name} ${
            o.name
          } ${o.params[1] || ty.name}\``,
        );
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
    } else if (this.isDuped) {
      // if it's duped, allow the instance to be `this` since
      // it needs to be tempConstrain-ed in order to have the
      // resulting instance type
      return this;
    } else {
      throw new Error(`Could not resolve interface type`);
    }
  }

  tempConstrain(that: Type, scope: Scope) {
    if (this === that || this.eq(that)) {
      // do nothing
    } else if (
      that instanceof Interface &&
      this.fields.every(f => that.compatibleWithConstraint(f, scope)) &&
      this.methods.every(m => that.compatibleWithConstraint(m, scope)) &&
      this.operators.every(o => that.compatibleWithConstraint(o, scope))
    ) {
      // compatibleWithConstraint returned true if `that ⊆ this`
      // but when tempConstraining, it's valid for `this ⊆ that`
      if (this.tempDelegate !== null) {
        this.tempDelegate.tempConstrain(
          that.tempDelegate ?? that,
          scope,
        );
      } else {
        this.tempDelegate = that;
      }
    } else if (this.compatibleWithConstraint(that, scope)) {
      if (this.tempDelegate !== null) {
        if (this.tempDelegate instanceof Interface && !(this.tempDelegate instanceof Generated)) {
          this.tempDelegate.tempConstrain(that, scope);
        } else {
          this.tempDelegate.constrain(that, scope);
        }
      } else {
        this.tempDelegate = that;
      }
    } else {
      throw new Error(`type ${this.ammName} is not compatible with ${that.ammName}`);
    }
  }

  resetTemp() {
    if (this.tempDelegate !== null) {
      this.tempDelegate.resetTemp();
      this.tempDelegate = null;
    }
  }

  dupIfNotLocalInterface(): Type | null {
    if (this.isDuped) return null;
    const dup = new Interface(
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
      TODO(
        `figure out how Interface should return from size() if there's not tempDelegate`,
      );
    }
  }

  fnselectOptions(): Type[] {
    if (this.tempDelegate !== null) {
      return this.tempDelegate.fnselectOptions();
    } else {
      return [this];
    }
  }
}

// technically, generated types are a kind of interface - we just get to build up
// the interface through type constraints instead of through explicit requirements.
// this'll make untyped fn parameters easier once they're implemented.
class Generated extends Interface {
  delegate: Type | null;

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
      throw new Error('ugh');
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
      throw new Error(
        `cannot process temporary type constraints before permanent type constraints`,
      );
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
      if (to.isDuped) {
        this.delegate = to;
      } else {
        this.fields.push(...to.fields);
        this.methods.push(...to.methods);
        this.operators.push(...to.operators);
        if (to instanceof Generated && to.delegate !== null) {
          this.delegate = to.delegate;
        }
      }
    } else {
      this.delegate = to;
      this.constrain(this.delegate, scope);
    }
  }

  eq(that: Equalable): boolean {
    if (that instanceof Generated) {
      return (
        this.delegate !== null &&
        that.delegate !== null &&
        this.delegate.eq(that.delegate)
      );
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
      TODO(
        `figure out how Generated should return from size() if there's not tempDelegate`,
      );
    }
  }

  fnselectOptions(): Type[] {
    if (this.delegate) {
      return this.delegate.fnselectOptions();
    } else {
      return super.fnselectOptions();
    }
  }
}

class OneOf extends Type {
  private __original: Type[];
  selection: Type[];
  tempSelect: Type[] | null;
  private selected: Type | null;

  get ammName(): string {
    return this.select().ammName;
  }

  constructor(selection: Type[], tempSelect: Type[] = null) {
    super(genName());
    // ensure there's no duplicates. This fixes an issue with duplicates
    // in matrix selection. Since no other values are added to the list,
    // there's no need to do this any time later.
    selection = selection.reduce(
      (sel, fn) => (sel.some((selFn) => selFn.eq(fn)) ? sel : [...sel, fn]),
      new Array<Type>(),
    );
    this.selection = selection;
    this.tempSelect = tempSelect;
    this.selected = null;
    if (this.selection.length === 0) {
      throw new Error('cannot have one of 0 types');
    }
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
      console.log('-------------');
      console.log('before:', this.selected);
      console.log('after:', selected);
      console.log('this:', this);
      TODO('uh somehow selected different types - check on this');
    }
    return selected;
  }

  compatibleWithConstraint(constraint: Type, scope: Scope): boolean {
    return this.selection.some((ty) =>
      ty.compatibleWithConstraint(constraint, scope),
    );
  }

  constrain(constraint: Type, scope: Scope) {
    this.selection = this.selection.filter((ty) =>
      ty.compatibleWithConstraint(constraint, scope),
    );
  }

  eq(that: Equalable): boolean {
    return (
      that instanceof OneOf &&
      this.selection.length === that.selection.length &&
      this.selection.every((ty, ii) => ty.eq(that.selection[ii]))
    );
  }

  instance(): Type {
    const selected = this.select();
    if (selected === undefined) {
      throw new Error('uh whaaaaat');
    }
    return selected.instance();
  }

  tempConstrain(to: Type, scope: Scope) {
    this.tempSelect = this.selection.filter((ty) =>
      ty.compatibleWithConstraint(to, scope),
    );
    this.selected = null;
  }

  resetTemp() {
    this.tempSelect = [];
    this.selected = null;
  }

  size(): number {
    return this.select().size();
  }

  fnselectOptions(): Type[] {
    // this still maintains preference order: say that this OneOf is somehow:
    // [string, OneOf(int64, float64), bool]
    // after the reduce, the result should be:
    // [string, int64, float64, bool]
    // which makes sense - highest preference no matter what should be `bool`,
    // but the 2nd-level preference locally is either `int64` or `float64`.
    // However, the nested `OneOf` prefers `float64`, so it wins the tie-breaker.
    //
    // An optimization step for `FunctionType.matrixSelect` *could* be to ensure
    // that this list does not contain types that `.eq` each other (eliminating
    // the element that is earlier in the list) but we'd need to perform
    // profiling to see if that doesn't drastically reduce performance in the
    // common case
    return this.selection.reduce(
      (options, ty) => [...options, ...ty.fnselectOptions()],
      new Array<Type>(),
    );
  }
}
