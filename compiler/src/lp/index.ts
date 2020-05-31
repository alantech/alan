import * as fs from 'fs' // This syntax is so dumb

export class LP {
  filename: string
  data: string
  line: number
  char: number
  i: number

  constructor(filename: string, loadData: boolean = true) {
    this.filename = filename
    this.data = loadData ? fs.readFileSync(filename, 'utf8') : ''
    this.line = 0
    this.char = 0
    this.i = 0
  }

  advance(n: number) {
    for (let i = 0; i < n; i++) {
      this.i += 1
      if (this.data[this.i] === '\n') {
        this.line += 1
        this.char = 0
      } else {
        this.char += 1
      }
    }
  }

  clone(): LP {
    const clone = new LP(this.filename, false)
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
  apply(lp: LP): LPish | Error
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
    let matches = true
    const t = this.t
    const len = t.length
    const data = lp.data
    const j = lp.i
    for (let i = 0; i < len; i++) {
      if (t[i] !== data[i + j]) {
        matches = false
        break
      }
    }
    return matches
  }
  apply(lp: LP): Token | Error {
    if (this.check(lp)) {
      lp.advance(this.t.length)
      return new Token(
        this.t,
        lp.filename,
        lp.line,
        lp.char,
      )
    }
    return lpError(`Token mismatch, ${this.t} not found`, lp)
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
      if (maybe instanceof Error) return new Maybe('', this.maybe, filename, line, char)
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
    for (let i = 0; i < this.and.length; i++) {
      if (this.and[i].apply(lpClone) instanceof Error) {
        works = false
        break
      }
    }
    return works
  }

  apply(lp: LP): And | Error {
    const filename = lp.filename
    const line = lp.line
    const char = lp.char
    let t = ''
    let and = []
    // This can fail, allow the underlying error to bubble up
    for (let i = 0; i < this.and.length; i++) {
      const a = this.and[i].apply(lp)
      if (a instanceof Error) return a;
      t += a.toString()
      and.push(a)
    }
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
    for (let i = 0; i < this.or.length; i++) {
      const lpClone = lp.clone()
      if (!(this.or[i].apply(lpClone) instanceof Error)) {
        works = true
        break
      }
    }
    return works
  }

  apply(lp: LP): Or | Error {
    const filename = lp.filename
    const line = lp.line
    const char = lp.char
    let t = ''
    let or = []
    // Return the first match (if there are multiple matches, it is the first one)
    for (let i = 0; i < this.or.length; i++) {
      // We need to test which one will work without mutating the original one
      const lpClone = lp.clone()
      const ofake = this.or[i].apply(lpClone)
      if (ofake instanceof Error) continue;
      // We have a match!
      const o = this.or[i].apply(lp)
      t = o.toString()
      or.push(o)
      break
    }
    if (or.length === 0) return lpError('No matching tokens found', lp)
    return new Or(t, or, filename, line, char)
  }
}

export const CharSet = (lowerChar: string, upperChar: string): LPish | Error => {
  let chars = []
  for (let i = lowerChar.charCodeAt(0); i <= upperChar.charCodeAt(0); i++) {
    chars.push(String.fromCharCode(i))
  }
  return Or.build(chars.map(c => Token.build(c)))
}

export const RangeSet = (toRepeat: LPish, min: number, max: number): LPish | Error => {
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