use syntax::ast::{Expr, Lit, TokenTree, TtToken};
use syntax::ext::base::ExtCtxt;
use syntax::parse;
use syntax::parse::token;
use syntax::ptr::P;

#[deriving(Show)]
pub enum Markup {
    Empty,
    Element(Vec<(String, Value)>, Vec<Markup>),
    Value(Value),
}

#[deriving(Show)]
pub struct Value {
    pub value: Value_,
    pub escape: Escape,
}

impl Value {
    pub fn escape(value: Value_) -> Value {
        Value {
            value: value,
            escape: Escape::Escape,
        }
    }

    pub fn no_escape(value: Value_) -> Value {
        Value {
            value: value,
            escape: Escape::NoEscape,
        }
    }
}

#[deriving(Show)]
pub enum Value_ {
    Literal(String),
    Splice(P<Expr>),
}

#[deriving(Copy, PartialEq, Show)]
pub enum Escape {
    NoEscape,
    Escape,
}

pub fn parse(cx: &mut ExtCtxt, mut args: &[TokenTree]) -> Option<Vec<Markup>> {
    let mut result = vec![];
    loop {
        match parse_markup(cx, &mut args) {
            Markup::Empty => break,
            markup => result.push(markup),
        }
    }
    // If not all tokens were consumed, then there must have been an
    // error somewhere
    match args {
        [] => Some(result),
        _ => None,
    }
}

fn parse_markup(cx: &mut ExtCtxt, args: &mut &[TokenTree]) -> Markup {
    if let Some(s) = parse_literal(cx, args) {
        Markup::Value(Value::escape(Value_::Literal(s)))
    } else {
        match *args {
            [] => Markup::Empty,
            [ref tt, ..] => {
                cx.span_err(tt.get_span(), "invalid syntax");
                Markup::Empty
            },
        }
    }
}

fn parse_literal(cx: &mut ExtCtxt, args: &mut &[TokenTree]) -> Option<String> {
    let minus = match *args {
        [TtToken(_, token::BinOp(token::Minus)), ..] => {
            args.shift(1);
            true
        },
        _ => false,
    };

    match *args {
        [ref tt @ TtToken(_, token::Literal(..)), ..] => {
            args.shift(1);
            let mut parser = parse::tts_to_parser(cx.parse_sess, vec![tt.clone()], cx.cfg.clone());
            let lit = parser.parse_lit();
            lit_to_string(cx, lit, minus)
        },
        _ => None,
    }
}

fn lit_to_string(cx: &mut ExtCtxt, lit: Lit, minus: bool) -> Option<String> {
    use syntax::ast::Lit_::*;
    let mut result = String::new();
    if minus {
        result.push('-');
    }
    match lit.node {
        LitStr(s, _) => result.push_str(s.get()),
        LitBinary(..) | LitByte(..) => {
            cx.span_err(lit.span, "cannot splice binary data");
            return None;
        },
        LitChar(c) => result.push(c),
        LitInt(x, _) => result.push_str(&*x.to_string()),
        LitFloat(s, _) | LitFloatUnsuffixed(s) => result.push_str(s.get()),
        LitBool(b) => result.push_str(if b { "true" } else { "false" }),
    };
    Some(result)
}

trait Shift {
    fn shift(&mut self, n: uint);
}

impl<'a, T> Shift for &'a [T] {
    fn shift(&mut self, n: uint) {
        *self = self.slice_from(n);
    }
}
