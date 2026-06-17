use crate::parse::*;

pub trait Render {
    fn render(&self) -> String;
}

impl<T: Render> Render for Vec<T> {
    fn render(&self) -> String {
        self.iter().map(|t| t.render()).collect()
    }
}

impl<T: Render> Render for Option<T> {
    fn render(&self) -> String {
        match self {
            Some(t) => t.render(),
            None => String::new(),
        }
    }
}

impl<T: Render> Render for Box<T> {
    fn render(&self) -> String {
        self.as_ref().render()
    }
}

impl Render for Ln {
    fn render(&self) -> String {
        format!("{}{}", self.a, self.body.render())
    }
}

impl Render for RootElements {
    fn render(&self) -> String {
        match self {
            RootElements::Whitespace(s) => s.clone(),
            RootElements::Exports(e) => e.render(),
            RootElements::OperatorMapping(o) => o.render(),
            RootElements::TypeOperatorMapping(t) => t.render(),
            RootElements::Functions(f) => f.render(),
            RootElements::Types(t) => t.render(),
            RootElements::CTypes(c) => c.render(),
            RootElements::CFns(c) => c.render(),
            RootElements::ConstDeclaration(c) => c.render(),
            RootElements::Interfaces(i) => i.render(),
        }
    }
}

impl Render for Exports {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}",
            self.export,
            self.a,
            self.opttypegenerics.render(),
            self.b,
            self.exportable.render(),
        )
    }
}

impl Render for Exportable {
    fn render(&self) -> String {
        match self {
            Exportable::OperatorMapping(o) => o.render(),
            Exportable::TypeOperatorMapping(t) => t.render(),
            Exportable::Functions(f) => f.render(),
            Exportable::ConstDeclaration(c) => c.render(),
            Exportable::Types(t) => t.render(),
            Exportable::Intefaces(i) => i.render(),
            Exportable::Ref(s) => s.clone(),
        }
    }
}

impl Render for OperatorMapping {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}{}",
            self.fix.render(),
            self.a,
            self.opttypegenerics.render(),
            self.blank,
            self.opmap.render(),
            self.optsemicolon,
        )
    }
}

impl Render for Fix {
    fn render(&self) -> String {
        match self {
            Fix::Prefix(s) => s.clone(),
            Fix::Infix(s) => s.clone(),
            Fix::Postfix(s) => s.clone(),
        }
    }
}

impl Render for OpMap {
    fn render(&self) -> String {
        match self {
            OpMap::FnOpPrecedence(fop) => fop.render(),
            OpMap::PrecedenceFnOp(pfo) => pfo.render(),
        }
    }
}

impl Render for FnOpPrecedence {
    fn render(&self) -> String {
        format!("{}{}{}", self.fntoop.render(), self.blank, self.opprecedence.render())
    }
}

impl Render for PrecedenceFnOp {
    fn render(&self) -> String {
        format!("{}{}{}", self.opprecedence.render(), self.blank, self.fntoop.render())
    }
}

impl Render for FnToOp {
    fn render(&self) -> String {
        format!("{}{}{}{}{}", self.fnname, self.a, self.asn, self.b, self.operator)
    }
}

impl Render for OpPrecedence {
    fn render(&self) -> String {
        format!("{}{}{}", self.precedence, self.blank, self.num)
    }
}

impl Render for TypeOperatorMapping {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}",
            self.typen,
            self.a,
            self.fix.render(),
            self.b,
            self.opttypegenerics.render(),
            self.blank,
            self.opmap.render(),
            self.optsemicolon,
        )
    }
}

impl Render for TypeOpMap {
    fn render(&self) -> String {
        match self {
            TypeOpMap::FnOpPrecedence(fop) => fop.render(),
            TypeOpMap::PrecedenceFnOp(pfo) => pfo.render(),
        }
    }
}

impl Render for TypeFnOpPrecedence {
    fn render(&self) -> String {
        format!("{}{}{}", self.fntoop.render(), self.blank, self.opprecedence.render())
    }
}

impl Render for TypePrecedenceFnOp {
    fn render(&self) -> String {
        format!("{}{}{}", self.opprecedence.render(), self.blank, self.fntoop.render())
    }
}

impl Render for TypeFnToOp {
    fn render(&self) -> String {
        format!("{}{}{}{}{}", self.fnname, self.a, self.asn, self.b, self.operator)
    }
}

impl Render for Functions {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}{}{}{}",
            self.fnn,
            self.a,
            self.opttypegenerics.render(),
            self.b,
            self.optname.as_deref().unwrap_or(""),
            self.c,
            self.optgenerics.render(),
            self.d,
            self.opttype.render(),
            self.e,
            self.fullfunctionbody.render(),
        )
    }
}

impl Render for FullFunctionBody {
    fn render(&self) -> String {
        match self {
            FullFunctionBody::FunctionBody(fb) => fb.render(),
            FullFunctionBody::AssignFunction(af) => af.render(),
            FullFunctionBody::DecOnly(s) => s.clone(),
        }
    }
}

impl Render for FunctionBody {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}",
            self.opencurly,
            self.a,
            self.statements.render(),
            self.b,
            self.closecurly,
        )
    }
}

impl Render for AssignFunction {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}",
            self.eq,
            self.a,
            self.assignables.render(),
            self.b,
        )
    }
}

impl Render for Statement {
    fn render(&self) -> String {
        match self {
            Statement::Declarations(d) => d.render(),
            Statement::Returns(r) => r.render(),
            Statement::Conditional(c) => c.render(),
            Statement::ArrayAssignment(a) => a.render(),
            Statement::Assignables(a) => a.render(),
            Statement::A(s) => s.clone(),
        }
    }
}

impl Render for Declarations {
    fn render(&self) -> String {
        match self {
            Declarations::Const(c) => c.render(),
            Declarations::Let(l) => l.render(),
        }
    }
}

impl Render for ConstDeclaration {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}{}{}{}{}",
            self.constn,
            self.a,
            self.opttypegenerics.render(),
            self.whitespace,
            self.variable,
            self.b,
            self.typedec.render(),
            self.c,
            self.eq,
            self.d,
            self.assignables.render(),
            self.semicolon,
        )
    }
}

impl Render for LetDeclaration {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}{}{}{}",
            self.letn,
            self.a,
            self.opttypegenerics.render(),
            self.whitespace,
            self.variable,
            self.b,
            self.typedec.render(),
            self.eq,
            self.c,
            self.assignables.render(),
            self.semicolon,
        )
    }
}

impl Render for TypeDec {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}",
            self.colon,
            self.a,
            self.fulltypename.render(),
            self.b,
        )
    }
}

impl Render for RetVal {
    fn render(&self) -> String {
        format!("{}{}", self.assignables.render(), self.a)
    }
}

impl Render for Returns {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}",
            self.returnn,
            self.a,
            self.retval.render(),
            self.semicolon,
        )
    }
}

impl Render for Conditional {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}{}",
            self.ifn,
            self.a,
            self.assignables.render(),
            self.b,
            self.blocklike.render(),
            self.optelsebranch.render(),
        )
    }
}

impl Render for Blocklike {
    fn render(&self) -> String {
        match self {
            Blocklike::Functions(f) => f.render(),
            Blocklike::FunctionBody(fb) => fb.render(),
        }
    }
}

impl Render for CondOrBlock {
    fn render(&self) -> String {
        match self {
            CondOrBlock::Conditional(c) => c.render(),
            CondOrBlock::Blocklike(b) => b.render(),
        }
    }
}

impl Render for ElseBranch {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}",
            self.a,
            self.elsen,
            self.b,
            self.condorblock.render(),
        )
    }
}

impl Render for AssignableStatement {
    fn render(&self) -> String {
        format!("{}{}", self.assignables.render(), self.semicolon)
    }
}

impl Render for ArrayAssignment {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}",
            self.name.render(),
            self.a,
            self.array.render(),
            self.b,
            self.eq,
            self.c,
            self.assignables.render(),
            self.semicolon,
        )
    }
}

impl Render for Types {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}",
            self.typen,
            self.a,
            self.opttypegenerics.render(),
            self.b,
            self.fulltypename.render(),
            self.c,
            self.typedef.render(),
            self.optsemicolon,
        )
    }
}

impl Render for TypeDef {
    fn render(&self) -> String {
        format!(
            "{}{}{}",
            self.a.as_deref().unwrap_or(""),
            self.b,
            self.typeassignables.render(),
        )
    }
}

impl Render for CTypes {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}",
            self.ctype,
            self.a,
            self.name,
            self.b,
            self.opttypegenerics.render(),
            self.c,
            self.optsemicolon,
        )
    }
}

impl Render for CFns {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}{}",
            self.cfn,
            self.a,
            self.name,
            self.b,
            self.opttypegenerics.render(),
            self.c,
            self.typesignature.render(),
            self.d,
            self.semicolon,
        )
    }
}

impl Render for Interfaces {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}",
            self.interface,
            self.a,
            self.opttypegenerics.render(),
            self.b,
            self.variable,
            self.c,
            self.interfacedef.render(),
        )
    }
}

impl Render for InterfaceDef {
    fn render(&self) -> String {
        match self {
            InterfaceDef::InterfaceBody(ib) => ib.render(),
            InterfaceDef::InterfaceAlias(ia) => ia.render(),
        }
    }
}

impl Render for InterfaceBody {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}{}",
            self.opencurly,
            self.a,
            self.interfacelist.render(),
            self.b,
            self.c,
            self.closecurly,
        )
    }
}

impl Render for InterfaceAlias {
    fn render(&self) -> String {
        format!("{}{}{}", self.eq, self.blank, self.variable)
    }
}

impl Render for InterfaceLine {
    fn render(&self) -> String {
        match self {
            InterfaceLine::FunctionTypeline(ft) => ft.render(),
        }
    }
}

impl Render for FunctionTypeline {
    fn render(&self) -> String {
        format!("{}{}{}", self.variable, self.a, self.functiontype.render())
    }
}

impl Render for WithOperators {
    fn render(&self) -> String {
        match self {
            WithOperators::BaseAssignableList(bal) => bal.render(),
            WithOperators::Operators(s) => s.clone(),
        }
    }
}

impl Render for BaseAssignable {
    fn render(&self) -> String {
        match self {
            BaseAssignable::Functions(f) => f.render(),
            BaseAssignable::FnCall(fc) => fc.render(),
            BaseAssignable::GnCall(gc) => gc.render(),
            BaseAssignable::Array(a) => a.render(),
            BaseAssignable::Variable(s) => s.clone(),
            BaseAssignable::MethodSep(s) => s.clone(),
            BaseAssignable::Constants(c) => c.render(),
        }
    }
}

impl Render for Constants {
    fn render(&self) -> String {
        match self {
            Constants::Bool(s) => s.clone(),
            Constants::Num(n) => n.render(),
            Constants::Strn(s) => s.clone(),
        }
    }
}

impl Render for Number {
    fn render(&self) -> String {
        match self {
            Number::RealNum(s) => s.clone(),
            Number::IntNum(s) => s.clone(),
        }
    }
}

impl Render for FnCall {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}",
            self.openparen,
            self.a,
            self.assignablelist.render(),
            self.b,
            self.closeparen,
        )
    }
}

impl Render for ArrayBase {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}{}",
            self.openarr,
            self.a,
            self.assignablelist.render(),
            self.b,
            self.c,
            self.closearr,
        )
    }
}

impl Render for AssignableList {
    fn render(&self) -> String {
        let mut out = String::new();
        for (i, element) in self.elements.iter().enumerate() {
            out.push_str(&element.render());
            if i < self.separators.len() {
                out.push_str(&self.separators[i]);
            }
        }
        out
    }
}

impl Render for WithTypeOperators {
    fn render(&self) -> String {
        match self {
            WithTypeOperators::TypeBaseList(tbl) => tbl.render(),
            WithTypeOperators::Operators(o) => o.render(),
        }
    }
}

impl Render for TypeOperatorsWithWhitespace {
    fn render(&self) -> String {
        format!("{}{}{}", self.a, self.op, self.b)
    }
}

impl Render for TypeBase {
    fn render(&self) -> String {
        match self {
            TypeBase::GnCall(gc) => gc.render(),
            TypeBase::TypeGroup(tg) => tg.render(),
            TypeBase::Constants(c) => c.render(),
            TypeBase::Variable(s) => s.clone(),
        }
    }
}

impl Render for GnCall {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}",
            self.opencurly,
            self.a,
            self.typecalllist.render(),
            self.b,
            self.closecurly,
        )
    }
}

impl Render for TypeGroup {
    fn render(&self) -> String {
        format!(
            "{}{}{}{}{}",
            self.openparen,
            self.a,
            self.typeassignables.render(),
            self.b,
            self.closeparen,
        )
    }
}

impl Render for FullTypename {
    fn render(&self) -> String {
        format!("{}{}", self.typename, self.opttypegenerics.render())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(path: &str) {
        let src = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));
        let ast = crate::parse::get_ast(&src)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {:?}", path, e));
        let rendered = ast.render();
        assert_eq!(
            rendered, src,
            "Round-trip failed for {}\n--- expected (original) ---\n{}\n--- actual (rendered) ---\n{}",
            path, src, rendered
        );
    }

    #[test]
    fn roundtrip_root() {
        roundtrip("src/std/root.ln");
    }

    #[test]
    fn roundtrip_fs() {
        roundtrip("src/std/fs.ln");
    }

    #[test]
    fn roundtrip_seq() {
        roundtrip("src/std/seq.ln");
    }

    #[test]
    fn roundtrip_test_ln() {
        roundtrip("../alan/test.ln");
    }
}
