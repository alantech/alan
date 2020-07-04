import Event from './Event'
import Microstatement from './Microstatement'
import Operator from './Operator'
import Scope from './Scope'
import Type from './Type'
import Fn from './Function'

type Boxish = Type | Scope | Microstatement | Array<Operator> | Array<Fn> | Event

class Box {
  val: Boxish
  type: Type

  constructor(
    val: Boxish,
    type: Type,
  ) {
    this.val = val
    this.type = type
  }
}

export default Box
