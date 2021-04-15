import { LPNode } from "../lp";
import Output from "./Amm";
import Fn from './Fn';
import opcodes from "./opcodes";
import Scope from "./Scope";
import Type, { Builtin } from "./Types";
import { genName, TODO } from "./util";

var allEvents: Event[] = [];

export default class Event {
  ammName: string
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
    if (allEvents.some(event => event.name === this.name)) {
      this.ammName = genName();
    } else {
      this.ammName = this.name;
    }
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
    amm.global('event', this.eventTy as Builtin, this.ammName);
    for (let handler of this.handlers) {
      if (handler instanceof Array) {
        return TODO('Event handler selection');
      }
      if (!(handler instanceof Fn)) {
        throw new Error('Too many possible event handlers');
      }
      // let [varConstraints, retConstraints] = handler.constraints([this.eventTy]);
      // for (let {dec, constraints} of varConstraints) {
      //   if (!constraints.every(con => dec.ty.compatibleWithConstraint(con))) {
      //     throw new Error(`failed to type-check: declaration ${dec}`);
      //   }
      // }
      // retConstraints.push(opcodes().get('void') as Type);
      // really tsc????? you're gonna complain that handler could be an array
      // here but not above? wat?
      // if (!retConstraints.every(con => (handler as Fn).retTy.compatibleWithConstraint(con))) {
      //   throw new Error(`failed to type-check: event handler ${handler}`);
      // }
      handler.asHandler(amm, this.ammName);
    }
  }

  // typeCheck() {
  //   for (let handler of this.handlers) {
  //     if (handler instanceof Array) {
  //       TODO('Event handler selection')
  //     }
  //     if (!(handler instanceof Fn)) {
  //       throw new Error('Too many possible event handlers');
  //     }
  //     let [varConstraints, retConstraints] = (handler as Fn).constraints();
  //     console.log(`------ handler for ${this.name} returns ${retConstraints.map(t => t.name).join(',')}`)
  //     for (let {dec, constraints} of varConstraints) {
  //       if (dec ===  null) {
  //         retConstraints.push(...constraints);
  //         continue;
  //       }
  //       console.log(dec.name, 'is', dec.ty.name, 'constraints:', constraints);
  //       console.log(`${dec.name} is ${dec.ty.name}, constrained: ${constraints.map(t => t.name).join(',')}`)
  //     }
  //   }
  // }
}
