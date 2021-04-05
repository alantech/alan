import { LPNode } from "../lp";
import Fn from './Fn';
import Scope from "./Scope";
import { Type } from "./Types";

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
}
