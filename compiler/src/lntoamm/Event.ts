import Scope from './Scope'
import Type from './Type'
import UserFunction from './UserFunction'

class Event {
  name: string
  type: Type
  builtIn: boolean
  handlers: Array<UserFunction>
  static allEvents: Array<Event> = []

  constructor(name: string, type: any, builtIn: boolean) {
    this.name = name,
    this.type = type
    this.builtIn = builtIn
    this.handlers = []
    Event.allEvents.push(this)
  }

  toString() {
    return `event ${this.name}: ${this.type.typename}`
  }

  static fromAst(eventAst: any, scope: Scope) { // TODO: Eliminate ANTLR
    const name = eventAst.VARNAME().getText()
    const type = scope.deepGet(eventAst.fulltypename().getText()) as Type
    if (!type) {
      throw new Error("Could not find specified type: " + eventAst.fulltypename().getText())
    } else if (!(type instanceof Type)) {
      throw new Error(eventAst.fulltypename().getText() + " is not a type")
    }
    return new Event(name, type, false)
  }
}

export default Event
