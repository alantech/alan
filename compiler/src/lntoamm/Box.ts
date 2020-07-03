import Event from './Event'
import Float32 from './Float32'
import Float64 from './Float64'
import Int16 from './Int16'
import Int32 from './Int32'
import Int64 from './Int64'
import Int8 from './Int8'
import Microstatement from './Microstatement'
import Operator from './Operator'
import Scope from './Scope'
import Type from './Type'
import UserFunction from './UserFunction'

class Box {
  type: Type
  readonly: boolean
  typeval: Type | undefined
  scopeval: Scope | undefined
  microstatementval: Microstatement | undefined
  operatorval: Array<Operator> | undefined
  int8val: Int8 | undefined
  int16val: Int16 | undefined
  int32val: Int32 | undefined
  int64val: Int64 | undefined
  float32val: Float32 | undefined
  float64val: Float64 | undefined
  boolval: boolean | undefined
  stringval: string | undefined
  functionval: Array<UserFunction> | undefined
  eventval: Event | undefined
  arrayval: Array<any> | undefined
  mapval: Map<any, any> | undefined
  typevalval: object | undefined

  constructor(...args: Array<any>) {
    if (args.length === 0) {
      this.type = Type.builtinTypes.void
      this.readonly = true
    } else if (args.length === 1) {
      if (typeof args[0] === "boolean") {
        this.type = Type.builtinTypes.void
        this.readonly = args[0]
      } else if (args[0] instanceof Type) {
        this.type = Type.builtinTypes.type
        this.typeval = args[0]
        this.readonly = true // Type declarations are always read-only
      } else if (args[0] instanceof Scope) {
        this.type = Type.builtinTypes.scope
        this.scopeval = args[0]
        this.readonly = true // Boxed scopes are always read-only
      } else if (args[0] instanceof Microstatement) {
        this.type = Type.builtinTypes.microstatement
        this.microstatementval = args[0]
        this.readonly = true
      } else if (args[0] instanceof Array) {
        // This is only operator declarations right now
        this.type = Type.builtinTypes.operator
        this.operatorval = args[0]
        this.readonly = true
      }
    } else if (args.length === 2) {
      if (args[0] instanceof Int8) {
        this.type = Type.builtinTypes.int8
        this.int8val = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Int16) {
        this.type = Type.builtinTypes.int16
        this.int16val = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Int32) {
        this.type = Type.builtinTypes.int32
        this.int32val = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Int64) {
        this.type = Type.builtinTypes.int64
        this.int64val = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Float32) {
        this.type = Type.builtinTypes.float32
        this.float32val = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Float64) {
        this.type = Type.builtinTypes.float64
        this.float64val = args[0]
        this.readonly = args[1]
      } else if (typeof args[0] === "boolean") {
        this.type = Type.builtinTypes.bool
        this.boolval = args[0]
        this.readonly = args[1]
      } else if (typeof args[0] === "string") {
        this.type = Type.builtinTypes.string
        this.stringval = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Array) {
        // This is only function declarations right now
        this.type = Type.builtinTypes["function"]
        this.functionval = args[0]
        this.readonly = args[1]
      } else if (args[0] instanceof Event) {
        this.type = Type.builtinTypes.Event
        this.eventval = args[0]
        this.readonly = args[1]
      }
      // Technically there's also supposed to be something for an "Object blank" but I don't know
      // what that is supposed to be for
    } else if (args.length === 3) {
      if (args[0] instanceof Array) {
        // It's an array, like a real one
        this.type = args[1]
        this.arrayval = args[0]
        this.readonly = args[2]
      } else if (args[0] instanceof Map) {
        // It's a map, also a real one
        this.type = args[1]
        this.mapval = args[0]
        this.readonly = args[2]
      } else if (args[0] instanceof Object) {
        // It's an event or user-defined type
        if (args[1].originalType == Type.builtinTypes.Event) {
          let eventval = new Event(args[0].name.stringval, args[1].properties.type, false)
          this.eventval = eventval
          this.readonly = args[2]
        } else {
          this.type = args[1]
          this.typevalval = args[0]
          this.readonly = args[2]
        }
      }
    }
  }
}

export default Box
