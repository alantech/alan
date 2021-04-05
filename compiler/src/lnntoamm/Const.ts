import { LPNode } from "../lp";
import Scope from "./Scope";
import { Type } from "./Types";

export default class Const {
  name: string
  ty: Type | null
  assignablesAst: LPNode

  constructor(
    name: string,
    ty: Type | null,
    assignablesAst: LPNode,
  ) {
    this.name = name;
    this.ty = ty;
    this.assignablesAst = assignablesAst;
  }

  static fromAst(ast: LPNode, scope: Scope): Const {
    const name = ast.get('variable').t.trim();
    let constTy = null;
    if (ast.get('typedec').has()) {
      // TODO: gonna have to support generics (just not yet)
      const tyName = ast.get('typedec').get().get('fulltypename').t.trim();
      const inScope = scope.get(tyName);
      if (!(inScope instanceof Type)) {
        throw new Error(`${tyName} is not a type`);
      }
      constTy = inScope as Type;
    }
    const assignablesAst = ast.get('assignables');
    return new Const(name, constTy, assignablesAst);
  }
}
