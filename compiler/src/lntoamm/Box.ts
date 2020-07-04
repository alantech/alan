import Event from './Event'
import Microstatement from './Microstatement'
import Operator from './Operator'
import Scope from './Scope'
import Type from './Type'
import UserFunction from './UserFunction'

type Boxish = Type | Scope | Microstatement | Array<Operator> | boolean | string |
  Array<UserFunction> | Event | Array<any> | Map<any, any> | object | undefined

class Box {
  val: Boxish
  type: Type
  readonly: boolean

  constructor(
    val: Boxish = undefined,
    type: Type = Type.builtinTypes.void,
    readonly: boolean = true
  ) {
    this.val = val
    this.type = type
    this.readonly = readonly
  }
}

export default Box
