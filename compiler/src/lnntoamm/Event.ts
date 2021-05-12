import { LPNode } from '../lp';
import Output from './Amm';
import Fn from './Fn';
import Scope from './Scope';
import Type, { Builtin } from './Types';
import { genName, TODO } from './util';

let allEvents: Event[] = [];

export default class Event {
  ammName: string
  name: string
  eventTy: Type
  // if it's a single fn
  handlers: Array<Fn | Fn[]>
  runtimeDefined: boolean

  static get allEvents(): Event[] {
    return allEvents;
  }

  constructor(
    name: string,
    eventTy: Type,
    handlers: Array<Fn | Fn[]> = [],
    runtimeDefined: boolean = false,
  ) {
    this.name = name;
    this.eventTy = eventTy;
    this.handlers = handlers;
    if (allEvents.some(event => event.name === this.name)) {
      this.ammName = genName();
    } else {
      this.ammName = this.name;
    }
    this.runtimeDefined = runtimeDefined;
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

  compile(amm: Output) {
    if (this.runtimeDefined === false) {
      amm.global('event', this.eventTy as Builtin, this.ammName);
    }
    for (let handler of this.handlers) {
      if (handler instanceof Array) {
        // select all of the handlers that accept `this.eventTy` and return `void`.
        return TODO('Event handler selection');
      }
      if (!(handler instanceof Fn)) {
        throw new Error('Too many possible event handlers');
      }
      handler.asHandler(amm, this.ammName);
    }
  }
}
