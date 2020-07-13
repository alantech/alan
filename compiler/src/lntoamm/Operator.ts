import * as Ast from './Ast'
import Scope from './Scope'
import Type from './Type'
import UserFunction from './UserFunction'

class Operator {
  name: string
  precedence: number
  isPrefix: boolean
  potentialFunctions: Array<UserFunction>

  constructor(name: string, precedence: number, isPrefix: boolean, potentialFunctions: Array<any>) {
    this.name = name
    this.precedence = precedence
    this.isPrefix = isPrefix
    this.potentialFunctions = potentialFunctions
  }

  applicableFunction(left: Type, right: Type, scope: Scope) {
    let argumentTypeList = []
    if (!this.isPrefix) {
      if (left == null) return null
      argumentTypeList.push(left)
    }
    argumentTypeList.push(right)
    const fns = this.potentialFunctions
    for (let i = 0; i < fns.length; i++) {
      const isNary = fns[i].isNary()
      const args = fns[i].getArguments()
      const argList: Array<Type> = Object.values(args)
      if (!isNary && argList.length != argumentTypeList.length) continue
      if (isNary && argList.length > argumentTypeList.length) continue
      let skip = false
      for (let j = 0; j < argList.length; j++) {
        if (argList[j].typename === argumentTypeList[j].typename) continue
        if (argList[j].iface &&
          argList[j].iface.typeApplies(argumentTypeList[j], scope)
        ) continue
        if (argList[j].generics.length > 0 && argumentTypeList[j].originalType == argList[j]) {
          continue
        }
        if (
          argList[j].originalType != null &&
          argumentTypeList[j].originalType == argList[j].originalType
        ) {
          const argListAst = Ast.fulltypenameAstFromString(argList[j].typename)
          const argumentTypeListAst = Ast.fulltypenameAstFromString(argumentTypeList[j].typename)
          const len = argListAst.typegenerics() ?
            argListAst.typegenerics().fulltypename().length : 0
          let innerSkip = false
          for (let i = 0; i < len; i++) {
            const argListTypeProp = argListAst.typegenerics().fulltypename(i).getText()
            const argumentTypeListTypeProp =
              argumentTypeListAst.typegenerics().fulltypename(i).getText()
            if (argListTypeProp === argumentTypeListTypeProp) continue
            const argListProp = scope.deepGet(argListTypeProp) as Type
            const argumentTypeListProp = scope.deepGet(argumentTypeListTypeProp) as Type
            if (!argListProp || !(argListProp instanceof Type)) {
              innerSkip = true
              break
            }
            if (!argumentTypeListProp || !(argumentTypeListProp instanceof Type)) {
              innerSkip = true
              break
            }
            if (
              argListProp.iface != null &&
              argListProp.iface.typeApplies(argumentTypeListProp, scope)
            ) continue
            innerSkip = true
          }
          if (innerSkip) skip = true
          continue
        }
        skip = true
      }
      if (skip) continue
      return fns[i]
    }
    return null
  }
}

export default Operator
