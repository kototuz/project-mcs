use crate::lexer as lex;
use std::process;
use std::ops::Range;

#[derive(Debug)]
pub enum Stmt<'a> {
    VarAssign { name: &'a str, expr: Range<usize> }
}

#[derive(Debug,  Clone, PartialEq)]
pub enum OpKind {
    Sub,
    Add,
    Mul,
    Div,
}

#[derive(Debug, PartialEq)]
pub enum Expr<'a> {
    Var(&'a str),
    Num(i32),
    Op(OpKind)
}



pub fn parse<'a>(lex: &mut lex::Lexer<'a>) -> (Vec<Expr<'a>>, Vec<Stmt<'a>>) {
    use lex::*;

    let mut expr_buf: Vec<Expr> = Vec::new();
    let mut stmt_buf: Vec<Stmt> = Vec::new();

    loop {
        if let Some(tok) = lex.next() {
            match tok.kind {
                TokenKind::Name => {
                    let _ = lex.expect_specific_next(TokenKind::Eq);
                    let expr = parse_expr(&mut expr_buf, lex);
                    stmt_buf.push(Stmt::VarAssign { name: tok.data, expr });
                    let _ = lex.expect_specific_next(TokenKind::Semicolon);
                },
                _ => todo!("now only variable assign stmt is avilable")
            }
        } else { return (expr_buf, stmt_buf); }
    }
}

fn parse_expr<'a>(expr_buf: &mut Vec<Expr<'a>>, lex: &mut lex::Lexer<'a>) -> Range<usize> {
    use lex::*;

    fn parse_num(src: &str) -> i32 {
        src.parse::<i32>().unwrap_or_else(|err| {
            eprintln!("ERROR: parser: could not parse num `{src}`: {err}");
            process::exit(1);
        })
    }

    // TODO: make seperate function for lexer. Something like `expect_oneof_next()`
    fn expect_read<'a>(lex: &mut Lexer<'a>) -> Expr<'a> {
        let tok = lex.expect_next();
        match tok.kind {
            TokenKind::Name => Expr::Var(tok.data),
            TokenKind::Num  => Expr::Num(parse_num(tok.data)),
            _ => unreachable!()
        }
    }

    // 1 + 1     => 11+
    // 1 + 1 + 1 => 11+ 1+ 1+ 1+ 1+ 1+
    // 1 - 1 - 1 => 11- 1-
    // 1 + 1 * 1 => 1 11* +
    // 1 + 1 / 1 => 1 11/ +

    let mut ret = Range { start: expr_buf.len(), end: 0 };

    expr_buf.push(expect_read(lex));

    let mut prev_op = match lex.expect_peek().kind {
        TokenKind::Plus  => OpKind::Add,
        TokenKind::Minus => OpKind::Sub,
        TokenKind::Star  => OpKind::Mul,
        TokenKind::Slash => OpKind::Div,
        _ => {
            ret.end = expr_buf.len();
            return ret;
        }
    };
    let _ = lex.next();

    expr_buf.push(expect_read(lex));
    loop {
        // 1 + 2 * 3 => 1 23* +
        // 1 * 2 + 3 => 12* 3 +
        // 1 * 2 * 3 => 1 23* *
        // 1 + 2 * 3 * 4 => 1 23* 4* 5* 6*
        // 1 + 2 * 3 + 2 => 1 23*+ 2
        // 1 + 2 / 3 * 4 => 123/ 4*+
        // 1 / 2 * 3 * 4 * 5 => 12/34*5*
        // 1 / 2 / 3 * 4 / 5 => 1 23/ /4*5/

        prev_op = match lex.expect_peek().kind {
            TokenKind::Plus  => {
                let _ = lex.next();
                expr_buf.push(Expr::Op(prev_op));
                OpKind::Add
            },
            TokenKind::Star  => {
                let _ = lex.next();
                if prev_op != OpKind::Div {
                    expr_buf.push(expect_read(lex));
                    expr_buf.push(Expr::Op(OpKind::Mul));
                    continue;
                }
                expr_buf.push(Expr::Op(prev_op));
                OpKind::Mul
            },
            TokenKind::Slash => {
                let _ = lex.next();
                if prev_op != OpKind::Mul {
                    expr_buf.push(expect_read(lex));
                    expr_buf.push(Expr::Op(OpKind::Div));
                    continue;
                }
                expr_buf.push(Expr::Op(prev_op));
                OpKind::Div
            },
            _  => {
                expr_buf.push(Expr::Op(prev_op));
                ret.end = expr_buf.len();
                return ret;
            }
        };

        expr_buf.push(expect_read(lex));
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() {
        use super::Expr::*;

        // Syntax: v1;v2;op
        // 1 + 2          =>   12+
        // 1 + 2 + 3      =>   12+ 3 +
        // 1 + 2*3        =>   1 23* +
        // 1 * 2 * 3      =>   1 23* *
        // 1 + 2*3*4      =>   1 23* 4* +
        // 1 + 2*3*4 + 5  =>   1 23* 4* + 5+
        // 1 + 2*3 + 4*5  =>   1 23* + 45* +
        // 1 / 2 * 3      =>   12/ 3*
        // 1 / 2 * 3 * 4  =>   12/ 34**
        // 1 / 2 * 3 / 4  =>   12/ 3* 4/
        let map: &[(&str, &[Expr])] = &[
            ("1 + 2;",         &[Num(1), Num(2), Op(OpKind::Add)]),
            ("1 + 2 + 3;",     &[Num(1), Num(2), Op(OpKind::Add), Num(3), Op(OpKind::Add)]),
            ("1 + 2*3;",       &[Num(1), Num(2), Num(3), Op(OpKind::Mul), Op(OpKind::Add)]),
            ("1 * 2 * 3;",     &[Num(1), Num(2), Num(3), Op(OpKind::Mul), Op(OpKind::Mul)]),
            ("1 + 2*3*4;",     &[Num(1), Num(2), Num(3), Op(OpKind::Mul), Num(4), Op(OpKind::Mul), Op(OpKind::Add)]),
            ("1 + 2*3*4 + 5;", &[Num(1), Num(2), Num(3), Op(OpKind::Mul), Num(4), Op(OpKind::Mul), Op(OpKind::Add), Num(5), Op(OpKind::Add)]),
            ("1 + 2*3 + 4*5;", &[Num(1), Num(2), Num(3), Op(OpKind::Mul), Op(OpKind::Add), Num(4), Num(5), Op(OpKind::Mul), Op(OpKind::Add)]),
            ("1 / 2 * 3;",     &[Num(1), Num(2), Op(OpKind::Div), Num(3), Op(OpKind::Mul)]),
            ("1 / 2 * 3 * 4;", &[Num(1), Num(2), Op(OpKind::Div), Num(3), Num(4), Op(OpKind::Mul), Op(OpKind::Mul)]),
            ("1 / 2 * 3 / 4;", &[Num(1), Num(2), Op(OpKind::Div), Num(3), Op(OpKind::Mul), Num(4), Op(OpKind::Div)])
        ];

        let mut exprs: Vec<Expr> = Vec::new();
        for test in map {
            let range = parse_expr(&mut exprs, &mut lex::Lexer::new(test.0));
            for x in range {
                assert_eq!(exprs[x], test.1[x]);
            }

            exprs.clear();
        }

    //    let Stmt::VarAssign { name, expr } = &stmts[0];
    //    assert_eq!(*name, "a");
    //    assert_eq!(*expr, (0..expected.len()));
    //
    //    for (i, expr) in exprs.iter().enumerate() {
    //        assert_eq!(*expr, expected[i]);
    //    }
    }
}
