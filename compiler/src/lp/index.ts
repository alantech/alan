import * as fs from 'fs' // This syntax is so dumb

export class LP {
  filename: string
  data: string
  line: number
  char: number
  i: number

  constructor(filename: string) {
    this.filename = filename
    this.data = fs.readFileSync(filename, 'utf8')
    this.line = 0
    this.char = 0
    this.i = 0
  }

  advance(n: number) {
    const strfrags = this.data.substring(this.i, this.i + n).split('\n')
    this.i += n
    if (strfrags.length === 1) {
      this.char += n
    } else {
      this.line += strfrags.length - 1
      this.char = strfrags[strfrags.length - 1].length
    }
  }

  clone(): LP {
    const clone = new LP(this.filename)
    clone.data = this.data
    clone.line = this.line
    clone.char = this.char
    clone.i = this.i
    return clone
  }
}

export interface LPmeta {
  filename: string
  line: number
  char: number
}

export interface LPish {
  t: string
  check(lp: LP): boolean
  apply(lp: LP): LPish
}

export const lpError = (message: string, obj: LPmeta) => new Error(`${message} in file ${obj.filename} line ${obj.line}:${obj.char}`)

export class Token implements LPish {
  t: string
  filename: string
  line: number
  char: number

  constructor(t: string, filename: string, line: number, char: number) {
    this.t = t
    this.filename = filename
    this.line = line
    this.char = char
  }

  static build(t: string): Token {
    return new Token(t, '', -1, -1)
  }

  toString(): string {
    return this.t
  }

  check(lp: LP): boolean {
    const tokenCheck = lp.data.substring(lp.i, lp.i + this.t.length)
    return this.t === tokenCheck
  }
  apply(lp: LP): Token {
    if (this.check(lp)) {
      lp.advance(this.t.length)
      return new Token(
        this.t,
        lp.filename,
        lp.line,
        lp.char,
      )
    }
    throw lpError(`Token mismatch, ${this.t} not found`, lp)
  }
}

export class Maybe implements LPish {
  t: string
  maybe: LPish
  filename: string
  line: number
  char: number

  constructor(t: string, maybe: LPish, filename: string, line: number, char: number) {
    this.t = t
    this.maybe = maybe
    this.filename = filename
    this.line = line
    this.char = char
  }

  static build(maybe: LPish): Maybe {
    return new Maybe('', maybe, '', -1, -1)
  }

  toString(): string {
    return this.t
  }

  check(lp: LP): boolean {
    return true
  }

  apply(lp: LP): Maybe {
    const filename = lp.filename
    const line = lp.line
    const char = lp.char
    if (this.maybe.check(lp)) {
      const maybe = this.maybe.apply(lp)
      const t = maybe.toString()
      return new Maybe(t, maybe, filename, line, char)
    }
    return new Maybe('', this.maybe, filename, line, char)
  }
}

export class ZeroOrMore implements LPish {
  t: string
  zeroOrMore: LPish[]
  filename: string
  line: number
  char: number

  constructor(t: string, zeroOrMore: LPish[], filename: string, line: number, char: number) {
    this.t = t
    this.zeroOrMore = zeroOrMore
    this.filename = filename
    this.line = line
    this.char = char
  }

  static build(zeroOrMore: LPish): ZeroOrMore {
    return new ZeroOrMore('', [zeroOrMore], '', -1, -1)
  }

  toString(): string {
    return this.t
  }

  check(lp: LP): boolean {
    return true
  }

  apply(lp: LP): ZeroOrMore {
    const filename = lp.filename
    const line = lp.line
    const char = lp.char
    let t = ''
    let zeroOrMore = []
    while (this.zeroOrMore[0].check(lp)) {
      const z = this.zeroOrMore[0].apply(lp)
      t += z.toString()
      zeroOrMore.push(z)
    }
    return new ZeroOrMore(t, zeroOrMore, filename, line, char)
  }
}

export class OneOrMore implements LPish {
  t: string
  oneOrMore: LPish[]
  filename: string
  line: number
  char: number

  constructor(t: string, oneOrMore: LPish[], filename: string, line: number, char: number) {
    this.t = t
    this.oneOrMore = oneOrMore
    this.filename = filename
    this.line = line
    this.char = char
  }

  static build(oneOrMore: LPish): OneOrMore {
    return new OneOrMore('', [oneOrMore], '', -1, -1)
  }

  toString(): string {
    return this.t
  }

  check(lp: LP): boolean {
    return this.oneOrMore[0].check(lp)
  }

  apply(lp: LP): OneOrMore {
    const filename = lp.filename
    const line = lp.line
    const char = lp.char
    if (!this.check(lp)) throw lpError(`Token mismatch, expected '${this.oneOrMore[0].t}'`, lp)
    let t = ''
    let oneOrMore = []
    while (this.oneOrMore[0].check(lp)) {
      const o = this.oneOrMore[0].apply(lp)
      t += o.toString()
      oneOrMore.push(o)
    }
    return new OneOrMore(t, oneOrMore, filename, line, char)
  }
}

export class And implements LPish {
  t: string
  and: LPish[]
  filename: string
  line: number
  char: number

  constructor(t: string, and: LPish[], filename: string, line: number, char: number) {
    this.t = t
    this.and = and
    this.filename = filename
    this.line = line
    this.char = char
  }

  static build(and: LPish[]): And {
    return new And('', and, '', -1, -1)
  }

  toString(): string {
    return this.t
  }

  check(lp: LP): boolean {
    const lpClone = lp.clone()
    let works = true
    try {
      this.and.forEach(a => a.apply(lpClone))
    } catch (e) {
      works = false
    }
    return works
  }

  apply(lp: LP): And {
    const filename = lp.filename
    const line = lp.line
    const char = lp.char
    let t = ''
    let and = []
    // This can fail, allow the underlying error to bubble up
    this.and.forEach((a) => {
      const a2 = a.apply(lp)
      t += a2.toString()
      and.push(a2)
    })
    return new And(t, and, filename, line, char)
  }
}

export class Or implements LPish {
  t: string
  or: LPish[]
  filename: string
  line: number
  char: number

  constructor(t: string, or: LPish[], filename: string, line: number, char: number) {
    this.t = t
    this.or = or 
    this.filename = filename
    this.line = line
    this.char = char
  }

  static build(or: LPish[]): Or {
    return new Or('', or, '', -1, -1)
  }

  toString(): string {
    return this.t
  }

  check(lp: LP): boolean {
    const lpClone = lp.clone()
    let works = false
    this.or.forEach((o) => {
      const lpClone = lp.clone()
      let failed = false
      try {
        o.apply(lpClone)
      } catch (e) {
        failed = true
      }
      if (!failed) works = true
    })
    return works
  }

  apply(lp: LP): Or {
    const filename = lp.filename
    const line = lp.line
    const char = lp.char
    let t = ''
    let or = []
    if (!this.check(lp)) throw lpError('No matching tokens found', lp)
    // Return the first match (if there are multiple matches, it is the first one)
    for (let i = 0; i < this.or.length; i++) {
      // We need to test which one will work without mutating the original one
      const lpClone = lp.clone()
      try {
        this.or[i].apply(lpClone)
      } catch (e) {
        continue;
      }
      // We have a match!
      const o = this.or[i].apply(lp)
      t = o.toString()
      or.push(o)
      break
    }
    return new Or(t, or, filename, line, char)
  }
}

export const CharSet = (lowerChar: string, upperChar: string): LPish => {
  let chars = []
  for (let i = lowerChar.charCodeAt(0); i <= upperChar.charCodeAt(0); i++) {
    chars.push(String.fromCharCode(i))
  }
  return Or.build(chars.map(c => Token.build(c)))
}

export const RangeSet = (toRepeat: LPish, min: number, max: number): LPish => {
  let sets = []
  for (let i = min; i <= max; i++) {
    if (i === 0) {
      sets.push(Token.build(''))
      continue
    } else {
      let set = []
      for (let j = 0; j < i; j++) {
        set.push(toRepeat)
      }
      sets.push(And.build(set))
    }
  }
  return Or.build(sets)
}