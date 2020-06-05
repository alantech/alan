import * as fs from 'fs' // This syntax is so dumb

export class LPBin {
  filename: string
  data: Buffer
  i: number

  constructor(filename: string, loadData: boolean = true) {
    this.filename = filename
    this.data = loadData ? fs.readFileSync(filename) : new Buffer(0)
    this.i = 0
  }

  advance(n: number) {
    this.i += n
  }

  clone(): LPBin {
    const clone = new LPBin(this.filename, false)
    clone.data = this.data
    clone.i = this.i
    return clone
  }
}

export interface LPBinish {
  b: Buffer
  check(lp: LP): boolean
  apply(lp: LP): LPBinish | Error
}

export const lpError = (message: string, obj: LPBin) => new Error(`${message} in file ${obj.filename}`)

export class Token implements LPBinish {
  b: Buffer
  filename: string
  i: number

  constructor(b: Buffer, filename: string, i: number) {
    this.b = b
    this.filename = filename
    this.i = i
  }

  static build(b: Buffer): Token {
    return new Token(b, '', -1)
  }

  toBuffer(): Buffer {
    return this.b
  }

  check(lp: LPBin): boolean {
    let matches = true
    const b = this.b
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

export class ZeroOrOne implements LPish {
  t: string
  zeroOrOne: LPish
  filename: string
  line: number
  char: number

  constructor(t: string, zeroOrOne: LPish, filename: string, line: number, char: number) {
    this.t = t
    this.zeroOrOne = zeroOrOne
    this.filename = filename
    this.line = line
    this.char = char
  }

  static build(zeroOrOne: LPish): ZeroOrOne {
    return new ZeroOrOne('', zeroOrOne, '', -1, -1)
  }

  toString(): string {
    return this.t
  }

  check(): boolean {
    return true
  }

  apply(lp: LP): ZeroOrOne {
    const filename = lp.filename
    const line = lp.line
    const char = lp.char
    if (this.zeroOrOne.check(lp)) {
      const zeroOrOne = this.zeroOrOne.apply(lp)
      if (zeroOrOne instanceof Error) return new ZeroOrOne('', this.zeroOrOne, filename, line, char)
      const t = zeroOrOne.toString()
      return new ZeroOrOne(t, zeroOrOne, filename, line, char)
    }
    return new ZeroOrOne('', this.zeroOrOne, filename, line, char)
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

  check(): boolean {
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

interface Named {
  [key: string]: LPish
}

export class NamedAnd implements LPish {
  t: string
  and: Named
  filename: string
  line: number
  char: number

  constructor(t: string, and: Named, filename: string, line: number, char: number) {
    this.t = t
    this.and = and
    this.filename = filename
    this.line = line
    this.char = char
  }

  static build(and: Named): NamedAnd {
    return new NamedAnd('', and, '', -1, -1)
  }

  toString(): string {
    return this.t
  }

  check(lp: LP): boolean {
    const lpClone = lp.clone()
    let works = true
    const andNames = Object.keys(this.and)
    for (let i = 0; i < andNames.length; i++) {
      if (this.and[andNames[i]].apply(lpClone) instanceof Error) {
        works = false
        break
      }
    }
    return works
  }

  apply(lp: LP): NamedAnd | Error {
    const filename = lp.filename
    const line = lp.line
    const char = lp.char
    let t = ''
    let and = {}
    const andNames = Object.keys(this.and)
    // This can fail, allow the underlying error to bubble up
    for (let i = 0; i < andNames.length; i++) {
      const a = this.and[andNames[i]].apply(lp)
      if (a instanceof Error) return a;
      t += a.toString()
      and[andNames[i]] = a
    }
    return new NamedAnd(t, and, filename, line, char)
  }
}

export class NamedOr implements LPish {
  t: string
  or: Named
  filename: string
  line: number
  char: number

  constructor(t: string, or: Named, filename: string, line: number, char: number) {
    this.t = t
    this.or = or
    this.filename = filename
    this.line = line
    this.char = char
  }

  static build(or: Named): NamedOr {
    return new NamedOr('', or, '', -1, -1)
  }

  toString(): string {
    return this.t
  }

  check(lp: LP): boolean {
    let works = false
    const orNames = Object.keys(this.or)
    for (let i = 0; i < orNames.length; i++) {
      const lpClone = lp.clone()
      if (!(this.or[orNames[i]].apply(lpClone) instanceof Error)) {
        works = true
        break
      }
    }
    return works
  }

  apply(lp: LP): NamedOr | Error {
    const filename = lp.filename
    const line = lp.line
    const char = lp.char
    let t = ''
    let or = {}
    const orNames = Object.keys(this.or)
    // Return the first match (if there are multiple matches, it is the first one)
    for (let i = 0; i < orNames.length; i++) {
      // We need to test which one will work without mutating the original one
      const lpClone = lp.clone()
      const ofake = this.or[orNames[i]].apply(lpClone)
      if (ofake instanceof Error) continue;
      // We have a match!
      const o = this.or[i].apply(lp)
      t = o.toString()
      or[orNames[i]] = o
      break
    }
    if (Object.keys(or).length === 0) return lpError('No matching tokens found', lp)
    return new NamedOr(t, or, filename, line, char)
  }
}

export class CharSet implements LPish {
  t: string
  lowerCharCode: number
  upperCharCode: number
  filename: string
  line: number
  char: number

  constructor(
    t: string,
    lowerChar: string,
    upperChar: string,
    filename: string,
    line: number,
    char: number
  ) {
    this.t = t
    this.lowerCharCode = lowerChar.charCodeAt(0)
    this.upperCharCode = upperChar.charCodeAt(0)
    this.filename = filename
    this.line = line
    this.char = char
  }

  static build(lowerChar: string, upperChar: string): Token {
    return new CharSet('', lowerChar, upperChar, '', -1, -1)
  }

  toString(): string {
    return this.t
  }

  check(lp: LP): boolean {
    let lpCharCode = lp.data.charCodeAt(lp.i)
    return this.lowerCharCode <= lpCharCode && this.upperCharCode >= lpCharCode
  }

  apply(lp: LP): CharSet | Error {
    if (this.check(lp)) {
      const outCharSet = new CharSet(
        lp.data[lp.i],
        String.fromCharCode(this.lowerCharCode),
        String.fromCharCode(this.upperCharCode),
        lp.filename,
        lp.line,
        lp.char,
      )
      lp.advance(1)
      return outCharSet
    }
    return lpError(`Token mismatch, expected character in range of ${String.fromCharCode(this.lowerCharCode)}-${String.fromCharCode(this.upperCharCode)}`, lp)
  }
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