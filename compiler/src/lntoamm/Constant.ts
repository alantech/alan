import Scope from './Scope'

class Constant {
  name: string
  assignablesAst: any
  scope: Scope

  constructor(name: string, assignablesAst: any, scope: Scope) {
    this.name = name
    this.assignablesAst = assignablesAst
    this.scope = scope
  }

  static fromAst(constdeclaration: any, scope: Scope) {
    const name = constdeclaration.VARNAME().getText()
    const outConst = new Constant(
      name,
      constdeclaration.assignables(),
      scope,
    )
    scope.put(name, outConst)
    return outConst
  }
}

export default Constant
