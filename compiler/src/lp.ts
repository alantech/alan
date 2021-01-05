import * as fs from 'fs' // This syntax is so dumb

// A snapshot of the metadata surrounding an LP record
export interface LPSnap {
  line: number
  char: number
  i: number
}

// An LP record and methods, used for keeping track of advancements through the text to parse
export class LP {
  filename: string
  data: string
  line: number
  char: number
  i: number

  constructor(filename: string, loadData: boolean = true) {
    this.filename = filename
    this.data = loadData ? fs.readFileSync(filename, 'utf8') : ''
    this.line = 1
    this.char = 1
    this.i = 0
  }

  advance(n: number) {
    for (let i = 0; i < n; i++) {
      this.i += 1
      if (this.data[this.i] === '\n') {
        this.line += 1
        this.char = 1
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

  snapshot(): LPSnap {
    return {
      line: this.line,
      char: this.char,
      i: this.i
    }
  }

  restore(snap: LPSnap) {
    this.line = snap.line
    this.char = snap.char
    this.i = snap.i
  }
}

// Any kind of type that provides enough data to attach metadata to error messages
export interface LPmeta {
  filename: string
  line: number
  char: number
}

// Any kind of type that can operate on LP records to build the AST.
export interface LPNode {
  t: string
  line: number
  char: number
  get(id?: string | number): LPNode
  getAll(): LPNode[]
  has(id?: string | number): boolean
  apply(lp: LP): LPNode | Error
}

export const lpError = (message: string, obj: LPmeta) => new Error(`${message} in file ${obj.filename} line ${obj.line}:${obj.char}`)

// A special AST node that indicates that you successfully matched nothing, useful for optional ASTs
export class NulLP implements LPNode {
  t: string
  line: number
  char: number

  constructor() {
    this.t = ''
    this.line = -1
    this.char = -1
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

  apply(): LPNode | Error {
    return new Error('nullish')
  }

  toString(): string {
    return this.t
  }
}

// One of the 'leaf' AST nodes. It declares a fixed set of characters in a row to match
export class Token implements LPNode {
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

  get(): LPNode {
    return this
  }

  getAll(): LPNode[] {
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
    return lpError(`Token mismatch, ${this.t} not found, instead ${lp.data[lp.i]}`, lp)
  }
}

// Another 'leaf' AST node. It matches any characters that DO NOT match the string provided
export class Not implements LPNode {
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

// An AST node that optionally matches the AST node below it
export class ZeroOrOne implements LPNode {
  t: string
  zeroOrOne: LPNode
  filename: string
  line: number
  char: number

  constructor(t: string, zeroOrOne: LPNode, filename: string, line: number, char: number) {
    this.t = t
    this.zeroOrOne = zeroOrOne
    this.filename = filename
    this.line = line
    this.char = char
  }

  static build(zeroOrOne: LPNode): ZeroOrOne {
    return new ZeroOrOne('', zeroOrOne, '', -1, -1)
  }

  toString(): string {
    return this.t
  }

  get(): LPNode {
    return this.zeroOrOne
  }

  getAll(): LPNode[] {
    return [this.zeroOrOne]
  }

  has(): boolean {
    return this.line > -1
  }

  apply(lp: LP): LPNode {
    const s = lp.snapshot()
    const zeroOrOne = this.zeroOrOne.apply(lp)
    if (zeroOrOne instanceof Error) {
      lp.restore(s)
      return new NulLP()
    }
    return zeroOrOne
  }
}

// An AST node that optionally matches the AST node below it as many times as possible
export class ZeroOrMore implements LPNode {
  t: string
  zeroOrMore: LPNode[]
  filename: string
  line: number
  char: number

  constructor(t: string, zeroOrMore: LPNode[], filename: string, line: number, char: number) {
    this.t = t
    this.zeroOrMore = zeroOrMore
    this.filename = filename
    this.line = line
    this.char = char
  }

  static build(zeroOrMore: LPNode): ZeroOrMore {
    return new ZeroOrMore('', [zeroOrMore], '', -1, -1)
  }

  toString(): string {
    return this.t
  }

  get(i: number): LPNode {
    if (this.zeroOrMore[i]) return this.zeroOrMore[i]
    return new NulLP()
  }

  getAll(): LPNode[] {
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

  apply(lp: LP): LPNode | Error {
    const filename = lp.filename
    const line = lp.line
    const char = lp.char
    let t = ''
    let zeroOrMore = []
    do {
      const s = lp.snapshot()
      const z = this.zeroOrMore[0].apply(lp)
      if (z instanceof Error) {
        lp.restore(s)
        return new ZeroOrMore(t, zeroOrMore, filename, line, char)
      }
      const t2 = z.toString()
      if (!t2 || t2.length === 0) {
        return lpError('ZeroOrMore made no forward progress, will infinite loop', lp)
      }
      t += t2
      zeroOrMore.push(z)
    } while(true)
  }
}

// An AST node that matches the node below it multiple times and fails if it finds no match
export class OneOrMore implements LPNode {
  t: string
  oneOrMore: LPNode[]
  filename: string
  line: number
  char: number

  constructor(t: string, oneOrMore: LPNode[], filename: string, line: number, char: number) {
    this.t = t
    this.oneOrMore = oneOrMore
    this.filename = filename
    this.line = line
    this.char = char
  }

  static build(oneOrMore: LPNode): OneOrMore {
    return new OneOrMore('', [oneOrMore], '', -1, -1)
  }

  toString(): string {
    return this.t
  }

  get(i: number): LPNode {
    if (this.oneOrMore[i]) return this.oneOrMore[i]
    return new NulLP()
  }

  getAll(): LPNode[] {
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

  apply(lp: LP): LPNode | Error {
    const filename = lp.filename
    const line = lp.line
    const char = lp.char
    let t = ''
    let oneOrMore = []
    do {
      const s = lp.snapshot()
      const o = this.oneOrMore[0].apply(lp)
      if (o instanceof Error) {
        lp.restore(s)
        if (oneOrMore.length === 0) {
          return lpError(`No match for OneOrMore ${this.oneOrMore.toString()}`, lp)
        }
        return new OneOrMore(t, oneOrMore, filename, line, char)
      }
      const t2 = o.toString()
      if (t2.length === 0) {
        return lpError('OneOrMore made no forward progress, will infinite loop', lp)
      }
      t += t2
      oneOrMore.push(o)
    } while(true)
  }
}

// An AST node that matches a sequence of child nodes in a row or fails
export class And implements LPNode {
  t: string
  and: LPNode[]
  filename: string
  line: number
  char: number

  constructor(t: string, and: LPNode[], filename: string, line: number, char: number) {
    this.t = t
    this.and = and
    this.filename = filename
    this.line = line
    this.char = char
  }

  static build(and: LPNode[]): And {
    return new And('', and, '', -1, -1)
  }

  toString(): string {
    return this.t
  }

  get(i: number): LPNode {
    if (this.and[i]) return this.and[i]
    return new NulLP()
  }

  getAll(): LPNode[] {
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
    const s = lp.snapshot()
    // This can fail, allow the underlying error to bubble up
    for (let i = 0; i < this.and.length; i++) {
      const a = this.and[i].apply(lp)
      if (a instanceof Error) {
        lp.restore(s)
        return a
      }
      t += a.toString()
      and.push(a)
    }
    return new And(t, and, filename, line, char)
  }
}

// An AST node that matches any of its child nodes or fails. Only returns the first match.
export class Or implements LPNode {
  t: string
  or: LPNode[]
  filename: string
  line: number
  char: number

  constructor(t: string, or: LPNode[], filename: string, line: number, char: number) {
    this.t = t
    this.or = or
    this.filename = filename
    this.line = line
    this.char = char
  }

  static build(or: LPNode[]): Or {
    return new Or('', or, '', -1, -1)
  }

  toString(): string {
    return this.t
  }

  get(): LPNode {
    if (this.or[0]) return this.or[0]
    return new NulLP()
  }

  getAll(): LPNode[] {
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
      const s = lp.snapshot()
      const o = this.or[i].apply(lp)
      if (o instanceof Error) {
        lp.restore(s)
        continue
      }
      // We have a match!
      t = o.toString()
      or.push(o)
      break
    }
    if (or.length === 0) return lpError('No matching tokens found', lp)
    return new Or(t, or, filename, line, char)
  }
}

interface Named {
  [key: string]: LPNode
}

// An AST node that matches all of the child nodes or fails. Also provides easier access to the
// matched child nodes.
export class NamedAnd implements LPNode {
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

  get(name: string): LPNode {
    if (this.and[name]) return this.and[name]
    return new NulLP()
  }

  getAll(): LPNode[] {
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
    const s = lp.snapshot()
    // This can fail, allow the underlying error to bubble up
    for (let i = 0; i < andNames.length; i++) {
      const a = this.and[andNames[i]].apply(lp)
      if (a instanceof Error) {
        lp.restore(s)
        return a
      }
      t += a.toString()
      and[andNames[i]] = a
    }
    return new NamedAnd(t, and, filename, line, char)
  }
}

// An AST node that matches one of the child nodes or fails. The first match is returned. Also
// provides easier access to the child node by name.
export class NamedOr implements LPNode {
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

  get(name: string): LPNode {
    if (this.or[name]) return this.or[name]
    return new NulLP()
  }

  getAll(): LPNode[] {
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
    // Return the first match (if there are multiple matches, it is the first one)
    for (let i = 0; i < orNames.length; i++) {
      const s = lp.snapshot()
      const o = this.or[orNames[i]].apply(lp)
      if (o instanceof Error) {
        lp.restore(s)
        continue
      }
      // We have a match!
      t = o.toString()
      or[orNames[i]] = o
      break
    }
    if (Object.keys(or).length === 0) return lpError('No matching or tokens found', lp)
    return new NamedOr(t, or, filename, line, char)
  }
}

// A 'leaf' AST node that matches a character within the specified range of characters. Useful for
// building regex-like matchers.
export class CharSet implements LPNode {
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

// A composite AST 'node' that matches the child node between the minimum and maximum repetitions or
// fails.
export const RangeSet = (toRepeat: LPNode, min: number, max: number): LPNode | Error => {
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