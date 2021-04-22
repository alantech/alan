import { LPNode } from '../lp'
import Fn from './Fn'
import Scope from './Scope'

class Operator {
  name: string
  precedence: number
  isPrefix: boolean
  potentialFunctions: Array<Fn>

  constructor(name: string, precedence: number, isPrefix: boolean, potentialFunctions: Array<Fn>) {
    this.name = name
    this.precedence = precedence
    this.isPrefix = isPrefix
    this.potentialFunctions = potentialFunctions
  }

  static fromAst(ast: LPNode, scope: Scope): Operator {
    return null;
  }
}

export default Operator
