import { LPNode } from '../lp'
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
    const precedence = ast.get('opprecedence').get('num');
    const symbol = ast.get('fntoop').get('operators');
    const fnName = ast.get('fntoop').get('fnname');
    const fns = scope.get(fnName.t);
    if (fns === null || !isFnArray(fns)) {
      throw new Error(`cannot create operator ${symbol.t} - no functions named ${fnName}`);
    }
    return new Operator(
      ast,
      symbol.t,
      Number.parseInt(precedence.t),
      isPrefix,
      fns,
    );
  }
}

export default Operator
