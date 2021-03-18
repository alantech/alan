import { v4 as uuid } from 'uuid';
import { Args, Fn} from './Function';
import Microstatement from "./Microstatement";
import Scope from './Scope';
import Statement from "./Statement";
import StatementType from './StatementType';
import Type from './Type';

export const isNested = (microstatements: Microstatement[]): boolean => {
  for (let ii = microstatements.length - 1; ii >= 0; ii--) {
    const m = microstatements[ii];
    if (m.statementType === StatementType.ENTERFN) {
      break;
    } else if (m.statementType === StatementType.ENTERCONDFN) {
      return true;
    }
  }
  return false;
}

export const handleTail = (
  microstatements: Microstatement[],
  tailed: Microstatement[],
  rest: Statement[],
) => {
  if (tailed.length === 0) return;
  const isNest = isNested(microstatements);
  const tail = tailed.shift();
  const tailFnName = '_' + uuid().replace(/-/g, '_');
  const closureMstmts: Microstatement[] = [...tailed];
  for (let next of rest) {
    Microstatement.fromStatement(next, closureMstmts, tail.scope);
  }
  // if we absolutely need a return value, insert a void return
  if (!isNest && (closureMstmts.length === 0 || (closureMstmts[closureMstmts.length - 1].statementType !== StatementType.EXIT))) {
    returnVoid(closureMstmts, tail.scope);
  }
  // insert the tail function definition
  microstatements.push(new Microstatement(
    StatementType.CLOSURE,
    tail.scope,
    undefined,
    tailFnName,
    undefined,
    undefined,
    undefined,
    undefined,
    false, // TODO: determine if true
    closureMstmts,
    {},
    Type.builtinTypes.void, // TODO: figure out how to consistently determine this
  ));
  // now fix and append the TAIL as a CONSTDEC
  tail.statementType = StatementType.CONSTDEC;
  tail.inputNames.push(tailFnName);
  tail.outputType = Type.builtinTypes.void; // TODO: figure out how to consistently determine this
  const isUnwrapReturn = tail.isUnwrapReturn; // TODO: determine if this is even necessary, or if it should replace isNest
  let retName = tail.outputName;
  microstatements.push(tail);
  // if we need to unwrap the tail, do so
  if (isUnwrapReturn) {
    const newName = '_' + uuid().replace(/-/g, '_');
    unwrapCondval.microstatementInlining([retName, newName], tail.scope, microstatements);
    retName = newName;
  }
  microstatements.push(new Microstatement(
    StatementType.EXIT,
    tail.scope,
    true,
    retName,
  ))
}

const returnVoid = (microstatements: Microstatement[], scope: Scope) => {
  const retName = '_' + uuid().replace(/-/g, '_');
  getVoid.microstatementInlining([retName], scope, microstatements);
  // can't just delegate to Microstatement.fromExitsAst because it doesn't guarantee
  // that it'll actually insert an EXIT
  microstatements.push(new Microstatement(
    StatementType.EXIT,
    scope,
    true,
    retName,
  ));
}

export const getVoid: Fn = {
  getName: () => 'noneM',
  getArguments: () => ({} as Args),
  getReturnType: () => Type.builtinTypes.void,
  isPure: () => true,
  isUnwrapReturn: () => false,
  microstatementInlining: (realArgNames: string[], scope: Scope, microstatements: Microstatement[]) => {
    microstatements.push(new Microstatement(
      StatementType.CONSTDEC,
      scope,
      true,
      realArgNames.shift(),
      Type.builtinTypes.void,
      [],
      [getVoid],
    ));
  },
};

const unwrapCondval: Fn = {
  getName: () => 'getR', // use the getR opcode since it pretty much accomplishes exactly what we want
  getArguments: () => ({} as Args),
  getReturnType: () => undefined, // TODO: replace this once it can be consistently determined
  isPure: () => true,
  isUnwrapReturn: () => false,
  microstatementInlining: (realArgNames: string[], scope: Scope, microstatements: Microstatement[]) => {
    microstatements.push(new Microstatement(
      StatementType.CONSTDEC,
      scope,
      true,
      realArgNames.pop(),
      undefined, // TODO: replace this once it can be consistently determined
      [...realArgNames],
      [unwrapCondval],
    ));
  },
};
