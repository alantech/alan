import * as Ast from './Ast'
import Fn from './Function'
import Scope from './Scope'
import Type from './Type'
import UserFunction from './UserFunction'

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

  applicableFunction(left: Type, right: Type, scope: Scope) {
    let argumentTypeList = []
    if (!this.isPrefix) {
      if (left == null) return null
      argumentTypeList.push(left)
    }
    argumentTypeList.push(right)
    const fns = this.potentialFunctions
    for (let i = 0; i < fns.length; i++) {
      const args = fns[i].getArguments()
      const argList: Array<Type> = Object.values(args)
      if (argList.length != argumentTypeList.length) continue
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
          const argGenericTypes = []
          if (argListAst.has('opttypegenerics')) {
            argGenericTypes.push(argListAst.get('opttypegenerics').get('fulltypename').t);
            argListAst.get('opttypegenerics').get('cdr').getAll().map(r => {
              argGenericTypes.push(r.get('fulltypename').t)
            })
          }
          const argumentGenericTypes = []
          if (argumentTypeListAst.has('opttypegenerics')) {
            argumentGenericTypes.push(
              argumentTypeListAst.get('opttypegenerics').get('fulltypename').t
            );
            argumentTypeListAst.get('opttypegenerics').get('cdr').getAll().map(
              r => { argumentGenericTypes.push(r.get('fulltypename').t) }
            )
          }
          let innerSkip = false
          for (let i = 0; i < argGenericTypes.length; i++) {
            const argListTypeProp = argGenericTypes[i]
            const argumentTypeListTypeProp = argumentGenericTypes[i]
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
