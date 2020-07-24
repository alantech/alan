import Constant from './Constant'
import Event from './Event'
import Fn from './Function'
import Microstatement from './Microstatement'
import Operator from './Operator'
import Type from './Type'

type Boxish = Type | Scope | Microstatement | Array<Operator> | Array<Fn> | Event | Constant | undefined

type BoxSet = {
  [K: string]: Boxish
}

class Scope {
  vals: BoxSet
  par: Scope | null

  constructor(par?: Scope) {
    this.vals = {}
    this.par = par ? par : null
  }

  get(name: string) {
    if (this.vals.hasOwnProperty(name)) {
      return this.vals[name]
    }
    if (!!this.par) {
      return this.par.get(name)
    }
    return null
  }

  deepGet(fullName: string) {
    const fullVar = fullName.trim().split(".")
    let boxedVar: Boxish
    for (let i = 0; i < fullVar.length; i++) {
      if (i === 0) {
        boxedVar = this.get(fullVar[i])
      } else if (!boxedVar) {
        return null
      } else {
        if (boxedVar instanceof Scope) {
          boxedVar = boxedVar.get(fullVar[i])
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
    if (!!this.par) {
      return this.par.has(name)
    }
    return false
  }

  put(name: string, val: Boxish) {
    this.vals[name.trim()] = val
  }
}

export default Scope
