import Event from './Event';
import { OpcodeFn } from './Fn';
import Scope from './Scope';
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
    // Array: ['T'],
    Error: [],
    Maybe: ['T'],
    Result: ['T'],
  }).forEach(([name, generics]: [string, string[]]) => {
    __opcodes.put(name, Type.opaque(name, generics));
  });

  __opcodes.put('any', Type.interface('any'));
  __opcodes.put('anythingElse', Type.interface('anythingElse'));

  Object.entries({
    start: 'void',
  }).forEach(([name, tyName]: [string, string]) => {
    const eventTy: Type = __opcodes.get(tyName);
    if (eventTy === null) {
      throw new Error(
        `builtin event ${name} has type ${tyName}, which isn't defined in the opcode scope`,
      );
    } else if (!(eventTy instanceof Type)) {
      throw new Error(
        `builtin event ${name} is declared with type ${tyName}, but that's not a valid type`,
      );
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
    i8f64: [{ a: 'int8' }, 'float64'],
    i16f64: [{ a: 'int16' }, 'float64'],
    i32f64: [{ a: 'int32' }, 'float64'],
    i64f64: [{ a: 'int64' }, 'float64'],
    f32f64: [{ a: 'float32' }, 'float64'],
    strf64: [{ a: 'string' }, 'float64'],
    boolf64: [{ a: 'bool' }, 'float64'],

    i8f32: [{ a: 'int8' }, 'float32'],
    i16f32: [{ a: 'int16' }, 'float32'],
    i32f32: [{ a: 'int32' }, 'float32'],
    i64f32: [{ a: 'int64' }, 'float32'],
    f64f32: [{ a: 'float64' }, 'float32'],
    strf32: [{ a: 'string' }, 'float32'],
    boolf32: [{ a: 'bool' }, 'float32'],

    i8i64: [{ a: 'int8' }, 'int64'],
    i16i64: [{ a: 'int16' }, 'int64'],
    i32i64: [{ a: 'int32' }, 'int64'],
    f32i64: [{ a: 'float32' }, 'int64'],
    f64i64: [{ a: 'float64' }, 'int64'],
    stri64: [{ a: 'string' }, 'int64'],
    booli64: [{ a: 'bool' }, 'int64'],

    i8i32: [{ a: 'int8' }, 'int32'],
    i16i32: [{ a: 'int16' }, 'int32'],
    i64i32: [{ a: 'int64' }, 'int32'],
    f32i32: [{ a: 'float32' }, 'int32'],
    f64i32: [{ a: 'float64' }, 'int32'],
    stri32: [{ a: 'string' }, 'int32'],
    booli32: [{ a: 'bool' }, 'int32'],

    i8i16: [{ a: 'int8' }, 'int16'],
    i32i16: [{ a: 'int32' }, 'int16'],
    i64i16: [{ a: 'int64' }, 'int16'],
    f32i16: [{ a: 'float32' }, 'int16'],
    f64i16: [{ a: 'float64' }, 'int16'],
    stri16: [{ a: 'string' }, 'int16'],
    booli16: [{ a: 'bool' }, 'int16'],

    i16i8: [{ a: 'int16' }, 'int8'],
    i32i8: [{ a: 'int32' }, 'int8'],
    i64i8: [{ a: 'int64' }, 'int8'],
    f32i8: [{ a: 'float32' }, 'int8'],
    f64i8: [{ a: 'float64' }, 'int8'],
    stri8: [{ a: 'string' }, 'int8'],
    booli8: [{ a: 'bool' }, 'int8'],

    i8bool: [{ a: 'int8' }, 'bool'],
    i16bool: [{ a: 'int16' }, 'bool'],
    i32bool: [{ a: 'int32' }, 'bool'],
    i64bool: [{ a: 'int64' }, 'bool'],
    f32bool: [{ a: 'float32' }, 'bool'],
    f64bool: [{ a: 'float64' }, 'bool'],
    strbool: [{ a: 'string' }, 'bool'],

    i8str: [{ a: 'int8' }, 'string'],
    i16str: [{ a: 'int16' }, 'string'],
    i32str: [{ a: 'int32' }, 'string'],
    i64str: [{ a: 'int64' }, 'string'],
    f32str: [{ a: 'float32' }, 'string'],
    f64str: [{ a: 'float64' }, 'string'],
    boolstr: [{ a: 'bool' }, 'string'],

    eqi8: [{ a: 'int8', b: 'int8' }, 'bool'],
    eqi16: [{ a: 'int16', b: 'int16' }, 'bool'],
    eqi32: [{ a: 'int32', b: 'int32' }, 'bool'],
    eqi64: [{ a: 'int64', b: 'int64' }, 'bool'],
    eqf32: [{ a: 'float32', b: 'float32' }, 'bool'],
    eqf64: [{ a: 'float64', b: 'float64' }, 'bool'],
    eqstr: [{ a: 'string', b: 'string' }, 'bool'],
    eqbool: [{ a: 'bool', b: 'bool' }, 'bool'],

    neqi8: [{ a: 'int8', b: 'int8' }, 'bool'],
    neqi16: [{ a: 'int16', b: 'int16' }, 'bool'],
    neqi32: [{ a: 'int32', b: 'int32' }, 'bool'],
    neqi64: [{ a: 'int64', b: 'int64' }, 'bool'],
    neqf32: [{ a: 'float32', b: 'float32' }, 'bool'],
    neqf64: [{ a: 'float64', b: 'float64' }, 'bool'],
    neqstr: [{ a: 'string', b: 'string' }, 'bool'],
    neqbool: [{ a: 'bool', b: 'bool' }, 'bool'],

    lti8: [{ a: 'int8', b: 'int8' }, 'bool'],
    lti16: [{ a: 'int16', b: 'int16' }, 'bool'],
    lti32: [{ a: 'int32', b: 'int32' }, 'bool'],
    lti64: [{ a: 'int64', b: 'int64' }, 'bool'],
    ltf32: [{ a: 'float32', b: 'float32' }, 'bool'],
    ltf64: [{ a: 'float64', b: 'float64' }, 'bool'],
    ltstr: [{ a: 'string', b: 'string' }, 'bool'],

    ltei8: [{ a: 'int8', b: 'int8' }, 'bool'],
    ltei16: [{ a: 'int16', b: 'int16' }, 'bool'],
    ltei32: [{ a: 'int32', b: 'int32' }, 'bool'],
    ltei64: [{ a: 'int64', b: 'int64' }, 'bool'],
    ltef32: [{ a: 'float32', b: 'float32' }, 'bool'],
    ltef64: [{ a: 'float64', b: 'float64' }, 'bool'],
    ltestr: [{ a: 'string', b: 'string' }, 'bool'],

    gti8: [{ a: 'int8', b: 'int8' }, 'bool'],
    gti16: [{ a: 'int16', b: 'int16' }, 'bool'],
    gti32: [{ a: 'int32', b: 'int32' }, 'bool'],
    gti64: [{ a: 'int64', b: 'int64' }, 'bool'],
    gtf32: [{ a: 'float32', b: 'float32' }, 'bool'],
    gtf64: [{ a: 'float64', b: 'float64' }, 'bool'],
    gtstr: [{ a: 'string', b: 'string' }, 'bool'],

    gtei8: [{ a: 'int8', b: 'int8' }, 'bool'],
    gtei16: [{ a: 'int16', b: 'int16' }, 'bool'],
    gtei32: [{ a: 'int32', b: 'int32' }, 'bool'],
    gtei64: [{ a: 'int64', b: 'int64' }, 'bool'],
    gtef32: [{ a: 'float32', b: 'float32' }, 'bool'],
    gtef64: [{ a: 'float64', b: 'float64' }, 'bool'],
    gtestr: [{ a: 'string', b: 'string' }, 'bool'],

    notbool: [{ b: 'bool' }, 'bool'],
    andbool: [{ a: 'bool', b: 'bool' }, 'bool'],
    nandboo: [{ a: 'bool', b: 'bool' }, 'bool'],
    orbool: [{ a: 'bool', b: 'bool' }, 'bool'],
    xorbool: [{ a: 'bool', b: 'bool' }, 'bool'],
    norbool: [{ a: 'bool', b: 'bool' }, 'bool'],
    xnorboo: [{ a: 'bool', b: 'bool' }, 'bool'],

    absi8: [{ a: 'Result<int8>' }, 'Result<int8>'],
    absi16: [{ a: 'Result<int16>' }, 'Result<int16>'],
    absi32: [{ a: 'Result<int32>' }, 'Result<int32>'],
    absi64: [{ a: 'Result<int64>' }, 'Result<int64>'],
    absf32: [{ a: 'Result<float32>' }, 'Result<float32>'],
    absf64: [{ a: 'Result<float64>' }, 'Result<float64>'],

    negi8: [{ a: 'Result<int8>' }, 'Result<int8>'],
    negi16: [{ a: 'Result<int16>' }, 'Result<int16>'],
    negi32: [{ a: 'Result<int32>' }, 'Result<int32>'],
    negi64: [{ a: 'Result<int64>' }, 'Result<int64>'],
    negf32: [{ a: 'Result<float32>' }, 'Result<float32>'],
    negf64: [{ a: 'Result<float64>' }, 'Result<float64>'],

    addi8: [{ a: 'Result<int8>', b: 'Result<int8>' }, 'Result<int8>'],
    addi16: [{ a: 'Result<int16>', b: 'Result<int16>' }, 'Result<int16>'],
    addi32: [{ a: 'Result<int32>', b: 'Result<int32>' }, 'Result<int32>'],
    addi64: [{ a: 'Result<int64>', b: 'Result<int64>' }, 'Result<int64>'],
    addf32: [{ a: 'Result<float32>', b: 'Result<float32>' }, 'Result<float32>'],
    addf64: [{ a: 'Result<float64>', b: 'Result<float64>' }, 'Result<float64>'],

    subi8: [{ a: 'Result<int8>', b: 'Result<int8>' }, 'Result<int8>'],
    subi16: [{ a: 'Result<int16>', b: 'Result<int16>' }, 'Result<int16>'],
    subi32: [{ a: 'Result<int32>', b: 'Result<int32>' }, 'Result<int32>'],
    subi64: [{ a: 'Result<int64>', b: 'Result<int64>' }, 'Result<int64>'],
    subf32: [{ a: 'Result<float32>', b: 'Result<float32>' }, 'Result<float32>'],
    subf64: [{ a: 'Result<float64>', b: 'Result<float64>' }, 'Result<float64>'],

    muli8: [{ a: 'Result<int8>', b: 'Result<int8>' }, 'Result<int8>'],
    muli16: [{ a: 'Result<int16>', b: 'Result<int16>' }, 'Result<int16>'],
    muli32: [{ a: 'Result<int32>', b: 'Result<int32>' }, 'Result<int32>'],
    muli64: [{ a: 'Result<int64>', b: 'Result<int64>' }, 'Result<int64>'],
    mulf32: [{ a: 'Result<float32>', b: 'Result<float32>' }, 'Result<float32>'],
    mulf64: [{ a: 'Result<float64>', b: 'Result<float64>' }, 'Result<float64>'],

    divi8: [{ a: 'Result<int8>', b: 'Result<int8>' }, 'Result<int8>'],
    divi16: [{ a: 'Result<int16>', b: 'Result<int16>' }, 'Result<int16>'],
    divi32: [{ a: 'Result<int32>', b: 'Result<int32>' }, 'Result<int32>'],
    divi64: [{ a: 'Result<int64>', b: 'Result<int64>' }, 'Result<int64>'],
    divf32: [{ a: 'Result<float32>', b: 'Result<float32>' }, 'Result<float32>'],
    divf64: [{ a: 'Result<float64>', b: 'Result<float64>' }, 'Result<float64>'],

    modi8: [{ a: 'int8', b: 'int8' }, 'int8'],
    modi16: [{ a: 'int16', b: 'int16' }, 'int16'],
    modi32: [{ a: 'int32', b: 'int32' }, 'int32'],
    modi64: [{ a: 'int64', b: 'int64' }, 'int64'],
    modf32: [{ a: 'float32', b: 'float32' }, 'float32'],
    modf64: [{ a: 'float64', b: 'float64' }, 'float64'],

    powi8: [{ a: 'Result<int8>', b: 'Result<int8>' }, 'Result<int8>'],
    powi16: [{ a: 'Result<int16>', b: 'Result<int16>' }, 'Result<int16>'],
    powi32: [{ a: 'Result<int32>', b: 'Result<int32>' }, 'Result<int32>'],
    powi64: [{ a: 'Result<int64>', b: 'Result<int64>' }, 'Result<int64>'],
    powf32: [{ a: 'Result<float32>', b: 'Result<float32>' }, 'Result<float32>'],
    powf64: [{ a: 'Result<float64>', b: 'Result<float64>' }, 'Result<float64>'],

    sqrtf32: [{ a: 'Result<float32>', b: 'Result<float32>' }, 'Result<float32>'],
    sqrtf64: [{ a: 'Result<float64>', b: 'Result<float64>' }, 'Result<float64>'],

    saddi8: [{ a: 'int8', b: 'int8' }, 'int8'],
    saddi16: [{ a: 'int16', b: 'int16' }, 'int16'],
    saddi32: [{ a: 'int32', b: 'int32' }, 'int32'],
    saddi64: [{ a: 'int64', b: 'int64' }, 'int64'],
    saddf32: [{ a: 'float32', b: 'float32' }, 'float32'],
    saddf64: [{ a: 'float64', b: 'float64' }, 'float64'],

    ssubi8: [{ a: 'int8', b: 'int8' }, 'int8'],
    ssubi16: [{ a: 'int16', b: 'int16' }, 'int16'],
    ssubi32: [{ a: 'int32', b: 'int32' }, 'int32'],
    ssubi64: [{ a: 'int64', b: 'int64' }, 'int64'],
    ssubf32: [{ a: 'float32', b: 'float32' }, 'float32'],
    ssubf64: [{ a: 'float64', b: 'float64' }, 'float64'],

    smuli8: [{ a: 'int8', b: 'int8' }, 'int8'],
    smuli16: [{ a: 'int16', b: 'int16' }, 'int16'],
    smuli32: [{ a: 'int32', b: 'int32' }, 'int32'],
    smuli64: [{ a: 'int64', b: 'int64' }, 'int64'],
    smulf32: [{ a: 'float32', b: 'float32' }, 'float32'],
    smulf64: [{ a: 'float64', b: 'float64' }, 'float64'],

    sdivi8: [{ a: 'int8', b: 'int8' }, 'int8'],
    sdivi16: [{ a: 'int16', b: 'int16' }, 'int16'],
    sdivi32: [{ a: 'int32', b: 'int32' }, 'int32'],
    sdivi64: [{ a: 'int64', b: 'int64' }, 'int64'],
    sdivf32: [{ a: 'float32', b: 'float32' }, 'float32'],
    sdivf64: [{ a: 'float64', b: 'float64' }, 'float64'],

    spowi8: [{ a: 'int8', b: 'int8' }, 'int8'],
    spowi16: [{ a: 'int16', b: 'int16' }, 'int16'],
    spowi32: [{ a: 'int32', b: 'int32' }, 'int32'],
    spowi64: [{ a: 'int64', b: 'int64' }, 'int64'],
    spowf32: [{ a: 'float32', b: 'float32' }, 'float32'],
    spowf64: [{ a: 'float64', b: 'float64' }, 'float64'],

    someM: [{ a: 'any', s: 'int64' }, 'Maybe<any>'],
    // noneM: [{}, 'Maybe<any>'],
    isSome: [{ a: 'Maybe<any>' }, 'bool'],
    isNone: [{ a: 'Maybe<any>' }, 'bool'],
    getOrM: [{ a: 'Maybe<any>', d: 'any' }, 'any'],
    getMaybe: [{ a: 'Maybe<any>' }, 'any'],

    okR: [{ a: 'any', s: 'int64' }, 'Result<any>'],
    // err: [{ a: 'string' }, 'Result<any>'],
    error: [{ a: 'string' }, 'Error'],
    isOk: [{ a: 'Result<any>' }, 'bool'],
    isErr: [{ a: 'Result<any>' }, 'bool'],
    getOrR: [{ a: 'Result<any>', d: 'any' }, 'any'],
    getR: [{ a: 'Result<any>' }, 'any'],
    getErr: [{ a: 'Result<any>', b: 'Error' }, 'Error'],
    noerr: [{}, 'Error'],
    errstr: [{ a: 'Error' }, 'string'],

    waitop: [{ t: 'int64' }, 'void'],

    catstr: [{ a: 'string', b: 'string' }, 'string'],
    indstr: [{ a: 'string', b: 'string' }, 'Maybe<int64>'],
    repstr: [{ s: 'string', n: 'int64' }, 'string'],
    matches: [{ a: 'string', b: 'string' }, 'bool'],
    lenstr: [{ s: 'string' }, 'int64'],
    trim: [{ s: 'string' }, 'string'],

    copyi8: [{ a: 'int8' }, 'int8'],
    copyi16: [{ a: 'int16' }, 'int16'],
    copyi32: [{ a: 'int32' }, 'int32'],
    copyi64: [{ a: 'int64' }, 'int64'],
    copyf32: [{ a: 'float32' }, 'float32'],
    copyf64: [{ a: 'float64' }, 'float64'],
    copybool: [{ a: 'bool' }, 'bool'],
    copystr: [{ a: 'string' }, 'string'],

    stdoutp: [{ out: 'string' }, 'void'],
    stderrp: [{ err: 'string' }, 'void'],
    exitop: [{ status: 'int8' }, 'void'],
  } as {
    [opcode: string]: [{ [arg: string]: string }, string];
    // Opcode constructor inserts into the opcode scope for us
  }).forEach(([name, [args, ret]]) => {
    new OpcodeFn(name, args, ret, __opcodes);
  });
};
