import {
  And,
  CharSet,
  LeftSubset,
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

// Defining LN Tokens
const space = Token.build(' ')
const blank = OneOrMore.build(space)
const optblank = ZeroOrOne.build(blank)
const newline = Or.build([Token.build('\n'), Token.build('\r\n')])
const notnewline = And.build([Not.build('\n'), Not.build('\r\n')])
const singlelinecomment = And.build([Token.build('//'), ZeroOrMore.build(notnewline), newline])
const star = Token.build('*')
const notstar = Not.build('*')
const notslash = Not.build('/')
const multilinecomment = And.build([
  Token.build('/*'),
  ZeroOrMore.build(Or.build([notstar, And.build([star, notslash])])),
  Token.build('*/'),
])
const whitespace = OneOrMore.build(Or.build([space, newline, singlelinecomment, multilinecomment]))
const optwhitespace = ZeroOrOne.build(whitespace)
const colon = Token.build(':')
const under = Token.build('_')
const negate = Token.build('-')
const dot = Token.build('.')
const par = Token.build('..')
const eq = Token.build('=')
const openParen = Token.build('(')
const closeParen = Token.build(')')
const openCurly = Token.build('{')
const closeCurly = Token.build('}')
const openCaret = Token.build('<')
const closeCaret = Token.build('>')
const openArr = Token.build('[')
const closeArr = Token.build(']')
const comma = Token.build(',')
const optcomma = ZeroOrOne.build(comma)
const semicolon = Token.build(';')
const optsemicolon = ZeroOrOne.build(semicolon)
const at = Token.build('@')
const slash = Token.build('/')
const base10 = CharSet.build('0', '9')
const natural = OneOrMore.build(base10)
const integer = And.build([ZeroOrOne.build(negate), natural])
const real = And.build([integer, ZeroOrOne.build(And.build([dot, natural]))])
const num = NamedOr.build({
  real,
  integer,
})
const t = Token.build('true')
const f = Token.build('false')
const bool = Or.build([t, f])
const lower = CharSet.build('a', 'z')
const upper = CharSet.build('A', 'Z')
const variable = LeftSubset.build(
  And.build([
    OneOrMore.build(Or.build([under, lower, upper])),
    ZeroOrMore.build(Or.build([under, lower, upper, base10])),
  ]),
  bool,
)
const operators = OneOrMore.build(Or.build([
  Token.build('+'),
  Token.build('-'),
  Token.build('/'),
  Token.build('\\'),
  Token.build('*'),
  Token.build('^'),
  Token.build('.'),
  Token.build('~'),
  Token.build('`'),
  Token.build('!'),
  Token.build('@'),
  Token.build('#'),
  Token.build('$'),
  Token.build('%'),
  Token.build('&'),
  Token.build('|'),
  Token.build(':'),
  Token.build('<'),
  Token.build('>'),
  Token.build('?'),
  Token.build('='),
]))
const interfacen = Token.build('interface')
const newn = Token.build('new')
const ifn = Token.build('if')
const elsen = Token.build('else')
const precedence = Token.build('precedence')
const infix = Token.build('infix')
const prefix = Token.build('prefix')
const asn = Token.build('as')
const exit = Token.build('return')
const emit = Token.build('emit')
const letn = Token.build('let')
const constn = Token.build('const')
const on = Token.build('on')
const event = Token.build('event')
const exportn = Token.build('export')
const typen = Token.build('type')
const importn = Token.build('import')
const fromn = Token.build('from')
const fn = Token.build('fn')
const quote = Token.build("'")
const doublequote = Token.build('"')
const escapeQuote = Token.build("\\'")
const escapeDoublequote = Token.build('\\"')
const notQuote = Not.build("'")
const notDoublequote = Not.build('"')
const sep = And.build([optwhitespace, comma, optwhitespace])
const optsep = ZeroOrOne.build(sep)
const str = Or.build([
  And.build([quote, ZeroOrMore.build(Or.build([escapeQuote, notQuote])), quote]),
  And.build([doublequote, ZeroOrMore.build(Or.build([escapeDoublequote, notDoublequote])), doublequote]),
])

const arrayaccess = NamedAnd.build({
  openArr,
  b: optwhitespace,
  assignables: new NulLP(), // Circular dep trick, see line 305
  c: optwhitespace,
  closeArr,
})
const varsegment = NamedOr.build({
  variable,
  methodsep: And.build([ optwhitespace, dot, optwhitespace ]),
  arrayaccess,
})
const varn = OneOrMore.build(varsegment)
const varop = NamedOr.build({
  variable,
  operators,
})
const renamed = ZeroOrOne.build(NamedAnd.build({
  a: blank,
  asn,
  b: blank,
  varop,
}))
const renameablevar = NamedAnd.build({
  varop,
  renamed,
})
const varlist = NamedAnd.build({
  renameablevar,
  cdr: ZeroOrMore.build(NamedAnd.build({
    sep,
    renameablevar,
  })),
  sep: ZeroOrOne.build(sep),
})
const depsegment = NamedOr.build({
  variable,
  slash,
})
const pardepsegment = NamedOr.build({
  variable,
  slash,
  par,
})
const localdependency = NamedOr.build({
  curDir: NamedAnd.build({
    dot,
    depsegments: OneOrMore.build(depsegment),
  }),
  parDir: NamedAnd.build({
    par,
    depsegments: OneOrMore.build(pardepsegment),
  }),
})
const globaldependency = NamedAnd.build({
  at,
  depsegments: OneOrMore.build(depsegment),
})
const dependency = NamedOr.build({
  localdependency,
  globaldependency,
})
const standardImport = NamedAnd.build({
  importn,
  blank,
  dependency,
  renamed,
  newline,
  optwhitespace,
})
const fromImport = NamedAnd.build({
  fromn,
  a: blank,
  dependency,
  b: blank,
  importn,
  c: blank,
  varlist,
  newline,
  optwhitespace,
})
const imports = ZeroOrMore.build(NamedOr.build({
  standardImport,
  fromImport,
}))
const typename = varn
const typegenerics = NamedAnd.build({
  openCaret,
  a: optwhitespace,
  generics: new NulLP(),
  b: optwhitespace,
  closeCaret,
})
export const fulltypename = NamedAnd.build({
  typename,
  opttypegenerics: ZeroOrOne.build(typegenerics),
});
typegenerics.and.generics = NamedAnd.build({
  fulltypename,
  cdr: ZeroOrMore.build(NamedAnd.build({
    sep,
    fulltypename,
  })),
})
const typeline = NamedAnd.build({
  variable,
  a: optwhitespace,
  colon,
  b: optwhitespace,
  fulltypename,
})
const typelist = NamedAnd.build({
  typeline,
  cdr: ZeroOrMore.build(NamedAnd.build({
    sep,
    typeline,
  })),
  optsep,
})
const typebody = NamedAnd.build({
  openCurly,
  a: optwhitespace,
  typelist,
  b: optwhitespace,
  closeCurly,
})
const types = NamedAnd.build({
  typen,
  blank,
  fulltypename,
  optwhitespace,
  typedef: NamedOr.build({
    typebody,
    typealias: NamedAnd.build({
      eq,
      blank,
      fulltypename,
    }),
  }),
})
const constants = NamedOr.build({
  bool,
  num,
  str,
})
const baseassignable = NamedOr.build({
  objectliterals: new NulLP(), // See line 525
  functions: new NulLP(), // See line 419
  fncall: new NulLP(), // See line 533
  variable,
  constants,
  methodsep: And.build([ optwhitespace, dot, optwhitespace ]),
})
const baseassignablelist = OneOrMore.build(NamedAnd.build({
  baseassignable,
}))
const withoperators = NamedOr.build({
  baseassignablelist,
  operators: And.build([optwhitespace, operators, optwhitespace]),
})
export const assignables = OneOrMore.build(NamedAnd.build({
  withoperators,
}))
arrayaccess.and.assignables = assignables
const constdeclaration = NamedAnd.build({
  constn,
  whitespace,
  variable,
  a: optwhitespace,
  typedec: ZeroOrOne.build(NamedAnd.build({
    colon,
    a: optwhitespace,
    fulltypename,
    b: optwhitespace,
  })),
  eq,
  b: optwhitespace,
  assignables,
  semicolon,
})
const letdeclaration = NamedAnd.build({
  letn,
  whitespace,
  variable,
  a: optwhitespace,
  typedec: ZeroOrOne.build(NamedAnd.build({
    colon,
    optwhitespace,
    fulltypename,
  })),
  b: optwhitespace,
  eq,
  c: optwhitespace,
  assignables,
  semicolon,
})
const declarations = NamedOr.build({
  constdeclaration,
  letdeclaration,
})
const assignments = NamedAnd.build({
  varn,
  a: optwhitespace,
  eq,
  b: optwhitespace,
  assignables,
  semicolon,
})
const retval = ZeroOrOne.build(NamedAnd.build({
  assignables,
  optwhitespace,
}))
const exits = NamedAnd.build({
  exit,
  optwhitespace,
  retval,
  semicolon,
})
const emits = NamedAnd.build({
  emit,
  a: optwhitespace,
  eventname: varn,
  b: optwhitespace,
  retval,
  semicolon,
})
const arglist = ZeroOrOne.build(NamedAnd.build({
  variable,
  a: optblank,
  colon,
  b: optblank,
  fulltypename,
  cdr: ZeroOrMore.build(NamedAnd.build({
    sep,
    variable,
    a: optblank,
    colon,
    b: optblank,
    fulltypename,
  })),
  optsep,
}))
const functionbody = NamedAnd.build({
  openCurly,
  statements: new NulLP(), // See line 458
  optwhitespace,
  closeCurly,
})
const assignfunction = NamedAnd.build({
  eq,
  optwhitespace,
  assignables,
  optsemicolon,
})
const fullfunctionbody = NamedOr.build({
  functionbody,
  assignfunction,
})
export const functions = NamedAnd.build({
  fn,
  a: optwhitespace,
  optname: ZeroOrOne.build(variable),
  b: optwhitespace,
  optargs: ZeroOrOne.build(NamedAnd.build({
    openParen,
    a: optwhitespace,
    arglist,
    b: optwhitespace,
    closeParen,
    c: optwhitespace,
  })),
  optreturntype: ZeroOrOne.build(NamedAnd.build({
    colon,
    a: optwhitespace,
    fulltypename,
    b: optwhitespace,
  })),
  fullfunctionbody,
})
baseassignable.or.functions = functions
const blocklike = NamedOr.build({
  functions,
  functionbody,
  fnname: varn,
})
const condorblock = NamedOr.build({
  conditionals: new NulLP(), // Circ dep trick, see line 442
  blocklike,
})
const conditionals = NamedAnd.build({
  ifn,
  whitespace,
  assignables,
  optwhitespace,
  blocklike,
  elsebranch: ZeroOrOne.build(NamedAnd.build({
    whitespace,
    elsen,
    optwhitespace,
    condorblock,
  })),
})
condorblock.or.conditionals = conditionals
export const statement = NamedOr.build({
  declarations,
  exits,
  emits,
  assignments,
  conditionals,
  assignables: NamedAnd.build({
    assignables,
    semicolon,
  }),
})
const statements = OneOrMore.build(NamedAnd.build({
  optwhitespace,
  statement,
}))
functionbody.and.statements = statements
const literaldec = NamedAnd.build({
  newn,
  blank,
  fulltypename,
  optblank,
})
const assignablelist = ZeroOrOne.build(NamedAnd.build({
  optwhitespace,
  assignables,
  cdr: ZeroOrMore.build(NamedAnd.build({
    sep,
    assignables,
  })),
  optsep,
}))
const arraybase = NamedAnd.build({
  openArr,
  a: optwhitespace,
  assignablelist,
  b: optwhitespace,
  closeArr,
})
const fullarrayliteral = NamedAnd.build({
  literaldec,
  arraybase,
})
const arrayliteral = NamedOr.build({
  arraybase,
  fullarrayliteral,
})
const typeassignlist = NamedAnd.build({
  variable,
  a: optwhitespace,
  colon,
  b: optwhitespace,
  assignables,
  c: optwhitespace,
  cdr: ZeroOrMore.build(NamedAnd.build({
    sep,
    variable,
    a: optwhitespace,
    colon,
    b: optwhitespace,
    assignables,
  })),
  optsep,
})
const typebase = NamedAnd.build({
  openCurly,
  a: optwhitespace,
  typeassignlist,
  b: optwhitespace,
  closeCurly,
})
const typeliteral = NamedAnd.build({
  literaldec,
  typebase,
})
const objectliterals = NamedOr.build({
  arrayliteral,
  typeliteral,
})
baseassignable.or.objectliterals = objectliterals
const fncall = NamedAnd.build({
  openParen,
  a: optwhitespace,
  assignablelist,
  b: optwhitespace,
  closeParen,
})
baseassignable.or.fncall = fncall
const fntoop = NamedAnd.build({
  fnname: variable,
  a: blank,
  asn,
  b: blank,
  operators,
})
const opprecedence = NamedAnd.build({
  precedence,
  blank,
  num,
})
const fix = NamedOr.build({
  prefix,
  infix,
})
const opmap = Or.build([
  NamedAnd.build({
    fntoop,
    blank,
    opprecedence,
  }),
  NamedAnd.build({
    opprecedence,
    blank,
    fntoop,
  }),
])
const operatormapping = NamedAnd.build({
  fix,
  blank,
  opmap,
})
const events = NamedAnd.build({
  event,
  whitespace,
  variable,
  a: optwhitespace,
  colon,
  b: optwhitespace,
  fulltypename,
})
const propertytypeline = NamedAnd.build({
  variable,
  a: blank,
  colon,
  b: blank,
  fulltypename,
})
const operatortypeline = NamedAnd.build({
  optleftarg: ZeroOrOne.build(NamedAnd.build({
    leftarg: fulltypename,
    whitespace,
  })),
  operators,
  whitespace,
  rightarg: fulltypename,
  a: optwhitespace,
  colon,
  b: optwhitespace,
  fulltypename,
})
const functiontype = NamedAnd.build({
  openParen,
  a: optwhitespace,
  fulltypename,
  b: optwhitespace,
  cdr: ZeroOrMore.build(NamedAnd.build({
    sep,
    a: optwhitespace,
    fulltypename,
    b: optwhitespace,
  })),
  optsep,
  c: optwhitespace,
  closeParen,
  d: optwhitespace,
  colon,
  e: optwhitespace,
  returntype: fulltypename,
})
const functiontypeline = NamedAnd.build({
  variable,
  optblank,
  functiontype,
})
const interfaceline = NamedOr.build({
  functiontypeline,
  operatortypeline,
  propertytypeline,
})
const interfacelist = ZeroOrOne.build(NamedAnd.build({
  a: optwhitespace,
  interfaceline,
  b: optwhitespace,
  cdr: ZeroOrMore.build(NamedAnd.build({
    sep,
    a: optwhitespace,
    interfaceline,
    b: optwhitespace,
  })),
  optsep,
}))
const interfacebody = NamedAnd.build({
  openCurly,
  interfacelist,
  optwhitespace,
  closeCurly,
})
const interfacealias = NamedAnd.build({
  eq,
  blank,
  variable,
})
const interfacedef = NamedOr.build({
  interfacebody,
  interfacealias,
})
const interfaces = NamedAnd.build({
  interfacen,
  a: optblank,
  variable,
  b: optblank,
  interfacedef,
})
const exportable = NamedOr.build({
  functions,
  constdeclaration,
  types,
  interfaces,
  operatormapping,
  events,
  ref: variable,
})
const exportsn = NamedAnd.build({
  exportn,
  blank,
  exportable,
})
const handler = NamedOr.build({
  functions,
  functionbody,
  fnname: variable,
})
const handlers = NamedAnd.build({
  on,
  a: whitespace,
  eventname: varn,
  b: whitespace,
  handler,
})
const body = OneOrMore.build(NamedOr.build({
  whitespace,
  exportsn,
  handlers,
  functions,
  types,
  constdeclaration,
  operatormapping,
  events,
  interfaces,
}))
export const ln = NamedAnd.build({
  optwhitespace,
  imports,
  body,
})

