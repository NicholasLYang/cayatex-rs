use std::char;
use std::fmt::Debug;
use std::ops::Range;
use thiserror::Error;

pub struct Parser {
    source: Vec<u8>,
    idx: usize,
}

#[derive(Debug)]
pub struct Loc<T: Debug> {
    range: Range<usize>,
    inner: T,
}

macro_rules! loc {
    ($start:expr, $end:expr, $inner:expr) => {
        Loc {
            range: ($start)..($end),
            inner: $inner,
        }
    };
}

type Span = Range<usize>;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("right bracket without matching left bracket. You can escape by prefixing the bracket with `\\`")]
    UnmatchedRightBracket,
    #[error("end of file reached, expected {}", expected)]
    EndOfFile { expected: String },
    #[error("expected {}, received {}", expected, received)]
    UnexpectedChar { expected: String, received: String },
}

#[derive(Debug)]
pub enum Expr {
    Inline {
        name: Span,
        args: Vec<Span>,
        body: Vec<Loc<Expr>>,
    },
    Block {
        name: Span,
        args: Vec<Span>,
        body: Vec<Loc<Expr>>,
    },
    Text(Span),
}

impl Parser {
    pub fn new<T: Into<Vec<u8>>>(source: T) -> Self {
        Parser {
            source: source.into(),
            idx: 0,
        }
    }

    fn peek(&mut self) -> Option<(usize, u8)> {
        self.source.get(self.idx).map(|c| (self.idx, *c))
    }

    fn bump(&mut self) -> Option<(usize, u8)> {
        if self.idx < self.source.len() {
            let idx = self.idx;
            self.idx += 1;
            Some((idx, self.source[idx]))
        } else {
            None
        }
    }

    pub fn parse_document(mut self) -> Result<Vec<Loc<Expr>>, Loc<ParseError>> {
        let mut exprs = Vec::new();
        let mut start_idx: usize = 0;
        while let Some((idx, c)) = self.bump() {
            match c {
                // Slight repetition here. If necessary will refactor
                b'[' => {
                    exprs.push(loc!(start_idx, idx, Expr::Text(start_idx..idx)));
                    let inline_expr = self.parse_inline(idx)?;
                    start_idx = inline_expr.range.end + 1;
                    exprs.push(inline_expr);
                }
                b'{' => {
                    exprs.push(loc!(start_idx, idx, Expr::Text(start_idx..idx)));
                    let block_expr = self.parse_block(idx)?;
                    start_idx = block_expr.range.end + 1;
                    exprs.push(block_expr);
                }
                b']' | b'}' => return Err(loc!(idx, idx, ParseError::UnmatchedRightBracket)),
                _ => {}
            }
        }
        Ok(exprs)
    }

    fn take_whitespace(&mut self) {
        while let Some((_, c)) = self.peek() {
            if c.is_ascii_whitespace() {
                self.bump();
            } else {
                return;
            }
        }
    }

    fn expect_char(&mut self, expected_char: u8) -> Result<(), Loc<ParseError>> {
        let (idx, c) = self.bump().ok_or_else(|| {
            loc!(
                self.source.len() - 1,
                self.source.len() - 1,
                ParseError::EndOfFile {
                    expected: char::from_digit(expected_char as u32, 10)
                        .unwrap()
                        .to_string(),
                }
            )
        })?;

        if c == expected_char {
            Ok(())
        } else {
            Err(loc!(
                idx,
                idx,
                ParseError::UnexpectedChar {
                    expected: (expected_char as char).to_string(),
                    received: (c as char).to_string(),
                }
            ))
        }
    }

    fn parse_inline(&mut self, start_idx: usize) -> Result<Loc<Expr>, Loc<ParseError>> {
        self.take_whitespace();
        let name = self.parse_name()?;
        self.take_whitespace();

        Ok(loc!(
            start_idx,
            name.end,
            Expr::Inline {
                name,
                args: Vec::new(),
                body: Vec::new()
            }
        ))
    }

    fn parse_block(&mut self, start_idx: usize) -> Result<Loc<Expr>, Loc<ParseError>> {
        self.take_whitespace();
        let name = self.parse_name()?;
        self.take_whitespace();
        self.expect_char(b'|')?;
        Ok(loc!(
            start_idx,
            name.end,
            Expr::Block {
                name,
                args: Vec::new(),
                body: Vec::new()
            }
        ))
    }

    fn parse_name(&mut self) -> Result<Span, Loc<ParseError>> {
        let (start_idx, c) = self.bump().ok_or_else(|| {
            loc!(
                self.source.len(),
                self.source.len(),
                ParseError::EndOfFile {
                    expected: "name".to_string()
                }
            )
        })?;

        if !c.is_ascii_alphabetic() {
            return Err(loc!(
                start_idx,
                start_idx,
                ParseError::UnexpectedChar {
                    expected: "letter".to_string(),
                    received: c.to_string()
                }
            ));
        }

        while let Some((idx, c)) = self.peek() {
            if c.is_ascii_alphanumeric() {
                self.bump();
            } else {
                return Ok(start_idx..idx);
            }
        }

        Ok((start_idx)..(self.source.len() - 1))
    }
}
