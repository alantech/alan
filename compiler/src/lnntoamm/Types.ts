import { stdout } from 'process';
import { LPNode, NulLP } from '../lp';
import { fulltypenameAstFromString } from './Ast';
import Fn from './Fn';
import Operator from './Operator';
import Scope from './Scope';
import {
  DBG,
  Equalable,
  genName,
  isFnArray,
  isOpArray,
  matrixIndices,
  TODO,
} from './util';

type Fields = { [name: string]: Type | null };
export type FieldIndices = { [name: string]: number };
type GenericArgs = { [name: string]: Type | null };
type TypeName = [string, TypeName[]];
interface Generalizable {
  generics: GenericArgs;
  solidify(types: Type[]): Type;
}
const generalizable = (val: Type): val is Type & Generalizable =>
  'generics' in val;

// note: if more opt types are used, use `InterfaceDupOpts & OtherDupOpts`
export type ConstrainOpts = OneOfConstrainOpts;
interface OneOfConstrainOpts {
  stopAt?: Type;
}

export type DupOpts = InterfaceDupOpts;
interface InterfaceDupOpts {
  isTyVar?: boolean;
}

export type InstanceOpts = InterfaceInstanceOpts;
interface InterfaceInstanceOpts {
  interfaceOk?: boolean;
  forSameDupIface?: [Type, Type][];
}

export type TempConstrainOpts = InterfaceTempConstrainOpts;
interface InterfaceTempConstrainOpts {
  isTest?: boolean;
}

const parseFulltypename = (node: LPNode): TypeName => {
  const name = node.get('typename').t.trim();
  const genericTys: TypeName[] = [];
  if (node.get('opttypegenerics').has()) {
    const generics = node.get('opttypegenerics').get('generics');
    genericTys.push(parseFulltypename(generics.get('fulltypename')));
    genericTys.push(
      ...generics
        .get('cdr')
        .getAll()
        .map((n) => n.get('fulltypename'))
        .map(parseFulltypename),
    );
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
  abstract constrain(to: Type, scope: Scope, opts?: ConstrainOpts): void;
  abstract contains(ty: Type): boolean;
  abstract eq(that: Equalable): boolean;
  abstract instance(opts?: InstanceOpts): Type;
  abstract tempConstrain(
    to: Type,
    scope: Scope,
    opts?: TempConstrainOpts,
  ): void;
  abstract resetTemp(): void;
  abstract size(): number;

  static getFromTypename(
    name: LPNode | string,
    scope: Scope,
    dupOpts?: DupOpts,
  ): Type {
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
          throw new Error(
            `Bad typename: type ${name} expects ${genericArgLen} type arguments, but ${generics.length} were provided`,
          );
        }
        const solidifiedTypeArgs = generics.map(solidify);
        // interfaces can't have generic type params so no need to call
        // dupIfNotLocalInterface
        return ty.solidify(solidifiedTypeArgs);
      } else if (generics.length !== 0) {
        throw new Error(
          `Bad typename: type ${name} doesn't expect any type arguments, but ${generics.length} were provided`,
        );
      } else {
        const duped = ty.dup(dupOpts);
        if (duped === null) {
          return ty;
        } else {
          // note: if scope isn't *only* for the function's arguments,
          // this'll override the module scope and that would be bad.
          scope.put(name, duped);
          return duped;
        }
      }
    };
    return solidify(parsed);
  }

  static builtinInterface(
    name: string,
    fields: HasField[],
    methods: HasMethod[],
    operators: HasOperator[],
  ) {
    return new Interface(name, new NulLP(), fields, methods, operators);
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

  // TODO: generalize for struct and any other generalizable type
  static flatten(tys: Type[]): Type {
    tys = tys.reduce((allTys, ty) => [...allTys, ...ty.fnselectOptions()], []);
    let outerType: Type = null;
    let isSquishy = true;
    const innerTys = tys.reduce((genTys, ty) => {
      // shallow equality (eg Result<int64> and Result<string> will match here)
      if (outerType === null) {
        outerType = ty;
      }
      if (generalizable(ty)) {
        if (ty.name !== outerType.name) {
          isSquishy = false;
        } else {
          return Object.values(ty.generics).map((genTy, ii) => [...(genTys[ii] || []), genTy]);
        }
      } else {
        isSquishy = false;
      }
    }, new Array<Type[]>());

    if (isSquishy) {
      return (outerType as Opaque).solidify(innerTys.map(Type.oneOf));
    } else {
      return null;
    }
  }

  dup(opts?: DupOpts): Type | null {
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

class Opaque extends Type implements Generalizable {
  // if any values are `null` that means that this isn't an instantiable type
  // and should be treated like the type needs to be duped
  generics: GenericArgs;

  get ammName(): string {
    let generics = '';
    if (Object.keys(this.generics).length !== 0) {
      const genNames = new Array<string>();
      for (const [tyVar, ty] of Object.entries(this.generics)) {
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

  constructor(name: string, generics: string[]) {
    super(name);
    this.generics = {};
    generics.forEach((g) => (this.generics[g] = null));
  }

  compatibleWithConstraint(that: Type, scope: Scope): boolean {
    if (this.eq(that)) {
      return true;
    }
    if (that instanceof Opaque) {
      const thisGens = Object.values(this.generics);
      const thatGens = Object.values(that.generics);
      if (this.name !== that.name || thisGens.length !== thatGens.length) {
        return false;
      }
      return (
        this.name === that.name &&
        thisGens.length === thatGens.length &&
        thisGens.every((thisGen, ii) => {
          const thatGen = thatGens[ii];
          if (thisGen === null || thatGen === null) {
            return true;
          } else {
            return thisGen.compatibleWithConstraint(thatGen, scope);
          }
        })
      );
    } else if (that instanceof Interface || that instanceof OneOf) {
      return that.compatibleWithConstraint(this, scope);
    } else if (that instanceof HasField) {
      return false;
    } else if (that instanceof HasOperator) {
      return Has.operator(that, scope, this).length !== 0;
    } else if (that instanceof HasMethod) {
      return Has.method(that, scope, this)[0].length !== 0;
    } else {
      TODO('Opaque constraint compatibility with other types');
    }
  }

  constrain(that: Type, scope: Scope, opts?: ConstrainOpts) {
    if (this.eq(that)) {
      return;
    }
    if (!this.compatibleWithConstraint(that, scope)) {
      throw new Error(
        `Cannot constrain type ${this.ammName} to ${that.ammName}`,
      );
    }
    if (that instanceof Opaque) {
      if (
        Object.values(this.generics).some((g) => g === null) ||
        Object.values(that.generics).some((g) => g === null)
      ) {
        // if any values are null values, that means that we just have to check
        // for constraint compatibilities which was already checked so we're good
        return;
      }
      const thisGens = Object.keys(this.generics);
      const thatGens = Object.keys(that.generics);
      thisGens.forEach((genName, ii) =>
        this.generics[genName].constrain(that.generics[thatGens[ii]], scope, opts),
      );
    } else if (that instanceof Interface || that instanceof OneOf) {
      that.constrain(this, scope, opts);
    } else if (that instanceof Struct) {
      throw new Error(`Cannot constrain Opaque type to Struct type`);
    } else {
      console.log(this);
      console.log(that);
      throw 'uh';
    }
  }

  contains(that: Type): boolean {
    return this.eq(that);
  }

  dup(): Type | null {
    const genKeys = Object.keys(this.generics);
    if (genKeys.length === 0) {
      return null;
    }
    const duped = new Opaque(this.name, genKeys);
    let isNothingNew = true;
    genKeys.forEach((genName) => {
      const thisGen = this.generics[genName];
      let tyVal: Type;
      if (thisGen === null) {
        tyVal = Type.generate();
      } else {
        const duped = thisGen.dup();
        if (duped === null) {
          tyVal = thisGen;
        } else {
          tyVal = duped;
          isNothingNew = false;
        }
      }
      duped.generics[genName] = tyVal;
    });
    return duped;
  }

  eq(that: Equalable): boolean {
    if (!(that instanceof Opaque) || this.name !== that.name) {
      return false;
    }
    const thisGens = Object.values(this.generics);
    const thatGens = Object.values(that.generics);
    return (
      thisGens.length === thatGens.length &&
      thisGens.every((thisGen, ii) => {
        const thatGen = thatGens[ii];
        if (thisGen === null || thatGen === null) {
          return thisGen === thatGen;
        } else {
          return thisGen.eq(thatGen);
        }
      })
    );
  }

  fnselectOptions(): Type[] {
    const genOptions = Object.values(this.generics).map(
      (g) => g?.fnselectOptions() ?? [g],
    );
    const opts = new Array<Type>();
    const toSolidify = new Opaque(this.name, Object.keys(this.generics));
    for (const indices of matrixIndices(genOptions)) {
      opts.push(
        toSolidify.solidify(
          indices.map((optIdx, tyVarIdx) => genOptions[tyVarIdx][optIdx]),
        ),
      );
    }
    return opts;
  }

  instance(opts?: InstanceOpts): Type {
    const genNames = Object.keys(this.generics);
    if (genNames.length === 0) {
      // minor optimization: if there's no generics then we keep the same JS
      // object to reduce the number of allocs
      return this;
    }
    const instance = new Opaque(this.name, genNames);
    for (const name of genNames) {
      const thisGen = this.generics[name];
      if (thisGen === null) {
        throw new Error(
          `Cannot get an instance of a generic Opaque type that hasn't been solidified`,
        );
      }
      instance.generics[name] = thisGen.instance(opts);
    }
    return instance;
  }

  isFixed(): boolean {
    switch (this.name) {
      case 'bool':
      case 'float32':
      case 'float64':
      case 'int8':
      case 'int16':
      case 'int32':
      case 'int64':
      case 'void':
        return true;
      default:
        return false;
    }
  }

  size(): number {
    switch (this.name) {
      case 'void':
        return 0;
      case 'bool':
      case 'float32':
      case 'float64':
      case 'int8':
      case 'int16':
      case 'int32':
      case 'int64':
        return 1;
      default:
        const containedTypes = Object.values(this.generics);
        return containedTypes
          .map((t) => {
            if (t === null) {
              throw new Error(`cannot compute size of ${this.ammName}`);
            } else {
              return t.size();
            }
          })
          .reduce((s1, s2) => s1 + s2, 1);
    }
  }

  solidify(tys: Type[]): Type {
    const genNames = Object.keys(this.generics);
    if (genNames.length < tys.length) {
      throw new Error(
        `Cannot solidify ${this.ammName} - too many type arguments were provided`,
      );
    } else if (genNames.length > tys.length) {
      throw new Error(
        `Cannot solidify ${this.ammName} - not enough type arguments were provided`,
      );
    } else if (genNames.length === 0) {
      return this;
    } else {
      const duped = new Opaque(this.name, genNames);
      genNames.forEach((name, ii) => (duped.generics[name] = tys[ii]));
      return duped;
    }
  }

  tempConstrain(that: Type, scope: Scope, opts?: TempConstrainOpts) {
    if (!this.compatibleWithConstraint(that, scope)) {
      throw new Error(
        `Cannot temporarily constrain type ${this.ammName} to ${that.ammName}`,
      );
    }
    if (that instanceof Opaque) {
      const thisGens = Object.keys(this.generics);
      const thatGens = Object.keys(that.generics);
      thisGens.forEach((thisGenName, ii) => {
        const thisGen = this.generics[thisGenName];
        const thatGen = that.generics[thatGens[ii]];
        if (thisGen === null || thatGen === null) {
          throw new Error(`Can't tempConstrain non-solidified Opaque types`);
        } else {
          thisGen.tempConstrain(thatGen, scope, opts);
        }
      });
    } else if (that instanceof Interface) {
      const tcTo = that.delegate ?? that.tempDelegate;
      if (tcTo !== null) {
        this.tempConstrain(tcTo, scope, opts);
      }
    } else if (that instanceof OneOf) {
      // we're happy, no need to tempConstrain
    } else {
      console.log(this);
      console.log(that);
      throw 'uh';
    }
  }

  resetTemp() {
    for (const generic in this.generics) {
      if (this.generics[generic] === null) {
        return;
      }
      this.generics[generic].resetTemp();
    }
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

  /*
  Original design comment (was written as a FIXME above `Expr.Call.inline`):
  Currently, this only works because of the way `root.lnn` is structured -
  functions that accept f32s are defined first and i64s are defined last.
  However, we can't rely on function declaration order to impact type checking
  or type inferrence, since that could unpredictably break users' code. Instead,
  if we have `OneOf` types, we should prefer the types in its list in ascending
  order. I think that the solution is to create a matrix of all of the possible
  types to each other, insert functions matching the types in each dimension,
  and pick the function furthest from the all-0 index. For example, given
  `1 + 2`, the matrix would be:
  |         |  float32   |  float64   |   int8   |   int16    |   int32    |   int64    |
  | float32 |add(f32,f32)|            |          |            |            |            |
  | float64 |            |add(f64,f64)|          |            |            |            |
  |  int8   |            |            |add(i8,i8)|            |            |            |
  |  int16  |            |            |          |add(i16,i16)|            |            |
  |  int32  |            |            |          |            |add(i32,i32)|            |
  |  int64  |            |            |          |            |            |add(i64,i64)|
  in this case, it would prefer `add(int64,int64)`. Note that constraining the
  type will impact this: given the code `const x: int8 = 0; const y = x + 1;`,
  the matrix would be:
  |         | float32 | float64 |    int8    | int16 | int32 | int64 |
  |  int8   |         |         | add(i8,i8) |       |       |       |
  where the columns represent the type of the constant `1`. There's only 1
  possibility, but we'd still have to check `int8,int64`, `int8,int32`, and
  `int8,int16` until it finds `int8,int8`.
  */
  static matrixSelect(
    fns: Fn[],
    args: Type[],
    expectResTy: Type,
    scope: Scope,
  ): [Fn[], Type[][], Type[]] {
    // super useful when debugging matrix selection
    const isDbg = false;
    isDbg && console.log('STARTING', fns);
    const original = [...fns];
    // remove any fns that shouldn't apply
    const callTy = new FunctionType(new NulLP(), args, expectResTy);
    isDbg && stdout.write('callTy: ') && console.dir(callTy, { depth: 6 });
    fns = fns.filter((fn) => fn.ty.compatibleWithConstraint(callTy, scope));
    isDbg && console.log('filtered:', fns);
    // if it's 0-arity then all we have to do is grab the retTy of the fn
    if (args.length === 0) {
      isDbg && console.log('nothin');
      return fns.reduce(
        ([fns, _pTys, retTys], fn) => {
          const alreadyFn = fns.findIndex((alreadyFn) => alreadyFn === fn);
          if (alreadyFn === -1) {
            return [
              [...fns, fn],
              _pTys,
              [...retTys, fn.retTy.instance({ interfaceOk: true, forSameDupIface: [] })],
            ];
          } else {
            return [fns, _pTys, retTys];
          }
        },
        [new Array<Fn>(), new Array<Type[]>(), new Array<Type>()],
      );
    }
    // and now to generate the matrix
    // every argument is a dimension within the matrix, but we're
    // representing each dimension _d_ as an index in the matrix
    const matrix: Array<Type[]> = args.map((arg) => {
      return arg.fnselectOptions();
    });
    isDbg && console.log('matrix:', matrix);
    // TODO: this weight system feels like it can be inaccurate
    // the weight of a particular function is computed by the sum
    // of the indices in each dimension, with the highest sum
    // having the greatest preference
    const fnsByWeight = new Map<number, [Fn, Type[], Type][]>();
    // keep it as for instead of while for debugging reasons
    for (const indices of matrixIndices(matrix)) {
      const weight = indices.reduce((w, c) => w + c);
      isDbg && console.log('weight', weight);
      const argTys = matrix.map((options, ii) => options[indices[ii]]);
      isDbg && console.log('argtys', argTys);
      const fnsForWeight = fnsByWeight.get(weight) || [];
      isDbg && console.log('for weight', fnsForWeight);
      fnsForWeight.push(
        ...fns.reduce((fns, fn) => {
          isDbg && console.log('getting result ty');
          const tys = fn.resultTyFor(argTys, expectResTy, scope, { isTest: true });
          isDbg && console.dir(tys, { depth: 4 });
          if (tys === null) {
            return fns;
          } else {
            return [...fns, [fn, ...tys] as [Fn, Type[], Type]];
          }
        }, new Array<[Fn, Type[], Type]>()),
      );
      isDbg && console.log('for weight', weight, 'now', fnsForWeight);
      fnsByWeight.set(weight, fnsForWeight);
    }
    const weights = Array.from(fnsByWeight.keys()).sort((a, b) => a - b);
    isDbg && console.log('assigned weights:', weights);
    // weights is ordered lowest->highest
    const ret: [Fn[], Type[][], Type[]] = weights.reduce(
      ([fns, pTys, retTys], weight) => {
        fnsByWeight
          .get(weight)
          .forEach(([weightedFn, weightedPTys, weightedRetTy]) => {
            const alreadyIdx = fns.findIndex((fn) => fn === weightedFn);
            if (alreadyIdx === -1) {
              fns.push(weightedFn);
            } else {
              // push to the end of the fns since technically the weight for the function
              // is indeed higher than what it was before
              fns.push(fns.splice(alreadyIdx, 1)[0]);
            }
            pTys = weightedPTys.map((pTy, ii) => [...(pTys[ii] || []), pTy]);
            retTys.push(weightedRetTy);
          });
        return [fns, pTys, retTys];
      },
      [new Array<Fn>(), new Array<Type[]>(), new Array<Type>()] as [
        Fn[],
        Type[][],
        Type[],
      ],
    );
    if (ret[0].length > original.length || ret[0].length === 0) {
      // these make debugging easier :)
      // console.log('~~~ ERROR');
      // console.log('original: ', original);
      // console.log('ret:      ', ret);
      // console.log('retLength:', ret[0].length);
      // console.log('args:     ', args);
      // console.log('matrix:   ', matrix);
      // console.log('byweight: ', fnsByWeight);
      if (ret[0].length === 0) {
        throw new Error('no more functions left');
      } else {
        throw new Error('somehow got more options when fn selecting');
      }
    }
    if (isDbg) {
      // const getStack = { stack: '' };
      // Error.captureStackTrace(getStack);
      // console.log('returning from', getStack.stack);
      console.dir(ret, { depth: 4 });
    }
    return ret;
  }

  compatibleWithConstraint(that: Type, scope: Scope): boolean {
    if (this.eq(that)) {
      return true;
    }
    // console.log('fn.compat', this, ty);
    if (that instanceof FunctionType) {
      return (
        this.params.length === that.params.length &&
        this.params.every((param, ii) => {
          // console.log('comparing my param', param, 'to', ty.params[ii]);
          return param.compatibleWithConstraint(that.params[ii], scope);
        }) &&
        this.retTy.compatibleWithConstraint(that.retTy, scope)
      );
    } else if (that instanceof OneOf || that instanceof Generated) {
      return that.compatibleWithConstraint(this, scope);
    } else {
      return false;
    }
  }

  constrain(to: Type, scope: Scope, opts?: ConstrainOpts): void {
    console.log(to);
    TODO('figure out what it means to constrain a function type');
  }

  contains(that: Type): boolean {
    return this.eq(that);
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

  instance(opts?: InstanceOpts): Type {
    return new FunctionType(
      this.ast,
      this.params.map((param) => param.instance(opts)),
      this.retTy.instance(opts),
    );
  }

  tempConstrain(to: Type, scope: Scope, opts?: TempConstrainOpts): void {
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

  compatibleWithConstraint(that: Type, scope: Scope): boolean {
    if (this.eq(that)) {
      return true;
    }
    if (that instanceof Struct) {
      return this.eq(that);
    } else if (that instanceof HasField) {
      return (
        this.fields.hasOwnProperty(that.name) &&
        this.fields[that.name].compatibleWithConstraint(that.ty, scope)
      );
    } else if (that instanceof HasMethod) {
      TODO(
        'get methods and operators for types? (probably during fn selection fix?)',
      );
    } else if (that instanceof Interface || that instanceof OneOf) {
      return that.compatibleWithConstraint(this, scope);
    } else {
      return false;
    }
  }

  constrain(that: Type, scope: Scope, opts?: ConstrainOpts) {
    // for every type T, when constraining with some `OneOf` U, T just needs
    // to ensure that T⊆U
    if (that instanceof OneOf) {
      that.constrain(this, scope, opts);
      return;
    }
    if (this.eq(that)) {
      return;
    }
    if (!this.compatibleWithConstraint(that, scope)) {
      throw new Error(
        `incompatible types: ${this.name} is not compatible with ${that.name}`,
      );
    }
    if (that instanceof HasField) {
      that.ty.constrain(this.fields[that.name], scope, opts);
    }
  }

  contains(that: Type): boolean {
    return this.eq(that);
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

  tempConstrain(to: Type, scope: Scope, opts?: TempConstrainOpts) {
    // TODO: can structs have temp constraints?
    // TODO: should be separate implementation when generics are implemented, pass opts
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

  static method(
    method: HasMethod,
    scope: Scope,
    ty: Type,
  ): [Fn[], Type[][], Type[]] {
    const fns = scope.get(method.name);
    if (!isFnArray(fns)) {
      return [[], [], []];
    }
    return FunctionType.matrixSelect(
      fns,
      method.params.map((p) => (p === null ? ty : p)),
      Type.generate(), // accept any type, it doesn't matter
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
        (op) =>
          op.select(scope, Type.generate(), operator.params[0] || ty) !== [],
      );
    } else {
      return ops.filter(
        (op) =>
          op.select(
            scope,
            Type.generate(),
            operator.params[0] || ty,
            operator.params[1] || ty,
          ) !== [],
      );
    }
  }

  // convenience for `Type.hasX(...).compatibleWithConstraint(ty)`
  compatibleWithConstraint(that: Type, scope: Scope): boolean {
    return this.eq(that) || that.compatibleWithConstraint(this, scope);
  }

  // convenience for `Type.hasX(...).constrain(ty)`
  constrain(that: Type, scope: Scope, opts?: ConstrainOpts) {
    if (this.eq(that)) {
      return;
    }
    that.constrain(this, scope);
  }

  contains(that: Type): boolean {
    return false;
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
  delegate: Type | null;
  tempDelegate: Type | null;
  private __isDuped: DupOpts;

  get ammName(): string {
    if (this.delegate !== null) {
      return this.delegate.ammName;
    } else if (this.tempDelegate !== null) {
      return this.tempDelegate.ammName;
    } else {
      throw new Error(`Could not determine ammName for ${this.name}`);
    }
  }

  get isDuped(): boolean {
    return this.__isDuped !== null;
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
    this.delegate = null;
    this.tempDelegate = null;
    this.__isDuped = null;
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

  compatibleWithConstraint(that: Type, scope: Scope): boolean {
    if (this.eq(that)) {
      return true;
    }
    if (that instanceof Has) {
      if (this.isDuped) {
        const checkFor = this.delegate ?? this.tempDelegate;
        if (checkFor !== null) {
          return checkFor.compatibleWithConstraint(that, scope);
        } else {
          // assume true for now, let later constraints give
          return true;
        }
      } else {
        if (that instanceof HasField) {
          return Has.field(that, this);
        } else if (that instanceof HasOperator) {
          return Has.operator(that, scope, this).length !== 0;
        } else if (that instanceof HasMethod) {
          return Has.method(that, scope, this)[0].length !== 0;
        } else {
          throw new Error(`unrecognized Has`);
        }
      }
    } else if (that instanceof Generated) {
      return that.compatibleWithConstraint(this, scope);
    }
    // always check all interface constraints first
    if (
      !(
        this.fields.every((f) => Has.field(f, that)) &&
        this.methods.every((f) => Has.method(f, scope, that)[0].length !== 0) &&
        this.operators.every((f) => Has.operator(f, scope, that).length !== 0)
      )
    ) {
      return false;
    }

    if (this.delegate !== null) {
      return this.delegate.compatibleWithConstraint(that, scope);
    } else if (this.tempDelegate !== null) {
      return this.tempDelegate.compatibleWithConstraint(that, scope);
    } else {
      return true;
    }
  }

  constrain(that: Type, scope: Scope, opts?: ConstrainOpts) {
    if (this.eq(that) || that.contains(this)) {
      return;
    }
    // if it's a `Has`, it's easy enough to process. Generated types should
    // handle the `Has` first before calling this method
    if (that instanceof Has) {
      const toCheck = this.delegate ?? this;
      const errorBase = `${toCheck.ammName} doesn't have`;
      if (that instanceof HasField && !Has.field(that, toCheck)) {
        throw new Error(`${errorBase} field ${that.name}`);
      } else if (
        that instanceof HasOperator &&
        Has.operator(that, scope, toCheck).length !== 0
      ) {
        const opString =
          that.params.length === 1
            ? `${that.name} ${that.params[0].ammName}`
            : `${that.params[0].ammName} ${that.name} ${that.params[1].ammName}`;
        throw new Error(`${errorBase} operator \`${opString}\``);
      } else if (
        that instanceof HasMethod &&
        Has.method(that, scope, toCheck)[0].length !== 0
      ) {
        const paramsString = `(${that.params
          .map((p) => p.ammName)
          .join(', ')})`;
        throw new Error(`${errorBase} method \`${that.name}${paramsString}\``);
      }
      // none of the other checks apply
      return;
    }

    const baseErrorString = `type ${that.name} was constrained to interface ${this.name} but doesn't have`;
    this.fields.forEach((f) => {
      if (!that.compatibleWithConstraint(f, scope)) {
        throw new Error(`${baseErrorString} field ${f.name} with type ${f.ty}`);
      }
    });
    this.methods.forEach((m) => {
      if (!Has.method(m, scope, that)) {
        throw new Error(
          `${baseErrorString} method ${m.name}(${m.params
            .map((p) => (p === null ? that : p))
            .map((t) => t.name)
            .join(', ')})`,
        );
      }
    });
    this.operators.forEach((o) => {
      if (Has.operator(o, scope, that)) return;
      if (o.isPrefix) {
        throw new Error(
          `${baseErrorString} prefix operator \`${o.name} ${that.name}\``,
        );
      } else {
        throw new Error(
          `${baseErrorString} infix operator \`${o.params[0] || that.name} ${
            o.name
          } ${o.params[1] || that.name}\``,
        );
      }
    });

    if (this.delegate !== null) {
      this.delegate.constrain(that, scope, opts);
    } else if (this.isDuped) {
      if (that.contains(this)) {
        that.constrain(this, scope, opts);
      } else {
        this.delegate = that;
      }
      // const getStack = { stack: undefined };
      // Error.captureStackTrace(getStack);
      // console.log('->', this.name, 'set delegate at', getStack.stack);
      if (this.tempDelegate !== null) {
        this.delegate.constrain(this.tempDelegate, scope, opts);
        this.tempDelegate = null;
      }
    }
    // if not duped, then don't set delegate - the interface was just being
    // used to ensure that the type of the `that` matches this interface
  }

  contains(that: Type): boolean {
    if (this.eq(that)) {
      return true;
    } else if (this.delegate !== null) {
      return this.delegate.contains(that);
    } else {
      // i don't think tempDelegate needs to be checked since theoretically
      // no other constraints happen
      return false;
    }
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
    } else if (this.delegate !== null) {
      return this.delegate.eq(that);
    } else if (this.tempDelegate !== null) {
      return this.tempDelegate.eq(that);
    } else {
      return false;
    }
  }

  instance(opts?: InstanceOpts): Type {
    if (this.delegate !== null) {
      return this.delegate.instance(opts);
    } else if (this.tempDelegate !== null) {
      return this.tempDelegate.instance(opts);
    } else if (
      opts &&
      opts.interfaceOk &&
      this.__isDuped &&
      this.__isDuped.isTyVar &&
      opts.forSameDupIface
    ) {
      const already = opts.forSameDupIface
        .find(([iface, _duped]) => iface === this);
      if (already) {
        return already[1];
      } else {
        const duped = this.dup();
        opts.forSameDupIface.push([this, duped]);
        return duped;
      }
    } else if (
      opts &&
      opts.interfaceOk &&
      this.__isDuped
    ) {
      return this;
    } else {
      // console.log(this);
      throw new Error(`Could not resolve type ${this.name}`);
    }
  }

  tempConstrain(that: Type, scope: Scope, opts?: TempConstrainOpts) {
    if (this === that) {
      throw new Error('huh?');
    } else if (this.delegate !== null) {
      this.delegate.tempConstrain(that, scope);
    } else if (this.tempDelegate !== null) {
      // ensure that `this.tempDelegate` is equal to `that`
      if (!this.tempDelegate.eq(that)) {
        if (opts && opts.isTest) {
          if (!this.tempDelegate.compatibleWithConstraint(that, scope)) {
            throw new Error(
              `${this.tempDelegate.ammName} is not compatible with ${that.ammName}`,
            );
          }
        } else {
          // TODO: if more ConstrainOpts are added, make TempConstrainOpts
          // be `& ConstrainOpts` and pass the opts through
          that.constrain(this.tempDelegate, scope);
        }
      }
    } else {
      this.tempDelegate = that;
    }
  }

  resetTemp() {
    if (this.delegate !== null) {
      this.delegate.resetTemp();
      if (this.tempDelegate !== null) {
        throw new Error(`somehow, tempDelegate and delegate are both set`);
      }
    } else {
      this.tempDelegate = null;
    }
  }

  dup(dupOpts: DupOpts = {}): Type | null {
    if (this.isDuped && (!this.__isDuped.isTyVar || dupOpts.isTyVar)) {
      return null;
    }
    const dup = new Interface(
      // name isn't really used for anything in Interfaces
      // (not used for ammName, not used for equality check, etc)
      genName(this.name),
      this.ast,
      [...this.fields],
      [...this.methods],
      [...this.operators],
    );
    dup.__isDuped = dupOpts;
    return dup;
  }

  size(): number {
    if (this.delegate !== null) {
      return this.delegate.size();
    } else if (this.tempDelegate !== null) {
      return this.tempDelegate.size();
    } else {
      throw new Error(`Non-concrete interface types can't have a size`);
    }
  }

  fnselectOptions(): Type[] {
    const del = this.delegate ?? this.tempDelegate;
    if (del !== null) {
      return del.fnselectOptions();
    } else {
      return [this];
    }
  }
}

// technically, generated types are a kind of interface - we just get to build up
// the interface through type constraints instead of through explicit requirements.
// this'll make untyped fn parameters easier once they're implemented.
class Generated extends Interface {
  // don't override `get ammName` since its Error output is unique but generic
  // over both Generated and Interface types
  get isDuped(): boolean {
    return true;
  }

  constructor() {
    // Generated types are just Interface types that are more
    // lenient when handling `Has` constraints
    super(genName('Generated'), new NulLP(), [], [], []);
  }

  compatibleWithConstraint(that: Type, scope: Scope): boolean {
    if (this.eq(that)) {
      return true;
    }
    if (that instanceof Has) {
      // if it's a field, make sure there's no other field by the
      // HasField's name, otherwise there's no conflict since
      // functions and operators can have the same symbol but be
      // selected by params and return type
      if (that instanceof HasField) {
        return !this.fields.some((field) => {
          if (field.name !== that.name) return false;
          if (field.eq(that)) return false;
          field.name === that.name && !field.eq(that);
        });
      } else {
        return true;
      }
    }
    if (this.delegate !== null) {
      return this.delegate.compatibleWithConstraint(that, scope);
    } else if (this.tempDelegate !== null) {
      return this.tempDelegate.compatibleWithConstraint(that, scope);
    } else {
      return true;
    }
  }

  constrain(that: Type, scope: Scope, opts?: ConstrainOpts) {
    if (this.eq(that)) {
      return;
    }
    // if `tempDelegate` is set, something is *very* wrong because
    // all permanent constraints should already be processed...
    // if we need to allow `tempConstrain`s to get processed `constrain`s,
    // then this check should be at the end of this method and pass the
    // removed `tempDelegate` to the new permanent delegate's `tempConstrain`
    if (this.tempDelegate !== null) {
      throw new Error(
        `cannot process temporary type constraints before permanent type constraints`,
      );
    } else if (that instanceof Interface) {
      if (this.delegate ?? that.delegate === null) {
        const oFields = [...this.fields];
        const oMethods = [...this.methods];
        const oOperators = [...this.operators];
        this.fields.push(...that.fields);
        this.methods.push(...that.methods);
        this.operators.push(...that.operators);
        if (that.isDuped) {
          that.fields.push(...oFields);
          that.methods.push(...oMethods);
          that.operators.push(...oOperators);
        }
        this.delegate = that;
      } else if (this.tempDelegate ?? that.tempDelegate !== null) {
        console.log('-------------');
        console.dir(this, { depth: 4 });
        console.dir(that, { depth: 4 });
        TODO('figure out what to do here');
      } else if (this.delegate !== null) {
        if (that.delegate === null) {
          that.delegate = this;
        } else if (that.isDuped) {
          this.delegate.constrain(that.delegate, scope, opts);
        } else {
          that.fields.forEach((f) => this.delegate.constrain(f, scope, opts));
          that.methods.forEach((m) => this.delegate.constrain(m, scope, opts));
          that.operators.forEach((o) => this.delegate.constrain(o, scope, opts));
        }
      } else if (that.delegate !== null) {
        this.delegate = that.delegate;
        this.fields.forEach((f) => this.delegate.constrain(f, scope, opts));
        this.methods.forEach((m) => this.delegate.constrain(m, scope, opts));
        this.operators.forEach((o) => this.delegate.constrain(o, scope, opts));
      } else {
        console.log('-------------');
        console.dir(this, { depth: 4 });
        console.dir(that, { depth: 4 });
        TODO('i thought i covered all the branches here?');
      }
    } else if (that instanceof Has) {
      if (this.delegate !== null) {
        this.delegate.constrain(that, scope, opts);
      } else if (that instanceof HasField) {
        const already =
          this.fields.find((field) => field.name === that.name) ?? null;
        if (already !== null) {
          if (
            !already.eq(that) &&
            !already.ty.compatibleWithConstraint(that.ty, scope)
          ) {
            throw new Error(
              `generated type ${this.name} already has a field called`,
            );
          }
        } else {
          this.fields.push(that);
        }
      } else if (that instanceof HasOperator) {
        if (!this.operators.some((o) => o.eq(that))) {
          this.operators.push(that);
        }
      } else if (that instanceof HasMethod) {
        if (!this.methods.some((m) => m.eq(that))) {
          this.methods.push(that);
        }
      }
    } else if (this.delegate !== null) {
      this.delegate.constrain(that, scope, opts);
    } else {
      this.delegate = that;
      this.fields.forEach((f) => that.constrain(f, scope, opts));
      this.methods.forEach((m) => that.constrain(m, scope, opts));
      this.operators.forEach((o) => that.constrain(o, scope, opts));
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
      const mine = this.delegate ?? this.tempDelegate;
      const other = that.delegate ?? that.tempDelegate;
      if (mine === null || other === null) {
        return mine === other;
      } else {
        return mine.eq(other);
      }
    } else {
      return this.delegate !== null && this.delegate.eq(that);
    }
  }

  tempConstrain(to: Type, scope: Scope, opts?: TempConstrainOpts) {
    if (this.delegate !== null) {
      this.delegate.tempConstrain(to, scope, opts);
    } else if (this.tempDelegate !== null) {
      TODO('temp constraints to a temporary constraint???');
    } else if (to instanceof Has) {
      TODO("i'm not sure");
    } else {
      this.tempDelegate = to;
    }
  }
}

class OneOf extends Type {
  selection: Type[];
  tempSelect: Type[] | null;

  get ammName(): string {
    return this.select().ammName;
  }

  constructor(selection: Type[], tempSelect: Type[] = null) {
    super(genName('OneOf'));
    // ensure there's no duplicates. This fixes an issue with duplicates
    // in matrix selection. Since no other values are added to the list,
    // there's no need to do this any time later. Ensure that the
    // precedence is order is maintained though - if there is a duplicate,
    // keep the one that's later in the list.
    selection = selection.reduceRight(
      (sel, fn) => (sel.some((selFn) => selFn.eq(fn)) ? sel : [fn, ...sel]),
      new Array<Type>(),
    );
    this.selection = selection;
    this.tempSelect = tempSelect;
  }

  private select(): Type {
    let selected: Type;
    if (this.tempSelect !== null) {
      if (this.tempSelect.length === 0) {
        throw new Error(`expected to select from tempSelect, but it's empty`);
      }
      selected = this.tempSelect[this.tempSelect.length - 1];
    } else if (this.selection.length > 0) {
      selected = this.selection[this.selection.length - 1];
    } else {
      throw new Error(`type selection impossible - no possible types left`);
    }
    return selected;
  }

  compatibleWithConstraint(that: Type, scope: Scope): boolean {
    if (this.eq(that)) {
      return true;
    }
    return this.selection.some((ty) =>
      ty.compatibleWithConstraint(that, scope),
    );
  }

  constrain(that: Type, scope: Scope, opts?: ConstrainOpts) {
    if (this.name === 'OneOf-n4413') {
      stdout.write('~~> constraining ' + this.name + ' to:');
      console.dir(that, { depth: 4 });
    }
    if (this.eq(that)) {
      return;
    } else if (opts && opts.stopAt === this) {
      return;
    }
    const original = [...this.selection];
    const thatOpts = that.fnselectOptions();
    // ok so it's easy enough to implement a Set intersection (which is what
    // this method does). However, since there *is* an order of preference,
    // we have to somehow unify that, which is harder to determine. We
    // could do an averaging system, but that takes a *lot* of work that
    // this module is not ready to implement. Right now, this algorithm
    // just assumes that the first `OneOf` (`this` in this context) will
    // assume the ordering of the RHS (if necessary) - in this context,
    // it's `that`.
    this.selection = thatOpts.reduce((sel, thatOpt) => {
      const myApplies = this.selection.filter((ty) =>
        ty.compatibleWithConstraint(thatOpt, scope),
      );
      // the filter is to prevent duping elements. we must maintain
      // preference order, so remove any elements from `sel` that apply
      // at the current "preference level" (ie, the index of `thatOpts`
      // that is `thatOpt`)
      return [...sel.filter((ty) => !myApplies.includes(ty)), ...myApplies];
    }, new Array<Type>());
    /*
    squishy:
    [Result<int8>, Result<int16>]
    [Result<any>, Result<int8>]
    [Result<void>, Result<int8>]

    not-squishy:
    [any, Result<int8>]
    [void, bool]
    [int8, int16]
    */
    if (thatOpts.length > 1) {
      this.selection.forEach((sel) => {
        const optsThatApply = thatOpts.filter((opt) =>
          opt.compatibleWithConstraint(sel, scope),
        );
        // const oneOfApply = new OneOf(optsThatApply);
        // try {
        const flattened = Type.flatten(optsThatApply);
        if (flattened !== null) {
          sel.constrain(flattened, scope, opts);
        }
        // } catch (e) {
        //   console.log('original sel:', original);
        //   console.log('now:', this.selection);
        //   console.log('during:', sel);
        //   stdout.write('that: ');
        //   console.dir(that, { depth: 4 });
        //   console.log('opts:', thatOpts);
        //   throw e;
        // }
      });
    }
    if (!opts || opts.stopAt === undefined) {
      opts = { stopAt: this };
    }
    that.constrain(this, scope, opts)
  }

  contains(that: Type): boolean {
    if (this === that) {
      return true;
    }
    return this.selection.some((s) => s.contains(that));
  }

  eq(that: Equalable): boolean {
    return (
      (that instanceof OneOf &&
        this.selection.length === that.selection.length &&
        this.selection.every((ty, ii) => ty.eq(that.selection[ii]))) ||
      this.select().eq(that)
    );
  }

  instance(opts?: InstanceOpts): Type {
    const selected = this.select();
    if (selected === undefined) {
      throw new Error('uh whaaaaat');
    }
    const res = selected.instance(opts);
    return res;
  }

  tempConstrain(to: Type, scope: Scope, opts?: TempConstrainOpts) {
    const toOpts = to.fnselectOptions();
    this.tempSelect = toOpts.reduce((tempSel, toOpt) => {
      const myApplies = this.selection.filter((ty) =>
        ty.compatibleWithConstraint(toOpt, scope),
      );
      return [...tempSel.filter((ty) => myApplies.includes(ty)), ...myApplies];
    }, new Array<Type>());
  }

  resetTemp() {
    this.selection.forEach((ty) => ty.resetTemp());
    this.tempSelect = null;
  }

  size(): number {
    return this.select().size();
  }

  fnselectOptions(): Type[] {
    const selFrom = this.tempSelect === null ? this.selection : this.tempSelect;
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
    return selFrom.reduce(
      (options, ty) => [...options, ...ty.fnselectOptions()],
      new Array<Type>(),
    );
  }
}
