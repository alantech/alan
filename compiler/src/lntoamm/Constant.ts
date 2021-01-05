import Scope from './Scope'
import { LPNode, } from '../lp'

class Constant {
  name: string
  assignablesAst: LPNode
  scope: Scope

  constructor(name: string, assignablesAst: LPNode, scope: Scope) {
    this.name = name
    this.assignablesAst = assignablesAst
    this.scope = scope
  }

  static fromAst(constdeclaration: LPNode, scope: Scope) {
    const name = constdeclaration.get('variable').t
    const outConst = new Constant(
      name,
      constdeclaration.get('assignables'),
      scope,
    )
    scope.put(name, outConst)
    return outConst
  }
}

export default Constant
