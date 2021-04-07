import Event from './Event'
import Fn from './Fn';
import opcodes from './opcodes';
import Statement from './Statement';
import { Interface, Type } from './Types';

export type Constraint = [string, Type | Interface];

export default class TypeChecker {
  static checkEvents(events: Event[]) {
    const opcodeScope = opcodes();

    for (let event of events) {
      const handlers = event.handlers.map(possibilities => {
        if (possibilities instanceof Fn) {
          possibilities.transform();
          return possibilities;
        }
        const fnName = possibilities[0].name;
        possibilities.forEach(fn => fn.transform());
        if (event.eventTy === opcodeScope.get('void')) {
          possibilities = possibilities.filter(fn => Object.keys(fn.args).length === 0);
        } else {
          possibilities = possibilities.filter(fn => {
            const args = Object.keys(fn.args);
            return args.length === 1 &&
                    // TODO: don't just check on the type name
                    fn.args[args[0]].name === event.eventTy.name;
          });
        }
        if (possibilities.length === 0) {
          throw new Error(`Tried to assign function ${fnName} to event ${event.name}, but no functions by that name were eligible`);
        } else if (possibilities.length > 1) {
          throw new Error(`Too many functions called ${fnName} that can subscribe to event ${event.name}`);
        }
        return possibilities.pop();
      });

      for (let handler of handlers) {
        TypeChecker.checkStatements(handler.body as Statement[]);
      }
    }
  }

  static checkStatements(statements: Statement[]) {}
}
