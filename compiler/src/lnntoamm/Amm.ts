import { listenerCount } from 'process';
import { OpcodeFn } from './Fn';
import { Builtin } from './Types';
import { genName } from './util';

const INDENT = '  ';

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
      return constants[val];
    } else {
      if (this.events[val]) {
        throw new Error(`AMM can't handle multiple events of the same name`);
      }
      this.events[val] = ty;
      return val;
    }
  }

  addHandler(event: string, arg: [string, Builtin]) {
    this.handlers.unshift('on '.concat(event, ' fn (', arg[0], ': ', arg[1].ammName, '): void {'));
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
    if (kind !== '') {
      line.concat(kind, ' ');
    }
    line = line.concat(name, ': ', ty.ammName, ' = ');
    if (assign instanceof OpcodeFn) {
      line.concat(assign.name, '(');
      if (args === null) {
        throw new Error(`attempting to call opcode ${assign.name} but there are no args defined`)
      }
      for (let ii = 0; ii < args.length; ii++) {
        line.concat(args[ii]);
        if (ii !== args.length - 1) {
          line.concat(', ');
        }
      }
    } else {
      let constants = this.constants.get(ty) || {};
      if (Object.keys(constants).findIndex(c => c === assign) === -1) {
        constants[assign] = genName();
        this.constants.set(ty, constants);
      }
      let assignName = constants[assign];
      line.concat(assignName);
    }
    this.handlers[0].concat(line.concat('\n'));
  }

  emit(
    eventName: string,
    val: string,
  ) {
    this.handlers[0].concat(this.indent, 'emit ', eventName, ' ', val, '\n');
  }

  return(
    val: string | null = null,
  ) {
    if (val !== null) {
      this.handlers[0].concat(this.indent, 'return ', val, '\n');
    }
    // only replace the first newline with nothing
    this.indent.replace(/\n/, '');
    this.handlers[0].concat(this.indent, '}');
  }
}
