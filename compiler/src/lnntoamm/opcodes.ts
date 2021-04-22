import Event from './Event';
import { OpcodeFn } from './Fn';
import Scope from "./Scope";
import Type from './Types';

let __opcodes: Scope = null;

const opcodes = (): Scope => {
  if (__opcodes === null) load();
  return __opcodes;
};

export default opcodes;

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
    __opcodes.put(name, Type.newBuiltin(name, generics))
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
    let event: Event;
    if (name === 'start') {
      event = new Event('_start', eventTy, [], true);
    } else {
      event = new Event(name, eventTy, [], false);
    }
    __opcodes.put(name, event);
  });

  Object.entries({
    i64f64: [{a: 'int64'}, 'float64'],

    i64f32: [{a: 'int64'}, 'float32'],

    i64str: [{a: 'int64'}, 'string'],

    boolstr: [{a: 'bool'}, 'string'],

    stdoutp: [{out: 'string'}, 'void'],
    stderrp: [{err: 'string'}, 'void'],
    exitop: [{status: 'int8'}, 'void'],
  } as {
    [opcode: string]: [{[arg: string]: string}, string]
    // Opcode constructor inserts into the opcode scope for us
  }).forEach(([name, [args, ret]]) => {
    new OpcodeFn(name, args, ret, __opcodes);
  });
};
