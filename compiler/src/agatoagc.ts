import {
  LP,
  LPNode,
  LPError,
  NamedAnd,
} from './lp'

import aga from './aga'

// This project depends on BigNum and associated support in Node's Buffer, so must be >= Node 10.20
// and does not work in the browser. It would be possible to implement a browser-compatible version
// but there is no need for it and it would make it harder to work with.
const agcHeader = Buffer.from('agc00001', 'utf8').readBigUInt64LE(0)
const eventdd   = Buffer.from('eventdd:', 'utf8').readBigUInt64LE(0)
const handlerd  = Buffer.from('handler:', 'utf8').readBigUInt64LE(0)
const lineno    = Buffer.from('lineno: ', 'utf8').readBigUInt64LE(0)

const ceil8 = (n: number) => Math.ceil(n / 8) * 8
const int64ToUint64 = (n: bigint): bigint => {
  const buf = Buffer.alloc(8)
  buf.writeBigInt64LE(n, 0)
  return buf.readBigUInt64LE(0)
}

const loadGlobalMem = (globalMemAst: LPNode): bigint[] => {
  const globalMem = []
  const memoryLines = globalMemAst.get('memoryLines')
  for (const globalConst of memoryLines.getAll()) {
    const memoryLine = globalConst.get('memoryLine')
    const value = memoryLine.get('value')
    if (value.has('i64')) {
      const val = BigInt(value.t.replace(/i64$/, ''))
      globalMem.push(val)
    } else if (value.has('i32')) {
      const val = BigInt(value.t.replace(/i32$/, ''))
      globalMem.push(val)
    } else if (value.has('i16')) {
      const val = BigInt(value.t.replace(/i16$/, ''))
      globalMem.push(val)
    } else if (value.has('i8')) {
      const val = BigInt(value.t.replace(/i8$/, ''))
      globalMem.push(val)
    } else if (value.has('f32')) {
      const buf = Buffer.alloc(8)
      buf.writeFloatLE(parseFloat(value.t.replace(/f32$/, '')))
      const val = buf.readBigUInt64LE(0)
      globalMem.push(val)
    } else if (value.has('f64')) {
      const buf = Buffer.alloc(8)
      buf.writeDoubleLE(parseFloat(value.t.replace(/f64$/, '')))
      const val = buf.readBigUInt64LE(0)
      globalMem.push(val)
    } else if (value.has('str')) {
      let str: string
      try {
        str = JSON.parse(value.t) // Will fail on strings with escape chars
      } catch (e) {
        // Hackery to get these strings to work
        str = JSON.stringify(value.t.replace(/^["']/, '').replace(/["']$/, ''))
      }
      let len = BigInt(ceil8(str.length) + 8)
      const buf = Buffer.alloc(Number(len))
      buf.writeBigInt64LE(BigInt(str.length), 0)
      for (let i = 8; i < str.length + 8; i++) {
        buf.writeInt8(str.charCodeAt(i - 8), i)
      }
      for (let i = 0; i < Number(len) / 8; i++) {
        globalMem.push(buf.readBigUInt64LE(i * 8))
      }
    } else if (value.has('bool')) {
      const val = value.t === "true" ? BigInt(1) : BigInt(0)
      globalMem.push(val)
    } else {
      throw new Error(`Strange AST parsing error, this should be unreachable: ${value}`)
    }
  }
  return globalMem
}

const loadEventDecs = (eventAst: LPNode, eventLookup: Object): bigint[] => {
  const eventLines = eventAst.get('eventLines')
  let customEventIdOffset = BigInt(0)
  const eventMem = []
  for (const evt of eventLines.getAll()) {
    const eventLine = evt.get('eventLine')
    const evtName = eventLine.get('variable').t
    const evtSize = int64ToUint64(BigInt(eventLine.get('integer').t))
    eventMem.push(eventdd, customEventIdOffset, evtSize)
    eventLookup[evtName] = {
      eventId: customEventIdOffset,
    }
    customEventIdOffset++
  }
  return eventMem
}

const fill8 = (name: string) => {
  const buf = Buffer.alloc(8, ' '.charCodeAt(0))
  for (let i = 0; i < name.length; i++) {
    buf.writeInt8(name.charCodeAt(i), i)
  }
  return buf.readBigUInt64LE(0)
}

const loadStatements = (statements: LPNode, eventLookup: Object): bigint[] => {
  let vec = []
  for (const statementAst of statements.getAll()) {
    const statement = statementAst.get('statement')
    const line = BigInt(statement.get('line').get(1).t)
    const dependsOn = statement.get('dependsOn')
    const deps = dependsOn.get('deps').getAll().map(d => BigInt(d.get('line').get(1).t))
    const fn = fill8(statement.get('variable').t)
    const args = statement.get('args').getAll().map(a => {
      const argOpt = a.get('arg')
      let out: bigint
      if (argOpt.has('variable')) {
        out = eventLookup[argOpt.get('variable').t].eventId
      } else if (argOpt.has('memoryAddress')) {
        out = int64ToUint64(BigInt((argOpt.get('memoryAddress').get(1).t)))
      } else if (argOpt.has('i64')) {
        out = BigInt(argOpt.t.replace(/i64$/, ''))
      } else if (argOpt.has('i8')) {
        out = BigInt(argOpt.t.replace(/i8$/, ''))
      } else if (argOpt.has('f64')) {
        const buf = Buffer.alloc(8)
        buf.writeDoubleLE(parseFloat(argOpt.t.replace(/f64$/, '')))
        out = buf.readBigUInt64LE(0)
      }
      return out
    })
    if (args.length < 3) {
      const resultAddress = statement.get('result').t === '' ?
        BigInt(0) :
        BigInt(statement.get('result').get('memoryAddress').get(1).t)
      args.push(resultAddress)
    }
    vec.push(lineno, line, BigInt(deps.length), ...deps, fn, ...args)
  }
  return vec
}

const loadHandlers = (handlersAst: LPNode, eventLookup: Object): bigint[] => {
  const handlers = handlersAst.getAll()
  const vec = []
  for (let i = 0; i < handlers.length; i++) {
    const handler = handlers[i]
    const handlerHead = handler.get('handlerLine')
    const { eventId, } = eventLookup[handlerHead.get('variable').t]
    const memSize = BigInt(handlerHead.get('integer').t)
    vec.push(handlerd, eventId, memSize)
    const statementVec = loadStatements(
      handler.get('statements'),
      eventLookup,
    )
    vec.push(...statementVec)
  }
  return vec
}

const astToAgc = (ast: NamedAnd): Buffer => {
  // Declare the AGC header
  const vec: bigint[] = [agcHeader]
  if (ast.has('globalMemory')) {
    const globalMemoryAst = ast.get('globalMemory')
    // Get the global memory
    const globalMem = loadGlobalMem(globalMemoryAst)
    // Compute the global memory size and declare that and add all of the global memory
    const memSize = BigInt(globalMem.length * 8)
    vec.push(memSize, ...globalMem)
  } else {
    vec.push(BigInt(0))
  }
  // Declare the event lookup table (event string to id) with the singular special event `"start"`
  const eventLookup = {
    _start: {
      eventId: (() => {
        const buf = Buffer.from('"start" ', 'utf8')
        buf.writeUInt8(0x80, 7)
        return buf.readBigUInt64LE(0)
      })(),
    },
    __conn: {
      eventId: (() => {
        const buf = Buffer.from('__conn  ', 'utf8')
        buf.writeUInt8(0x80, 7)
        return buf.readBigUInt64LE(0)
      })(),
    },
    __ctrl: {
      eventId: (() => {
        const buf = Buffer.from('__ctrl  ', 'utf8')
        buf.writeUInt8(0x80, 7)
        return buf.readBigUInt64LE(0)
      })(),
    },
  }
  // Load the events, get the event id offset (for reuse with closures) and the event declarations
  const customEvents = ast.get('customEvents')
  const eventDecs = loadEventDecs(customEvents, eventLookup)
  // Then add that to the output vector
  vec.push(...eventDecs)
  // Load the handlers
  const handlers = ast.get('handlers')
  const handlerVec = loadHandlers(handlers, eventLookup)
  vec.push(...handlerVec)
  // All done, convert the BigInt array to a big buffer to write to a file
  const outBuf = Buffer.alloc(vec.length * 8)
  vec.forEach((n, i) => {
    outBuf.writeBigUInt64LE(n < 0 ? int64ToUint64(n) : n, i * 8)
  })
  return outBuf
}

export const fromFile = (filename: string): Buffer => {
  const lp = new LP(filename)
  const ast = aga.apply(lp)
  if (ast instanceof LPError) {
    throw new Error(ast.msg)
  }
  return astToAgc(ast)
}

export const fromString = (str: string): Buffer => {
  const lp = LP.fromText(str)
  const ast = aga.apply(lp)
  if (ast instanceof LPError) {
    throw new Error(ast.msg)
  }
  return astToAgc(ast)
}
