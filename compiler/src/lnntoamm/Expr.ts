import { LPNode, NamedAnd, NamedOr, NulLP } from '../lp';
import Output, { AssignKind } from './Amm';
import Fn from './Fn';
import opcodes from './opcodes';
import Operator from './Operator';
import Scope from './Scope';
import Stmt, { Dec, MetaData, VarDef } from './Stmt';
import Type, { FunctionType } from './Types';
import { isFnArray, isOpArray, TODO } from './util';

export default abstract class Expr {
  ast: LPNode;
  abstract get ty(): Type;

  constructor(ast: LPNode) {
    this.ast = ast;
  }

  abstract inline(amm: Output, kind: AssignKind, name: string, ty: Type): void;

  private static fromBaseassignablelist(
    ast: LPNode,
    metadata: MetaData,
  ): [Stmt[], Expr] {
    const asts = ast.getAll().map((a) => a.get('baseassignable'));
    const generated: Stmt[] = [];
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
          const [stmts, newVal] = New.fromTypeLiteral(
            work.get('typeliteral'),
            metadata,
          );
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
          const text = `${
            expr !== null ? expr.ast.get('variable').t + '.' : ''
          }${varName}${next.get('fncall').t.trim()}`;
          const and: any = {
            fnname: work.get('variable'),
            fncall: next.get('fncall'),
          };
          let accessed: Ref | null = null;
          // DO NOT access `expr` past this block until it is set.
          if (expr !== null) {
            and.fnaccess = expr.ast;
            if (!(expr instanceof Ref)) {
              const dec = Dec.gen(expr, metadata);
              generated.push(dec);
              accessed = dec.ref();
            } else {
              accessed = expr;
            }
            expr = null;
          }
          const callAst = new NamedAnd(
            text,
            and,
            (work as NamedOr).filename,
            work.line,
            work.char,
          );
          const [intermediates, call] = Call.fromCallAst(
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
          const fieldTy = Type.generate();
          const hasField = Type.hasField(varName, fieldTy);
          if (!expr.ty.compatibleWithConstraint(hasField, metadata.scope)) {
            throw new Error(
              `cannot access ${varName} on type ${expr.ty.name} because it doesn't have that field`,
            );
          }
          expr.ty.constrain(hasField, metadata.scope);
          // TODO: better ast - currently only gives the ast for the field name
          // (instead of giving the way the struct is accessed as well)
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
        const [int, constant] = Const.fromConstantsAst(work, metadata);
        generated.push(...int);
        expr = constant;
        skipDotIfNext();
      } else if (work.has('fncall')) {
        work = work.get('fncall');
        if (expr === null) {
          const assignableList = work.get('assignablelist');
          if (
            !assignableList.has('assignables') ||
            assignableList.get('cdr').has(0)
          ) {
            console.log(
              assignableList,
              assignableList.has('assignables'),
              assignableList.get('cdr').has(0),
            );
            throw new Error(
              `unexpected token: found ${work.t.trim()} but it's not applied to a function`,
            );
          }
          const [intermediates, res] = Expr.fromAssignablesAst(
            assignableList.get('assignables'),
            metadata,
          );
          generated.push(...intermediates);
          expr = res;
          skipDotIfNext();
        } else {
          // it's probably an HOF
          const text = `${expr.ast.t.trim()}${work.t.trim()}`;
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
        throw new Error(
          `unexpected token: expected variable or value, found ${work.t.trim()}`,
        );
      }
    }
    return [generated, expr];
  }

  static fromAssignablesAst(ast: LPNode, metadata: MetaData): [Stmt[], Expr] {
    const asts = ast.getAll();
    // break it up so that we're only working on one base assignable list or operator at a time.
    const operated: Array<[Stmt[], Expr] | Operator[]> = asts.map((work) => {
      work = work.get('withoperators');
      if (work.has('baseassignablelist')) {
        return Expr.fromBaseassignablelist(
          work.get('baseassignablelist'),
          metadata,
        );
      } else if (work.has('operators')) {
        // TODO: this won't work with operators associated with interfaces.
        // Will have to iterate through all of the interfaces in-scope and collect
        // the applicable types as well
        const op = work.get('operators').t.trim();
        const operators = metadata.scope.get(op) as Operator[];
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
    const stmts: Stmt[] = [];
    let infixPosition = false;
    const operation = operated.map((op) => {
      if (!isOpArray(op)) {
        if (infixPosition) {
          throw new Error(
            `invalid expression: expected operator, found ${op[1].ast.t.trim()}`,
          );
        }
        infixPosition = true;
        stmts.push(...op[0]);
        return op[1];
      } else if (infixPosition) {
        infixPosition = false;
        return op.filter((op) => !op.isPrefix);
      } else {
        return op.filter((op) => op.isPrefix);
      }
    });

    // Now we build the precedence table for this application
    const precedences = operation.map((opOrRef) => {
      if (opOrRef instanceof Expr) {
        return opOrRef;
      } else {
        // return opOrRef.reduce((prec, op) => prec.add(op.precedence), new Set<number>());
        // TODO: do i need this?
        return opOrRef.reduce(
          (prec, op) =>
            prec.set(op.precedence, [...(prec.get(op.precedence) || []), op]),
          new Map<number, Operator[]>(),
        );
      }
    });

    // now to try to solve operators.
    // TODO: this might not work if there are multiple operator precedences for
    // the same symbol. If that's the case, then we'll have to create an Expr
    // that acts as a permutation over the different possible operator expansions
    // (it can be done after eliminating operators that aren't compatible with
    // the provided types)
    // eslint-disable-next-line no-constant-condition
    while (true) {
      // find the highest-precedence operations
      let prec = -1;
      let idxs: number[] = precedences.reduce((idxs, opOrRef, ii) => {
        if (opOrRef instanceof Expr) return idxs;
        const precs = Array.from(opOrRef.keys());
        if (precs.length > 1) {
          // TODO: this is just a stop for future cases. might have
          // to revisit this whole loop, but just to remind myself
          TODO('figure out multiple precedences?');
        }
        const maxPrec = precs.sort().pop();
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
      const prefixModeOf = (vals: Operator[]) =>
        vals.reduce((mode, op) => {
          if (mode === null) return mode;
          return mode === op.isPrefix ? mode : null;
        }, vals[0].isPrefix);

      idxs.forEach((idx) => {
        const val = precedences[idx];
        // heat-death-of-the-universe check
        if (val instanceof Expr) {
          throw new Error(`uh, how?`);
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
        const idx = idxs[jj];
        const item = precedences[idx] as Map<number, Operator[]>;
        const operators = [...item.get(prec)];
        const isPrefix = prefixModeOf(operators);
        if (!isPrefix) continue;
        // prefix operators are right-associated, so we have to go ahead
        // in the indices to ensure that the right-most is handled first
        const applyIdx = precedences
          .slice(idx)
          .findIndex((val) => val instanceof Expr);
        // make sure all of the operators between are prefix operators
        // with the same precedence
        precedences.slice(idx + 1, applyIdx).forEach((opOrExpr, idx) => {
          if (opOrExpr instanceof Expr) {
            throw new Error(`this error should not be thrown`);
          } else if (!idxs.includes(idx)) {
            throw new Error(
              `unable to resolve operators - operator precedence ambiguity`,
            );
          } else if (prefixModeOf(opOrExpr.get(prec)) !== true) {
            throw new Error(`unable to resolve operators - operator ambiguity`);
          }
        });
        // slice copies the array, so this is ok :)
        for (const op of precedences.slice(idx, applyIdx).reverse()) {
          if (op instanceof Expr) {
            throw new Error(
              `unexpected expression during computation? this error should never happen`,
            );
          }
          if (!(precedences[applyIdx] instanceof Ref)) {
            const dec = Dec.gen(precedences[applyIdx] as Expr, metadata);
            stmts.push(dec);
            precedences[applyIdx] = dec.ref();
          }
          const applyTo = precedences[applyIdx] as Ref;
          const retTy = Type.generate();
          const [fns, paramTys, retTys] = operators.reduce(
            ([fns, paramTys, retTys], op) => {
              const [selFns, selPTys, selRTys] = op.select(
                metadata.scope,
                retTy,
                applyTo.ty,
              );
              fns = [...fns, ...selFns];
              // assume that `selPTys[i].length === 1`
              paramTys = [...paramTys, ...selPTys.map((pTys) => pTys[0])];
              retTys = [...retTys, ...selRTys];
              return [fns, paramTys, retTys];
            },
            [new Array<Fn>(), new Array<Type>(), new Array<Type>()],
          );
          const argConstraint = Type.oneOf(paramTys);
          applyTo.ty.constrain(argConstraint, metadata.scope);
          retTy.constrain(Type.oneOf(retTys), metadata.scope);
          precedences[applyIdx] = new Call(
            new NulLP(),
            fns,
            null,
            [applyTo],
            metadata.scope,
            retTy,
          );
          const rm = precedences.splice(idx, applyIdx);
          // update indices
          idxs = idxs.map((idx, kk) => (kk > jj ? idx - rm.length : kk));
          // remove the operators we used so they aren't looked at later
          idxs.splice(jj, rm.length);
          jj -= 1;
        }
      }
      // now suffix operators
      for (let jj = 0; jj < idxs.length; jj++) {
        const idx = idxs[jj];
        const item = precedences[idx];
        // heat-death-of-the-universe check
        if (item instanceof Expr) {
          console.log('-> prec', prec);
          console.log('-> idxs', idxs);
          console.log('-> idx', idx);
          throw new Error(`uh, how?`);
        }
        // prefer the last-defined operators, so we must pop()
        const ops = [...item.get(prec)];
        if (prefixModeOf(ops) === true) {
          throw new Error(
            `prefix operators at precedence level ${prec} should've already been handled`,
          );
        }
        // since infix operators are left-associated, and we iterate
        // left->right anyways, this impl is easy
        const fns = [];
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
        const argTys: [Type[], Type[]] = [[], []];
        const retTys: Type[] = [];
        while (ops.length > 0) {
          const op = ops.pop();
          const retTy = Type.generate();
          const selected = op.select(metadata.scope, retTy, left.ty, right.ty);
          fns.push(...selected[0]);
          // assume `selected[1].length === 2`
          selected[1].forEach((pTys) => {
            argTys[0].push(pTys[0]);
            argTys[1].push(pTys[1]);
          });
          retTy.constrain(Type.oneOf(selected[2]), metadata.scope);
          retTys.push(retTy);
        }
        const retTy = Type.oneOf(retTys);
        const call = new Call(
          new NulLP(),
          fns,
          null,
          [left, right],
          metadata.scope,
          retTy,
        );
        precedences[idx - 1] = call;
        precedences.splice(idx, 2);
        idxs = idxs.map((idx, kk) => (kk > jj ? idx - 2 : kk));
      }
    }

    if (precedences.length !== 1) {
      throw new Error(`couldn't resolve operators`);
    }
    return [stmts, precedences.pop() as Ref];
  }

  /**
   * @returns true if more cleanup might be required
   */
  cleanup(expectResTy: Type): boolean {
    // most implementing Exprs don't have anything they need to do.
    // I just didn't want to expose any of the Expr classes except
    // for Ref to prevent split handling of the classes.
    return false;
  }
}

class AccessField extends Expr {
  struct: Ref;
  fieldName: string;
  fieldTy: Type;

  get ty(): Type {
    return this.fieldTy;
  }

  constructor(ast: LPNode, struct: Ref, fieldName: string, fieldTy: Type) {
    super(ast);
    this.struct = struct;
    this.fieldName = fieldName;
    this.fieldTy = fieldTy;
  }

  inline(amm: Output, kind: AssignKind, name: string, ty: Type): void {
    const fieldIndices = this.struct.ty.fieldIndices();
    const index = fieldIndices[this.fieldName];
    const indexVal = amm.global('const', opcodes().get('int64'), `${index}`);
    amm.assign(kind, name, ty, 'register', [this.struct.ammName, indexVal]);
  }
}

class Call extends Expr {
  private dbg = () => this.ast.t.includes('ok');

  fns: Fn[];
  maybeClosure: VarDef | null;
  args: Ref[];
  retTy: Type;
  // FIXME: once the matrix as below is implemented, get rid of this field
  // and just pass it into the selection phase
  scope: Scope;

  get ty(): Type {
    return this.retTy;
  }

  constructor(
    ast: LPNode,
    fns: Fn[],
    maybeClosure: VarDef | null,
    args: Ref[],
    scope: Scope,
    retTy: Type,
  ) {
    super(ast);
    if (fns.length === 0 && maybeClosure === null) {
      throw new Error(`no function possibilities provided for ${ast}`);
    }
    this.fns = fns;
    this.maybeClosure = maybeClosure;
    this.args = args;
    this.retTy = retTy;
    this.scope = scope;
  }

  static fromCallAst(
    ast: LPNode,
    fnName: string,
    accessed: Ref | null,
    metadata: MetaData,
  ): [Stmt[], Expr] {
    const stmts = [];
    const argAst = ast.get('fncall').get('assignablelist');
    const argAsts: LPNode[] = [];
    if (argAst.has('assignables')) {
      argAsts.push(argAst.get('assignables'));
      if (argAst.has('cdr')) {
        argAsts.push(
          ...argAst
            .get('cdr')
            .getAll()
            .map((a) => a.get('assignables')),
        );
      }
    }
    const args: Ref[] = [];
    if (accessed !== null) {
      args.push(accessed);
    }
    args.push(
      ...argAsts.map((a) => {
        const [generated, argExpr] = Expr.fromAssignablesAst(a, metadata);
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
      }),
    );
    let fns = metadata.scope.deepGet(fnName);
    const closure = metadata.get(fnName);
    if ((fns === null || !isFnArray(fns)) && closure === null) {
      throw new Error(`no functions found for ${fnName}`);
    }
    if (fns === null || !isFnArray(fns)) {
      fns = [] as Fn[];
    }
    // first reduction
    const argTys = args.map((arg) => arg.ty);
    const retTy = Type.generate();
    const [selFns, selPTys, selRetTys] = FunctionType.matrixSelect(
      fns,
      argTys,
      retTy,
      metadata.scope,
    );
    fns = selFns;
    // console.log('initial select for', ast.t.trim(), '->', fns);
    // stdout.write('-> args ');
    // console.dir(args, { depth: 4 });
    // now, constrain all of the args to their possible types
    argTys.forEach((ty, ii) =>
      ty.constrain(Type.oneOf(selPTys[ii]), metadata.scope),
    );
    retTy.constrain(Type.oneOf(selRetTys), metadata.scope);
    if (closure !== null) {
      TODO('closures');
    }
    return [stmts, new Call(ast, fns, closure, args, metadata.scope, retTy)];
  }

  private fnSelect(): [Fn[], Type[][], Type[]] {
    const ret = FunctionType.matrixSelect(
      this.fns,
      this.args.map((a) => a.ty),
      this.retTy,
      this.scope,
    );
    // console.log('fnSelect for', this.ast.t.trim(), '->', ret[0]);
    return ret;
  }

  cleanup() {
    const [fns, pTys, retTys] = this.fnSelect();
    const isChanged = this.fns.length !== fns.length;
    this.fns = fns;
    this.args.forEach((arg, ii) =>
      arg.ty.constrain(Type.oneOf(pTys[ii]), this.scope),
    );
    this.retTy.constrain(Type.oneOf(retTys), this.scope);
    return isChanged;
  }

  inline(amm: Output, kind: AssignKind, name: string, ty: Type) {
    // ignore selTys because if there's a mismatch between `ty`
    // and the return type of the selected function, there will
    // be an error when we inline
    const [selFns, _selTys] = this.fnSelect();
    if (selFns.length === 0) {
      // TODO: to get better error reporting, we need to pass an ast when using
      // operators. i'm not worried about error reporting yet, though :)
      console.log('~~~ ERROR');
      console.log('selection pool:', this.fns);
      console.log('args:', this.args);
      console.log('kind:', kind);
      console.log('expected output type:', ty);
      throw new Error(`no function selected`);
    }
    // FunctionType.matrixSelect implements the matrix so that the most
    // reasonable choice is last in the fn array. "Reasonableness" is computed
    // with 2 factors: 1st is alignment with given OneOf types. If `add(1, 0)`
    // is called, the literal types should prefer `int64` to `int32` etc. The
    // other factor is order of declaration - Alan should always prefer using
    // functions that are defined last.
    const fn = selFns.pop();
    fn.inline(amm, this.args, kind, name, ty, this.scope);
  }
}

class Const extends Expr {
  val: string;
  private detectedTy: Type;

  get ty(): Type {
    return this.detectedTy;
  }

  constructor(ast: LPNode, val: string, detectedTy: Type) {
    super(ast);
    this.val = val;
    this.detectedTy = detectedTy;
  }

  static fromConstantsAst(ast: LPNode, _metadata: MetaData): [Stmt[], Expr] {
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
        const sanitized = val.substring(1, val.length - 1).replace(/'/g, "\\'");
        val = `"${sanitized.replace(/"/g, '\\"')}"`;
      }
    } else if (ast.has('num')) {
      if (val.indexOf('.') !== -1) {
        detectedTy = Type.oneOf(
          ['float32', 'float64'].map((t) => opcodes().get(t)),
        );
      } else {
        detectedTy = Type.oneOf(
          ['float32', 'float64', 'int8', 'int16', 'int32', 'int64'].map((t) =>
            opcodes().get(t),
          ),
        );
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
  valTy: Type;
  fields: { [name: string]: Ref };

  get ty(): Type {
    return this.valTy;
  }

  /**
   * NOTE: does NOT check to make sure that the fields
   * are valid. Ensure that the caller has already done
   * validated the fields (fromTypeLiteral does this
   * already).
   */
  constructor(ast: LPNode, valTy: Type, fields: { [name: string]: Ref }) {
    super(ast);
    this.valTy = valTy;
    this.fields = fields;
  }

  static fromTypeLiteral(ast: LPNode, metadata: MetaData): [Stmt[], New] {
    const stmts: Stmt[] = [];

    // get the constructed type
    const typename = ast.get('literaldec').get('fulltypename');
    const valTy = Type.getFromTypename(typename, metadata.scope);

    const fieldsAst = ast.get('typebase').get('typeassignlist');
    const fieldAsts: LPNode[] = [fieldsAst, ...fieldsAst.get('cdr').getAll()];
    const fields: { [name: string]: Ref } = {};
    // type that we're generating to make sure that the constructed object
    // has the appropriate fields.
    const fieldCheck = Type.generate();

    for (const fieldAst of fieldAsts) {
      const fieldName = fieldAst.get('variable').t.trim();
      // assign the value of the field to a variable
      // can't use const here but eslint doesn't like the newStmts isn't const
      // eslint-disable-next-line prefer-const
      let [newStmts, fieldVal] = Expr.fromAssignablesAst(
        fieldAst.get('assignables'),
        metadata,
      );
      stmts.push(...newStmts);
      if (!(fieldVal instanceof Ref)) {
        const fieldDef = Dec.gen(fieldVal, metadata);
        stmts.push(fieldDef);
        fieldVal = fieldDef.ref();
      }
      // assign the field to our pseudo-object
      fields[fieldName] = fieldVal as Ref;
      // add the field to our generated type
      fieldCheck.constrain(
        Type.hasField(fieldName, fieldVal.ty),
        metadata.scope,
      );
    }

    // ensure that the type we just constructed matches the type intended
    // to be constructed. if our generated type isn't compatible with the
    // intended type, then that means we don't have all of its fields. If
    // the intended type isn't compatible with our generated type, that
    // means we have some unexpected fields
    // TODO: MUCH better error handling. Ideally without exposing the
    // internal details of the `Type`.
    if (!fieldCheck.compatibleWithConstraint(valTy, metadata.scope)) {
      throw new Error(
        `Constructed value doesn't have all of the fields in type ${valTy.name}`,
      );
    } else if (!valTy.compatibleWithConstraint(fieldCheck, metadata.scope)) {
      throw new Error(
        `Constructed value has fields that don't exist in ${valTy.name}`,
      );
    }

    // *new* new
    return [stmts, new New(ast, valTy, fields)];
  }

  inline(amm: Output, kind: AssignKind, name: string, ty: Type): void {
    const int64 = opcodes().get('int64');
    const size = amm.global('const', int64, this.ty.size().toString());
    amm.assign(kind, name, ty, 'newarr', [size]);
    for (const field in this.fields) {
      const fieldTy = this.fields[field].ty.instance();
      const sizeHint = amm.global('const', int64, `${fieldTy.size()}`);
      const pushCall = fieldTy.isFixed() ? 'pushf' : 'pushv';
      amm.call(pushCall, [name, this.fields[field].ammName, sizeHint]);
    }
  }
}

export class Ref extends Expr {
  def: VarDef;

  get ammName(): string {
    return this.def.ammName;
  }

  get ty(): Type {
    return this.def.ty;
  }

  constructor(def: VarDef) {
    super(def.ast);
    this.def = def;
  }

  inline(_amm: Output, _kind: AssignKind, _name: string, _ty: Type) {
    throw new Error(`did not expect to inline a variable reference`);
  }
}
