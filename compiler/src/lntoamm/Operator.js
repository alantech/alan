class Operator {
  constructor(name, precedence, isPrefix, potentialFunctions) {
    this.name = name
    this.precedence = precedence
    this.isPrefix = isPrefix
    this.potentialFunctions = potentialFunctions
  }

  applicableFunction(left, right, scope) {
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
      const argList = Object.values(args)
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
          for (const propKey of Object.keys(argList[j].properties)) {
            const propVal = argList[j].properties[propKey]
            if (propVal == argumentTypeList[j].properties[propKey]) continue
            if (
              propVal.iface != null &&
              propVal.iface.typeApplies(
                argumentTypeList[j].properties[propKey],
                scope
              )
            ) continue
            skip = true
          }
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

module.exports = Operator
