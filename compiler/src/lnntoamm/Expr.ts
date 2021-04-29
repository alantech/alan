import { LPNode, NamedAnd, NamedOr } from '../lp';
import Output, { AssignKind } from './Amm';
import Fn from './Fn';
import opcodes from './opcodes';
import Stmt, { Dec, MetaData, VarDef } from './Stmt';
import Type, { Builtin } from './Types';
import { isFnArray, TODO } from './util';

export default abstract class Expr {
  ast: LPNode
  abstract get ty(): Type;

  constructor(
    ast: LPNode,
  ) {
    this.ast = ast;
  }

  abstract inline(amm: Output, kind: AssignKind, name: string, ty: Builtin): void;

  private static fromBaseassignablelist(ast: LPNode, metadata: MetaData): [Stmt[], Expr] {
    let asts = ast.getAll();
    let generated = [];
    let expr: Expr = null;
    for (let ii = 0; ii < asts.length; ii++) {
      let work = asts[ii];
      if (work.has('objectliterals')) {
        TODO('object literals');
      } else if (work.has('functions')) {
        TODO('functions in functions');
      } else if (work.has('variable')) {
        const varName = work.get('variable').t;
        if (ii === asts.length - 1) {
          let dec = metadata.get(varName);
          if (dec === null) {
            throw new Error(`${varName} not defined`);
          }
          expr = dec.ref();
          break;
        }
        const next = asts[ii + 1];
        if (next.has('fncall')) {
          let text = `${expr !== null ? expr.ast.t.trim() + '.' : ''}${varName}${next.get('fncall').t.trim()}`;
          let and: any = {
            fnname: work.get('variable'),
            fncall: next.get('fncall'),
          };
          let accessed: Ref | null = null;
          // DO NOT access `expr` past this block until it is set.
          if (expr !== null) {
            and.fnaccess = expr.ast;
            let dec = Dec.gen(expr, metadata);
            generated.push(dec);
            accessed = dec.ref();
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
        } else if (next.has('methodsep')) {
          TODO('accesses/methods on non-constants');
        } else {
          throw new Error(`unexpected token: expected dor or call, found ${next.t.trim()}`);
        }
      } else if (work.has('constants')) {
        work = work.get('constants');
        if (expr !== null) {
          throw new Error(`unexpected constant found`);
        }
        let [int, constant] = Const.fromConstantsAst(work, metadata);
        generated.push(...int);
        expr = constant;
      } else {
        // TODO: don't lump in HOF and chains
        throw new Error(`unexpected token: expected variable or value, found ${work.t.trim()}`);
      }
    }
    return [generated, expr];
  }

  static fromAssignablesAst(ast: LPNode, metadata: MetaData): [Stmt[], Expr] {
    // break it up so that we're only working on one base assignable list or operator at a time.
    let operated = ast.getAll().map(work => {
      work = work.get('withoperators');
      if (work.has('baseassignablelist')) {
        return Expr.fromBaseassignablelist(work.get('baseassignablelist'), metadata);
      } else if (work.has('operators')) {
        return work;
      } else {
        throw new Error(`unexpected assignable ast: ${work}`);
      }
    });
    if (operated.length === 0) {
      throw new Error(`no expressions generated for ast: ${ast}`);
    } else if (operated.length === 1) {
      if (operated[0][0] instanceof Array) {
        return operated[0] as [Stmt[], Expr];
      } else {
        throw new Error(`variables can't be assigned to operators`);
      }
    } else {
      return TODO('operators');
    }
  }
}

class Call extends Expr {
  fns: Fn[]
  maybeClosure: VarDef | null
  args: Ref[]
  retTy: Type

  get ty(): Type {
    return this.retTy;
  }

  constructor(
    ast: LPNode,
    fns: Fn[],
    maybeClosure: VarDef | null,
    args: Ref[],
    retTy: Type,
  ) {
    super(ast);
    if (fns.length === 0 && maybeClosure === null) {
      throw new Error(`no function possibilities provided for ${ast.t.trim()}`);
    }
    this.fns = fns;
    this.maybeClosure = maybeClosure;
    this.args = args;
    this.retTy = retTy;
  }

  static fromCallAst(
    ast: LPNode,
    fnName: string,
    accessed: Ref | null,
    metadata: MetaData,
  ): [Stmt[], Expr] {
    let stmts = [];
    let argAst = ast.get('fncall').get('assignablelist');
    const argAsts: LPNode[] = [
      argAst.get('assignables'),
      ...argAst.get('cdr').getAll().map(a => a.get('assignables')),
    ];
    let args = [];
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
    let retPossibilities = [];
    retPossibilities.push(...fns.map(fn => fn.retTy));
    if (closure !== null) {
      TODO('closures');
    }
    return [stmts, new Call(ast, fns, closure, args, Type.oneOf(retPossibilities))];
  }

  inline(amm: Output, kind: AssignKind, name: string, ty: Builtin) {
  }
}

class Const extends Expr {
  val: string
  private detectedTy: Builtin

  get ty(): Type {
    return this.detectedTy;
  }

  constructor(
    ast: LPNode,
    val: string,
    detectedTy: Builtin,
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

  inline(amm: Output, kind: AssignKind, name: string, ty: Builtin) {
    const globalName = amm.global('const', ty, this.val);
    amm.assign(kind, name, ty, globalName);
  }
}

export class Ref extends Expr {
  def: VarDef

  get ty(): Type {
    return this.def.ty;
  }

  constructor(
    def: VarDef,
  ) {
    super(def.ast);
    this.def = def;
  }

  inline(_amm: Output, _kind: AssignKind, _name: string, _ty: Builtin) {
    throw new Error(`did not expect to inline a variable reference`);
  }
}
