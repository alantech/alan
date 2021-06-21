import { LPNode, NamedAnd, NamedOr, NulLP } from '../lp';
import Output, { AssignKind } from './Amm';
import Fn from './Fn';
import opcodes from './opcodes';
import Operator from './Operator';
import Scope from './Scope';
import Stmt, { Dec, MetaData, VarDef } from './Stmt';
import Type from './Types';
import { isFnArray, isOpArray, TODO } from './util';

export default abstract class Expr {
  ast: LPNode
  abstract get ty(): Type;

  constructor(
    ast: LPNode,
  ) {
    this.ast = ast;
  }

  abstract inline(amm: Output, kind: AssignKind, name: string, ty: Type): void;

  private static fromBaseassignablelist(ast: LPNode, metadata: MetaData): [Stmt[], Expr] {
    let asts = ast.getAll().map(a => a.get('baseassignable'));
    let generated: Stmt[] = [];
    let expr: Expr = null;
    for (let ii = 0; ii < asts.length; ii++) {
      const skipDotIfNext = () => {
        if (ii + 1 < asts.length && asts[ii + 1].has('methodsep')) {
          ii += 1;
        }
      };
      let work = asts[ii];
      if (work.has('objectliterals')) {
        if (expr !== null) {
          throw new Error(`unexpected object literal following an expression`);
        }
        work = work.get('objectliterals');
        if (work.has('typeliteral')) {
          let [stmts, newVal] = New.fromTypeLiteral(work.get('typeliteral'), metadata);
          generated.push(...stmts);
          expr = newVal;
        } else {
          TODO('arrays');
        }
      } else if (work.has('functions')) {
        TODO('functions in functions');
      } else if (work.has('variable')) {
        const varName = work.get('variable').t;
        const next = asts[ii + 1] || new NulLP();
        if (next.has('fncall')) {
          // it's a function call
          // TODO: this is broken because operators don't pass their AST yet
          // let text = `${expr !== null ? expr.ast.t.trim() + '.' : ''}${varName}${next.get('fncall').t.trim()}`;
          let text = `${expr !== null ? expr.ast.get('variable').t + '.' : ''}${varName}${next.get('fncall').t.trim()}`;
          let and: any = {
            fnname: work.get('variable'),
            fncall: next.get('fncall'),
          };
          let accessed: Ref | null = null;
          // DO NOT access `expr` past this block until it is set.
          if (expr !== null) {
            and.fnaccess = expr.ast;
            if (!(expr instanceof Ref)) {
              let dec = Dec.gen(expr, metadata);
              generated.push(dec);
              accessed = dec.ref();
            } else {
              accessed = expr;
            }
            expr = null;
          }
          let callAst = new NamedAnd(
            text,
            and,
            (work as NamedOr).filename,
            work.line,
            work.char,
          );
          let [intermediates, call] = Call.fromCallAst(
            callAst,
            varName,
            accessed,
            metadata,
          );
          generated.push(...intermediates);
          expr = call;
          ii += 1;
          skipDotIfNext();
        } else if (expr !== null) {
          // it's a field access
          if (!(expr instanceof Ref)) {
            const dec = Dec.gen(expr, metadata);
            generated.push(dec);
            expr = dec.ref();
          }
          // ensure that the value has the field
          let fieldTy = Type.generate();
          const hasField = Type.hasField(varName, fieldTy);
          if (!expr.ty.compatibleWithConstraint(hasField, metadata.scope)) {
            throw new Error(`cannot access ${varName} on type ${expr.ty.name} because it doesn't have that field`);
          }
          expr.ty.constrain(hasField, metadata.scope);
          // TODO: better ast
          expr = new AccessField(asts[ii], expr as Ref, varName, fieldTy);
          skipDotIfNext();
        } else {
          // it's a variable reference
          const val = metadata.get(varName);
          if (!val) {
            throw new Error(`${varName} not defined`);
          }
          expr = val.ref();
          ii += 1;
        }
      } else if (work.has('constants')) {
        work = work.get('constants');
        if (expr !== null) {
          throw new Error(`unexpected constant found`);
        }
        let [int, constant] = Const.fromConstantsAst(work, metadata);
        generated.push(...int);
        expr = constant;
        skipDotIfNext();
      } else if (work.has('fncall')) {
        work = work.get('fncall');
        if (expr === null) {
          let assignableList = work.get('assignablelist');
          if (!assignableList.has('assignables') || assignableList.get('cdr').has(0)) {
            console.log(assignableList, assignableList.has('assignables'), assignableList.get('cdr').has(0));
            throw new Error(`unexpected token: found ${work.t.trim()} but it's not applied to a function`);
          }
          const [intermediates, res] = Expr.fromAssignablesAst(assignableList.get('assignables'), metadata);
          generated.push(...intermediates);
          expr = res;
          skipDotIfNext();
        } else {
          // it's probably an HOF
          let text = `${expr.ast.t.trim()}${work.t.trim()}`;
          const and = {
            fnaccess: expr.ast,
            fncall: work,
          };
          const callAst = new NamedAnd(
            text,
            and,
            (work as NamedAnd).filename,
            work.line,
            work.char,
          );
          TODO('closures/HOFs');
        }
      } else {
        console.error(asts);
        throw new Error(`unexpected token: expected variable or value, found ${work.t.trim()}`);
      }
    }
    return [generated, expr];
  }

  static fromAssignablesAst(ast: LPNode, metadata: MetaData): [Stmt[], Expr] {
    const asts = ast.getAll();
    // break it up so that we're only working on one base assignable list or operator at a time.
    let operated: Array<[Stmt[], Expr] | Operator[]> = asts.map(work => {
      work = work.get('withoperators');
      if (work.has('baseassignablelist')) {
        return Expr.fromBaseassignablelist(work.get('baseassignablelist'), metadata);
      } else if (work.has('operators')) {
        // TODO: this won't work with operators associated with interfaces.
        // Will have to iterate through all of the interfaces in-scope and collect
        // the applicable types as well
        const op = work.get('operators').t.trim();
        let operators = metadata.scope.get(op) as Operator[];
        if (operators === null) {
          throw new Error(`can't find operator ${op}`);
        } else if (!isOpArray(operators)) {
          // sanity check
          console.log(operators);
          throw new Error(`somehow ${op} isn't an operator?`);
        }
        return operators;
      } else {
        console.error(work);
        console.error(ast);
        throw new Error(`unexpected assignable ast: ${work}`);
      }
    });
    if (operated.length === 0) {
      throw new Error(`no expressions generated for ast: ${ast}`);
    } else if (operated.length === 1) {
      if (isOpArray(operated)) {
        throw new Error(`variables can't be assigned to operators`);
      }
      return operated[0] as [Stmt[], Expr];
    }
    // now we have to resolve operators - start by filtering out operators if they
    // are in a position that must be prefix or infix
    // since there are no suffix operators, this is relatively easy - operators
    // immediately following an expression must be infix, while all others must be
    // a prefix
    // TODO: make sure errors match lntoamm
    let stmts: Stmt[] = [];
    let infixPosition = false;
    let operation = operated.map(op => {
      if (!isOpArray(op)) {
        if (infixPosition) {
          throw new Error(`invalid expression: expected operator, found ${op[1].ast.t.trim()}`);
        }
        infixPosition = true;
        stmts.push(...op[0]);
        return op[1];
      } else if (infixPosition) {
        infixPosition = false;
        return op.filter(op => !op.isPrefix);
      } else {
        return op.filter(op => op.isPrefix);
      }
    });

    // Now we build the precedence table for this application
    const precedences = operation.map(opOrRef => {
      if (opOrRef instanceof Expr) {
        return opOrRef;
      } else {
        // return opOrRef.reduce((prec, op) => prec.add(op.precedence), new Set<number>());
        // TODO: do i need this?
        return opOrRef.reduce((prec, op) => prec.set(op.precedence, [...(prec.get(op.precedence) || []), op]), new Map<number, Operator[]>());
      }
    });

    // now to try to solve operators.
    // TODO: this might not work if there are multiple operator precedences for
    // the same symbol. If that's the case, then we'll have to create an Expr
    // that acts as a permutation over the different possible operator expansions
    // (it can be done after eliminating operators that aren't compatible with
    // the provided types)
    while (true) {
      // find the highest-precedence operations
      let prec = -1;
      let idxs: number[] = precedences.reduce((idxs, opOrRef, ii) => {
        if (opOrRef instanceof Expr) return idxs;
        let precs = Array.from(opOrRef.keys());
        if (precs.length > 1) {
          // TODO: this is just a stop for future cases. might have
          // to revisit this whole loop, but just to remind myself
          TODO('figure out multiple precedences?');
        }
        let maxPrec = precs.sort().pop();
        if (maxPrec > prec) {
          prec = maxPrec;
          return [ii];
        } else if (maxPrec === prec) {
          return [...idxs, ii];
        } else {
          return idxs;
        }
      }, []);
      if (prec === -1 || idxs.length === 0) {
        break;
      }

      // all of the selected operators should be the same infix/prefix mode
      // if the result is null, that means they're not - idk if that's
      // ever a case so just TODO it
      const prefixModeOf = (vals: Operator[]) => vals.reduce(
        (mode, op) => {
          if (mode === null) return mode;
          return mode === op.isPrefix ? mode : null;
        },
        vals[0].isPrefix
      );

      idxs.forEach(idx => {
        const val = precedences[idx];
        // heat-death-of-the-universe check
        if (val instanceof Expr) {
          throw new Error(`uh, how?`)
        }
        // ensure that none of the operators disagree on fixity
        const mode = prefixModeOf(val.get(prec));
        if (mode === null) {
          TODO('operator is both prefix and infix - how to determine?');
        }
      });
      // first, prefix operators - we need to mutate idxs so no `.forEach`
      // do prefix operators first to ensure that there's no operator
      // ambiguity. If there's a prefix before an infix operator (not an
      // expression), this still gets caught below.
      // TODO: this can be iterated through in reverse, but it's a quick
      // refactor during PR review so do that later
      for (let jj = 0; jj < idxs.length; jj++) {
        let idx = idxs[jj];
        let item = precedences[idx] as Map<number, Operator[]>;
        let operators = [...item.get(prec)];
        const isPrefix = prefixModeOf(operators);
        if (!isPrefix) continue;
        // prefix operators are right-associated, so we have to go ahead
        // in the indices to ensure that the right-most is handled first
        let applyIdx = precedences.slice(idx).findIndex(val => val instanceof Expr);
        // make sure all of the operators between are prefix operators
        // with the same precedence
        precedences.slice(idx + 1, applyIdx).forEach((opOrExpr, idx) => {
          if (opOrExpr instanceof Expr) {
            throw new Error(`this error should not be thrown`);
          } else if (!idxs.includes(idx)) {
            throw new Error(`unable to resolve operators - operator precedence ambiguity`);
          } else if (prefixModeOf(opOrExpr.get(prec)) !== true) {
            throw new Error(`unable to resolve operators - operator ambiguity`);
          }
        });
        // slice copies the array, so this is ok :)
        for (let op of precedences.slice(idx, applyIdx).reverse()) {
          if (op instanceof Expr) {
            throw new Error(`unexpected expression during computation? this error should never happen`);
          }
          if (!(precedences[applyIdx] instanceof Ref)) {
            const dec = Dec.gen(precedences[applyIdx] as Expr, metadata);
            stmts.push(dec);
            precedences[applyIdx] = dec.ref();
          }
          const applyTo = precedences[applyIdx] as Ref;
          let fns = operators.reduce((fns, op) => [...fns, ...op.select(metadata.scope, applyTo.ty)], new Array<Fn>());
          precedences[applyIdx] = new Call(new NulLP(), fns, null, [applyTo], metadata.scope);
          let rm = precedences.splice(idx, applyIdx);
          // update indices
          idxs = idxs.map((idx, kk) => kk > jj ? idx - rm.length : kk);
          // remove the operators we used so they aren't looked at later
          idxs.splice(jj, rm.length);
          jj -= 1;
        }
      }
      // now suffix operators
      for (let jj = 0; jj < idxs.length; jj++) {
        let idx = idxs[jj];
        let item = precedences[idx];
        // heat-death-of-the-universe check
        if (item instanceof Expr) {
          console.log('-> prec', prec);
          console.log('-> idxs', idxs);
          console.log('-> idx', idx);
          throw new Error(`uh, how?`);
        }
        // prefer the last-defined operators, so we must pop()
        let ops = [...item.get(prec)];
        if (prefixModeOf(ops) === true) {
          throw new Error(`prefix operators at precedence level ${prec} should've already been handled`);
        }
        // since infix operators are left-associated, and we iterate
        // left->right anyways, this impl is easy
        let fns = [];
        let left = precedences[idx - 1] as Ref;
        let right = precedences[idx + 1] as Ref;
        if (!left || !right) {
          throw new Error(`operator in invalid position`);
        } else if (!(left instanceof Expr) || !(right instanceof Expr)) {
          throw new Error(`operator ambiguity`);
        }
        if (!(left instanceof Ref)) {
          const dec = Dec.gen(left, metadata);
          stmts.push(dec);
          left = dec.ref();
        }
        if (!(right instanceof Ref)) {
          const dec = Dec.gen(right, metadata);
          stmts.push(dec);
          right = dec.ref();
        }
        while (ops.length > 0) {
          const op = ops.pop();
          const selected = op.select(metadata.scope, left.ty, right.ty);
          fns.push(...selected);
        }
        const call = new Call(
          new NulLP(),
          fns,
          null,
          [left, right],
          metadata.scope,
        );
        precedences[idx - 1] = call;
        precedences.splice(idx, 2);
        idxs = idxs.map((idx, kk) => kk > jj ? idx - 2 : kk);
      }
    }

    if (precedences.length !== 1) {
      throw new Error(`couldn't resolve operators`);
    }
    return [stmts, precedences.pop() as Ref];
  }
}

class AccessField extends Expr {
  struct: Ref
  fieldName: string
  fieldTy: Type

  get ty(): Type {
    return this.fieldTy;
  }

  constructor(
    ast: LPNode,
    struct: Ref,
    fieldName: string,
    fieldTy: Type,
  ) {
    super(ast);
    this.struct = struct;
    this.fieldName = fieldName;
    this.fieldTy = fieldTy;
  }

  inline(amm: Output, kind: AssignKind, name: string, ty: Type): void {
    const fieldIndices = this.struct.ty.fieldIndices();
    const index = fieldIndices[this.fieldName];
    const indexVal = amm.global('const', opcodes().get('int64'), `${index}`)
    amm.assign(kind, name, ty, 'register', [this.struct.ammName, indexVal]);
  }
}

class Call extends Expr {
  fns: Fn[]
  maybeClosure: VarDef | null
  args: Ref[]
  retTy: Type
  // FIXME: once the matrix as below is implemented, get rid of this field
  // and just pass it into the selection phase
  scope: Scope

  get ty(): Type {
    return this.retTy;
  }

  constructor(
    ast: LPNode,
    fns: Fn[],
    maybeClosure: VarDef | null,
    args: Ref[],
    scope: Scope,
  ) {
    // console.log('~~~ generating call ', ast, fns, maybeClosure, args, scope);
    super(ast);
    if (fns.length === 0 && maybeClosure === null) {
      throw new Error(`no function possibilities provided for ${ast}`);
    }
    this.fns = fns;
    this.maybeClosure = maybeClosure;
    this.args = args;
    this.retTy = Type.oneOf(Array.from(new Set(fns.map(fn => fn.retTy))));
    this.scope = scope;
  }

  static fromCallAst(
    ast: LPNode,
    fnName: string,
    accessed: Ref | null,
    metadata: MetaData,
  ): [Stmt[], Expr] {
    let stmts = [];
    let argAst = ast.get('fncall').get('assignablelist');
    let argAsts: LPNode[] = [];
    if (argAst.has('assignables')) {
      argAsts.push(argAst.get('assignables'));
      if (argAst.has('cdr')) {
        argAsts.push(...argAst.get('cdr').getAll().map(a => a.get('assignables')));
      }
    }
    let args: Ref[] = [];
    if (accessed !== null) {
      args.push(accessed);
    }
    args.push(...argAsts.map(a => {
      let [generated, argExpr] = Expr.fromAssignablesAst(a, metadata);
      stmts.push(...generated);
      let arg: Ref;
      if (argExpr instanceof Ref) {
        arg = argExpr;
      } else {
        const dec = Dec.gen(argExpr, metadata);
        stmts.push(dec);
        arg = dec.ref();
      }
      return arg;
    }));
    let fns = metadata.scope.deepGet(fnName);
    let closure = metadata.get(fnName);
    if ((fns === null || !isFnArray(fns)) && closure === null) {
      throw new Error(`no functions found for ${fnName}`);
    }
    if (fns === null || !isFnArray(fns)) {
      fns = [] as Fn[];
    }
    // first reduction
    let argTys = args.map(arg => arg.ty);
    fns = Fn.select(fns, argTys, metadata.scope);
    // now, constrain all of the args to their possible types
    // makes it so that the type of the parameters in each position are in their own list
    // ie, given `do(int8, int16)` and `do(int8, int8)`, will result in this 2D array:
    // [ [int8, int8],
    //   [int16, int8] ]
    // for some reason TS thinks that `fns` is `Boxish` but *only* in the lambda here,
    // which is why I have to specify `fns: Fn[]`...
    argTys.forEach((ty, ii) => {
      let paramTys = (fns as Fn[]).map(fn => fn.params[ii].ty);
      // console.log('constraining', ty, 'to', paramTys);
      ty.constrain(Type.oneOf(paramTys), metadata.scope);
      // console.log('constrained:', ty);
    });
    if (closure !== null) {
      TODO('closures');
    }
    return [stmts, new Call(ast, fns, closure, args, metadata.scope)];
  }

  /*
  FIXME:
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


  This should also happen in the unimplemented "solidification" phase.
  */
  inline(amm: Output, kind: AssignKind, name: string, ty: Type) {
    const argTys = this.args.map(arg => arg.ty.instance());
    const selected = Fn.select(this.fns, argTys, this.scope) || [];
    // console.log('!!!!!!!!!!', this.ast.t.trim(), selected);
    if (selected.length === 0) {
      // TODO: to get better error reporting, we need to pass an ast when using
      // operators. i'm not worried about error reporting yet, though :)
      console.log('~~~ ERROR')
      console.log('selection pool:', this.fns);
      console.log('args:', this.args);
      console.log('kind:', kind);
      console.log('expected output type:', ty);
      throw new Error(`no function selected`);
    }
    const fn = selected.pop();
    fn.inline(amm, this.args, kind, name, ty);
  }
}

class Const extends Expr {
  val: string
  private detectedTy: Type

  get ty(): Type {
    return this.detectedTy;
  }

  constructor(
    ast: LPNode,
    val: string,
    detectedTy: Type,
  ) {
    super(ast);
    this.val = val;
    this.detectedTy = detectedTy;
  }

  static fromConstantsAst(
    ast: LPNode,
    _metadata: MetaData,
  ): [Stmt[], Expr] {
    let val = ast.t.trim();
    let detectedTy = null;
    if (ast.has('bool')) {
      detectedTy = opcodes().get('bool');
    } else if (ast.has('str')) {
      detectedTy = opcodes().get('string');
      // sanitize single-quoted strings
      // don't need to for double-quoted strings, since the string output
      // is double-quoted
      if (val.startsWith("'")) {
        let sanitized = val.substring(1, val.length - 1).replace(/'/g, "\\'");
        val = `"${sanitized.replace(/"/g, '\\"')}"`;
      }
    } else if (ast.has('num')) {
      if (val.indexOf('.') !== -1) {
        detectedTy = Type.oneOf([
          'float32',
          'float64',
        ].map(t => opcodes().get(t)));
      } else {
        detectedTy = Type.oneOf([
          'float32',
          'float64',
          'int8',
          'int16',
          'int32',
          'int64',
        ].map(t => opcodes().get(t)));
      }
    } else {
      throw new Error(`unrecognized constants node: ${ast}`);
    }
    return [[], new Const(ast, val, detectedTy)];
  }

  inline(amm: Output, kind: AssignKind, name: string, ty: Type) {
    const suffixes = {
      int8: 'i8',
      int16: 'i16',
      int32: 'i32',
      int64: 'i64',
      float32: 'f32',
      float64: 'f64',
      string: 'str',
      bool: 'bool',
    };

    const globalName = amm.global('const', ty, this.val);
    let copyOp = 'copy';
    if (suffixes[ty.ammName]) {
      copyOp += suffixes[ty.ammName];
    } else {
      // sanity check
      throw new Error(`unhandled const type ${ty.ammName}`);
    }
    amm.assign(kind, name, ty, copyOp, [globalName]);
  }
}

class New extends Expr {
  valTy: Type
  fields: {[name: string]: Ref}

  get ty(): Type {
    return this.valTy;
  }

  /**
   * NOTE: does NOT check to make sure that the fields
   * are valid.
   */
  constructor(
    ast: LPNode,
    valTy: Type,
    fields: {[name: string]: Ref},
  ) {
    super(ast);
    this.valTy = valTy;
    this.fields = fields;
  }

  static fromTypeLiteral(
    ast: LPNode,
    metadata: MetaData,
  ): [Stmt[], New] {
    let stmts: Stmt[] = [];

    // get the constructed type
    let typename = ast.get('literaldec').get('fulltypename');
    let valTy = Type.getFromTypename(typename, metadata.scope);

    let fieldsAst = ast.get('typebase').get('typeassignlist');
    let fieldAsts: LPNode[] = [
      fieldsAst,
      ...fieldsAst.get('cdr').getAll(),
    ];
    let fields: {[name: string]: Ref} = {};
    // type that we're generating to make sure that the constructed object
    // has the appropriate fields.
    let fieldCheck = Type.generate();

    for (let fieldAst of fieldAsts) {
      const fieldName = fieldAst.get('variable').t.trim();
      // assign the value of the field to a variable
      let [newStmts, fieldVal] = Expr.fromAssignablesAst(fieldAst.get('assignables'), metadata);
      stmts.push(...newStmts);
      if (!(fieldVal instanceof Ref)) {
        const fieldDef = Dec.gen(fieldVal, metadata);
        stmts.push(fieldDef);
        fieldVal = fieldDef.ref();
      }
      // assign the field to our pseudo-object
      fields[fieldName] = fieldVal as Ref;
      // add the field to our generated type
      fieldCheck.constrain(Type.hasField(fieldName, fieldVal.ty), metadata.scope);
    }

    // ensure that the type we just constructed matches the type intended
    // to be constructed. if our generated type isn't compatible with the
    // intended type, then that means we don't have all of its fields. If
    // the intended type isn't compatible with our generated type, that
    // means we have some unexpected fields
    // TODO: MUCH better error handling. Ideally without exposing the
    // internal details of the `Type`.
    if (!fieldCheck.compatibleWithConstraint(valTy, metadata.scope)) {
      throw new Error(`Constructed value doesn't have all of the fields in type ${valTy.name}`);
    } else if (!valTy.compatibleWithConstraint(fieldCheck, metadata.scope)) {
      throw new Error(`Constructed value has fields that don't exist in ${valTy.name}`);
    }

    // *new* new
    return [stmts, new New(ast, valTy, fields)];
  }

  inline(amm: Output, kind: AssignKind, name: string, ty: Type): void {
    const int64 = opcodes().get('int64');
    const size = amm.global('const', int64, this.ty.size().toString());
    amm.assign(kind, name, ty, 'newarr', [size]);
    for (let field in this.fields) {
      const fieldTy = this.fields[field].ty.instance();
      const sizeHint = amm.global('const', int64, `${fieldTy.size()}`);
      const pushCall = fieldTy.isFixed() ? 'pushf' : 'pushv';
      amm.call(pushCall, [name, this.fields[field].ammName, sizeHint]);
    }
  }
}

export class Ref extends Expr {
  def: VarDef

  get ammName(): string {
    return this.def.ammName;
  }

  get ty(): Type {
    return this.def.ty;
  }

  constructor(
    def: VarDef,
  ) {
    super(def.ast);
    this.def = def;
  }

  inline(_amm: Output, _kind: AssignKind, _name: string, _ty: Type) {
    throw new Error(`did not expect to inline a variable reference`);
  }
}
