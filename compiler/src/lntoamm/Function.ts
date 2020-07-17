import Microstatement from './Microstatement'
import Scope from './Scope'
import Type from './Type'

export type Args = {
  [K: string]: Type
}

export interface Fn {
  getName(): string
  getArguments(): Args
  getReturnType(): Type
  isPure(): boolean
  microstatementInlining(
    realArgNames: Array<string>,
    scope: Scope,
    microstatements: Array<Microstatement>
  ): void
}

export default Fn