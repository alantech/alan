import Const from './Const';
import Event from './Event';
import Fn from './Fn';
import Operator from './Operator';
import { Interface, Type } from './Types';

// type Boxish = Type | Scope | Microstatement | Array<Operator> | Array<Fn> | Event | Constant | undefined
// Scope instead of a Module
type Boxish = Scope | Const | Event | Fn[] | Interface | Operator[] | Type

type BoxSet = {
  [K: string]: Boxish
}

class Scope {
  vals: BoxSet
  par: Scope | null
  secondaryPar: Scope | null

  constructor(par?: Scope) {
    this.vals = {}
    this.par = par ? par : null
    this.secondaryPar = null
  }

  get(name: string) {
    if (this.vals.hasOwnProperty(name)) {
      return this.vals[name]
    }
    if (!!this.par) {
      const val = this.par.get(name)
      if (!val && !!this.secondaryPar) {
        return this.secondaryPar.get(name)
      } else {
        return val
      }
    }
    return null
  }

  shallowGet(name: string) {
    if (this.vals.hasOwnProperty(name)) {
      return this.vals[name]
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
