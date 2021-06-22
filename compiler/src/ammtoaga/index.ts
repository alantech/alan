import { LP, LPNode, LPError, NamedAnd, NulLP } from '../lp';

import amm from '../amm';
import { DepGraph } from './depgraph';
import { Block, Statement } from './aga';

type AddressMap = {
  [name: string]: bigint | number;
};
type EventDecs = {
  [name: string]: number;
};
type MemoryMap = {
  [name: string]: number;
};
type HandlerMem = {
  memSize: number;
  addressMap: MemoryMap;
};

// This project depends on BigNum and associated support in Node's Buffer, so must be >= Node 10.20
// and does not work in the browser. It would be possible to implement a browser-compatible version
// but there is no need for it and it would make it harder to work with.
const ceil8 = (n: number) => Math.ceil(n / 8) * 8;
const CLOSURE_ARG_MEM_START = BigInt(Math.pow(-2, 63));

// special closure that does nothing
const NOP_CLOSURE = BigInt('-9223372036854775808');

const loadGlobalMem = (globalMemAst: LPNode[], addressMap: AddressMap) => {
  const suffixes = {
    int64: 'i64',
    int32: 'i32',
    int16: 'i16',
    int8: 'i8',
    float64: 'f64',
    float32: 'f32',
  };

  const globalMem = {};
  let currentOffset = -1;
  for (const globalConst of globalMemAst) {
    const rec = globalConst.get();
    if (!(rec instanceof NamedAnd)) continue;
    let offset = 8;
    let val = rec.get('assignables').t.trim();
    const typename = rec.get('fulltypename').t.trim();
    if (typename === 'string') {
      let str: string;
      try {
        // Will fail on strings with escape chars
        str = JSON.parse(val);
      } catch (e) {
        // Hackery to get these strings to work
        str = JSON.stringify(val.replace(/^["']/, '').replace(/["']$/, ''));
      }
      offset = ceil8(str.length) + 8;
    } else if (suffixes.hasOwnProperty(typename)) {
      val += suffixes[typename];
    } else if (typename !== 'bool') {
      throw new Error(rec.get('fulltypename').t + ' not yet implemented');
    }
    globalMem[`@${currentOffset}`] = val;
    addressMap[rec.get('decname').t] = currentOffset;
    currentOffset -= offset;
  }
  return globalMem;
};

const loadEventDecs = (eventAst: LPNode[]) => {
  const eventMem = {};
  for (const evt of eventAst) {
    const rec = evt.get();
    if (!(rec instanceof NamedAnd)) continue;
    const evtName = rec.get('variable').t.trim();
    const evtSize =
      rec.get('fulltypename').t.trim() === 'void'
        ? 0
        : [
            'int8',
            'int16',
            'int32',
            'int64',
            'float32',
            'float64',
            'bool',
          ].includes(rec.get('fulltypename').t.trim())
        ? 8
        : -1;
    eventMem[evtName] = evtSize;
  }
  return eventMem;
};

const getFunctionbodyMem = (functionbody: LPNode) => {
  let memSize = 0;
  const addressMap = {};
  for (const statement of functionbody.get('statements').getAll()) {
    if (statement.has('declarations')) {
      if (statement.get('declarations').has('constdeclaration')) {
        if (
          statement
            .get('declarations')
            .get('constdeclaration')
            .get('assignables')
            .has('functions')
        ) {
          // Because closures re-use their parent memory space, their own memory needs to be included
          const closureMem = getFunctionbodyMem(
            statement
              .get('declarations')
              .get('constdeclaration')
              .get('assignables')
              .get('functions')
              .get('functionbody'),
          );
          Object.keys(closureMem.addressMap).forEach(
            (name) =>
              (addressMap[name] = closureMem.addressMap[name] + memSize),
          );
          memSize += closureMem.memSize;
        } else {
          addressMap[
            statement
              .get('declarations')
              .get('constdeclaration')
              .get('decname')
              .t.trim()
          ] = memSize;
          memSize += 1;
        }
      } else {
        addressMap[
          statement
            .get('declarations')
            .get('letdeclaration')
            .get('decname')
            .t.trim()
        ] = memSize;
        memSize += 1;
      }
    }
  }
  return {
    memSize,
    addressMap,
  };
};

const getHandlersMem = (handlers: LPNode[]) =>
  handlers
    .map((h) => h.get())
    .filter((h) => h instanceof NamedAnd)
    .map((handler) => {
      const handlerMem = getFunctionbodyMem(
        handler.get('functions').get('functionbody'),
      );
      let arg = handler.get('functions').get('args').get(0).get(0).get('arg');
      if (arg instanceof NulLP) {
        arg = handler.get('functions').get('args').get(1).get('arg');
      }
      if (!(arg instanceof NulLP)) {
        // Increase the memory usage and shift *everything* down, then add the new address
        handlerMem.memSize += 1;
        Object.keys(handlerMem.addressMap).forEach(
          (name) => (handlerMem.addressMap[name] += 1),
        );
        handlerMem.addressMap[arg.get('variable').t.trim()] = 0;
      }
      return handlerMem;
    });

const closuresFromDeclaration = (
  declaration: LPNode,
  closureMem: HandlerMem,
  eventDecs: EventDecs,
  addressMap: AddressMap,
  // For each scope branch, determine a unique argument rereference so nested scopes can access
  // parent scope arguments
  argRerefOffset: number,
  scope: string[],
  depGraph: DepGraph,
) => {
  const name = declaration.get('constdeclaration').get('decname').t.trim();
  if (
    depGraph.byVar[name] === null ||
    depGraph.byVar[name] === undefined ||
    depGraph.byVar[name].length === 0 ||
    depGraph.byVar[name][0].closure === null
  ) {
    throw new Error(
      'trying to build a closure, but the dependency graph did not build a closure',
    );
  }
  const graph = depGraph.byVar[name][0].closure;
  const fn = declaration
    .get('constdeclaration')
    .get('assignables')
    .get('functions');
  let fnArgs = [];
  fn.get('args')
    .getAll()[0]
    .getAll()
    .forEach((argdef) => {
      fnArgs.push(argdef.get('arg').get('variable').t);
    });
  if (fn.get('args').getAll()[1].has()) {
    fnArgs.push(
      ...fn
        .get('args')
        .getAll()[1]
        .getAll()
        .map((t) => t.get('variable').t),
    );
    fnArgs = fnArgs.filter((t) => t !== '');
  }
  fnArgs.forEach((arg) => {
    addressMap[arg + name] = CLOSURE_ARG_MEM_START + BigInt(argRerefOffset);
    argRerefOffset++;
  });
  const allStatements = declaration
    .get('constdeclaration')
    .get('assignables')
    .get('functions')
    .get('functionbody')
    .get('statements')
    .getAll();
  const statements = allStatements.filter(
    (statement) =>
      !(
        statement.has('declarations') &&
        statement.get('declarations').has('constdeclaration') &&
        statement
          .get('declarations')
          .get('constdeclaration')
          .get('assignables')
          .has('functions')
      ),
  );
  const otherClosures = allStatements
    .filter(
      (statement) =>
        statement.has('declarations') &&
        statement.get('declarations').has('constdeclaration') &&
        statement
          .get('declarations')
          .get('constdeclaration')
          .get('assignables')
          .has('functions'),
    )
    .map((s) =>
      closuresFromDeclaration(
        s.get('declarations'),
        closureMem,
        eventDecs,
        addressMap,
        argRerefOffset,
        [name, ...scope], // Newest scope gets highest priority
        graph,
      ),
    )
    .filter((clos) => clos !== null)
    .reduce(
      (obj, rec) => ({
        ...obj,
        ...rec,
      }),
      {},
    );
  eventDecs[name] = 0;

  if (!graph.isNop) {
    otherClosures[name] = {
      name,
      fn,
      statements,
      closureMem,
      scope: [name, ...scope],
      graph,
    };
  } else {
    addressMap[name] = NOP_CLOSURE;
  }

  return otherClosures;
};

const extractClosures = (
  handlers: LPNode[],
  handlerMem: HandlerMem[],
  eventDecs: EventDecs,
  addressMap: AddressMap,
  depGraphs: DepGraph[],
) => {
  let closures = {};
  const recs = handlers.filter((h) => h.get() instanceof NamedAnd);
  for (let i = 0; i < recs.length; i++) {
    const rec = recs[i].get();
    const closureMem = handlerMem[i];
    const handlerGraph = depGraphs[i];
    for (const statement of rec
      .get('functions')
      .get('functionbody')
      .get('statements')
      .getAll()) {
      if (
        statement.has('declarations') &&
        statement.get('declarations').has('constdeclaration') &&
        statement
          .get('declarations')
          .get('constdeclaration')
          .get('assignables')
          .has('functions')
      ) {
        // It's a closure, first try to extract any inner closures it may have
        const innerClosures = closuresFromDeclaration(
          statement.get('declarations'),
          closureMem,
          eventDecs,
          addressMap,
          5,
          [],
          handlerGraph,
        );
        closures = {
          ...closures,
          ...innerClosures,
        };
      }
    }
  }
  return Object.values(closures);
};

const loadStatements = (
  statements: LPNode[],
  localMem: MemoryMap,
  globalMem: MemoryMap,
  fn: LPNode,
  fnName: string,
  isClosure: boolean,
  closureScope: string[],
  depGraph: DepGraph,
) => {
  const vec = [];
  let line = 0;
  const localMemToLine = {};
  statements = statements.filter((s) => !s.has('whitespace'));
  let fnArgs = [];
  fn.get('args')
    .getAll()[0]
    .getAll()
    .forEach((argdef) => {
      fnArgs.push(argdef.get('arg').get('variable').t);
    });
  if (fn.get('args').getAll()[1].has()) {
    fnArgs.push(
      ...fn
        .get('args')
        .getAll()[1]
        .getAll()
        .map((t) => t.get('variable').t),
    );
    fnArgs = fnArgs.filter((t) => t !== '');
  }
  fnArgs.forEach((arg, i) => {
    if (globalMem.hasOwnProperty(arg + fnName)) {
      const resultAddress = globalMem[arg + fnName];
      const val = CLOSURE_ARG_MEM_START + BigInt(1) + BigInt(i);
      const s = new Statement(
        'refv',
        [`@${val}`, '@0'],
        `@${resultAddress}`,
        line,
        [],
        depGraph.params[arg] || null,
      );
      vec.push(s);
      line += 1;
    }
  });
  for (let idx = 0; idx < statements.length; idx++) {
    const statement = statements[idx];
    if (
      statement.has('declarations') &&
      statement.get('declarations').has('constdeclaration') &&
      statement
        .get('declarations')
        .get('constdeclaration')
        .get('assignables')
        .has('functions')
    ) {
      // It's a closure, skip it
      continue;
    }
    const node = depGraph.byLP.get(statement);
    const hasClosureArgs = isClosure && fnArgs.length > 0;
    let s: Statement;
    if (statement.has('declarations')) {
      const dec = statement.get('declarations').has('constdeclaration')
        ? statement.get('declarations').get('constdeclaration')
        : statement.get('declarations').get('letdeclaration');
      const resultAddress = localMem[dec.get('decname').t.trim()];
      localMemToLine[dec.get('decname').t.trim()] = line;
      const assignables = dec.get('assignables');
      if (assignables.has('functions')) {
        throw new Error("This shouldn't be possible!");
      } else if (assignables.has('calls')) {
        const call = assignables.get('calls');
        const fnName = call.get('variable').t.trim();
        const vars = (
          call.has('calllist') ? call.get('calllist').getAll() : []
        ).map((v) => v.get('variable').t.trim());
        const args = vars
          .map((v) => {
            if (localMem.hasOwnProperty(v)) {
              return localMem[v];
            } else if (globalMem.hasOwnProperty(v)) {
              return globalMem[v];
            } else if (
              Object.keys(globalMem).some((k) =>
                closureScope.map((s) => v + s).includes(k),
              )
            ) {
              return globalMem[
                closureScope
                  .map((s) => v + s)
                  .find((k) => Object.keys(globalMem).includes(k))
              ];
            } else if (hasClosureArgs) {
              return (
                CLOSURE_ARG_MEM_START + BigInt(1) + BigInt(fnArgs.indexOf(v))
              );
            } else {
              return v;
            }
          })
          .map((a) => (typeof a === 'string' ? a : `@${a}`));
        while (args.length < 2) args.push('@0');
        s = new Statement(
          fnName,
          args as [string, string],
          `@${resultAddress}`,
          line,
          [],
          node,
        );
      } else if (assignables.has('value')) {
        // Only required for `let` statements
        let fn: string;
        let val: string;
        switch (dec.get('fulltypename').t.trim()) {
          case 'int64':
            fn = 'seti64';
            val = assignables.t + 'i64';
            break;
          case 'int32':
            fn = 'seti32';
            val = assignables.t + 'i32';
            break;
          case 'int16':
            fn = 'seti16';
            val = assignables.t + 'i16';
            break;
          case 'int8':
            fn = 'seti8';
            val = assignables.t + 'i8';
            break;
          case 'float64':
            fn = 'setf64';
            val = assignables.t + 'f64';
            break;
          case 'float32':
            fn = 'setf32';
            val = assignables.t + 'f32';
            break;
          case 'bool':
            fn = 'setbool';
            val = assignables.t === 'true' ? '1i8' : '0i8'; // Bools are bytes in the runtime
            break;
          case 'string':
            fn = 'setestr';
            val = '0i64';
            break;
          default:
            throw new Error(
              `Unsupported variable type ${dec.get('fulltypename').t}`,
            );
        }
        s = new Statement(fn, [val, '@0'], `@${resultAddress}`, line, [], node);
      } else if (assignables.has('variable')) {
        throw new Error('This should have been squashed');
      }
    } else if (statement.has('assignments')) {
      const asgn = statement.get('assignments');
      const resultAddress = localMem[asgn.get('decname').t.trim()];
      localMemToLine[resultAddress] = line;
      const assignables = asgn.get('assignables');
      if (assignables.has('functions')) {
        throw new Error("This shouldn't be possible!");
      } else if (assignables.has('calls')) {
        const call = assignables.get('calls');
        const fnName = call.get('variable').t.trim();
        const vars = (
          call.has('calllist') ? call.get('calllist').getAll() : []
        ).map((v) => v.get('variable').t.trim());
        const hasClosureArgs = isClosure && vars.length > 0;
        const args = vars
          .map((v) => {
            if (localMem.hasOwnProperty(v)) {
              return localMem[v];
            } else if (globalMem.hasOwnProperty(v)) {
              return globalMem[v];
            } else if (
              Object.keys(globalMem).some((k) =>
                closureScope.map((s) => v + s).includes(k),
              )
            ) {
              return globalMem[
                closureScope
                  .map((s) => v + s)
                  .find((k) => Object.keys(globalMem).includes(k))
              ];
            } else if (hasClosureArgs) {
              return (
                CLOSURE_ARG_MEM_START + BigInt(1) + BigInt(fnArgs.indexOf(v))
              );
            } else return v;
          })
          .map((a) => (typeof a === 'string' ? a : `@${a}`));
        while (args.length < 2) args.push('@0');
        s = new Statement(
          fnName,
          args as [string, string],
          `@${resultAddress}`,
          line,
          [],
          node,
        );
      } else if (assignables.has('value')) {
        // Only required for `let` statements
        let fn: string;
        let val: string;
        // TODO: Relying on little-endian trimming integers correctly and doesn't support float32
        // correctly. Need to find the correct type data from the original variable.
        const valStr = assignables.t;
        if (valStr[0] === '"' || valStr[0] === "'") {
          // It's a string, which doesn't work here...
          fn = 'setestr';
          val = '0i64';
        } else if (valStr[0] === 't' || valStr[0] === 'f') {
          // It's a bool
          fn = 'setbool';
          val = assignables.t === 'true' ? '1i8' : '0i8'; // Bools are bytes in the runtime
        } else if (valStr.indexOf('.') > -1) {
          // It's a floating point number, assume 64-bit
          fn = 'setf64';
          val = valStr + 'f64';
        } else {
          // It's an integer. i64 will "work" for now
          fn = 'seti64';
          val = valStr + 'i64';
        }
        s = new Statement(fn, [val, '@0'], `@${resultAddress}`, line, [], node);
      } else if (assignables.has('variable')) {
        throw new Error('This should have been squashed');
      }
    } else if (statement.has('calls')) {
      const call = statement.get('calls');
      const fnName = call.get('variable').t.trim();
      const vars = (
        call.has('calllist') ? call.get('calllist').getAll() : []
      ).map((v) => v.get('variable').t.trim());
      const hasClosureArgs = isClosure && vars.length > 0;
      const args = vars
        .map((v) => {
          if (localMem.hasOwnProperty(v)) {
            return localMem[v];
          } else if (globalMem.hasOwnProperty(v)) {
            return globalMem[v];
          } else if (
            Object.keys(globalMem).some((k) =>
              closureScope.map((s) => v + s).includes(k),
            )
          ) {
            return globalMem[
              closureScope
                .map((s) => v + s)
                .find((k) => Object.keys(globalMem).includes(k))
            ];
          } else if (hasClosureArgs) {
            return (
              CLOSURE_ARG_MEM_START + BigInt(1) + BigInt(fnArgs.indexOf(v))
            );
          } else return v;
        })
        .map((a) => (typeof a === 'string' ? a : `@${a}`));
      while (args.length < 3) args.push('@0');
      s = new Statement(
        fnName,
        args as [string, string, string],
        null,
        line,
        [],
        node,
      );
    } else if (statement.has('emits')) {
      const emit = statement.get('emits');
      const evtName = emit.get('variable').t.trim();
      const payloadVar = emit.has('value')
        ? emit.get('value').t.trim()
        : undefined;
      const payload = !payloadVar
        ? 0
        : localMem.hasOwnProperty(payloadVar)
        ? localMem[payloadVar]
        : globalMem.hasOwnProperty(payloadVar)
        ? globalMem[payloadVar]
        : payloadVar;
      s = new Statement(
        'emit',
        [evtName, typeof payload === 'string' ? payload : `@${payload}`],
        null,
        line,
        [],
        node,
      );
    } else if (statement.has('exits')) {
      const exit = statement.get('exits');
      const exitVar = exit.get('variable').t.trim();
      const exitVarType = localMem.hasOwnProperty(exitVar)
        ? 'variable'
        : globalMem.hasOwnProperty(exitVar) &&
          typeof globalMem[exitVar] !== 'string'
        ? 'fixed'
        : 'variable';
      const vars = [exitVar];
      const args = vars
        .map((v) => {
          if (localMem.hasOwnProperty(v)) {
            return localMem[v];
          } else if (globalMem.hasOwnProperty(v)) {
            return globalMem[v];
          } else if (
            Object.keys(globalMem).some((k) =>
              closureScope.map((s) => v + s).includes(k),
            )
          ) {
            return globalMem[
              closureScope
                .map((s) => v + s)
                .find((k) => Object.keys(globalMem).includes(k))
            ];
          } else if (hasClosureArgs) {
            return (
              CLOSURE_ARG_MEM_START + BigInt(1) + BigInt(fnArgs.indexOf(v))
            );
          } else return v;
        })
        .map((a) => (typeof a === 'string' ? a : `@${a}`));
      while (args.length < 2) args.push('@0');
      const ref = exitVarType === 'variable' ? 'refv' : 'reff';
      s = new Statement(
        ref,
        args as [string, string],
        `@${CLOSURE_ARG_MEM_START}`,
        line,
        [],
        node,
      );
    }
    vec.push(s);
    line += 1;
  }
  return vec;
};

const loadHandlers = (
  handlers: LPNode[],
  handlerMem: HandlerMem[],
  globalMem: MemoryMap,
  depGraphs: DepGraph[],
) => {
  const vec = [];
  const recs = handlers.filter((h) => h.get() instanceof NamedAnd);
  for (let i = 0; i < recs.length; i++) {
    const handler = recs[i].get();
    const eventName = handler.get('variable').t.trim();
    const memSize = handlerMem[i].memSize;
    const localMem = handlerMem[i].addressMap;
    const h = new Block(
      'handler',
      eventName,
      memSize,
      loadStatements(
        handler.get('functions').get('functionbody').get('statements').getAll(),
        localMem,
        globalMem,
        handler.get('functions'),
        eventName,
        false,
        [],
        depGraphs[i],
      ),
      [],
    );
    vec.push(h);
  }
  return vec;
};

const loadClosures = (closures: any[], globalMem: MemoryMap) => {
  const vec = [];
  for (let i = 0; i < closures.length; i++) {
    const closure = closures[i];
    const eventName = closure.name;
    const memSize = closure.closureMem.memSize;
    const localMem = closure.closureMem.addressMap;
    const c = new Block(
      'closure',
      eventName,
      memSize,
      loadStatements(
        closure.statements,
        localMem,
        globalMem,
        closure.fn,
        eventName,
        true,
        closure.scope,
        closure.graph,
      ),
      [],
    );
    vec.push(c);
  }
  return vec;
};

const ammToAga = (amm: LPNode) => {
  // Declare the AGA header
  let outStr = 'Alan Graphcode Assembler v0.0.1\n\n';
  // Get the global memory and the memory address map (var name to address ID)
  const addressMap = {};
  const globalMem = loadGlobalMem(amm.get('globalMem').getAll(), addressMap);
  if (Object.keys(globalMem).length > 0) {
    // Output the global memory
    outStr += 'globalMem\n';
    Object.keys(globalMem).forEach(
      (addr) => (outStr += `  ${addr}: ${globalMem[addr]}\n`),
    );
    outStr += '\n';
  }
  // Load the events, get the event id offset (for reuse with closures) and the event declarations
  const eventDecs = loadEventDecs(amm.get('eventDec').getAll());
  // Determine the amount of memory to allocate per handler and map declarations to addresses
  const handlerMem = getHandlersMem(amm.get('handlers').getAll());
  const depGraphs: DepGraph[] = [];
  for (let handler of amm.get('handlers').getAll()) {
    handler = handler.get();
    if (handler instanceof NamedAnd) {
      depGraphs.push(new DepGraph(handler));
    }
  }
  // console.log(depGraphs.map(g => JSON.stringify(g.toJSON())).join(','))
  const closures = extractClosures(
    amm.get('handlers').getAll(),
    handlerMem,
    eventDecs,
    addressMap,
    depGraphs,
  );
  // Make sure closures are accessible as addresses for statements to use
  closures.forEach((c: any) => {
    if (addressMap[c.name] !== NOP_CLOSURE) {
      addressMap[c.name] = c.name;
    }
  });
  // Then output the custom events, which may include closures, if needed
  if (Object.keys(eventDecs).length > 0) {
    outStr += 'customEvents\n';
    Object.keys(eventDecs).forEach(
      (evt) => (outStr += `  ${evt}: ${eventDecs[evt]}\n`),
    );
    outStr += '\n';
  }
  // Load the handlers and load the closures (as handlers) if present
  const handlerVec = loadHandlers(
    amm.get('handlers').getAll(),
    handlerMem,
    addressMap,
    depGraphs,
  );
  const closureVec = loadClosures(closures, addressMap);
  [...handlerVec, ...closureVec].map((b) => b.build());
  // console.log(([...handlerVec, ...closureVec]).map(b => b.build()).join(','))
  const blockVec = [...handlerVec, ...closureVec].map((b) => b.toString());
  outStr += blockVec.join('\n');
  return outStr;
};

export const fromFile = (filename: string) => {
  const lp = new LP(filename);
  const ast = amm.apply(lp);
  if (ast instanceof LPError) {
    throw new Error(ast.msg);
  }
  return ammToAga(ast);
};
export const fromString = (str: string) => {
  const lp = LP.fromText(str);
  const ast = amm.apply(lp);
  if (ast instanceof LPError) {
    throw new Error(ast.msg);
  }
  return ammToAga(ast);
};
