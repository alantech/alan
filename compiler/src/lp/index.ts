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

  static fromText(data: string): LP {
    const lp = new LP('fakeFile', false)
    lp.data = data
    return lp
  }
}

export interface LPmeta {
  filename: string
  line: number
  char: number
}

export interface LPish {
  t: string
  get(id?: string | number): LPish
  getAll(): LPish[]
  has(id?: string | number): boolean
  check(lp: LP): boolean
  apply(lp: LP): LPish | Error
}

export const lpError = (message: string, obj: LPmeta) => new Error(`${message} in file ${obj.filename} line ${obj.line}:${obj.char}`)

export class NulLP implements LPish {
  t: string

  constructor() {
    this.t = ''
  }

  get(): NulLP {
    return this
  }

  getAll(): NulLP[] {
    return [this]
  }

  has(): boolean {
    return false
  }

  check(): boolean {
    return false
  }

  apply(): LPish | Error {
    return new Error('nullish')
  }

  toString(): string {
    return this.t
  }
}

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

  get(): LPish {
    return this
  }

  getAll(): LPish[] {
    return [this]
  }

  has(): boolean {
    return this.line > -1
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

export class Not implements LPish {
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

  static build(t: string): Not {
    return new Not(t, '', -1, -1)
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
    return !matches
  }

  get(): Not {
    return this
  }

  getAll(): Not[] {
    return [this]
  }

  has(): boolean {
    return this.line > -1
  }

  apply(lp: LP): Not | Error {
    if (this.check(lp)) {
      const newT = lp.data[lp.i]
      lp.advance(this.t.length)
      return new Not(
        newT,
        lp.filename,
        lp.line,
        lp.char,
      )
    }
    return lpError(`Not mismatch, ${this.t} found`, lp)
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

  get(): LPish {
    return this.zeroOrOne
  }

  getAll(): LPish[] {
    return [this.zeroOrOne]
  }

  has(): boolean {
    return this.line > -1
  }

  apply(lp: LP): LPish {
    if (this.zeroOrOne.check(lp)) {
      const zeroOrOne = this.zeroOrOne.apply(lp)
      if (zeroOrOne instanceof Error) return new NulLP()
      return zeroOrOne
    }
    return new NulLP()
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

  get(i: number): LPish {
    if (this.zeroOrMore[i]) return this.zeroOrMore[i]
    return new NulLP()
  }

  getAll(): LPish[] {
    return this.zeroOrMore
  }

  has(id?: number): boolean {
    if (typeof id === 'number') {
      if (this.zeroOrMore[id]) {
        return this.zeroOrMore[id].has()
      }
      return false
    }
    return this.line > -1
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

  get(i: number): LPish {
    if (this.oneOrMore[i]) return this.oneOrMore[i]
    return new NulLP()
  }

  getAll(): LPish[] {
    return this.oneOrMore
  }

  has(id?: number): boolean {
    if (typeof id === 'number') {
      if (this.oneOrMore[id]) {
        return this.oneOrMore[id].has()
      }
      return false
    }
    return this.line > -1
  }

  apply(lp: LP): OneOrMore | Error {
    const filename = lp.filename
    const line = lp.line
    const char = lp.char
    if (!this.check(lp)) return lpError(`Token mismatch, expected '${this.oneOrMore[0].t}'`, lp)
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

  get(i: number): LPish {
    if (this.and[i]) return this.and[i]
    return new NulLP()
  }

  getAll(): LPish[] {
    return this.and
  }

  has(id?: number): boolean {
    if (typeof id === 'number') {
      if (this.and[id]) {
        return this.and[id].has()
      }
      return false
    }
    return this.line > -1
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

  get(): LPish {
    if (this.or[0]) return this.or[0]
    return new NulLP()
  }

  getAll(): LPish[] {
    return this.or
  }

  has(id?: number): boolean {
    if (typeof id === 'number') {
      if (this.or[id]) {
        return this.or[id].has()
      }
      return false
    }
    return this.line > -1
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
    return new NamedAnd(Object.keys(and).join(' '), and, '', -1, -1)
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

  get(name: string): LPish {
    if (this.and[name]) return this.and[name]
    return new NulLP()
  }

  getAll(): LPish[] {
    return Object.values(this.and)
  }

  has(id?: string): boolean {
    if (typeof id === 'string') {
      if (this.and[id]) {
        return this.and[id].has()
      }
      return false
    }
    return this.line > -1
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
      if (a instanceof Error) {
        return a
      }
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
    return new NamedOr(Object.keys(or).join(' '), or, '', -1, -1)
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

  get(name: string): LPish {
    if (this.or[name]) return this.or[name]
    return new NulLP()
  }

  getAll(): LPish[] {
    return Object.values(this.or)
  }

  has(id?: string): boolean {
    if (typeof id === 'string') {
      if (this.or[id]) {
        return this.or[id].has()
      }
      return false
    }
    return this.line > -1
  }

  apply(lp: LP): NamedOr | Error {
    const filename = lp.filename
    const line = lp.line
    const char = lp.char
    let t = ''
    let or = {}
    const orNames = Object.keys(this.or)
    const errors = []
    // Return the first match (if there are multiple matches, it is the first one)
    for (let i = 0; i < orNames.length; i++) {
      // We need to test which one will work without mutating the original one
      const lpClone = lp.clone()
      const ofake = this.or[orNames[i]].apply(lpClone)
      if (ofake instanceof Error) {
        errors.push(ofake)
        continue
      }
      // We have a match!
      const o = this.or[orNames[i]].apply(lp)
      t = o.toString()
      or[orNames[i]] = o
      break
    }
    if (Object.keys(or).length === 0) return lpError(`No matching tokens found: ${errors.map(e => e.message).join(', ')}`, lp)
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

  static build(lowerChar: string, upperChar: string): CharSet {
    return new CharSet(`[${lowerChar}-${upperChar}]`, lowerChar, upperChar, '', -1, -1)
  }

  toString(): string {
    return this.t
  }

  check(lp: LP): boolean {
    let lpCharCode = lp.data.charCodeAt(lp.i)
    return this.lowerCharCode <= lpCharCode && this.upperCharCode >= lpCharCode
  }

  get(): CharSet {
    return this
  }

  getAll(): CharSet[] {
    return [this]
  }

  has(): boolean {
    return this.line > -1
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