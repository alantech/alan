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
} from '../lp'

// Defining LN Tokens
const space = Token.build(' ')
const blank = OneOrMore.build(space)
const optblank = ZeroOrOne.build(blank)
const newline = Token.build('\n')
const whitespace = OneOrMore.build(Or.build([space, newline]))
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
  integer,
  real,
})
const lower = CharSet.build('a', 'z')
const upper = CharSet.build('A', 'Z')
const variable = And.build([
  OneOrMore.build(Or.build([under, lower, upper])),
  ZeroOrMore.build(Or.build([under, lower, upper, natural])),
])
const generaloperators = And.build([
  Or.build([
    Token.build('+'),
    Token.build('-'),
    Token.build('/'),
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
    Token.build('?'),
    Token.build('='),
  ]),
  ZeroOrMore.build(Or.build([
    Token.build('+'),
    Token.build('-'),
    Token.build('/'),
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
  ])),
])
const interfacen = Token.build('interface')
const newn = Token.build('new')
const ifn = Token.build('if')
const elsen = Token.build('else')
const precedence = Token.build('precedence')
const infix = Token.build('infix')
const prefix = Token.build('prefix')
const asn = Token.build('as')
const exit = Token.build('return')
const t = Token.build('true')
const f = Token.build('false')
const bool = Or.build([t, f])
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
const quote = Token.build('"')
const escapeQuote = Token.build('\\"')
const notQuote = Not.build('"')
const sep = And.build([comma, optblank])
const optsep = ZeroOrOne.build(sep)
const str = And.build([quote, ZeroOrMore.build(Or.build([escapeQuote, notQuote])), quote])

const arrayaccess = NamedAnd.build({
  openArr,
  b: ZeroOrOne.build(whitespace),
  assignables: new NulLP(), // Circular dep trick, see line 305
  c: ZeroOrOne.build(whitespace),
  closeArr,
})
const varsegment = NamedOr.build({
  variable,
  methodsep: And.build([ whitespace, dot, ]),
  arrayaccess,
})
const varn = OneOrMore.build(varsegment)
const operators = NamedOr.build({
  generaloperators,
  dot,
  at,
  slash,
  openCaret,
  genericsWorkaround: NamedAnd.build({
    closeCarets: OneOrMore.build(closeCaret),
    maybeMore: ZeroOrOne.build(NamedOr.build({
      withEquals: NamedAnd.build({
        eqs: OneOrMore.build(eq),
        maybeoperators: ZeroOrOne.build(generaloperators),
      }),
      generaloperators,
    })),
  }),
})
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
const typename = variable
const typegenerics = NamedAnd.build({
  openCaret,
  generics: OneOrMore.build(NamedAnd.build({
    optblank,
    fulltypename: new NulLP(), // Circular dependency trick, see line 245
    optsep,
  })),
  closeCaret,
})
const fulltypename = NamedAnd.build({
  typename,
  opttypegenerics: ZeroOrOne.build(NamedAnd.build({
    optblank,
    typegenerics
  })),
});
// Ugly hackery around circular dependency
((typegenerics.and.generics as OneOrMore).oneOrMore[0] as NamedAnd).and.fulltypename = fulltypename
const typeline = NamedAnd.build({
  variable,
  a: optblank,
  colon,
  b: optblank,
  fulltypename,
})
const typelist = NamedAnd.build({
  typeline,
  optwhitespace,
  cdr: ZeroOrMore.build(NamedAnd.build({
    sep,
    a: optwhitespace,
    typeline,
    b: optwhitespace,
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
  a: blank,
  fulltypename,
  b: blank,
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
  num,
  str,
  bool,
})
const baseassignable = NamedOr.build({
  dot,
  variable,
  constants,
  functions: new NulLP(), // See line 419
  fncall: new NulLP(), // See line 533
  objectliterals: new NulLP(), // See line 525
})
const baseassignablelist = OneOrMore.build(NamedAnd.build({
  baseassignable,
  optwhitespace,
}))
const withoperators = NamedOr.build({
  baseassignablelist,
  operators,
})
const assignables = OneOrMore.build(NamedAnd.build({
  withoperators,
  optwhitespace,
}))
arrayaccess.and.assignables = assignables
const constdeclaration = NamedAnd.build({
  constn,
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
  eventname: variable,
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
  a: optwhitespace,
  statements: new NulLP(), // See line 458
  b: optwhitespace,
  closeCurly,
})
const assignfunction = NamedAnd.build({
  eq,
  optwhitespace,
  assignables,
})
const fullfunctionbody = NamedOr.build({
  functionbody,
  assignfunction,
})
const functions = NamedAnd.build({
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
  optsemicolon,
})
baseassignable.or.functions = functions
const blocklike = NamedOr.build({
  functions,
  functionbody,
  fnname: variable,
})
const condorblock = NamedOr.build({
  blocklike,
  conditionals: new NulLP(), // Circ dep trick, see line 442
})
const conditionals = NamedAnd.build({
  ifn,
  a: whitespace,
  assignables,
  b: whitespace,
  blocklike,
  elsebranch: ZeroOrOne.build(NamedAnd.build({
    optwhitespace,
    elsen,
    whitespace,
    condorblock,
  })),
})
condorblock.or.conditionals = conditionals
const statement = NamedOr.build({
  declarations,
  exits,
  emits,
  assignments,
  assignables: NamedAnd.build({
    assignables,
    semicolon,
  }),
  conditionals,
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
  assignables,
  optwhitespace,
  cdr: ZeroOrMore.build(NamedAnd.build({
    sep,
    a: optwhitespace,
    assignables,
    b: optwhitespace,
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
    a: optwhitespace,
    variable,
    b: optwhitespace,
    colon,
    c: optwhitespace,
    assignables,
    d: optwhitespace,
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
  events,
  types,
  constdeclaration,
  functions,
  operatormapping,
  interfaces,
  ref: variable,
})
const exportsn = NamedAnd.build({
  exportn,
  blank,
  exportable,
})
const handler = NamedOr.build({
  functions,
  fnname: variable,
  functionbody,
})
const handlers = NamedAnd.build({
  on,
  a: whitespace,
  eventname: variable,
  b: whitespace,
  handler,
})
const body = OneOrMore.build(NamedOr.build({
  types,
  constdeclaration,
  functions,
  operatormapping,
  events,
  handlers,
  interfaces,
  exportsn,
  whitespace,
}))
const ln = NamedAnd.build({
  optwhitespace,
  imports,
  body,
})
export const stripcomments = (str: string) => str
  .replace(/\/\/[^\r\n]*[\r\n]/mg, '\n')
  .replace(/\/\*(\*[^\/]|[^\*])*\*\//mg, (m) => m.split('\n').map(_ => '').join('\n'))
export default ln
