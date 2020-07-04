import Box from './Box'
import Type from './Type'

class Scope {
  vals: object
  par: Scope | null

  constructor(par?: Scope) {
    this.vals = {}
    this.par = par ? par : null
  }

  get(name: string) {
    if (this.vals.hasOwnProperty(name)) {
      return this.vals[name]
    }
    if (this.par != null) {
      return this.par.get(name)
    }
    return null
  }

  deepGet(fullName: string) {
    const fullVar = fullName.trim().split(".")
    let boxedVar: any
    for (let i = 0; i < fullVar.length; i++) {
      if (i === 0) {
        boxedVar = this.get(fullVar[i])
      } else if (boxedVar === null) {
        return null
      } else {
        if (boxedVar.type === Type.builtinTypes['scope']) {
          boxedVar = boxedVar.val.get(fullVar[i])
        } else if (!Object.values(Type.builtinTypes).includes(boxedVar.type)) {
          boxedVar = boxedVar.val[fullVar[i]]
        } else {
          return null
        }
      }
    }
    return boxedVar
  }

  has(name: string) {
    if (this.vals.hasOwnProperty(name)) {
      return true
    }
    if (this.par != null) {
      return this.par.has(name)
    }
    return false
  }

  put(name: string, val: Box) {
    this.vals[name.trim()] = val
  }
}

export default Scope
