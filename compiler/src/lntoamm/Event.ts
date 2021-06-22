import Scope from './Scope';
import Type from './Type';
import UserFunction from './UserFunction';
import { LPNode } from '../lp';

class Event {
  name: string;
  type: Type;
  builtIn: boolean;
  handlers: Array<UserFunction>;
  static allEvents: Array<Event> = [];

  constructor(name: string, type: any, builtIn: boolean) {
    (this.name = name), (this.type = type);
    this.builtIn = builtIn;
    this.handlers = [];
    Event.allEvents.push(this);
  }

  toString() {
    return `event ${this.name}: ${this.type.typename}`;
  }

  static fromAst(eventAst: LPNode, scope: Scope) {
    const name = eventAst.get('variable').t;
    const type = scope.deepGet(eventAst.get('fulltypename').t) as Type;
    if (!type) {
      throw new Error(
        'Could not find specified type: ' + eventAst.get('fulltypename').t,
      );
    } else if (!(type instanceof Type)) {
      throw new Error(eventAst.get('fulltypename').t + ' is not a type');
    }
    return new Event(name, type, false);
  }
}

export default Event;
