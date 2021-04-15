import { listenerCount } from 'process';
import { OpcodeFn } from './Fn';
import { Builtin } from './Types';
import { genName } from './util';

const INDENT = '  ';
// keep this since this compiler is WIP - once it's finished we shouldn't need it anymore, and in fact it'll hurt performance (barely? i think?)
const DEBUG_MODE_PRINTING = false;

export default class Output {
  private constants: Map<Builtin, {[val: string]: string}>
  private events: {[name: string]: Builtin}
  private handlers: string[]
  private indent: string

  constructor() {
    this.constants = new Map();
    this.events = {};
    this.handlers = [];
  }

  toString(): string {
    let res = '';
    for (let [ty, constants] of this.constants.entries()) {
      for (let constVal of Object.keys(constants)) {
        res = res.concat('const ', constants[constVal], ': ', ty.ammName, ' = ', constVal, '\n');
      }
    }
    for (let eventName of Object.keys(this.events)) {
      res = res.concat('event ', eventName, ': ', this.events[eventName].ammName, '\n');
    }
    for (let handler of this.handlers) {
      res = res.concat(handler);
    }
    return res;
  }

  global(
    kind: 'const' | 'event',
    ty: Builtin,
    val: string,
  ): string {
    if (kind === 'const') {
      let constants = this.constants.get(ty) || {};
      if (Object.keys(constants).findIndex(c => c === val) === -1) {
        constants[val] = genName();
        this.constants.set(ty, constants);
      }
      DEBUG_MODE_PRINTING && console.log('-> const', constants[val], ':', ty.ammName, '=', val);
      return constants[val];
    } else {
      if (this.events[val]) {
        throw new Error(`AMM can't handle multiple events of the same name`);
      }
      this.events[val] = ty;
      DEBUG_MODE_PRINTING && console.log('-> event', val, ':', ty.ammName);
      return val;
    }
  }

  addHandler(
    event: string,
    args: [string, Builtin][],
    retTy?: Builtin,
  ) {
    let line = 'on '.concat(event, ' fn (');
    for (let ii = 0; ii < args.length; ii++) {
      if (ii !== 0) {
        line = line.concat(', ');
      }
      line = line.concat(args[ii][0], ': ', args[ii][1].ammName);
    }
    line = line.concat('): ', retTy ? retTy.ammName : 'void', ' {');
    DEBUG_MODE_PRINTING && console.log(line);
    this.handlers.unshift(line.concat('\n'));
    this.indent = INDENT;
  }

  // made it as linear and DRY as possible :)
  assign(
    kind: '' | 'const' | 'let',
    name: string,
    ty: Builtin,
    assign: string | OpcodeFn,
    args: string[] | null = null,
  ) {
    let line = this.indent
    if (kind === '') {
      line = line.concat(name, ' = ');
    } else {
      line = line.concat(kind, ' ', name, ': ', ty.ammName, ' = ');
    }
    if (assign instanceof OpcodeFn) {
      line = line.concat(assign.name, '(');
      if (args === null) {
        throw new Error(`attempting to call opcode ${assign.name} but there are no args defined`)
      }
      for (let ii = 0; ii < args.length; ii++) {
        line = line.concat(args[ii]);
        if (ii !== args.length - 1) {
          line = line.concat(', ');
        }
      }
      line = line.concat(')');
    } else {
      let assignName = this.global('const', ty, assign);
      line = line.concat(assignName);
    }
    DEBUG_MODE_PRINTING && console.log(line);
    this.handlers[0] = this.handlers[0].concat(line.concat('\n'));
  }

  emit(
    eventName: string,
    val: string,
  ) {
    const line = this.indent.concat('emit ', eventName, ' ', val);
    DEBUG_MODE_PRINTING && console.log(line);
    this.handlers[0] = this.handlers[0].concat(line.concat('\n'));
  }

  exit(
    val: string | null = null,
  ) {
    if (val !== null) {
      const line = this.indent.concat('return ', val, '\n');
      DEBUG_MODE_PRINTING && console.log(line);
      this.handlers[0] = this.handlers[0].concat(line);
    }
    // only replace the first newline with nothing
    this.indent = this.indent.replace(/  /, '');
    const line = this.indent.concat('}');
    DEBUG_MODE_PRINTING && console.log(line);
    this.handlers[0] = this.handlers[0].concat(line.concat('\n'));
  }
}
