//! Scope analysis: it walks the AST and decides, for every identifier token, whether it names a
//! local (and what new name it should get) or a global or field (left alone). It also marks the
//! string and number literals that sit in plain expression positions, so [`crate::emit`] can
//! encode them without touching the `f"x"` / `t.field` / `{key=v}` forms where that would be
//! wrong.

use std::collections::HashMap;

use full_moon::ast::{
    Ast, Block, Call, Expression, Field, FunctionArgs, FunctionBody, Index, LastStmt, Prefix, Stmt, Suffix,
    TableConstructor, Var,
};
use full_moon::tokenizer::{TokenReference, TokenType};

use crate::Options;

/// Everything the serializer needs, keyed by each token's start byte.
#[derive(Default)]
pub struct Plan {
    /// Identifier token byte -> its new name.
    pub renames: HashMap<usize, String>,
    /// String literal token byte -> the bytes it stands for (only safe, quoted literals).
    pub strings: HashMap<usize, Vec<u8>>,
    /// Number literal token byte -> its integer value (only plain decimal integers).
    pub numbers: HashMap<usize, u64>,
    /// Every identifier that stays a global (so the bundler can pick a non-clashing helper name).
    pub globals: std::collections::BTreeSet<String>,
}

struct Resolver {
    plan: Plan,
    scopes: Vec<HashMap<String, String>>,
    counter: usize,
    opts: Options,
}

const ALPHABET: [char; 4] = ['l', 'I', 'o', 'O'];

fn ident(token: &TokenReference) -> Option<&str> {
    match token.token().token_type() {
        TokenType::Identifier { identifier } => Some(identifier.as_str()),
        _ => None,
    }
}

impl Resolver {
    fn fresh(&mut self) -> String {
        let mut n = self.counter;
        self.counter += 1;
        let mut s = String::new();
        loop {
            s.push(ALPHABET[n % 4]);
            if n < 4 {
                break;
            }
            n = n / 4 - 1;
        }
        s
    }

    fn bind(&mut self, token: &TokenReference) {
        let Some(name) = ident(token) else { return };
        if !self.opts.rename {
            return;
        }
        let new = self.fresh();
        self.plan.renames.insert(token.token().start_position().bytes(), new.clone());
        self.scopes.last_mut().expect("a scope is open").insert(name.to_string(), new);
    }

    fn reference(&mut self, token: &TokenReference) {
        let Some(name) = ident(token) else { return };
        for scope in self.scopes.iter().rev() {
            if let Some(new) = scope.get(name) {
                self.plan.renames.insert(token.token().start_position().bytes(), new.clone());
                return;
            }
        }
        self.plan.globals.insert(name.to_string());
    }

    fn block(&mut self, block: &Block) {
        self.scopes.push(HashMap::new());
        self.stmts(block);
        self.scopes.pop();
    }

    fn stmts(&mut self, block: &Block) {
        for stmt in block.stmts() {
            self.stmt(stmt);
        }
        if let Some(LastStmt::Return(ret)) = block.last_stmt() {
            for e in ret.returns() {
                self.expr(e);
            }
        }
    }

    fn stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assignment(a) => {
                for v in a.variables() {
                    self.var(v);
                }
                for e in a.expressions() {
                    self.expr(e);
                }
            }
            Stmt::LocalAssignment(la) => {
                for e in la.expressions() {
                    self.expr(e);
                }
                for name in la.names() {
                    self.bind(name);
                }
            }
            Stmt::LocalFunction(lf) => {
                self.bind(lf.name());
                self.function_body(lf.body());
            }
            Stmt::FunctionDeclaration(fd) => {
                // `function a.b.c:d()`: only the first name is a variable; the rest are fields.
                if let Some(first) = fd.name().names().iter().next() {
                    self.reference(first);
                }
                self.function_body(fd.body());
            }
            Stmt::FunctionCall(fc) => self.prefix_suffixes(fc.prefix(), fc.suffixes()),
            Stmt::Do(d) => self.block(d.block()),
            Stmt::While(w) => {
                self.expr(w.condition());
                self.block(w.block());
            }
            Stmt::Repeat(r) => {
                // The `until` condition can see the locals declared in the loop body.
                self.scopes.push(HashMap::new());
                self.stmts(r.block());
                self.expr(r.until());
                self.scopes.pop();
            }
            Stmt::If(f) => {
                self.expr(f.condition());
                self.block(f.block());
                if let Some(elseifs) = f.else_if() {
                    for ei in elseifs {
                        self.expr(ei.condition());
                        self.block(ei.block());
                    }
                }
                if let Some(eb) = f.else_block() {
                    self.block(eb);
                }
            }
            Stmt::NumericFor(nf) => {
                self.expr(nf.start());
                self.expr(nf.end());
                if let Some(step) = nf.step() {
                    self.expr(step);
                }
                self.scopes.push(HashMap::new());
                self.bind(nf.index_variable());
                self.stmts(nf.block());
                self.scopes.pop();
            }
            Stmt::GenericFor(gf) => {
                for e in gf.expressions() {
                    self.expr(e);
                }
                self.scopes.push(HashMap::new());
                for name in gf.names() {
                    self.bind(name);
                }
                self.stmts(gf.block());
                self.scopes.pop();
            }
            // goto / label and any luau-only statements carry no local reads that need renaming.
            _ => {}
        }
    }

    fn var(&mut self, var: &Var) {
        match var {
            Var::Name(token) => self.reference(token),
            Var::Expression(ve) => self.prefix_suffixes(ve.prefix(), ve.suffixes()),
            _ => {}
        }
    }

    fn prefix_suffixes<'a>(&mut self, prefix: &Prefix, suffixes: impl Iterator<Item = &'a Suffix>) {
        match prefix {
            Prefix::Name(token) => self.reference(token),
            Prefix::Expression(e) => self.expr(e),
            _ => {}
        }
        for s in suffixes {
            self.suffix(s);
        }
    }

    fn suffix(&mut self, suffix: &Suffix) {
        match suffix {
            Suffix::Call(Call::AnonymousCall(args)) => self.args(args),
            Suffix::Call(Call::MethodCall(mc)) => self.args(mc.args()),
            Suffix::Index(Index::Brackets { expression, .. }) => self.expr(expression),
            Suffix::Index(Index::Dot { .. }) => {}
            _ => {}
        }
    }

    fn args(&mut self, args: &FunctionArgs) {
        match args {
            FunctionArgs::Parentheses { arguments, .. } => {
                for e in arguments {
                    self.expr(e);
                }
            }
            FunctionArgs::TableConstructor(tc) => self.table(tc),
            // A bare `f"text"` string argument is left as it is (encoding it would need `f(...)`).
            FunctionArgs::String(_) => {}
            _ => {}
        }
    }

    fn table(&mut self, tc: &TableConstructor) {
        for field in tc.fields() {
            match field {
                Field::ExpressionKey { key, value, .. } => {
                    self.expr(key);
                    self.expr(value);
                }
                Field::NameKey { value, .. } => self.expr(value),
                Field::NoKey(e) => self.expr(e),
                _ => {}
            }
        }
    }

    fn expr(&mut self, expr: &Expression) {
        match expr {
            Expression::BinaryOperator { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            Expression::Parentheses { expression, .. } => self.expr(expression),
            Expression::UnaryOperator { expression, .. } => self.expr(expression),
            Expression::Function(anon) => self.function_body(anon.body()),
            Expression::FunctionCall(fc) => self.prefix_suffixes(fc.prefix(), fc.suffixes()),
            Expression::TableConstructor(tc) => self.table(tc),
            Expression::Var(v) => self.var(v),
            Expression::Number(token) => self.mark_number(token),
            Expression::String(token) => self.mark_string(token),
            // Symbols (nil/true/false/...) need nothing; luau-only forms cannot occur here.
            _ => {}
        }
    }

    fn function_body(&mut self, body: &FunctionBody) {
        self.scopes.push(HashMap::new());
        for param in body.parameters() {
            if let full_moon::ast::Parameter::Name(token) = param {
                self.bind(token);
            }
        }
        self.stmts(body.block());
        self.scopes.pop();
    }

    fn mark_number(&mut self, token: &TokenReference) {
        if !self.opts.rewrite_numbers {
            return;
        }
        if let TokenType::Number { text } = token.token().token_type() {
            let t = text.as_str();
            if !t.is_empty()
                && t.bytes().all(|b| b.is_ascii_digit())
                && let Ok(v) = t.parse::<u64>()
            {
                self.plan.numbers.insert(token.token().start_position().bytes(), v);
            }
        }
    }

    fn mark_string(&mut self, token: &TokenReference) {
        if !self.opts.encode_strings {
            return;
        }
        if let TokenType::StringLiteral { literal, multi_line_depth, quote_type } = token.token().token_type() {
            use full_moon::tokenizer::StringLiteralQuoteType::{Double, Single};
            if *multi_line_depth == 0
                && matches!(quote_type, Single | Double)
                && let Some(bytes) = crate::unescape::unescape(literal.as_str())
            {
                self.plan.strings.insert(token.token().start_position().bytes(), bytes);
            }
        }
    }
}

/// Analyses the whole chunk.
pub fn plan(ast: &Ast, opts: &Options) -> Plan {
    let mut r = Resolver { plan: Plan::default(), scopes: Vec::new(), counter: 0, opts: opts.clone() };
    r.block(ast.nodes());
    r.plan
}
