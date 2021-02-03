import {
  And,
  CharSet,
  NamedAnd,
  NamedOr,
  Not,
  NulLP,
  OneOrMore,
  Or,
  Token,
  ZeroOrMore,
  ZeroOrOne,
} from './lp'

// Defining AMM Tokens
const space = Token.build(' ')
const blank = OneOrMore.build(space)
const optblank = ZeroOrOne.build(blank)
const newline = Token.build('\n')
const whitespace = OneOrMore.build(Or.build([space, newline]))
const colon = Token.build(':')
const under = Token.build('_')
const negate = Token.build('-')
const dot = Token.build('.')
const eq = Token.build('=')
const openParen = Token.build('(')
const closeParen = Token.build(')')
const openCurly = Token.build('{')
const closeCurly = Token.build('}')
const openCaret = Token.build('<')
const closeCaret = Token.build('>')
const comma = Token.build(',')
const optcomma = ZeroOrOne.build(comma)
const sep = And.build([optblank, comma, optblank])
const base10 = CharSet.build('0', '9')
const natural = OneOrMore.build(base10)
const integer = And.build([ZeroOrOne.build(negate), natural])
const real = And.build([integer, dot, natural])
const lower = CharSet.build('a', 'z')
const upper = CharSet.build('A', 'Z')
const variable = And.build([
  OneOrMore.build(Or.build([under, lower, upper])),
  ZeroOrMore.build(Or.build([under, lower, upper, natural])),
])
const exit = Token.build('return')
const t = Token.build('true')
const f = Token.build('false')
const bool = Or.build([t, f])
const voidn = Token.build('void')
const emit = Token.build('emit')
const letn = Token.build('let')
const constn = Token.build('const')
const on = Token.build('on')
const event = Token.build('event')
const fn = Token.build('fn')
const quote = Token.build('"')
const escapeQuote = Token.build('\\"')
const notQuote = Not.build('"')
const str = And.build([quote, ZeroOrMore.build(Or.build([escapeQuote, notQuote])), quote])
const value = NamedOr.build({ str, bool, real, integer, })
const decname = variable
const typename = variable
const typegenerics = NamedAnd.build({
  openCaret,
  a: optblank,
  generics: new NulLP(), // Circular dependency trick
  b: optblank,
  closeCaret,
})
const fulltypename = NamedAnd.build({
  typename,
  opttypegenerics: ZeroOrOne.build(typegenerics),
})
// Ugly hackery around circular dependency
typegenerics.and.generics = NamedAnd.build({
  fulltypename,
  cdr: ZeroOrMore.build(NamedAnd.build({
    sep,
    fulltypename,
  })),
})

const emits = NamedAnd.build({ emit, blank, variable, value: ZeroOrOne.build(NamedAnd.build({
  blank, variable
}))})
const events = NamedAnd.build({ event, blank, variable, a: optblank, colon, b: optblank, fulltypename })
const exits = NamedAnd.build({ exit, blank, variable, a: optblank })
const calllist = ZeroOrMore.build(NamedAnd.build({ variable, optcomma, optblank }))
const calls = NamedAnd.build({
  variable,
  a: optblank,
  openParen,
  b: optblank,
  calllist,
  c: optblank,
  closeParen
})
const assignables = NamedOr.build({
  functions: new NulLP(), // Circular dep trick
  calls,
  value,
  variable,
})
const constdeclaration = NamedAnd.build({
  constn,
  a: blank,
  decname,
  b: optblank,
  colon,
  c: optblank,
  fulltypename,
  d: optblank,
  eq,
  e: optblank,
  assignables,
})
const letdeclaration = NamedAnd.build({
  letn,
  a: blank,
  decname,
  b: optblank,
  colon,
  c: optblank,
  fulltypename,
  d: blank,
  eq,
  e: blank,
  assignables,
})
const declarations = NamedOr.build({ constdeclaration, letdeclaration })
const assignments = NamedAnd.build({ decname, a: blank, eq, b: blank, assignables, })
const statements = OneOrMore.build(NamedOr.build({
  declarations,
  assignments,
  calls,
  emits,
  exits,
  whitespace,
}))
const functionbody = NamedAnd.build({
  openCurly,
  statements,
  closeCurly,
})
const arg = NamedAnd.build({ variable, a: optblank, colon, b: optblank, fulltypename, })
const functions = NamedAnd.build({
  fn,
  blank,
  openParen,
  args: And.build([
    ZeroOrMore.build(NamedAnd.build({ arg, sep, })),
    ZeroOrOne.build(NamedAnd.build({ arg, optblank, }))
  ]),
  closeParen,
  a: optblank,
  colon,
  b: optblank,
  fulltypename,
  c: optblank,
  functionbody,
})
assignables.or.functions = functions
const handler = NamedAnd.build({ on, a: blank, variable, b: blank, functions, })
const amm = NamedAnd.build({
  a: optblank,
  globalMem: ZeroOrMore.build(Or.build([ constdeclaration, whitespace ])),
  eventDec: ZeroOrMore.build(Or.build([ events, whitespace ])),
  handlers: OneOrMore.build(Or.build([ handler, whitespace ])),
})
export default amm
