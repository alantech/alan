import { LPNode } from '../lp'
import { Ref } from './Expr'
import Fn from './Fn'
import Scope from './Scope'
import { isFnArray } from './util'

class Operator {
  ast: LPNode
  symbol: string
  precedence: number
  isPrefix: boolean
  fns: Array<Fn>

  constructor(ast: LPNode, symbol: string, precedence: number, isPrefix: boolean, fns: Array<Fn>) {
    this.ast = ast;
    this.symbol = symbol;
    this.precedence = precedence;
    this.isPrefix = isPrefix;
    this.fns = fns;
  }

  static fromAst(ast: LPNode, scope: Scope): Operator {
    const isPrefix = ast.get('fix').has('prefix');
    const opmap = ast.get('opmap').get();
    const precedence = opmap.get('opprecedence').get('num');
    const symbol = opmap.get('fntoop').get('operators');
    const fnName = opmap.get('fntoop').get('fnname');
    const scoped = scope.get(fnName.t);
    if (scoped === null) {
      throw new Error(`${fnName.t} cannot be used as an operator - not found`);
    } else if (!isFnArray(scoped)) {
      throw new Error(`${fnName.t} cannot be used as an operator - it's not a function`);
    }
    const fns = scoped.filter(fn => fn.params.length === (isPrefix ? 1 : 2));
    return new Operator(
      ast,
      symbol.t,
      Number.parseInt(precedence.t),
      isPrefix,
      fns,
    );
  }

  select(arg1: Ref, arg2?: Ref): Fn[] {
    if ((this.isPrefix && arg2) || (!this.isPrefix && !arg2)) {
      throw new Error(`nope`);
    }
    const tys = [arg1.ty, ...(arg2 ? [arg2.ty] : [])];
    return this.fns.filter(fn => fn.acceptsTypes(tys));
  }
}

export default Operator
