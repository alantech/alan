import { NulLP } from '../lp';
import Event from './Event';
import Scope from "./Scope";
import Type from './Types';

let __opcodes: Scope = null;

const opcodes = (): Scope => {
  if (__opcodes === null) load();
  return __opcodes;
};

export default opcodes;

const addOpcode = (
  name: string,
  argDecs: {[name: string]: string},
  retTyName: string,
) => {
  let args = {};
  for (let argName of Object.keys(argDecs)) {
    let argTy = argDecs[argName];
    let ty = __opcodes.get(argTy);
    if (ty === null) {
      throw new Error(`opcode ${name} arg ${argName} uses a type that's not defined`);
    } else if (!(ty instanceof Type)) {
      throw new Error(`opcode ${name} arg ${argName} doesn't have a valid type`);
    } else {
      args[ty[0]] = ty;
    }
  }
  let retTy = __opcodes.get(retTyName);
  if (retTy === null || !(retTy instanceof Type)) {
    throw new Error(`opcode ${name} doesn't return a valid type`);
  }
  const opcode = {
    name,
    ast: new NulLP(),
    scope: __opcodes,
    args,
    retTy,
    body: [],
    stmtMeta: null,
    transform: () => {},
    constraints: () => [[], []] as [any[], any[]], // ugh
    getReturnType: () => retTy,
  };
  // if this line errors, that's because we're not successfully duck-typing the opcode as a Fn
  // (duck-typing necessary because otherwise Node complains about circular dependencies)
  __opcodes.put(name, [opcode]);
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
  }).forEach(([name, [args, ret]]) => addOpcode(name, args, ret));
};
