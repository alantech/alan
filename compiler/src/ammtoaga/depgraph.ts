import { LPNode } from '../lp';

// this is just here for debugging purposes
const unhandled = (val: any, reason?: string) => {
  console.error(`========== UNHANDLED: ${reason}`)
  console.error(val)
  console.error()
  throw new Error()
}

export class DepGraph {
  /**
   * Goes by order of the statements in the AMM input. Note that this ordering
   * is not preserved in the AGA output, so this can't be used for AGA dependencies.
   */
  byOrder: DepNode[]
  /**
   * The keys are the AMM variable name. Multiple nodes can be assigned to a given key,
   * since a variable can be assigned to or mutated multiple times. Note that I said
   * "mutated" - all mutations are inserted into the array, with the initial declaration
   * (or in-scope assignment/mutation) being the first value in the list.
   */
  byVar: {[varname: string]: DepNode[]}
  /**
   * Uses the `LPNode` as the key to a `DepNode`. Note that this does not act
   * recursively - it'll only work for the top-level nodes of the current handler
   * or function.
   */
  byLP: Map<LPNode, DepNode>
  outerGraph?: DepGraph
  outerDeps: DepNode[]
  outerMuts: string[]

  constructor(fn?: LPNode, outer?: DepGraph) {
    this.byOrder = []
    this.byVar = {}
    this.byLP = new Map()
    this.outerGraph = outer || null
    this.outerDeps = []
    this.outerMuts = []

    if (fn) {
      let stmts = fn.get('functions')
        .get('functionbody')
        .get('statements').getAll()
        .filter(s => !s.has('whitespace'))
        .filter(s => !s.has('exits'))
      this.build(stmts)
    }
  }

  build(stmts: LPNode[]) {
    for (let stmt of stmts) {
      let node = new DepNode(stmt, this)
      // console.log(`mutates:`)
      // console.log(node.mutates)
      for (let mutated of node.mutates) {
        if (this.outerGraph !== null && this.outerGraph.getLastMutationFor(mutated) !== null) {
          this.outerMuts.push(mutated)
        }

        if (this.byVar[mutated] === null || this.byVar[mutated] === undefined) {
          this.byVar[mutated] = []
        }
        this.byVar[mutated].push(node)
      }
      this.byOrder.push(node)
      this.byLP.set(stmt, node)
    }
    this.outerDeps = [ ...new Set(this.outerDeps) ]
  }

  getLastMutationFor(varName: string): DepNode {
    let nodes = this.byVar[varName]
    // console.log(`------- ${varName}`)
    // console.log('nodes:')
    // console.log(nodes)
    if ((nodes !== null && nodes !== undefined) && nodes.length !== 0) {
      return nodes[nodes.length - 1]
    }

    // console.log('og:')
    // console.log(this.outerGraph)
    if (this.outerGraph !== null) {
      let outer = this.outerGraph.getLastMutationFor(varName)
      if (outer !== null) {
        // console.log(`found outer for ${varName}`)
        this.outerDeps.push(outer)
        return outer
      }
    }

    return null
  }

  toJSON(): object {
    return {
      byOrder: this.byOrder.map(n => n.toJSON()),
      byVar: Object.keys(this.byVar),
      byLP: this.byLP.size,
      outerGraph: this.outerGraph !== null,
      outerDeps: this.outerDeps.map(n => n.toJSON()),
      outerMuts: this.outerMuts,
    }
  }
}

export class DepNode {
  stmt: string
  upstream: DepNode[]
  downstream: DepNode[]
  closure?: DepGraph
  mutates: string[]
  graph: DepGraph

  constructor(stmt: LPNode, graph: DepGraph) {
    this.stmt = stmt.t.trim()
    this.upstream = []
    this.downstream = []
    this.closure = null
    this.mutates = []
    this.graph = graph

    if (stmt.has('declarations')) {
      let dec = stmt.get('declarations')
      if (dec.has('constdeclaration')) {
        dec = dec.get('constdeclaration')
      } else if (dec.has('letdeclaration')) {
        dec = dec.get('letdeclaration')
      } else {
        unhandled(dec, 'dec kind')
      }
      this.fromAssignment(dec)
    } else if (stmt.has('assignments')) {
      this.fromAssignment(stmt.get('assignments'))
    } else if (stmt.has('calls')) {
      this.fromCall(stmt.get('calls'))
    } else if (stmt.has('emits')) {
      let upstream = graph.getLastMutationFor(stmt.get('emits').get('value').t.trim())
      if (upstream) {
        this.upstream.push(upstream)
        upstream.downstream.push(this)
      }
    } else {
      unhandled(stmt, 'node top-level')
    }
    this.upstream = [ ...new Set(this.upstream) ]
    this.mutates = [ ...new Set(this.mutates) ]
  }

  fromAssignment(assign: LPNode) {
    // console.log(assign)
    if (!assign.has('assignables')) {
      unhandled(assign, 'non-assignment assignment?')
    }

    let decname = assign.get('decname').t.trim()
    // console.log(`decname: ${decname}`)
    let prev = this.graph.getLastMutationFor(decname)
    if (prev !== null) {
      this.upstream.push(prev)
    }

    this.mutates.push(decname)
    if (assign.get('fulltypename').t.trim() === 'function') {
      this.closure = new DepGraph(assign.get('assignables'), this.graph)
      // for closures, only add upstream since the closure isn't actually
      // evaluated until its called. this just makes it so that the actual
      // use-site of the closure can inherit the upstream dependencies.
      this.upstream.push(...this.closure.outerDeps)
      this.mutates.push(...this.closure.outerMuts)
    } else if (assign.has('assignables')) {
      if (prev !== null) {
        prev.downstream.push(this)
      }
      assign = assign.get('assignables')
      if (assign.has('calls')) {
        this.fromCall(assign.get('calls'))
      } else if (assign.has('value')) {
        // do nothing
      } else {
        unhandled(assign, 'assignable')
      }
    } else {
      unhandled(assign, 'non-assignable... assignable... ?')
    }
  }

  fromCall(call: LPNode) {
    let opcodeName = call.get('variable').t.trim()
    let args = call.get('calllist').getAll().map(c => c.get('variable'))
    let mutated = []
    let opMutability = opcodeParamMutabilities[opcodeName]
    if (opMutability === undefined || opMutability === null) {
      unhandled(opMutability, 'opcode ' + opcodeName)
    }
    for (let ii = 0; ii < opMutability.length; ii++) {
      if (opMutability[ii] === true) {
        mutated.push(args[ii].t.trim())
      } else if (opMutability[ii] === null) {
        // null indicates that the parameter expects a closure,
        // so the mutability of the overall call depends on the
        // mutability of the specified closure. Because of this,
        // we have to grab the node for the closure declaration
        // and use its mutabilities instead

        // the closure def will be the first node in the list
        let closure = this.graph.byVar[args[ii].t.trim()][0]
        if (closure === null || closure === undefined) {
          unhandled(this.graph.byVar, `no nodes declared for ${args[ii].t.trim()}`)
        } else if (closure.closure === null) {
          unhandled(closure, 'expected a closure')
        }
        mutated.push(...closure.mutates)
      }
    }
    this.mutates.push(...mutated)
    // console.log('---')
    // console.log(this.stmt)
    for (let arg of args) {
      // console.log(arg)
      let upstream = this.graph.getLastMutationFor(arg.t.trim())
      // console.log(upstream)
      if (upstream !== null) {
        if (upstream.closure !== null) {
          // if it's a closure, inherit the upstreams
          this.upstream.push(...upstream.upstream)
        } else {
          this.upstream.push(upstream)
          upstream.downstream.push(this)
        }
      }
    }
  }

  toJSON(): object {
    let closure = null;
    if (this.closure) closure = this.closure.toJSON()
    return {
      stmt: this.stmt.replace(/\n/g, '\\n'),
      upstream: this.upstream.length,
      downstream: this.downstream.length,
      closure: this.closure,
      mutates: this.mutates,
    }
  }
}

export const opcodeParamMutabilities = {
  i8f64: [false],
  i16f64: [false],
  i32f64: [false],
  i64f64: [false],
  f32f64: [false],
  strf64: [false],
  boolf64: [false],
  i8f32: [false],
  i16f32: [false],
  i32f32: [false],
  i64f32: [false],
  f64f32: [false],
  strf32: [false],
  boolf32: [false],
  i8i64: [false],
  i16i64: [false],
  i32i64: [false],
  f32i64: [false],
  f64i64: [false],
  stri64: [false],
  booli64: [false],
  i8i32: [false],
  i16i32: [false],
  i64i32: [false],
  f32i32: [false],
  f64i32: [false],
  stri32: [false],
  booli32: [false],
  i8i16: [false],
  i32i16: [false],
  i64i16: [false],
  f32i16: [false],
  f64i16: [false],
  stri16: [false],
  booli16: [false],
  i16i8: [false],
  i32i8: [false],
  i64i8: [false],
  f32i8: [false],
  f64i8: [false],
  stri8: [false],
  booli8: [false],
  i8bool: [false],
  i16bool: [false],
  i32bool: [false],
  i64bool: [false],
  f32bool: [false],
  f64bool: [false],
  strbool: [false],
  i8str: [false],
  i16str: [false],
  i32str: [false],
  i64str: [false],
  f32str: [false],
  f64str: [false],
  boolstr: [false],
  addi8: [false, false],
  addi16: [false, false],
  addi32: [false, false],
  addi64: [false, false],
  addf32: [false, false],
  addf64: [false, false],
  subi8: [false, false],
  subi16: [false, false],
  subi32: [false, false],
  subi64: [false, false],
  subf32: [false, false],
  subf64: [false, false],
  negi8: [false, false],
  negi16: [false, false],
  negi32: [false, false],
  negi64: [false, false],
  negf32: [false, false],
  negf64: [false, false],
  absi8: [false, false],
  absi16: [false, false],
  absi32: [false, false],
  absi64: [false, false],
  absf32: [false, false],
  absf64: [false, false],
  muli8: [false, false],
  muli16: [false, false],
  muli32: [false, false],
  muli64: [false, false],
  mulf32: [false, false],
  mulf64: [false, false],
  divi8: [false, false],
  divi16: [false, false],
  divi32: [false, false],
  divi64: [false, false],
  divf32: [false, false],
  divf64: [false, false],
  modi8: [false, false],
  modi16: [false, false],
  modi32: [false, false],
  modi64: [false, false],
  powi8: [false, false],
  powi16: [false, false],
  powi32: [false, false],
  powi64: [false, false],
  powf32: [false, false],
  powf64: [false, false],
  sqrtf32: [false, false],
  sqrtf64: [false, false],
  andi8: [false, false],
  andi16: [false, false],
  andi32: [false, false],
  andi64: [false, false],
  andbool: [false, false],
  ori8: [false, false],
  ori16: [false, false],
  ori32: [false, false],
  ori64: [false, false],
  orbool: [false, false],
  xori8: [false, false],
  xori16: [false, false],
  xori32: [false, false],
  xori64: [false, false],
  xorbool: [false, false],
  noti8: [false, false],
  noti16: [false, false],
  noti32: [false, false],
  noti64: [false, false],
  notbool: [false, false],
  nandi8: [false, false],
  nandi16: [false, false],
  nandi32: [false, false],
  nandi64: [false, false],
  nandboo: [false, false],
  nori8: [false, false],
  nori16: [false, false],
  nori32: [false, false],
  nori64: [false, false],
  norbool: [false, false],
  xnori8: [false, false],
  xnori16: [false, false],
  xnori32: [false, false],
  xnori64: [false, false],
  xnorboo: [false, false],
  eqi8: [false, false],
  eqi16: [false, false],
  eqi32: [false, false],
  eqi64: [false, false],
  eqf32: [false, false],
  eqf64: [false, false],
  eqbool: [false, false],
  eqstr: [false, false],
  neqi8: [false, false],
  neqi16: [false, false],
  neqi32: [false, false],
  neqi64: [false, false],
  neqf32: [false, false],
  neqf64: [false, false],
  neqbool: [false, false],
  neqstr: [false, false],
  lti8: [false, false],
  lti16: [false, false],
  lti32: [false, false],
  lti64: [false, false],
  ltf32: [false, false],
  ltf64: [false, false],
  ltstr: [false, false],
  ltei8: [false, false],
  ltei16: [false, false],
  ltei32: [false, false],
  ltei64: [false, false],
  ltef32: [false, false],
  ltef64: [false, false],
  ltestr: [false, false],
  gti8: [false, false],
  gti16: [false, false],
  gti32: [false, false],
  gti64: [false, false],
  gtf32: [false, false],
  gtf64: [false, false],
  gtstr: [false, false],
  gtei8: [false, false],
  gtei16: [false, false],
  gtei32: [false, false],
  gtei64: [false, false],
  gtef32: [false, false],
  gtef64: [false, false],
  gtestr: [false, false],
  httpget: [false],
  httppost: [false],
  httplsn: [false],
  httpsend: [false],
  execop: [false],
  waitop: [false],
  catstr: [false, false],
  catarr: [false, false],
  split: [false, false],
  repstr: [false, false],
  reparr: [false, false],
  matches: [false, false],
  indstr: [false, false],
  indarrf: [false, false],
  indarrv: [false, false],
  lenstr: [false],
  lenarr: [false],
  trim: [false],
  condfn: [false, null],
  pusharr: [true, false, false],
  poparr: [true],
  delindx: [true, false],
  each: [false, null],
  eachl: [false, null],
  map: [false, null],
  mapl: [false, null],
  reducel: [false, null],
  reducep: [false, null],
  foldl: [false, null],
  foldp: [false, null],
  filter: [false, null],
  filterl: [false, null],
  find: [false, null],
  findl: [false, null],
  every: [false, null],
  everyl: [false, null],
  some: [false, null],
  somel: [false, null],
  join: [false, false],
  newarr: [false],
  stdoutp: [false],
  stderrp: [false],
  exitop: [false],
  copyfrom: [false, false],
  copytof: [true, false, false],
  copytov: [true, false, false],
  register: [false, false],
  copyi8: [false],
  copyi16: [false],
  copyi32: [false],
  copyi64: [false],
  copyvoid: [false],
  copyf32: [false],
  copyf64: [false],
  copybool: [false],
  copystr: [false],
  copyarr: [false],
  zeroed: [],
  lnf64: [false],
  logf64: [false],
  sinf64: [false],
  cosf64: [false],
  tanf64: [false],
  asinf64: [false],
  acosf64: [false],
  atanf64: [false],
  sinhf64: [false],
  coshf64: [false],
  tanhf64: [false],
  error: [false],
  reff: [false],
  refv: [false],
  noerr: [],
  errorstr: [false],
  someM: [false, false],
  noneM: [],

  // TODO: RFC-12 might impact these:
  isSome: [false],
  isNone: [false],
  getOrM: [false, false],
  okR: [false, false],
  err: [false],
  isOk: [false],
  isErr: [false],
  getOrR: [false, false],
  getOrRS: [false, false],
  getR: [false],
  getErr: [false, false],
  resfrom: [false, false],
  mainE: [false, false],
  altE: [false, false],
  isMain: [false],
  isAlt: [false],
  mainOr: [false, false],
  altOr: [false, false],

  hashf: [false],
  hashv: [false],
  dssetf: [true, false, false], // TODO: is this right? should i be marking the DS name as being mutated?
  dssetv: [true, false, false],
  dshas: [false, false],
  dsdel: [true, false],
  dsgetf: [false, false],
  dsgetv: [false, false],
  newseq: [false],
  seqnext: [false],
  seqeach: [false, null],
  seqwhile: [false, null, null], // TODO: ok so i don't *want* to make the 2nd value `null`, but it's not impossible for someone to mutate a value in the second function...
  seqdo: [false, null],
  selfrec: [null, null], // TODO: figure this out. maybe just mark both as false??
  seqrec: [false, null],
}