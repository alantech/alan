import { LPNode } from '../lp';
import Scope from './Scope';
import Type from './Types';

/*
Ugh, I forgot about this class, but it should be pretty easy to do.
I think all that needs to be done is to convert the `assignablesAst`
into an `Expr`. What most compilers then do is effectively "copy-paste"
the Expr into wherever it's referenced. However, I've seen the GH
issue that mentions the desire to do something akin to Rust's `const fn`
stuff (ie, evaluating expressions at compile time). For that, there
should probably be work here to expand on this idea, with a potential
`class ConstRef extends Expr` to be able to reference the result of
those constant values.
*/
export default class Const {
  name: string;
  ty: Type;
  assignablesAst: LPNode;

  constructor(name: string, ty: Type | null, assignablesAst: LPNode) {
    this.name = name;
    this.ty = ty !== null ? ty : Type.generate();
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
