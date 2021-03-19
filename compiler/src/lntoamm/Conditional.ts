import { v4 as uuid } from 'uuid';
import { Args, Fn} from './Function';
import * as Ast from './Ast';
import Microstatement from "./Microstatement";
import Scope from './Scope';
import Statement from "./Statement";
import StatementType from './StatementType';
import Type from './Type';

export const determineEvalCondReturn = (microstatements: Microstatement[], scope: Scope, interfaceMap?: Map<Type, Type>): [Type, boolean] => {
  const opcodeScope = require('./opcodes').default.exportScope;
  const MaybeVoid: Type = Type.builtinTypes.Maybe.solidify([Type.builtinTypes.void.typename], opcodeScope)
  // defaults, since there won't be an `ENTERFN` or an `ENTERCONDFN` at the top level of a handler
  let retTy = MaybeVoid; // handlers return `void` always
  let isUnwrap = false; // handlers don't care about the void value
  for (let ii = microstatements.length - 1; ii >= 0; ii--) {
    const m = microstatements[ii];
    // if the first thing we come across is an ENTERFN, that means that we're in a function,
    // and we need to mirror the return value of the function, except wrapped in a Maybe. if
    // it's an ENTERCONDFN, then we're in a nested conditional, and we need to keep whatever
    // the return value is as-is (except maybe wrapping it in `Some`, that's up to the
    // Microstatement.fromConditionalsAst to implement).
    // note that an ENTERFN should only exist for the containing function - for other functions
    // that happen to be in the same containing function, the ENTERFN should be deleted.
    if (m.statementType === StatementType.ENTERFN) {
      // the return type is a Maybe-wrapped value of the return type of the function
      retTy = Type.builtinTypes.Maybe.solidify([m.closureOutputType.typename], scope);
      // if the return type is `Maybe<void>`, then we don't really care about the return value,
      // it's just there to ensure that things compile and run correctly. The value doesn't
      // need to get unwrapped, as it shouldn't get used anywhere else anyways.
      isUnwrap = !(MaybeVoid.typeApplies(retTy, scope, interfaceMap));
      break;
    } else if (m.statementType === StatementType.ENTERCONDFN) {
      // the type *should* already be wrapped in a Maybe
      retTy = m.closureOutputType
      break;
    }
  }
  return [retTy, isUnwrap];
}

export const handleTail = (
  microstatements: Microstatement[],
  tailed: Microstatement[],
  rest: Statement[],
) => {
  if (tailed.length === 0) return;
  const tail = tailed.shift();
  const isUnwrapReturn = tail.isUnwrapReturn;
  const tailFnName = '_' + uuid().replace(/-/g, '_');
  const closureMstmts: Microstatement[] = [...tailed];
  for (let next of rest) {
    Microstatement.fromStatement(next, closureMstmts, tail.scope);
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
    tail.outputType,
  ));
  // now fix and append the TAIL as a CONSTDEC (it's an evalcond or you can cut my legs and call me shorty)
  tail.statementType = StatementType.CONSTDEC;
  tail.inputNames.push(tailFnName);
  let retName = tail.outputName;
  microstatements.push(tail);
  // if we need to unwrap the tail, do so
  if (isUnwrapReturn) {
    const newName = '_' + uuid().replace(/-/g, '_');
    unwrapEvalcond.microstatementInlining([retName, newName], tail.scope, microstatements);
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

const unwrapEvalcond: Fn = {
  getName: () => 'getR', // use the getR opcode since it pretty much accomplishes exactly what we want
  getArguments: () => ({} as Args),
  getReturnType: () => undefined, // TODO: replace this once it can be consistently determined
  isPure: () => true,
  isUnwrapReturn: () => false,
  microstatementInlining: (realArgNames: string[], scope: Scope, microstatements: Microstatement[]) => {
    // assume that the previous microstatement is an evalcond call
    const evalCondRetTy = microstatements[microstatements.length - 1].outputType;
    const getRRetTy = evalCondRetTy.remappedGenerics.values().next().value as Type;
    microstatements.push(new Microstatement(
      StatementType.CONSTDEC,
      scope,
      true,
      realArgNames.pop(),
      getRRetTy,
      [...realArgNames],
      [unwrapEvalcond],
    ));
  },
};
