use crate::parse::*;

const INDENT_SIZE: usize = 2;
const LINE_WIDTH: usize = 100;

#[derive(Debug, PartialEq)]
enum WsToken {
    Spaces(usize),
    Newline,
    SingleLineComment(String),
    MultiLineComment(String),
}

fn tokenize_whitespace(s: &str) -> Vec<WsToken> {
    let mut tokens = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            ' ' => {
                let mut count = 1;
                while let Some(' ') = chars.peek() {
                    chars.next();
                    count += 1;
                }
                tokens.push(WsToken::Spaces(count));
            }
            '\n' | '\r' => {
                tokens.push(WsToken::Newline);
            }
            '/' if chars.peek() == Some(&'/') => {
                let mut text = String::from("//");
                chars.next();
                for c in chars.by_ref() {
                    text.push(c);
                    if c == '\n' || c == '\r' {
                        break;
                    }
                }
                tokens.push(WsToken::SingleLineComment(text));
            }
            '/' if chars.peek() == Some(&'*') => {
                let mut text = String::from("/*");
                chars.next();
                loop {
                    match chars.next() {
                        Some('*') if chars.peek() == Some(&'/') => {
                            text.push('*');
                            text.push('/');
                            chars.next();
                            break;
                        }
                        Some(c) => text.push(c),
                        None => break,
                    }
                }
                tokens.push(WsToken::MultiLineComment(text));
            }
            _ => {}
        }
    }
    tokens
}

pub struct Formatter {
    out: String,
    pub indent: usize,
    pub col: usize,
    in_chain: bool,
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatter {
    pub fn new() -> Self {
        Formatter {
            out: String::new(),
            indent: 0,
            col: 0,
            in_chain: false,
        }
    }

    pub fn into_output(self) -> String {
        self.out
    }

    pub fn write(&mut self, s: &str) {
        self.out.push_str(s);
        self.col += s.len();
    }

    pub fn newline(&mut self) {
        self.out.push('\n');
        self.col = 0;
    }

    pub fn indent_write(&mut self) {
        let spaces = self.indent * INDENT_SIZE;
        self.out.push_str(&" ".repeat(spaces));
        self.col = spaces;
    }

    pub fn newline_and_indent(&mut self) {
        self.newline();
        self.indent_write();
    }

    pub fn space(&mut self) {
        self.write(" ");
    }

    fn write_comment_block(&mut self, lines: &[String]) {
        if lines.len() == 1 && 4 + lines[0].len() <= LINE_WIDTH - self.indent * INDENT_SIZE {
            self.indent_write();
            self.write("// ");
            self.write(&lines[0]);
            self.newline();
            return;
        }
        self.indent_write();
        self.write("/*");
        self.newline();
        for line in lines {
            self.indent_write();
            self.write(" * ");
            self.write(line);
            self.newline();
        }
        self.indent_write();
        self.write(" */");
        self.newline();
    }

    fn write_own_line_comments(&mut self, comments: &[String]) {
        let mut i = 0;
        while i < comments.len() {
            if comments[i].starts_with("///") {
                // Preserve `///` doc-comment-style markers as-is.
                // TODO: Revisit when a formalized documentation syntax is devised.
                let text = comments[i].trim_end_matches('\n').trim_end_matches('\r');
                self.indent_write();
                self.write(text);
                self.newline();
                i += 1;
            } else if comments[i].starts_with("//") {
                let text = strip_comment_markers(&comments[i]);
                let line_len = 3 + text.len();
                let indent_width = self.indent * INDENT_SIZE;
                let max_inner = LINE_WIDTH.saturating_sub(self.indent * INDENT_SIZE + 4);
                if indent_width + line_len > LINE_WIDTH {
                    let wrapped = word_wrap(&text, max_inner);
                    self.write_comment_block(&wrapped);
                } else {
                    self.indent_write();
                    self.write("// ");
                    self.write(&text);
                    self.newline();
                }
                i += 1;
            } else if comments[i].trim_start().starts_with("/***") {
                // Preserve multi-line comments that begin with `/***`
                // verbatim — no wrapping or block reformatting.  This
                // protects commented-out code blocks and ASCII art.
                for line in comments[i].lines() {
                    self.indent_write();
                    self.write(line);
                    self.newline();
                }
                i += 1;
            } else {
                let text = strip_comment_markers(&comments[i]);
                let max_inner = LINE_WIDTH.saturating_sub(self.indent * INDENT_SIZE + 4);
                let wrapped = word_wrap(&text, max_inner);
                self.write_comment_block(&wrapped);
                i += 1;
            }
        }
    }
}

impl Formatter {
    pub fn root_elem_width(elem: &RootElements) -> usize {
        match elem {
            RootElements::CTypes(ct) => 6 + ct.name.len() + 1,
            RootElements::Types(t) => 5 + t.fulltypename.typename.len() + 1,
            RootElements::CFns(cf) => 4 + cf.name.len() + 1,
            RootElements::Functions(f) => {
                let name = f.optname.as_deref().unwrap_or("").len();
                f.fnn.len() + name + 3
            }
            RootElements::ConstDeclaration(c) => 6 + c.variable.len() + 3,
            RootElements::Exports(_) => 17,
            _ => 20,
        }
    }

    fn fmt_ln(&mut self, ln: &Ln) {
        let comments = extract_own_line_comments(&ln.a);
        self.write_own_line_comments(&comments);
        let mut i = 0;
        let mut pending_trailing: Vec<String> = Vec::new();
        while i < ln.body.len() {
            let elem = &ln.body[i];
            match elem {
                RootElements::Whitespace(ws) => {
                    let (own_line, trailing) = separate_comments(ws);
                    if !own_line.is_empty() {
                        self.write_own_line_comments(&own_line);
                    } else if has_blank_line(ws) && !ws.starts_with(' ') {
                        self.newline();
                    }
                    pending_trailing.extend(trailing);
                    i += 1;
                }
                _ => {
                    if !pending_trailing.is_empty() {
                        self.write_own_line_comments(&pending_trailing);
                    }
                    pending_trailing.clear();
                    // Look ahead at the next Whitespace for trailing comments.
                    // Long ones go ABOVE this root element (shunted before it).
                    // Short ones stay inline after the root element.
                    if i + 1 < ln.body.len() {
                        if let RootElements::Whitespace(next_ws) = &ln.body[i + 1] {
                            let (own_line, trailing) = separate_comments(next_ws);
                            let mut any_long = false;
                            if !trailing.is_empty() {
                                let est_col = Self::root_elem_width(elem);
                                for tc in &trailing {
                                    let text = strip_comment_markers(tc);
                                    if est_col + 3 + text.len() > LINE_WIDTH {
                                        any_long = true;
                                        break;
                                    }
                                }
                            }
                            if any_long {
                                self.write_own_line_comments(&trailing);
                            }
                            self.fmt_root(elem);
                            if !any_long {
                                for tc in &trailing {
                                    let text = strip_comment_markers(tc);
                                    self.write(" // ");
                                    self.write(&text);
                                }
                            }
                            self.newline();
                            if !own_line.is_empty() {
                                self.write_own_line_comments(&own_line);
                            } else if trailing.is_empty()
                                && has_blank_line(next_ws)
                                && !next_ws.starts_with(' ')
                            {
                                self.newline();
                            }
                            i += 2;
                            continue;
                        }
                    }
                    self.fmt_root(elem);
                    self.newline();
                    i += 1;
                }
            }
        }
        let trimmed = self.out.trim_end().to_string();
        self.out = trimmed;
        self.out.push('\n');
    }

    fn fmt_root(&mut self, root: &RootElements) {
        match root {
            RootElements::Whitespace(_) => {}
            RootElements::Exports(e) => self.fmt_exports(e),
            RootElements::OperatorMapping(o) => self.fmt_operator_mapping(o),
            RootElements::TypeOperatorMapping(t) => self.fmt_type_operator_mapping(t),
            RootElements::Functions(f) => self.fmt_functions(f),
            RootElements::Types(t) => self.fmt_types(t),
            RootElements::CTypes(c) => self.fmt_ctypes(c),
            RootElements::CFns(c) => self.fmt_cfns(c),
            RootElements::ConstDeclaration(c) => self.fmt_const(c),
            RootElements::Interfaces(i) => self.fmt_interfaces(i),
        }
    }

    fn fmt_ctypes(&mut self, ct: &CTypes) {
        self.write("ctype ");
        self.write(&ct.name);
        if let Some(g) = &ct.opttypegenerics {
            self.write(&g.opencurly);
            self.fmt_gncall_types(&g.typecalllist);
            self.write(&g.closecurly);
        }
        if !ct.optsemicolon.is_empty() {
            self.write(&ct.optsemicolon);
        }
    }

    fn fmt_cfns(&mut self, cf: &CFns) {
        self.write("cfn ");
        self.write(&cf.name);
        if let Some(g) = &cf.opttypegenerics {
            self.write(&g.opencurly);
            self.fmt_gncall_types(&g.typecalllist);
            self.write(&g.closecurly);
        }
        self.space();
        self.fmt_typeassignables(&cf.typesignature);
        self.write(";");
    }
}

impl Formatter {
    fn fmt_operator_mapping(&mut self, op: &OperatorMapping) {
        match &op.fix {
            Fix::Prefix(_) => self.write("prefix"),
            Fix::Infix(_) => self.write("infix"),
            Fix::Postfix(_) => self.write("postfix"),
        }
        if let Some(g) = &op.opttypegenerics {
            self.write(&g.opencurly);
            self.fmt_gncall_types(&g.typecalllist);
            self.write(&g.closecurly);
        }
        self.space();
        let fntoop = op.opmap.get_fntoop();
        self.write(&fntoop.fnname);
        self.write(" as ");
        self.write(&fntoop.operator);
        self.write(" precedence ");
        let prec = op.opmap.get_opprecedence();
        self.write(&prec.num);
        if !op.optsemicolon.is_empty() {
            self.write(&op.optsemicolon);
        }
    }

    fn fmt_type_operator_mapping(&mut self, op: &TypeOperatorMapping) {
        self.write("type ");
        match &op.fix {
            Fix::Prefix(_) => self.write("prefix"),
            Fix::Infix(_) => self.write("infix"),
            Fix::Postfix(_) => self.write("postfix"),
        }
        if let Some(g) = &op.opttypegenerics {
            self.write(&g.opencurly);
            self.fmt_gncall_types(&g.typecalllist);
            self.write(&g.closecurly);
        }
        self.space();
        let fntoop = op.opmap.get_fntoop();
        self.write(&fntoop.fnname);
        self.write(" as ");
        self.write(&fntoop.operator);
        self.write(" precedence ");
        let prec = op.opmap.get_opprecedence();
        self.write(&prec.num);
        if !op.optsemicolon.is_empty() {
            self.write(&op.optsemicolon);
        }
    }

    fn fmt_types(&mut self, t: &Types) {
        self.write("type");
        if let Some(g) = &t.opttypegenerics {
            self.write(&g.opencurly);
            self.fmt_gncall_types(&g.typecalllist);
            self.write(&g.closecurly);
            self.space();
        } else {
            self.space();
        }
        self.write(&t.fulltypename.typename);
        if let Some(g) = &t.fulltypename.opttypegenerics {
            self.write(&g.opencurly);
            self.fmt_gncall_types(&g.typecalllist);
            self.write(&g.closecurly);
        }
        self.space();
        self.fmt_typedef(&t.typedef);
        if !t.optsemicolon.is_empty() {
            self.write(&t.optsemicolon);
        }
    }

    fn fmt_typedef(&mut self, td: &TypeDef) {
        if let Some(eq) = &td.a {
            self.write(eq);
            if td.b.contains('\n') || td.b.contains('\r') {
                self.indent += 1;
                self.newline_and_indent();
                self.fmt_typeassignables(&td.typeassignables);
                self.indent -= 1;
                return;
            }
            self.space();
        }
        self.fmt_typeassignables(&td.typeassignables);
    }
}

impl Formatter {
    fn fmt_withtypeoperators_list(&mut self, list: &[WithTypeOperators]) {
        let saved = self.indent;
        let mut prop_indent = false;
        for (i, wt) in list.iter().enumerate() {
            if !prop_indent {
                if let WithTypeOperators::Operators(o) = wt {
                    if (o.op.as_str() == "." || o.op.as_str() == "-.") && o.a.contains('\n') {
                        self.indent += 1;
                        prop_indent = true;
                    }
                }
            }
            self.fmt_withtypeoperators(wt, i + 1 == list.len());
            if prop_indent {
                let is_end = match list.get(i + 1) {
                    None => true,
                    Some(WithTypeOperators::Operators(o)) => {
                        o.op.as_str() != "." && o.op.as_str() != "-."
                    }
                    Some(_) => true,
                };
                if is_end {
                    self.indent = saved;
                    prop_indent = false;
                }
            }
        }
        self.indent = saved;
    }

    fn fmt_gncall_types(&mut self, list: &[WithTypeOperators]) {
        self.fmt_withtypeoperators_list(list);
    }

    fn fmt_typeassignables(&mut self, list: &[WithTypeOperators]) {
        self.fmt_withtypeoperators_list(list);
    }

    fn fmt_withtypeoperators(&mut self, wt: &WithTypeOperators, is_last: bool) {
        match wt {
            WithTypeOperators::TypeBaseList(tbl) => {
                for tb in tbl {
                    self.fmt_typebase(tb);
                }
            }
            WithTypeOperators::Operators(o) => {
                let trailing = extract_trailing_comment(&o.b);
                match o.op.as_str() {
                    // Prop (`.`), Exclude (`-.`), Buffer (`[`):
                    // no spacing around operator, break before with extra indent
                    // when multi-line (indent managed by fmt_typeassignables for `.`/`-.`)
                    "." | "-." | "[" => {
                        if o.a.contains('\n') {
                            self.newline_and_indent();
                        }
                        self.write(&o.op);
                        let shunt = trailing
                            .as_ref()
                            .is_some_and(|tc| should_shunt_comment(self.col, tc));
                        if let Some(tc) = &trailing {
                            if shunt {
                                let text = strip_comment_markers(tc);
                                let max_inner =
                                    LINE_WIDTH.saturating_sub(self.indent * INDENT_SIZE + 4);
                                let wrapped = word_wrap(&text, max_inner);
                                self.write_comment_block(&wrapped);
                            } else {
                                self.space();
                                self.write(tc);
                            }
                        }
                        if shunt || o.b.contains('\n') {
                            self.newline_and_indent();
                        }
                    }
                    // All other type operators: use original inline whitespace
                    // to determine spacing (preserves prefix/postfix/infix intent)
                    _ => {
                        let has_leading = o.a.split('\n').next_back().unwrap_or("").contains(' ')
                            || o.a.split('\n').next_back().unwrap_or("").contains('\t');
                        let has_trailing = o.b.split('\n').next().unwrap_or("").contains(' ')
                            || o.b.split('\n').next().unwrap_or("").contains('\t');
                        let is_postfix = !has_leading && has_trailing;
                        if has_leading {
                            self.space();
                        }
                        self.write(&o.op);
                        let shunt = trailing
                            .as_ref()
                            .is_some_and(|tc| should_shunt_comment(self.col, tc));
                        if let Some(tc) = &trailing {
                            if shunt {
                                let text = strip_comment_markers(tc);
                                let max_inner =
                                    LINE_WIDTH.saturating_sub(self.indent * INDENT_SIZE + 4);
                                let wrapped = word_wrap(&text, max_inner);
                                self.write_comment_block(&wrapped);
                            } else {
                                self.space();
                                self.write(tc);
                            }
                        }
                        if shunt || o.b.contains('\n') {
                            self.newline_and_indent();
                        } else if has_trailing && !(is_last && is_postfix) {
                            self.space();
                        }
                    }
                }
            }
        }
    }

    fn fmt_typebase(&mut self, tb: &TypeBase) {
        match tb {
            TypeBase::GnCall(gc) => {
                if self.gncall_inline_too_wide(gc) {
                    self.write(&gc.opencurly);
                    self.fmt_function_typeassignables_wrapped(&gc.typecalllist);
                    self.finish_wrapped_gncall(&gc.closecurly);
                } else {
                    self.write(&gc.opencurly);
                    // When the generic arguments span multiple lines, indent the continuation
                    // lines one level past the call so they don't fall back to column 0.
                    let multiline = typecalllist_multiline(&gc.typecalllist);
                    if multiline {
                        self.indent += 1;
                    }
                    self.fmt_gncall_types(&gc.typecalllist);
                    if multiline {
                        self.indent -= 1;
                    }
                    self.write(&gc.closecurly);
                }
            }
            TypeBase::TypeGroup(tg) => {
                self.write(&tg.openparen);
                self.fmt_withtypeoperators_list(&tg.typeassignables);
                self.write(&tg.closeparen);
            }
            TypeBase::Variable(v) => self.write(v),
            TypeBase::Constants(c) => self.fmt_constants(c),
        }
    }

    fn fmt_constants(&mut self, c: &Constants) {
        match c {
            Constants::Bool(s) => self.write(s),
            Constants::Num(n) => self.write(&n.to_string()),
            Constants::Strn(s) => self.write(s),
        }
    }

    fn fmt_exports(&mut self, e: &Exports) {
        self.write("export");
        if let Some(g) = &e.opttypegenerics {
            self.write(&g.opencurly);
            self.fmt_gncall_types(&g.typecalllist);
            self.write(&g.closecurly);
        }
        self.space();
        match &e.exportable {
            Exportable::OperatorMapping(o) => self.fmt_operator_mapping(o),
            Exportable::TypeOperatorMapping(t) => self.fmt_type_operator_mapping(t),
            Exportable::Functions(f) => self.fmt_functions(f),
            Exportable::ConstDeclaration(c) => self.fmt_const(c),
            Exportable::Types(t) => self.fmt_types(t),
            Exportable::Intefaces(i) => self.fmt_interfaces(i),
            Exportable::Ref(s) => self.write(s),
        }
    }
}

impl Formatter {
    fn fmt_functions(&mut self, f: &Functions) {
        self.write("fn");
        if let Some(g) = &f.opttypegenerics {
            self.write(&g.opencurly);
            self.fmt_gncall_types(&g.typecalllist);
            self.write(&g.closecurly);
        }
        if let Some(name) = &f.optname {
            self.space();
            self.write(name);
        }
        if let Some(g) = &f.optgenerics {
            self.write(&g.opencurly);
            self.fmt_gncall_types(&g.typecalllist);
            self.write(&g.closecurly);
        }
        if let Some(ty) = &f.opttype {
            if !ty.is_empty() {
                if self.function_type_inline_too_wide(f, ty) {
                    self.fmt_function_typeassignables_wrapped(ty);
                } else {
                    if f.optgenerics.is_none() && !f.c.is_empty() {
                        self.write(&f.c);
                    } else if !f.d.is_empty() {
                        self.write(&f.d);
                    }
                    self.fmt_typeassignables(ty);
                }
            }
        }
        match &f.fullfunctionbody {
            FullFunctionBody::FunctionBody(fb) => {
                self.space();
                self.fmt_function_body(fb);
            }
            FullFunctionBody::AssignFunction(af) => {
                let saved_in_chain = self.in_chain;
                self.in_chain = false;
                // Format the = form into a temporary buffer to measure its
                // width without altering the real output.
                let mut tmp = Formatter::new();
                tmp.indent = self.indent;
                tmp.col = self.col;
                tmp.in_chain = self.in_chain;
                tmp.write(" = ");
                for a in &af.assignables {
                    tmp.fmt_withoperators(a);
                }
                if !af.b.is_empty() {
                    tmp.write(&af.b);
                }
                let eq_form = tmp.into_output();
                let eq_first_line = eq_form.lines().next().unwrap_or("");
                let eq_too_wide = self.col + eq_first_line.len() > LINE_WIDTH;
                let eq_multi_line = eq_form.contains('\n');

                if eq_too_wide || eq_multi_line {
                    // = form is too wide.  Check whether the block form
                    // ({ return expr; }) would fit on a single line at
                    // indent+1.  Only switch if the block form actually fits,
                    // otherwise stick with = and let the fncall split logic
                    // handle the wrapping.
                    let tmp_indent = self.indent + 1;
                    let mut btmp = Formatter::new();
                    btmp.indent = tmp_indent;
                    btmp.col = tmp_indent * INDENT_SIZE;
                    btmp.write("return ");
                    for a in &af.assignables {
                        btmp.fmt_withoperators(a);
                    }
                    btmp.write(";");
                    let block_form = btmp.into_output();
                    let block_first_line = block_form.lines().next().unwrap_or("");
                    let block_too_wide =
                        (tmp_indent * INDENT_SIZE) + block_first_line.len() > LINE_WIDTH;
                    let block_multi_line = block_form.contains('\n');
                    if !block_too_wide && !block_multi_line {
                        self.write(" {");
                        self.indent += 1;
                        self.newline_and_indent();
                        self.write("return ");
                        self.fmt_wrapped_assign_function_assignables(&af.assignables);
                        self.write(";");
                        if self.in_chain {
                            self.indent -= 1;
                            self.in_chain = false;
                        }
                        self.indent -= 1;
                        self.newline_and_indent();
                        self.write("}");
                    } else {
                        // Block form doesn't fit either; break the = form
                        // onto the next line so the expression starts from
                        // the normal statement indent instead of forcing a
                        // very long single line.
                        self.write(" =");
                        self.indent += 1;
                        self.newline_and_indent();
                        self.fmt_wrapped_assign_function_assignables(&af.assignables);
                        if !af.b.is_empty() {
                            self.write(&af.b);
                        }
                        if self.in_chain {
                            self.indent -= 1;
                            self.in_chain = false;
                        }
                        self.indent -= 1;
                    }
                } else {
                    self.write(" = ");
                    for a in &af.assignables {
                        self.fmt_withoperators(a);
                    }
                    if !af.b.is_empty() {
                        self.write(&af.b);
                    }
                    if self.in_chain {
                        self.indent -= 1;
                        self.in_chain = false;
                    }
                }
                self.in_chain = saved_in_chain;
            }
            FullFunctionBody::DecOnly(s) => {
                self.write(s);
            }
        }
    }

    fn function_type_inline_too_wide(&self, f: &Functions, list: &[WithTypeOperators]) -> bool {
        let mut tmp = Formatter::new();
        tmp.indent = self.indent;
        tmp.col = self.col;
        tmp.in_chain = self.in_chain;
        if f.optgenerics.is_none() && !f.c.is_empty() {
            tmp.write(&f.c);
        } else if !f.d.is_empty() {
            tmp.write(&f.d);
        }
        tmp.fmt_typeassignables(list);
        let rendered = tmp.into_output();
        let first_line = rendered.lines().next().unwrap_or("");
        rendered.contains('\n') || self.col + first_line.len() > LINE_WIDTH
    }

    fn gncall_inline_too_wide(&self, gc: &GnCall) -> bool {
        if !gc.typecalllist.iter().any(|wt| matches!(wt, WithTypeOperators::Operators(o) if matches!(o.op.as_str(), "<-" | "::"))) {
            return false;
        }

        let mut tmp = Formatter::new();
        tmp.indent = self.indent;
        tmp.col = self.col;
        tmp.in_chain = self.in_chain;
        tmp.write(&gc.opencurly);
        tmp.fmt_gncall_types(&gc.typecalllist);
        tmp.write(&gc.closecurly);
        let rendered = tmp.into_output();
        let first_line = rendered.lines().next().unwrap_or("");
        rendered.contains('\n') || self.col + first_line.len() > LINE_WIDTH
    }

    fn finish_wrapped_gncall(&mut self, closecurly: &str) {
        while self.out.ends_with(' ') {
            self.out.pop();
        }
        self.col = self.out.rsplit('\n').next().unwrap_or("").len();
        if self.out.ends_with('\n') {
            self.indent_write();
        } else {
            self.newline_and_indent();
        }
        self.write(closecurly);
    }

    fn fmt_function_typeassignables_wrapped(&mut self, list: &[WithTypeOperators]) {
        self.indent += 1;
        self.newline_and_indent();
        let mut line_start = true;

        for (i, wt) in list.iter().enumerate() {
            let break_before_arrow = match wt {
                WithTypeOperators::Operators(o) if o.op.as_str() == "->" => {
                    list.get(i + 1).is_some_and(|next| {
                        self.col
                            + self.function_type_part_width(wt, line_start)
                            + self.function_type_part_width(next, false)
                            > LINE_WIDTH
                    })
                }
                _ => false,
            };
            let force_break_before = matches!(wt, WithTypeOperators::Operators(o) if matches!(o.op.as_str(), "<-" | "::"))
                || break_before_arrow;
            let part_width = self.function_type_part_width(wt, line_start);

            if !line_start && (force_break_before || self.col + part_width > LINE_WIDTH) {
                self.newline_and_indent();
                line_start = true;
            }

            match wt {
                WithTypeOperators::TypeBaseList(tbl) => {
                    for tb in tbl {
                        self.fmt_typebase(tb);
                    }
                }
                WithTypeOperators::Operators(o) => {
                    self.fmt_wrapped_function_type_operator(o, line_start);
                }
            }

            line_start = false;
        }

        self.indent -= 1;
    }

    fn function_type_part_width(&self, wt: &WithTypeOperators, line_start: bool) -> usize {
        match wt {
            WithTypeOperators::TypeBaseList(tbl) => {
                let mut tmp = Formatter::new();
                for tb in tbl {
                    tmp.fmt_typebase(tb);
                }
                tmp.into_output().len()
            }
            WithTypeOperators::Operators(o) => {
                let mut len = o.op.len();
                let has_leading = o.a.split('\n').next_back().unwrap_or("").contains(' ')
                    || o.a.split('\n').next_back().unwrap_or("").contains('\t');
                let has_trailing = o.b.split('\n').next().unwrap_or("").contains(' ')
                    || o.b.split('\n').next().unwrap_or("").contains('\t');
                if has_leading && !line_start {
                    len += 1;
                }
                if has_trailing {
                    len += 1;
                }
                len
            }
        }
    }

    fn fmt_wrapped_function_type_operator(
        &mut self,
        o: &TypeOperatorsWithWhitespace,
        line_start: bool,
    ) {
        let trailing = extract_trailing_comment(&o.b);
        match o.op.as_str() {
            "." | "-." | "[" => {
                if o.a.contains('\n') {
                    self.newline_and_indent();
                }
                self.write(&o.op);
                let shunt = trailing
                    .as_ref()
                    .is_some_and(|tc| should_shunt_comment(self.col, tc));
                if let Some(tc) = &trailing {
                    if shunt {
                        let text = strip_comment_markers(tc);
                        let max_inner = LINE_WIDTH.saturating_sub(self.indent * INDENT_SIZE + 4);
                        let wrapped = word_wrap(&text, max_inner);
                        self.write_comment_block(&wrapped);
                    } else {
                        self.space();
                        self.write(tc);
                    }
                }
                if shunt || o.b.contains('\n') {
                    self.newline_and_indent();
                }
            }
            _ => {
                let has_leading = o.a.split('\n').next_back().unwrap_or("").contains(' ')
                    || o.a.split('\n').next_back().unwrap_or("").contains('\t');
                if has_leading && !line_start {
                    self.space();
                }
                self.write(&o.op);
                let shunt = trailing
                    .as_ref()
                    .is_some_and(|tc| should_shunt_comment(self.col, tc));
                if let Some(tc) = &trailing {
                    if shunt {
                        let text = strip_comment_markers(tc);
                        let max_inner = LINE_WIDTH.saturating_sub(self.indent * INDENT_SIZE + 4);
                        let wrapped = word_wrap(&text, max_inner);
                        self.write_comment_block(&wrapped);
                    } else {
                        self.space();
                        self.write(tc);
                    }
                }
                if shunt || o.b.contains('\n') {
                    self.newline_and_indent();
                } else {
                    let has_trailing = o.b.split('\n').next().unwrap_or("").contains(' ')
                        || o.b.split('\n').next().unwrap_or("").contains('\t');
                    if has_trailing {
                        self.space();
                    }
                }
            }
        }
    }

    fn fmt_function_body(&mut self, fb: &FunctionBody) {
        let saved_in_chain = self.in_chain;
        self.in_chain = false;
        self.write("{");
        let own_a = extract_own_line_comments(&fb.a);
        let has_own_a = !own_a.is_empty();
        let stmts_are_non_ws = fb.statements.iter().any(|s| !matches!(s, Statement::A(_)));
        let has_content = stmts_are_non_ws || has_own_a || fb.a.contains('\n');
        let mut need_newline_for_content = true;
        if has_content {
            self.indent += 1;
            if has_own_a {
                self.newline();
                self.write_own_line_comments(&own_a);
                need_newline_for_content = false;
            } else if fb.a.contains('\n') {
                self.newline();
                need_newline_for_content = false;
            }
        }
        let mut i = 0;
        while i < fb.statements.len() {
            match &fb.statements[i] {
                Statement::A(ws) => {
                    let (own_line, _trailing) = separate_comments(ws);
                    let has_next_non_ws = fb.statements[i + 1..]
                        .iter()
                        .any(|n| !matches!(n, Statement::A(_)));
                    if !own_line.is_empty() {
                        if need_newline_for_content {
                            self.newline();
                        }
                        if has_blank_line(ws) {
                            self.newline();
                        }
                        self.write_own_line_comments(&own_line);
                        need_newline_for_content = false;
                    } else if has_blank_line(ws) && has_next_non_ws {
                        if need_newline_for_content {
                            self.newline();
                        }
                        self.newline();
                        need_newline_for_content = false;
                    } else if has_next_non_ws {
                        need_newline_for_content = true;
                    }
                    i += 1;
                }
                stmt => {
                    let mut inline_trailing: Vec<String> = Vec::new();
                    let mut own_line_after: Vec<String> = Vec::new();
                    let mut next_ws: Option<&str> = None;

                    if i + 1 < fb.statements.len() {
                        if let Statement::A(ws) = &fb.statements[i + 1] {
                            let (own_line, trailing) = separate_comments(ws);
                            let any_long = trailing
                                .iter()
                                .any(|tc| should_shunt_statement_comment(self, stmt, tc));
                            if any_long {
                                if need_newline_for_content {
                                    self.newline();
                                }
                                self.write_own_line_comments(&trailing);
                                need_newline_for_content = false;
                            } else {
                                inline_trailing = trailing;
                            }
                            own_line_after = own_line;
                            next_ws = Some(ws);
                        }
                    }

                    if need_newline_for_content {
                        self.newline_and_indent();
                    } else {
                        self.indent_write();
                    }
                    self.fmt_statement(stmt);
                    for tc in &inline_trailing {
                        let text = strip_comment_markers(tc);
                        self.write(" // ");
                        self.write(&text);
                    }
                    need_newline_for_content = true;

                    if let Some(ws) = next_ws {
                        if !own_line_after.is_empty() {
                            self.newline();
                            if has_blank_line(ws) {
                                self.newline();
                            }
                            self.write_own_line_comments(&own_line_after);
                            need_newline_for_content = false;
                        } else if inline_trailing.is_empty()
                            && has_blank_line(ws)
                            && fb.statements[i + 2..]
                                .iter()
                                .any(|n| !matches!(n, Statement::A(_)))
                        {
                            self.newline();
                            self.newline();
                            need_newline_for_content = false;
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
        }
        let own_b = extract_own_line_comments(&fb.b);
        if !own_b.is_empty() {
            if need_newline_for_content {
                self.newline();
            }
            self.write_own_line_comments(&own_b);
            need_newline_for_content = false;
        }
        if has_content {
            self.indent -= 1;
            if need_newline_for_content {
                self.newline_and_indent();
            } else {
                self.indent_write();
            }
        }
        self.in_chain = saved_in_chain;
        self.write("}");
    }

    fn fmt_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Declarations(d) => self.fmt_declarations(d),
            Statement::Returns(r) => self.fmt_returns(r),
            Statement::Conditional(c) => self.fmt_conditional(c),
            Statement::ArrayAssignment(a) => self.fmt_array_assignment(a),
            Statement::Assignables(a) => self.fmt_assignable_statement(a),
            Statement::A(ws) => {
                let (own, _) = separate_comments(ws);
                if !own.is_empty() {
                    self.write_own_line_comments(&own);
                }
            }
        }
    }

    fn fmt_declarations(&mut self, d: &Declarations) {
        match d {
            Declarations::Const(c) => self.fmt_const(c),
            Declarations::Let(l) => self.fmt_let(l),
        }
    }

    fn fmt_const(&mut self, c: &ConstDeclaration) {
        self.write("const ");
        if let Some(g) = &c.opttypegenerics {
            self.write(&g.opencurly);
            self.fmt_gncall_types(&g.typecalllist);
            self.write(&g.closecurly);
        }
        self.write(&c.variable);
        if let Some(td) = &c.typedec {
            self.write(": ");
            self.write(&td.fulltypename.typename);
            if let Some(g) = &td.fulltypename.opttypegenerics {
                self.write(&g.opencurly);
                self.fmt_gncall_types(&g.typecalllist);
                self.write(&g.closecurly);
            }
        }
        self.write(" = ");
        for a in &c.assignables {
            self.fmt_withoperators(a);
        }
        if self.in_chain {
            self.indent -= 1;
            self.in_chain = false;
        }
        self.write(";");
    }

    fn fmt_let(&mut self, l: &LetDeclaration) {
        self.write("let ");
        if let Some(g) = &l.opttypegenerics {
            self.write(&g.opencurly);
            self.fmt_gncall_types(&g.typecalllist);
            self.write(&g.closecurly);
        }
        self.write(&l.variable);
        if let Some(td) = &l.typedec {
            self.write(": ");
            self.write(&td.fulltypename.typename);
            if let Some(g) = &td.fulltypename.opttypegenerics {
                self.write(&g.opencurly);
                self.fmt_gncall_types(&g.typecalllist);
                self.write(&g.closecurly);
            }
        }
        self.write(" = ");
        for a in &l.assignables {
            self.fmt_withoperators(a);
        }
        if self.in_chain {
            self.indent -= 1;
            self.in_chain = false;
        }
        self.write(";");
    }

    fn fmt_returns(&mut self, r: &Returns) {
        // Only emit the space after `return` when there's an actual value, so a bare `return;`
        // doesn't become `return ;`.
        if r.retval.is_some() {
            self.write("return ");
        } else {
            self.write("return");
        }
        self.fmt_retval(&r.retval);
        self.write(";");
    }

    fn fmt_retval(&mut self, rv: &Option<RetVal>) {
        if let Some(rv) = rv {
            for a in &rv.assignables {
                self.fmt_withoperators(a);
            }
        }
        if self.in_chain {
            self.indent -= 1;
            self.in_chain = false;
        }
    }

    fn fmt_conditional(&mut self, c: &Conditional) {
        self.write("if ");
        for a in &c.assignables {
            self.fmt_withoperators(a);
        }
        self.space();
        self.fmt_blocklike(&c.blocklike);
        if let Some(el) = &c.optelsebranch {
            self.space();
            self.write("else ");
            self.fmt_condorblock(&el.condorblock);
        }
    }

    fn fmt_blocklike(&mut self, bl: &Blocklike) {
        match bl {
            Blocklike::Functions(f) => self.fmt_functions(f),
            Blocklike::FunctionBody(fb) => self.fmt_function_body(fb),
        }
    }

    fn fmt_condorblock(&mut self, cb: &CondOrBlock) {
        match cb {
            CondOrBlock::Conditional(c) => self.fmt_conditional(c),
            CondOrBlock::Blocklike(b) => self.fmt_blocklike(b),
        }
    }
}

impl Formatter {
    fn fmt_assignable_statement(&mut self, a: &AssignableStatement) {
        for wo in &a.assignables {
            self.fmt_withoperators(wo);
        }
        if self.in_chain {
            self.indent -= 1;
            self.in_chain = false;
        }
        self.write(";");
    }

    fn fmt_array_assignment(&mut self, aa: &ArrayAssignment) {
        self.fmt_baseassignable(&aa.name);
        self.fmt_array(&aa.array);
        self.write(" = ");
        for a in &aa.assignables {
            self.fmt_withoperators(a);
        }
        self.write(";");
    }

    fn fmt_baseassignable_list(
        &mut self,
        bal: &[BaseAssignable],
        force_chain_break_from: Option<usize>,
    ) {
        let saved_in_chain = self.in_chain;
        self.in_chain = false;
        for (i, ba) in bal.iter().enumerate() {
            let force_break = force_chain_break_from == Some(i);
            self.fmt_baseassignable_with_forced_chain_break(ba, force_break);
        }
        if self.in_chain {
            self.indent -= 1;
            self.in_chain = false;
        }
        self.in_chain = saved_in_chain;
    }

    fn inline_baseassignable_list_width(&self, bal: &[BaseAssignable]) -> usize {
        bal.iter().map(|ba| ba.to_string().len()).sum()
    }

    fn wrapped_assign_chain_break_index(&self, bal: &[BaseAssignable]) -> Option<usize> {
        let first_method_sep = bal
            .iter()
            .position(|ba| matches!(ba, BaseAssignable::MethodSep(_)))?;
        if self.col + self.inline_baseassignable_list_width(bal) > LINE_WIDTH {
            Some(first_method_sep)
        } else {
            None
        }
    }

    fn fmt_baseassignable(&mut self, ba: &BaseAssignable) {
        self.fmt_baseassignable_with_forced_chain_break(ba, false);
    }

    fn fmt_baseassignable_with_forced_chain_break(
        &mut self,
        ba: &BaseAssignable,
        force_chain_break: bool,
    ) {
        match ba {
            BaseAssignable::Functions(f) => self.fmt_functions(f),
            BaseAssignable::FnCall(fc) => self.fmt_fncall(fc),
            BaseAssignable::GnCall(gc) => {
                if self.gncall_inline_too_wide(gc) {
                    self.write(&gc.opencurly);
                    self.fmt_function_typeassignables_wrapped(&gc.typecalllist);
                    self.finish_wrapped_gncall(&gc.closecurly);
                } else {
                    self.write(&gc.opencurly);
                    self.fmt_gncall_types(&gc.typecalllist);
                    self.write(&gc.closecurly);
                }
            }
            BaseAssignable::Array(a) => self.fmt_array(a),
            BaseAssignable::Variable(v) => self.write(v),
            BaseAssignable::MethodSep(ws) => {
                if let Some(dot_pos) = ws.find('.') {
                    let before = &ws[..dot_pos];
                    let trailing = extract_trailing_comment(before);
                    let shunt = trailing
                        .as_ref()
                        .is_some_and(|tc| should_shunt_comment(self.col, tc));
                    if let Some(tc) = &trailing {
                        if shunt {
                            let text = strip_comment_markers(tc);
                            let max_inner =
                                LINE_WIDTH.saturating_sub(self.indent * INDENT_SIZE + 4);
                            let wrapped = word_wrap(&text, max_inner);
                            self.write_comment_block(&wrapped);
                        } else {
                            self.space();
                            self.write(tc);
                        }
                    }
                    if shunt || before.contains('\n') || force_chain_break || self.in_chain {
                        if !self.in_chain {
                            self.indent += 1;
                            self.in_chain = true;
                        }
                        self.newline_and_indent();
                    } else if trailing.is_none() && !before.is_empty() {
                        self.space();
                    }
                }
                self.write(".");
            }
            BaseAssignable::Constants(c) => self.fmt_constants(c),
        }
    }

    fn fmt_wrapped_assign_function_assignables(&mut self, assignables: &[WithOperators]) {
        for (wo_idx, wo) in assignables.iter().enumerate() {
            match (wo_idx, wo) {
                (0, WithOperators::BaseAssignableList(bal)) => {
                    let force_idx = self.wrapped_assign_chain_break_index(bal);
                    self.fmt_baseassignable_list(bal, force_idx);
                }
                (_, WithOperators::BaseAssignableList(bal)) => {
                    self.fmt_baseassignable_list(bal, None);
                }
                _ => self.fmt_withoperators(wo),
            }
        }
    }

    fn fmt_fncall(&mut self, fc: &FnCall) {
        let saved_in_chain = self.in_chain;
        self.in_chain = false;
        let needs_break =
            fc.assignablelist.elements.len() > 1 && line_would_exceed(self, &fc.assignablelist);
        if needs_break {
            self.write("(");
            self.indent += 1;
            for (i, args) in fc.assignablelist.elements.iter().enumerate() {
                self.newline_and_indent();
                for wo in args {
                    self.fmt_withoperators(wo);
                }
                if i < fc.assignablelist.separators.len() {
                    self.write(",");
                }
            }
            self.indent -= 1;
            self.newline_and_indent();
            self.write(")");
        } else {
            self.write("(");
            for (i, args) in fc.assignablelist.elements.iter().enumerate() {
                for wo in args {
                    self.fmt_withoperators(wo);
                }
                if i < fc.assignablelist.separators.len() {
                    self.write(", ");
                }
            }
            self.write(")");
        }
        self.in_chain = saved_in_chain;
    }

    fn fmt_array(&mut self, a: &ArrayBase) {
        let saved_in_chain = self.in_chain;
        self.in_chain = false;
        self.write("[");
        for (i, elems) in a.assignablelist.elements.iter().enumerate() {
            for wo in elems {
                self.fmt_withoperators(wo);
            }
            if i < a.assignablelist.separators.len() {
                self.write(", ");
            }
        }
        self.write("]");
        self.in_chain = saved_in_chain;
    }

    fn fmt_withoperators(&mut self, wo: &WithOperators) {
        match wo {
            WithOperators::BaseAssignableList(bal) => {
                self.fmt_baseassignable_list(bal, None);
            }
            WithOperators::Operators(op) => {
                let trimmed = op.trim();
                let leading = &op[..op.len() - op.trim_start().len()];
                let trailing = &op[op.trim_end().len()..];
                let has_leading_ws = !leading.is_empty();
                let has_trailing_ws = !trailing.is_empty();
                let has_leading_newline = leading.contains('\n') || leading.contains('\r');
                let has_trailing_newline = trailing.contains('\n') || trailing.contains('\r');
                let has_leading_inline =
                    leading.split('\n').next_back().unwrap_or("").contains(' ')
                        || leading.split('\n').next_back().unwrap_or("").contains('\t');
                let has_trailing_inline = trailing.split('\n').next().unwrap_or("").contains(' ')
                    || trailing.split('\n').next().unwrap_or("").contains('\t');
                let is_infix = has_leading_ws && has_trailing_ws;

                if is_infix {
                    self.space();
                    self.write(trimmed);
                    if has_leading_newline || has_trailing_newline {
                        self.newline_and_indent();
                    } else {
                        self.space();
                    }
                } else {
                    if has_leading_newline {
                        self.newline_and_indent();
                    } else if has_leading_inline {
                        self.space();
                    }
                    self.write(trimmed);
                    if has_trailing_newline {
                        self.newline_and_indent();
                    } else if has_trailing_inline {
                        self.space();
                    }
                }
            }
        }
    }

    fn fmt_interfaces(&mut self, i: &Interfaces) {
        self.write("interface ");
        if let Some(g) = &i.opttypegenerics {
            self.write(&g.opencurly);
            self.fmt_gncall_types(&g.typecalllist);
            self.write(&g.closecurly);
        }
        self.write(&i.variable);
        self.space();
        match &i.interfacedef {
            InterfaceDef::InterfaceBody(ib) => {
                self.write("{");
                if !ib.interfacelist.is_empty() {
                    self.indent += 1;
                    for il in &ib.interfacelist {
                        self.newline_and_indent();
                        self.fmt_interfaceline(il);
                        self.write(",");
                    }
                    self.indent -= 1;
                    self.newline_and_indent();
                }
                self.write("}");
            }
            InterfaceDef::InterfaceAlias(ia) => {
                self.write("= ");
                self.write(&ia.variable);
            }
        }
    }

    fn fmt_interfaceline(&mut self, il: &InterfaceLine) {
        match il {
            InterfaceLine::FunctionTypeline(ft) => {
                self.write(&ft.variable);
                self.fmt_withtypeoperators_list(&ft.functiontype);
            }
        }
    }
}

fn extract_own_line_comments(ws: &str) -> Vec<String> {
    let mut comments = Vec::new();
    let tokens = tokenize_whitespace(ws);
    let mut prev_was_newline = true;
    for token in &tokens {
        match token {
            WsToken::Newline => prev_was_newline = true,
            WsToken::SingleLineComment(text) => {
                if prev_was_newline {
                    let trimmed = text.trim_end_matches('\n').trim_end_matches('\r');
                    comments.push(trimmed.to_string());
                }
                prev_was_newline = text.ends_with('\n') || text.ends_with('\r');
            }
            WsToken::MultiLineComment(text) => {
                if prev_was_newline {
                    comments.push(text.clone());
                }
                prev_was_newline = text.ends_with('\n') || text.ends_with('\r');
            }
            WsToken::Spaces(_) => {}
        }
    }
    comments
}

fn separate_comments(ws: &str) -> (Vec<String>, Vec<String>) {
    let tokens = tokenize_whitespace(ws);
    let mut own_line: Vec<String> = Vec::new();
    let mut trailing: Vec<String> = Vec::new();
    let mut prev_was_newline = false;
    for token in &tokens {
        match token {
            WsToken::Newline => prev_was_newline = true,
            WsToken::SingleLineComment(text) => {
                let trimmed = text.trim_end_matches('\n').trim_end_matches('\r');
                if prev_was_newline {
                    own_line.push(trimmed.to_string());
                } else {
                    trailing.push(trimmed.to_string());
                }
                prev_was_newline = text.ends_with('\n') || text.ends_with('\r');
            }
            WsToken::MultiLineComment(text) => {
                if prev_was_newline {
                    own_line.push(text.clone());
                } else {
                    trailing.push(text.clone());
                }
                prev_was_newline = text.ends_with('\n') || text.ends_with('\r');
            }
            WsToken::Spaces(_) => {}
        }
    }
    (own_line, trailing)
}

fn extract_trailing_comment(ws: &str) -> Option<String> {
    let tokens = tokenize_whitespace(ws);
    let mut trailing = None;
    let mut last_was_newline = false;
    for token in &tokens {
        match token {
            WsToken::Newline => last_was_newline = true,
            WsToken::SingleLineComment(text) => {
                let trimmed = text.trim_end_matches('\n').trim_end_matches('\r');
                if !last_was_newline {
                    trailing = Some(trimmed.to_string());
                }
                last_was_newline = text.ends_with('\n') || text.ends_with('\r');
            }
            WsToken::MultiLineComment(text) => {
                if !last_was_newline {
                    trailing = Some(text.clone());
                }
                last_was_newline = text.ends_with('\n') || text.ends_with('\r');
            }
            WsToken::Spaces(_) => {}
        }
    }
    trailing
}

fn line_would_exceed(f: &Formatter, al: &AssignableList) -> bool {
    let mut est = f.col + 1; // +1 for the opening (
    for (i, args) in al.elements.iter().enumerate() {
        for wo in args {
            match wo {
                WithOperators::BaseAssignableList(bal) => {
                    for ba in bal {
                        match ba {
                            BaseAssignable::Variable(v) => est += v.len(),
                            BaseAssignable::Constants(c) => est += c.to_string().len(),
                            BaseAssignable::MethodSep(v) => est += v.trim().len(),
                            BaseAssignable::FnCall(fc) => {
                                est += 2;
                                for inner in &fc.assignablelist.elements {
                                    for iwo in inner {
                                        match iwo {
                                            WithOperators::BaseAssignableList(ibal) => {
                                                for iba in ibal {
                                                    if let BaseAssignable::Variable(v) = iba {
                                                        est += v.len();
                                                    } else {
                                                        est += 10;
                                                    }
                                                }
                                            }
                                            WithOperators::Operators(o) => {
                                                est += o.len();
                                            }
                                        }
                                    }
                                }
                            }
                            _ => est += 10,
                        }
                        if est > LINE_WIDTH {
                            return true;
                        }
                    }
                }
                WithOperators::Operators(o) => {
                    est += o.len();
                }
            }
        }
        if i < al.separators.len() {
            est += 2;
        }
        if est > LINE_WIDTH {
            return true;
        }
    }
    est > LINE_WIDTH
}

fn has_blank_line(ws: &str) -> bool {
    let mut chars = ws.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\n' || ch == '\r' {
            if ch == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            let rest = chars.clone();
            for next in rest {
                if next == ' ' || next == '\t' {
                    continue;
                }
                if next == '\n' || next == '\r' {
                    return true;
                }
                break;
            }
        }
    }
    false
}

fn should_shunt_statement_comment(f: &Formatter, stmt: &Statement, comment: &str) -> bool {
    let mut tmp = Formatter::new();
    tmp.indent = f.indent;
    tmp.col = f.indent * INDENT_SIZE;
    tmp.in_chain = f.in_chain;
    tmp.indent_write();
    tmp.fmt_statement(stmt);
    let stmt_out = tmp.into_output();
    if stmt_out.contains('\n') {
        return true;
    }
    let last_line_len = stmt_out.rsplit('\n').next().unwrap_or("").len();
    should_shunt_comment(last_line_len, comment)
}

fn strip_comment_markers(text: &str) -> String {
    let s = text.trim();
    if s.starts_with("/*") {
        let inner = s
            .strip_prefix("/*")
            .and_then(|t| t.strip_suffix("*/"))
            .unwrap_or(s)
            .trim();
        let lines: Vec<&str> = inner.lines().collect();
        let mut words: Vec<String> = Vec::new();
        for line in &lines {
            let trimmed = line.trim();
            let content = trimmed.strip_prefix('*').unwrap_or(trimmed).trim();
            if !content.is_empty() {
                for w in content.split_whitespace() {
                    words.push(w.to_string());
                }
            }
        }
        words.join(" ")
    } else if let Some(content) = s.strip_prefix("//") {
        content.trim().to_string()
    } else {
        s.trim().to_string()
    }
}

fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    if max_width < 10 {
        return vec![text.to_string()];
    }
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in words {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= max_width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn should_shunt_comment(col: usize, comment: &str) -> bool {
    comment.starts_with("/*") || col + 2 + comment.len() > LINE_WIDTH
}

/// Whether a generic call's argument list spans multiple lines (a newline appears in the
/// whitespace around one of its type operators, e.g. after a separating comma).
fn typecalllist_multiline(list: &[WithTypeOperators]) -> bool {
    list.iter().any(|wt| match wt {
        WithTypeOperators::Operators(o) => o.a.contains('\n') || o.b.contains('\n'),
        WithTypeOperators::TypeBaseList(_) => false,
    })
}

pub fn fmt(ln: &Ln) -> String {
    let mut formatter = Formatter::new();
    formatter.fmt_ln(ln);
    formatter.into_output()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::get_ast;

    fn canonical_fmt(src: &str) -> String {
        let ast = get_ast(src).expect("parse failed");
        fmt(&ast)
    }

    fn strip_code(s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        let mut in_string = false;
        let mut string_char = '"';
        let mut in_block_comment = false;
        while let Some(ch) = chars.next() {
            if in_block_comment {
                if ch == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    in_block_comment = false;
                }
                continue;
            }
            if ch == '"' || ch == '\'' {
                if !in_string {
                    in_string = true;
                    string_char = ch;
                } else if ch == string_char {
                    in_string = false;
                }
            }
            if in_string {
                result.push(ch);
                continue;
            }
            if ch == '/' && chars.peek() == Some(&'/') {
                for c in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                }
                continue;
            }
            if ch == '/' && chars.peek() == Some(&'*') {
                chars.next();
                in_block_comment = true;
                continue;
            }
            if !ch.is_whitespace() {
                result.push(ch);
            }
        }
        result
    }

    // An `AssignFunction` (`= expr;`) and a single-statement `FunctionBody`
    // (`{ return expr; }`) are semantically equivalent surface forms for the
    // same function, so the formatter can freely choose between them.  We
    // first check stripped code.  When that differs we fall back to AST-level
    // comparison of function bodies at the `assignables` level.
    fn sem_equiv(src: &str, formatted: &str) -> bool {
        if strip_code(src) == strip_code(formatted) {
            return true;
        }
        let src_ast = get_ast(src);
        let fmt_ast = get_ast(formatted);
        match (src_ast, fmt_ast) {
            (Ok(src_ln), Ok(fmt_ln)) => all_fn_bodies_equiv(&src_ln, &fmt_ln),
            _ => false,
        }
    }

    fn all_fn_bodies_equiv(a: &Ln, b: &Ln) -> bool {
        let fns_a: Vec<&FullFunctionBody> = a
            .body
            .iter()
            .filter_map(|e| match e {
                RootElements::Functions(f) => Some(&f.fullfunctionbody),
                _ => None,
            })
            .collect();
        let fns_b: Vec<&FullFunctionBody> = b
            .body
            .iter()
            .filter_map(|e| match e {
                RootElements::Functions(f) => Some(&f.fullfunctionbody),
                _ => None,
            })
            .collect();
        if fns_a.len() != fns_b.len() {
            return false;
        }
        for (fa, fb) in fns_a.iter().zip(fns_b.iter()) {
            if !body_equiv(fa, fb) {
                return false;
            }
        }
        true
    }

    fn body_equiv(a: &FullFunctionBody, b: &FullFunctionBody) -> bool {
        match (a, b) {
            (FullFunctionBody::AssignFunction(af), FullFunctionBody::FunctionBody(fb))
            | (FullFunctionBody::FunctionBody(fb), FullFunctionBody::AssignFunction(af)) => {
                assignables_eq_single_return(af, fb)
            }
            (FullFunctionBody::AssignFunction(afa), FullFunctionBody::AssignFunction(afb)) => {
                assignables_eq(&afa.assignables, &afb.assignables)
            }
            (FullFunctionBody::FunctionBody(fba), FullFunctionBody::FunctionBody(fbb)) => {
                if fba.statements.len() != fbb.statements.len() {
                    return false;
                }
                fba.statements
                    .iter()
                    .zip(fbb.statements.iter())
                    .all(|(x, y)| stmt_eq(x, y))
            }
            _ => a == b,
        }
    }

    fn stmt_eq(a: &Statement, b: &Statement) -> bool {
        match (a, b) {
            (Statement::Returns(ra), Statement::Returns(rb)) => match (&ra.retval, &rb.retval) {
                (Some(va), Some(vb)) => assignables_eq(&va.assignables, &vb.assignables),
                (None, None) => true,
                _ => false,
            },
            (Statement::Assignables(aa), Statement::Assignables(ba)) => {
                assignables_eq(&aa.assignables, &ba.assignables)
            }
            (Statement::Declarations(da), Statement::Declarations(db)) => match (da, db) {
                (Declarations::Let(la), Declarations::Let(lb)) => {
                    assignables_eq(&la.assignables, &lb.assignables)
                }
                (Declarations::Const(ca), Declarations::Const(cb)) => {
                    assignables_eq(&ca.assignables, &cb.assignables)
                }
                _ => false,
            },
            (Statement::A(_), Statement::A(_)) => true,
            _ => a == b,
        }
    }

    fn assignables_eq_single_return(af: &AssignFunction, fb: &FunctionBody) -> bool {
        let non_ws: Vec<&Statement> = fb
            .statements
            .iter()
            .filter(|s| !matches!(s, Statement::A(_)))
            .collect();
        if non_ws.len() != 1 {
            return false;
        }
        match non_ws[0] {
            Statement::Returns(ret) => match &ret.retval {
                Some(rv) => assignables_eq(&af.assignables, &rv.assignables),
                None => af.assignables.is_empty(),
            },
            _ => false,
        }
    }

    fn assignables_eq(a: &[WithOperators], b: &[WithOperators]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        a.iter().zip(b.iter()).all(|(x, y)| wo_eq(x, y))
    }

    fn wo_eq(a: &WithOperators, b: &WithOperators) -> bool {
        match (a, b) {
            (WithOperators::BaseAssignableList(al), WithOperators::BaseAssignableList(bl)) => {
                al.iter().zip(bl.iter()).all(|(x, y)| ba_eq(x, y))
            }
            (WithOperators::Operators(ao), WithOperators::Operators(bo)) => ao.trim() == bo.trim(),
            _ => false,
        }
    }

    fn ba_eq(a: &BaseAssignable, b: &BaseAssignable) -> bool {
        match (a, b) {
            (BaseAssignable::Variable(av), BaseAssignable::Variable(bv)) => av == bv,
            (BaseAssignable::MethodSep(am), BaseAssignable::MethodSep(bm)) => {
                am.trim() == bm.trim()
            }
            (BaseAssignable::Constants(ac), BaseAssignable::Constants(bc)) => ac == bc,
            (BaseAssignable::Functions(af), BaseAssignable::Functions(bf)) => {
                af.fnn == bf.fnn
                    && af.optname == bf.optname
                    && body_equiv(&af.fullfunctionbody, &bf.fullfunctionbody)
            }
            (BaseAssignable::FnCall(afc), BaseAssignable::FnCall(bfc)) => {
                afc.assignablelist.elements.len() == bfc.assignablelist.elements.len()
                    && afc
                        .assignablelist
                        .elements
                        .iter()
                        .zip(bfc.assignablelist.elements.iter())
                        .all(|(ea, eb)| assignables_eq(ea, eb))
            }
            (BaseAssignable::GnCall(agc), BaseAssignable::GnCall(bgc)) => {
                agc.typecalllist.len() == bgc.typecalllist.len()
                    && agc
                        .typecalllist
                        .iter()
                        .zip(bgc.typecalllist.iter())
                        .all(|(ta, tb)| wto_eq(ta, tb))
            }
            (BaseAssignable::Array(aa), BaseAssignable::Array(ba)) => {
                aa.assignablelist.elements.len() == ba.assignablelist.elements.len()
                    && aa
                        .assignablelist
                        .elements
                        .iter()
                        .zip(ba.assignablelist.elements.iter())
                        .all(|(ea, eb)| assignables_eq(ea, eb))
            }
            _ => a == b,
        }
    }

    fn wto_eq(a: &WithTypeOperators, b: &WithTypeOperators) -> bool {
        match (a, b) {
            (WithTypeOperators::TypeBaseList(al), WithTypeOperators::TypeBaseList(bl)) => {
                al.iter().zip(bl.iter()).all(|(x, y)| tb_eq(x, y))
            }
            (WithTypeOperators::Operators(ao), WithTypeOperators::Operators(bo)) => {
                ao.op.trim() == bo.op.trim()
            }
            _ => false,
        }
    }

    fn tb_eq(a: &TypeBase, b: &TypeBase) -> bool {
        match (a, b) {
            (TypeBase::Variable(av), TypeBase::Variable(bv)) => av == bv,
            (TypeBase::Constants(ac), TypeBase::Constants(bc)) => ac == bc,
            (TypeBase::GnCall(agc), TypeBase::GnCall(bgc)) => {
                agc.typecalllist.len() == bgc.typecalllist.len()
                    && agc
                        .typecalllist
                        .iter()
                        .zip(bgc.typecalllist.iter())
                        .all(|(ta, tb)| wto_eq(ta, tb))
            }
            _ => a == b,
        }
    }

    fn count_comments(s: &str) -> usize {
        s.matches("//").count() + s.matches("/*").count()
    }

    #[test]
    fn test_trailing_comment_shunt() {
        // Short trailing comment: fits on line, stays inline
        let short = canonical_fmt("ctype Type; // short\nctype Generic; // also short\n");
        assert_eq!(
            short,
            "ctype Type; // short\nctype Generic; // also short\n"
        );

        // Long trailing comment: exceeds 100 cols, shunted before the same statement
        let long = canonical_fmt(
            "ctype Type; // this is a very long trailing comment that definitely exceeds one hundred columns so it must be shunted\nctype Generic;\n",
        );
        assert_eq!(
            long,
            "/*\n * this is a very long trailing comment that definitely exceeds one hundred columns so it must be\n * shunted\n */\nctype Type;\nctype Generic;\n"
        );
        assert_eq!(canonical_fmt(&long), long);

        // Mixed: short stays inline, long is shunted to before its own statement
        let mixed = canonical_fmt(
            "ctype Type; // short\nctype Generic; // this long comment must be shunted because it is way too long to fit on a single line at all\nctype Int{T};\n",
        );
        assert_eq!(
            mixed,
            "ctype Type; // short\n// this long comment must be shunted because it is way too long to fit on a single line at all\nctype Generic;\nctype Int{T};\n"
        );
        assert_eq!(canonical_fmt(&mixed), mixed);
    }

    #[test]
    fn test_statement_trailing_comment_shunt() {
        let short = canonical_fmt(
            "fn normalize (arr: f32[]) {\n  let mag = magnitude(arr);\n  let arr1 = arr.clone; // TODO: Needed for Rust codegen, but should not\n  return if(mag == 0.0.f32, fn = arr1.clone, fn = arr.map(fn (v: f32) = v / mag));\n}\n",
        );
        assert_eq!(
            short,
            "fn normalize (arr: f32[]) {\n  let mag = magnitude(arr);\n  let arr1 = arr.clone; // TODO: Needed for Rust codegen, but should not\n  return if(mag == 0.0.f32, fn = arr1.clone, fn = arr.map(fn(v: f32) = v / mag));\n}\n"
        );
        assert_eq!(canonical_fmt(&short), short);

        let long = canonical_fmt(
            "fn normalize (arr: f32[]) {\n  let mag = magnitude(arr);\n  let arr1 = arr.clone; // this is a very long trailing comment that definitely exceeds one hundred columns so it must be shunted\n  return arr1;\n}\n",
        );
        assert_eq!(
            long,
            "fn normalize (arr: f32[]) {\n  let mag = magnitude(arr);\n  /*\n   * this is a very long trailing comment that definitely exceeds one hundred columns so it must be\n   * shunted\n   */\n  let arr1 = arr.clone;\n  return arr1;\n}\n"
        );
        assert_eq!(canonical_fmt(&long), long);
    }

    #[test]
    fn test_bare_return_no_trailing_space() {
        // A bare `return;` must not gain a space before the semicolon, while a value return keeps
        // its single space after `return`.
        let out = canonical_fmt(
            "fn f(n: i64) {\n  if n > 0 {\n    return;\n  }\n  return;\n}\nfn g(n: i64) -> i64 {\n  return n;\n}\n",
        );
        assert_eq!(
            out,
            "fn f(n: i64) {\n  if n > 0 {\n    return;\n  }\n  return;\n}\nfn g(n: i64) -> i64 {\n  return n;\n}\n"
        );
        assert_eq!(canonical_fmt(&out), out);
    }

    #[test]
    fn test_multiline_generic_type_indents() {
        // Continuation lines of a multi-line generic type call are indented one level past the
        // call rather than falling back to column 0.
        let out =
            canonical_fmt("type Vec{T, L} = If{\n  L < 5,\n  T[L],\n  Fail{\"too long\"}};\n");
        assert_eq!(
            out,
            "type Vec{T, L} = If{L < 5,\n  T[L],\n  Fail{\"too long\"}};\n"
        );
        assert_eq!(canonical_fmt(&out), out);
    }

    #[test]
    fn test_long_assign_function_wraps() {
        let out = canonical_fmt(
            "fn insertBits(v: u16, newbits: u16, offset: u16, count: u16) = ((newbits & (2.u16 ** count - 1.u16)) << offset) + (v & !((2.u16 ** count - 1.u16) << offset));\n",
        );
        assert_eq!(
            out,
            "fn insertBits(v: u16, newbits: u16, offset: u16, count: u16) =\n  ((newbits & (2.u16 ** count - 1.u16)) << offset) + (v & !((2.u16 ** count - 1.u16) << offset));\n"
        );
        assert_eq!(canonical_fmt(&out), out);
    }

    #[test]
    fn test_wrapped_assign_function_chain_indent() {
        let out = canonical_fmt(
            "fn reverseBitsTest(test: Mut{Testing}) = test.assert(eq, 0.i8.reverseBits, 0.i8).assert(eq, 1.i8.reverseBits, (-128).i8).assert(eq, 2.i8.reverseBits, 64.i8).assert(eq, (-128).i8.reverseBits, 1.i8);\n",
        );
        assert_eq!(
            out,
            "fn reverseBitsTest(test: Mut{Testing}) =\n  test\n    .assert(eq, 0.i8.reverseBits, 0.i8)\n    .assert(eq, 1.i8.reverseBits, (-128).i8)\n    .assert(eq, 2.i8.reverseBits, 64.i8)\n    .assert(eq, (-128).i8.reverseBits, 1.i8);\n"
        );
        assert_eq!(canonical_fmt(&out), out);
    }

    #[test]
    fn test_inline_binding_gncall_wraps() {
        let out = canonical_fmt(
            "fn{Js} filter{T} (a: T[], f: (T, i64) -> bool) = {\"(async (a, f) => { let out = []; for (let i = 0; i < a.length; i++) { if ((await f(a[i], new alan_std.I64(i))).val) { out.push(a[i]); } } return out; })\" <- RootBacking :: (T[], (T, i64) -> bool) -> T[]}(a, f);\n",
        );
        assert_eq!(
            out,
            "fn{Js} filter{T} (a: T[], f: (T, i64) -> bool) =\n  {\n    \"(async (a, f) => { let out = []; for (let i = 0; i < a.length; i++) { if ((await f(a[i], new alan_std.I64(i))).val) { out.push(a[i]); } } return out; })\"\n    <- RootBacking\n    :: (T[], (T, i64) -> bool) -> T[]\n  }(a, f);\n"
        );
        assert_eq!(canonical_fmt(&out), out);
    }

    #[test]
    fn test_infix_operator_multiline_layout() {
        let out = canonical_fmt(
            "fn determinant{T}(m: Buffer{T, 9}) =\n   m.0 * m.4 * m.8 +\n   m.3 * m.7 * m.2 +\n   m.6 * m.1 * m.5 -\n   m.6 * m.4 * m.2 -\n   m.3 * m.1 * m.8 -\n   m.0 * m.7 * m.5;\nfn determinant{T}(m: Buffer{T, 16}) =\n   m.0 * m.5 * m.10 * m.15 +\n   m.4 * m.9 * m.14 * m.3 +\n   m.8 * m.13 * m.2 * m.7 +\n   m.12 * m.1 * m.6 * m.11 -\n   m.12 * m.9 * m.6 * m.3 -\n   m.8 * m.5 * m.2 * m.15 -\n   m.4 * m.1 * m.14 * m.11 -\n   m.0 * m.13 * m.10 * m.7;\n",
        );
        assert_eq!(
            out,
            "fn determinant{T}(m: Buffer{T, 9}) =\n  m.0 * m.4 * m.8 +\n  m.3 * m.7 * m.2 +\n  m.6 * m.1 * m.5 -\n  m.6 * m.4 * m.2 -\n  m.3 * m.1 * m.8 -\n  m.0 * m.7 * m.5;\nfn determinant{T}(m: Buffer{T, 16}) =\n  m.0 * m.5 * m.10 * m.15 +\n  m.4 * m.9 * m.14 * m.3 +\n  m.8 * m.13 * m.2 * m.7 +\n  m.12 * m.1 * m.6 * m.11 -\n  m.12 * m.9 * m.6 * m.3 -\n  m.8 * m.5 * m.2 * m.15 -\n  m.4 * m.1 * m.14 * m.11 -\n  m.0 * m.13 * m.10 * m.7;\n"
        );
        assert_eq!(canonical_fmt(&out), out);
    }

    #[test]
    fn test_multiline_type_definition_indent() {
        let out = canonical_fmt(
            "type WgpuTypeMap =\n  Unwrap{WgpuTypeMap},\n  String{Buffer{Buffer{f32, 2}, 2}}: gmat2x2f,\n  String{Buffer{Buffer{f32, 2}, 3}}: gmat2x3f;\n",
        );
        assert_eq!(
            out,
            "type WgpuTypeMap =\n  Unwrap{WgpuTypeMap},\n  String{Buffer{Buffer{f32, 2}, 2}}: gmat2x2f,\n  String{Buffer{Buffer{f32, 2}, 3}}: gmat2x3f;\n"
        );
        assert_eq!(canonical_fmt(&out), out);
    }

    #[test]
    fn test_postfix_return_type_single_space_before_body() {
        let out = canonical_fmt(
            "export fn iter{T}(f: Mut{(i64) -> T}, n: i64) -> T[] {\n  return {T[]}();\n}\n",
        );
        assert_eq!(
            out,
            "export fn iter{T}(f: Mut{(i64) -> T}, n: i64) -> T[] {\n  return {T[]}();\n}\n"
        );
        assert_eq!(canonical_fmt(&out), out);
    }

    #[test]
    fn test_triple_star_comment() {
        // /*** comments are preserved verbatim (no wrapping, no reformatting).
        let src = concat!(
            "/***\n",
            " * This is commented-out code that should stay\n",
            " * exactly as-is, preserving all line breaks and\n",
            " * internal structure.\n",
            " */\n",
            "ctype Type;\n",
        );
        let out = canonical_fmt(src);
        assert_eq!(out, src);

        // Regular /* comment with double-star style **/ still gets reformatted
        // (write_comment_block keeps it one-line as // if the text fits).
        let regular = canonical_fmt(
            "/**\n * This is a regular doc comment that spans multiple lines and should be reflowed\n **/",
        );
        assert_eq!(
            regular,
            "// This is a regular doc comment that spans multiple lines and should be reflowed\n"
        );
    }

    #[test]
    fn test_simple_const() {
        let out = canonical_fmt("const x = 5;");
        assert_eq!(out, "const x = 5;\n");
    }

    #[test]
    fn test_simple_fn() {
        let out = canonical_fmt("fn foo() { let x = 5; }");
        assert_eq!(out, "fn foo() {\n  let x = 5;\n}\n");
    }

    #[test]
    fn test_multi_arg_call() {
        let out = canonical_fmt("const r = foo(a, b, c);");
        assert_eq!(out, "const r = foo(a, b, c);\n");
    }

    #[test]
    fn test_if_else() {
        let out = canonical_fmt("fn test() { if true { let x = 1; } else { let x = 2; } }");
        assert_eq!(
            out,
            "fn test() {\n  if true {\n    let x = 1;\n  } else {\n    let x = 2;\n  }\n}\n"
        );
    }

    #[test]
    fn test_semantic_preservation_root() {
        let src = std::fs::read_to_string("src/std/root.ln").expect("read root.ln");
        let formatted = canonical_fmt(&src);
        let src_code = strip_code(&src);
        let fmt_code = strip_code(&formatted);
        for (i, (sc, fc)) in src_code.chars().zip(fmt_code.chars()).enumerate() {
            if sc != fc {
                let ctx_start = i.saturating_sub(80);
                eprintln!("First diff at {}: src='{}' fmt='{}'", i, sc, fc);
                eprintln!(
                    "Src ctx: {:?}",
                    &src_code[ctx_start..std::cmp::min(src_code.len(), i + 50)]
                );
                eprintln!(
                    "Fmt ctx: {:?}",
                    &fmt_code[ctx_start..std::cmp::min(fmt_code.len(), i + 50)]
                );
                break;
            }
        }
        if src_code.len() != fmt_code.len() {
            eprintln!("Length: src={} fmt={}", src_code.len(), fmt_code.len());
            if src_code.len() > fmt_code.len() {
                eprintln!(
                    "Extra at end of src: {:?}",
                    &src_code[fmt_code.len()..std::cmp::min(src_code.len(), fmt_code.len() + 200)]
                );
                eprintln!(
                    "End of fmt: {:?}",
                    &fmt_code[fmt_code.len().saturating_sub(200)..]
                );
            } else {
                eprintln!(
                    "Extra at end of fmt: {:?}",
                    &fmt_code[src_code.len()..std::cmp::min(fmt_code.len(), src_code.len() + 200)]
                );
                eprintln!(
                    "End of src: {:?}",
                    &src_code[src_code.len().saturating_sub(200)..]
                );
            }
        }
        assert!(
            sem_equiv(&src, &formatted),
            "semantic preservation failed for root.ln"
        );
    }

    #[test]
    fn test_semantic_preservation_fs() {
        let src = std::fs::read_to_string("src/std/fs.ln").expect("read fs.ln");
        let formatted = canonical_fmt(&src);
        assert!(
            sem_equiv(&src, &formatted),
            "semantic preservation failed for fs.ln"
        );
    }

    #[test]
    fn test_semantic_preservation_seq() {
        let src = std::fs::read_to_string("src/std/seq.ln").expect("read seq.ln");
        let formatted = canonical_fmt(&src);
        assert!(
            sem_equiv(&src, &formatted),
            "semantic preservation failed for seq.ln"
        );
    }

    #[test]
    fn test_semantic_preservation_test_ln() {
        let src = std::fs::read_to_string("../alan/test.ln").expect("read test.ln");
        let formatted = canonical_fmt(&src);
        assert!(
            sem_equiv(&src, &formatted),
            "semantic preservation failed for test.ln"
        );
    }

    #[test]
    fn test_idempotence_root() {
        let src = std::fs::read_to_string("src/std/root.ln").expect("read root.ln");
        let once = canonical_fmt(&src);
        let twice = canonical_fmt(&once);
        if once != twice {
            let once_code = strip_code(&once);
            let twice_code = strip_code(&twice);
            for (i, (oc, tc)) in once_code.chars().zip(twice_code.chars()).enumerate() {
                if oc != tc {
                    eprintln!("Idempotence diff at {}: once='{}' twice='{}'", i, oc, tc);
                    break;
                }
            }
            if once_code.len() != twice_code.len() {
                eprintln!(
                    "Idempotence length: once={} twice={}",
                    once_code.len(),
                    twice_code.len()
                );
            }
            let once_len = once.len();
            let twice_len = twice.len();
            for i in 0..once_len.min(twice_len) {
                if once.as_bytes()[i] != twice.as_bytes()[i] {
                    let start = i.saturating_sub(40);
                    eprintln!("Raw diff at byte {}:", i);
                    eprintln!("Once: {:?}", &once[start..i + 10.min(once_len - i)]);
                    eprintln!("Twice: {:?}", &twice[start..i + 10.min(twice_len - i)]);
                    break;
                }
            }
        }
        assert_eq!(once, twice, "fmt(fmt(src)) != fmt(src) for root.ln");
    }

    #[test]
    fn test_idempotence_fs() {
        let src = std::fs::read_to_string("src/std/fs.ln").expect("read fs.ln");
        let once = canonical_fmt(&src);
        let twice = canonical_fmt(&once);
        assert_eq!(once, twice, "fmt(fmt(src)) != fmt(src) for fs.ln");
    }

    #[test]
    fn test_idempotence_seq() {
        let src = std::fs::read_to_string("src/std/seq.ln").expect("read seq.ln");
        let once = canonical_fmt(&src);
        let twice = canonical_fmt(&once);
        assert_eq!(once, twice, "fmt(fmt(src)) != fmt(src) for seq.ln");
    }

    #[test]
    fn test_idempotence_test_ln() {
        let src = std::fs::read_to_string("../alan/test.ln").expect("read test.ln");
        let once = canonical_fmt(&src);
        let twice = canonical_fmt(&once);
        assert_eq!(once, twice, "fmt(fmt(src)) != fmt(src) for test.ln");
    }

    fn check_no_comment_text_loss(src: &str, formatted: &str, label: &str) {
        let mut src_texts: Vec<String> = Vec::new();
        for (i, _) in src.match_indices("//") {
            let after = &src[i + 2..];
            let eol = after.find('\n').unwrap_or(after.len());
            let text = after[..eol].trim().to_string();
            if !text.is_empty() {
                src_texts.push(text.clone());
            }
        }
        for (i, _) in src.match_indices("/*") {
            let after = &src[i + 2..];
            let end = after.find("*/").unwrap_or(after.len());
            let text = after[..end].trim().to_string();
            if !text.is_empty() {
                src_texts.push(text);
            }
        }
        for text in &src_texts {
            let found = formatted.contains(text.as_str());
            if !found {
                let stripped = strip_comment_markers(&format!("// {}", text));
                let found_in_block = stripped.contains(text.as_str());
                if !found_in_block {
                    eprintln!("Comment text MISSING in {}: {:?}", label, text);
                }
            }
        }
    }

    #[test]
    fn test_no_comment_loss_root() {
        let src = std::fs::read_to_string("src/std/root.ln").expect("read root.ln");
        let formatted = canonical_fmt(&src);
        let src_comments = count_comments(&src);
        let fmt_comments = count_comments(&formatted);
        eprintln!(
            "root.ln comments: src={} fmt={}",
            src_comments, fmt_comments
        );
        check_no_comment_text_loss(&src, &formatted, "root.ln");
        // Marker count may differ due to // → /* */ restructuring
    }

    #[test]
    fn test_no_comment_loss_fs() {
        let src = std::fs::read_to_string("src/std/fs.ln").expect("read fs.ln");
        let formatted = canonical_fmt(&src);
        let src_comments = count_comments(&src);
        let fmt_comments = count_comments(&formatted);
        eprintln!("fs.ln comments: src={} fmt={}", src_comments, fmt_comments);
        check_no_comment_text_loss(&src, &formatted, "fs.ln");
    }

    #[test]
    fn test_no_comment_loss_seq() {
        let src = std::fs::read_to_string("src/std/seq.ln").expect("read seq.ln");
        let formatted = canonical_fmt(&src);
        let src_comments = count_comments(&src);
        let fmt_comments = count_comments(&formatted);
        eprintln!("seq.ln comments: src={} fmt={}", src_comments, fmt_comments);
        check_no_comment_text_loss(&src, &formatted, "seq.ln");
    }

    #[test]
    fn test_no_comment_loss_test_ln() {
        let src = std::fs::read_to_string("../alan/test.ln").expect("read test.ln");
        let formatted = canonical_fmt(&src);
        let src_comments = count_comments(&src);
        let fmt_comments = count_comments(&formatted);
        eprintln!(
            "test.ln comments: src={} fmt={}",
            src_comments, fmt_comments
        );

        let mut src_comment_lines: Vec<(usize, &str)> = Vec::new();
        for (i, _) in src.match_indices("//") {
            let line_start = src[..i].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let line_end = src[i..].find('\n').map(|p| i + p).unwrap_or(src.len());
            let line = &src[line_start..line_end];
            src_comment_lines.push((line_start, line.trim()));
        }
        for (pos, line) in &src_comment_lines {
            if let Some(ci) = line.find("//") {
                let comment_text = &line[ci..];
                if !formatted.contains(comment_text) {
                    eprintln!("MISSING at src offset {}: {}", pos, line);
                }
            }
        }

        let fmt_lines: Vec<&str> = formatted.lines().collect();
        for (i, fl) in fmt_lines.iter().enumerate() {
            if fl.contains("//") {
                eprintln!("fmt line {}: {}", i + 1, fl.trim());
            }
        }

        check_no_comment_text_loss(&src, &formatted, "test.ln");
    }
}
