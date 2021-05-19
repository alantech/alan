import {
  And,
  CharSet,
  NamedAnd,
  NamedOr,
  Not,
  OneOrMore,
  Or,
  Token,
  ZeroOrMore,
  ZeroOrOne,
} from './lp'

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
const i8 = And.build([integer, Token.build('i8')])
const i16 = And.build([integer, Token.build('i16')])
const i32 = And.build([integer, Token.build('i32')])
const i64 = And.build([integer, Token.build('i64')])
const f32 = And.build([real, Token.build('f32')])
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
const value = NamedOr.build({ str, bool, i8, i16, i32, i64, f32, f64 })
const header = Token.build('Alan Graphcode Assembler v0.0.1')
const globalMem = Token.build('globalMem')
const memoryAddress = And.build([at, integer])
const memoryLine = NamedAnd.build({ memoryAddress, colon, whitespace, value })
const customEvents = Token.build('customEvents')
const eventLine = NamedAnd.build({ variable, colon, whitespace, integer })
const handlerFor = Or.build([Token.build('handler for'), Token.build('closure for')])
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
const arg = NamedOr.build({ variable, memoryAddress, i8, i64, f64 })
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
export const aga = NamedAnd.build({
  header,
  a: whitespace,
  globalMemory: ZeroOrOne.build(memory),
  b: whitespace,
  customEvents: ZeroOrOne.build(events),
  c: whitespace,
  handlers: OneOrMore.build(handler),
})

export default aga