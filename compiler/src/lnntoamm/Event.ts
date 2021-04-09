import { LPNode } from "../lp";
import Fn from './Fn';
import Scope from "./Scope";
import Type from "./Types";
import { TODO } from "./util";

var allEvents: Event[] = [];

export default class Event {
  name: string
  eventTy: Type
  // if it's a single fn
  handlers: Array<Fn | Fn[]>

  static get allEvents(): Event[] {
    return allEvents;
  }

  constructor(
    name: string,
    eventTy: Type,
    handlers: Array<Fn | Fn[]> = []
  ) {
    this.name = name;
    this.eventTy = eventTy;
    this.handlers = handlers;
    allEvents.push(this);
  }

  static fromAst(ast: LPNode, scope: Scope): Event {
    const name = ast.get('variable').t.trim();
    const tyName = ast.get('fulltypename').t.trim();
    const ty = scope.get(tyName);
    if (ty === null) {
      throw new Error(`Could not find specified type: ${tyName}`);
    } else if (!(ty instanceof Type)) {
      throw new Error(`${tyName} is not a type`);
    }
    return new Event(name, ty);
  }

  typeCheck() {
    for (let handler of this.handlers) {
      if (handler instanceof Array) {
        TODO('Event handler selection')
      }
      if (!(handler instanceof Fn)) {
        throw new Error('Too many possible event handlers');
      }
      let [varConstraints, retConstraints] = (handler as Fn).constraints();
      console.log(`------ handler for ${this.name} returns ${retConstraints.map(t => t.name).join(',')}`)
      for (let {dec, constraints} of varConstraints) {
        if (dec ===  null) {
          retConstraints.push(...constraints);
          continue;
        }
        console.log(dec.name, 'is', dec.ty.name, 'constraints:', constraints);
        console.log(`${dec.name} is ${dec.ty.name}, constrained: ${constraints.map(t => t.name).join(',')}`)
      }
    }
  }
}
