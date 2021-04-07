import Event from './Event';
import Fn, { Args } from "./Fn";
import Scope from "./Scope";
import { Interface, Type } from './Types';

let __opcodes: Scope = null;

const opcodes = (): Scope => {
  if (__opcodes === null) load();
  return __opcodes;
};

export default opcodes;

class Opcode extends Fn {
  constructor(name: string, argDecs: {[name: string]: string}, retTyName: string) {
    let args: Args = {};
    for (let argName of Object.keys(argDecs)) {
      let argTy = argDecs[argName];
      let ty = __opcodes.get(argTy);
      if (ty === null) {
        throw new Error(`opcode ${name} arg ${argName} uses a type that's not defined in the opcode scope (${argTy})`);
      } else if (!(ty instanceof Type) && !(ty instanceof Interface)) {
        throw new Error(`opcode ${name} arg ${argName} doesn't have a valid type (${argTy})`);
      } else {
        args[ty[0]] = ty;
      }
    }
    let ret: Type = __opcodes.get(retTyName);
    if (ret === null || !(ret instanceof Type)) {
      throw new Error(`opcode ${name} doesn't return a valid type`);
    }
    super(null, __opcodes, name, args, ret, []);

    __opcodes.put(this.name, [this]);
  }
}

const load = (): void => {
  __opcodes = new Scope();

  Object.entries({
    void: [],
    int8: [],
    int16: [],
    int32: [],
    int64: [],
    float32: [],
    float64: [],
    bool: [],
    string: [],
  }).forEach(([name, generics]: [string, string[]]) => {
    let genericTypes = {};
    generics.forEach(gen => genericTypes[gen] = null);
    __opcodes.put(name, new Type(name, genericTypes, null, {}));
  });

  Object.entries({
    start: 'void',
  }).forEach(([name, tyName]: [string, string]) => {
    const eventTy: Type = __opcodes.get(tyName);
    if (eventTy === null) {
      throw new Error(`builtin event ${name} has type ${tyName}, which isn't defined in the opcode scope`);
    } else if (!(eventTy instanceof Type)) {
      throw new Error(`builtin event ${name} is declared with type ${tyName}, but that's not a valid type`);
    }
    const event = new Event(name, eventTy, []);
    __opcodes.put(name, event);
  });

  Object.entries({
    i64f64: [{a: 'int64'}, 'float64'],

    i64f32: [{a: 'int64'}, 'float32'],

    i64str: [{a: 'int64'}, 'string'],

    stdoutp: [{out: 'string'}, 'void'],
    stderrp: [{err: 'string'}, 'void'],
    exit: [{status: 'int8'}, 'void'],
  } as {
    [opcode: string]: [{[arg: string]: string}, string]
    // Opcode constructor inserts into the opcode scope for us
  }).forEach(([name, [args, ret]]) => new Opcode(name, args, ret));
};
