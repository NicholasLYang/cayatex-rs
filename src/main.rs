use crate::parser::Parser;

mod parser;

fn main() {
    let mut parser = Parser::new("hello world [bold");
    let exprs = parser.parse_document();
    println!("{:?}", exprs);
}
