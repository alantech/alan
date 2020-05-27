class Event {
  constructor(name, type, builtIn) {
    this.name = name,
    this.type = type
    this.builtIn = builtIn
    this.handlers = []
    Event.allEvents.push(this)
  }

  toString() {
    return `event ${this.name}: ${this.type.typename}`
  }

  static fromAst(eventAst, scope) {
    const name = eventAst.VARNAME().getText()
    const boxedVal = scope.deepGet(eventAst.varn())
    if (boxedVal === null) {
      console.error("Could not find specified type: " + eventAst.varn().getText())
      process.exit(-8)
    } else if (!boxedVal.type.typename === "type") {
      console.error(eventAst.varn().getText() + " is not a type")
      process.exit(-9)
    }
    const type = boxedVal.typeval
    return new Event(name, type, false)
  }
}

Event.allEvents = []

module.exports = Event
