import {
  And,
  CharSet,
  LP,
  LPish,
  NamedAnd,
  NamedOr,
  Not,
  NulLP,
  OneOrMore,
  Or,
  Token,
  ZeroOrMore,
  ZeroOrOne,
} from '../lp'

// Defining AGA Tokens
const space = Token.build(' ')
const newline = Token.build('\n')
const whitespace = ZeroOrMore.build(Or.build([space, newline]))
const at = Token.build('@')
const colon = Token.build(':')
const sharp = Token.build('#')
const under = Token.build('_')
const negate = Token.build('-')
const dot = Token.build('.')
const eq = Token.build('=')
const openParen = Token.build('(')
const closeParen = Token.build(')')
const backArrow = Token.build('<-')
const openBracket = Token.build('[')
const closeBracket = Token.build(']')
const comma = Token.build(',')
const base10 = CharSet.build('0', '9')
const natural = OneOrMore.build(base10)
const integer = And.build([ZeroOrOne.build(negate), natural])
const real = And.build([integer, ZeroOrOne.build(And.build([dot, natural]))])
const i64 = And.build([integer, Token.build('i64')])
const f64 = And.build([real, Token.build('f64')])
const lower = CharSet.build('a', 'z')
const upper = CharSet.build('A', 'Z')
const variable = And.build([
  OneOrMore.build(Or.build([under, lower, upper])),
  ZeroOrMore.build(Or.build([under, lower, upper, natural])),
])
const t = Token.build('true')
const f = Token.build('false')
const bool = Or.build([t, f])
const quote = Token.build('"')
const escapeQuote = Token.build('\\"')
const notQuote = Not.build('"')
const str = And.build([quote, ZeroOrMore.build(Or.build([escapeQuote, notQuote])), quote])
const value = NamedOr.build({ str, bool, i64, f64 })
const header = Token.build('Alan Graphcode Assembler v0.0.1')
const globalMem = Token.build('globalMem')
const memoryAddress = And.build([at, integer])
const memoryLine = NamedAnd.build({ memoryAddress, colon, whitespace, value })
const customEvents = Token.build('customEvents')
const eventLine = NamedAnd.build({ variable, colon, whitespace, integer })
const handlerFor = Token.build('handler for')
const withSize = Token.build('with size')
const handlerLine = NamedAnd.build({
  handlerFor,
  a: whitespace,
  variable,
  b: whitespace,
  withSize,
  c: whitespace,
  integer,
})
const arg = NamedOr.build({ variable, memoryAddress, i64, f64 })
const sep = And.build([comma, whitespace])
const args = ZeroOrMore.build(NamedAnd.build({ arg, sep: ZeroOrOne.build(sep), }))
const line = And.build([sharp, natural])
const deps = OneOrMore.build(NamedAnd.build({ line, sep: ZeroOrOne.build(sep), }))
const statement = NamedAnd.build({
  result: ZeroOrOne.build(NamedAnd.build({ memoryAddress, a: whitespace, eq, b: whitespace, })),
  variable,
  a: whitespace,
  openParen,
  args,
  closeParen,
  b: whitespace,
  line,
  dependsOn: ZeroOrOne.build(NamedAnd.build({
    a: whitespace,
    backArrow,
    b: whitespace,
    openBracket,
    deps,
    closeBracket,
  })),
})
const memory = NamedAnd.build({
  globalMem,
  memoryLines: OneOrMore.build(NamedAnd.build({ a: whitespace, memoryLine, b: whitespace, })),
})
const events = NamedAnd.build({
  customEvents,
  eventLines: OneOrMore.build(NamedAnd.build({ a: whitespace, eventLine, b: whitespace, })),
})
const handler = NamedAnd.build({
  handlerLine,
  statements: OneOrMore.build(NamedAnd.build({ a: whitespace, statement, b: whitespace, })),
  whitespace,
})
const aga = NamedAnd.build({
  header,
  a: whitespace,
  globalMemory: ZeroOrOne.build(memory),
  b: whitespace,
  customEvents: ZeroOrOne.build(events),
  c: whitespace,
  handlers: OneOrMore.build(handler),
})

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

const loadGlobalMem = (globalMemAst: LPish): bigint[] => {
  const globalMem = []
  const memoryLines = globalMemAst.get('memoryLines')
  for (const globalConst of memoryLines.getAll()) {
    const memoryLine = globalConst.get('memoryLine')
    const value = memoryLine.get('value')
    if (!(value.get('i64') instanceof NulLP)) {
      const val = BigInt(value.t.replace(/i64$/, ''))
      globalMem.push(val)
    } else if (!(value.get('f64') instanceof NulLP)) {
      const buf = Buffer.alloc(8)
      buf.writeDoubleLE(parseFloat(value.t.replace(/f64$/, '')))
      const val = buf.readBigUInt64LE(0)
      globalMem.push(val)
    } else if (!(value.get('str') instanceof NulLP)) {
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
    } else if (!(value.get('bool') instanceof NulLP)) {
      const val = value.t === "true" ? 1n : 0n
      globalMem.push(val)
    } else {
      console.error('Strange AST parsing error, this should be unreachable')
      console.error(value)
      process.exit(1)
    }
  }
  return globalMem
}

const loadEventDecs = (eventAst: LPish, eventLookup: Object): bigint[] => {
  const eventLines = eventAst.get('eventLines')
  let customEventIdOffset = 0n
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

const loadStatements = (statements: LPish, eventLookup: Object): bigint[] => {
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
      if (!(argOpt.get('variable') instanceof NulLP)) {
        out = eventLookup[argOpt.get('variable').t].eventId
      } else if (!(argOpt.get('memoryAddress') instanceof NulLP)) {
        out = int64ToUint64(BigInt((argOpt.get('memoryAddress').get(1).t)))
      } else if (!(argOpt.get('i64') instanceof NulLP)) {
        out = BigInt(argOpt.t.replace(/i64$/, ''))
      } else if (!(argOpt.get('f64') instanceof NulLP)) {
        const buf = Buffer.alloc(8)
        buf.writeDoubleLE(parseFloat(value.t.replace(/f64$/, '')))
        out = buf.readBigUInt64LE(0)
      }
      return out
    })
    const resultAddress = statement.get('result').t === '' ?
      0n :
      BigInt(statement.get('result').get('memoryAddress').get(1).t)
    vec.push(lineno, line, BigInt(deps.length), ...deps, fn, ...args, resultAddress)
  }
  return vec
}

const loadHandlers = (handlersAst: LPish, eventLookup: Object): bigint[] => {
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
  const globalMemoryAst = ast.get('globalMemory')
  if (!(globalMemoryAst instanceof NulLP)) {
    // Get the global memory
    const globalMem = loadGlobalMem(globalMemoryAst)
    // Compute the global memory size and declare that and add all of the global memory
    const memSize = BigInt(globalMem.length * 8)
    vec.push(memSize, ...globalMem)
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
    outBuf.writeBigUInt64LE(n, i * 8)
  })
  return outBuf
}

export const agaToAgc = (filename: string): Buffer => {
  const lp = new LP(filename)
  const ast = aga.apply(lp)
  if (ast instanceof Error) {
    console.error(ast)
    process.exit(1)
  }
  return astToAgc(ast)
}

export const agaTextToAgc = (str: string): Buffer => {
  const lp = LP.fromText(str)
  const ast = aga.apply(lp)
  if (ast instanceof Error) {
    console.error(ast)
    process.exit(1)
  }
  return astToAgc(ast)
}
