use std::error::Error;
use std::fmt;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum Op {
    Add,
    Div,
    Mul,
    Sub,
}

#[derive(Debug, Eq, PartialEq)]
enum Token {
    EOF,
    Number(i32),
    Operation(Op),
    LeftParen,
    RightParen,
}

#[derive(Debug, Eq, PartialEq)]
enum Side {
    Node(Box<AST>),
    Leaf(i32),
}

#[derive(Debug, Eq, PartialEq)]
struct AST {
    operation: Op,
    left: Side,
    right: Side,
}

#[derive(Debug, Eq, PartialEq)]
struct BadInput {
    unexpected: char,
}

impl fmt::Display for BadInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "invalid syntax. saw unexpected character: {:?}",
            self.unexpected
        )
    }
}

impl Error for BadInput {}

#[derive(Debug, Eq, PartialEq)]
struct ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "parse error")
    }
}

impl Error for ParseError {}

fn lex(input: &str) -> Result<Vec<Token>, BadInput> {
    let mut result: Vec<Token> = Vec::new();

    for character in input.chars() {
        use Op::*;
        use Token::*;

        match character {
            ' ' => continue,
            ';' | '\n' => {
                result.push(Token::EOF);
                break;
            }
            '+' => result.push(Operation(Add)),
            '/' => result.push(Operation(Div)),
            '*' => result.push(Operation(Mul)),
            '-' => result.push(Operation(Sub)),
            '(' => result.push(LeftParen),
            ')' => result.push(RightParen),
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                let num: i32 = (character as u8 - '0' as u8) as i32;

                if result.len() == 0 {
                    result.push(Number(num));
                    continue;
                }

                let last = result.pop().unwrap();

                match last {
                    Number(i) => {
                        result.push(Number((i * 10) + num));
                    }
                    _ => {
                        result.push(last);
                        result.push(Number(num));
                    }
                }
            }

            _ => {
                return Err(BadInput {
                    unexpected: character,
                })
            }
        }
    }

    Ok(result)
}

fn parse(tokens: &[Token]) -> Result<i32, ParseError> {
    use Token::*;

    let mut depth = 0;
    let mut op_stack: Vec<(usize, Op, i32)> = Vec::new();
    let mut value: Option<i32> = None;

    println!("{:?}", tokens);

    for token in tokens {
        println!("token: {:?}", token);
        match token {
            Operation(op) => {
                if let Some(operand) = value {
                    op_stack.push((depth, *op, operand));
                    value = None;
                } else {
                    return Err(ParseError {});
                }
            }
            Number(number) => {
                if let Some((op_depth, op, operand)) = op_stack.pop() {
                    op_stack.push((op_depth, op, operand));

                    if op_depth == depth {
                        return Ok(eval(&mut op_stack, *number));
                    } else {
                        value = Some(*number)
                    }
                } else {
                    value = Some(*number)
                }
            }
            LeftParen => depth += 1,
            RightParen => depth -= 1,
            EOF => {}
        }
        println!(
            "depth: {:?} value: {:?} op_stack: {:?}",
            depth, value, op_stack
        );
    }

    Err(ParseError {})
}

fn eval(op_stack: &mut Vec<(usize, Op, i32)>, initial_value: i32) -> i32 {
    println!(
        "eval called with op_stack={:?} initial_value={:?}",
        op_stack, initial_value
    );

    if let Some((_op_depth, op, operand)) = op_stack.pop() {
        eval(op_stack, eval_op(op, operand, initial_value))
    } else {
        initial_value
    }
}

fn eval_op(op: Op, left: i32, right: i32) -> i32 {
    println!("eval_op({:?}, {:?}, {:?})", op, left, right);
    use Op::*;

    match op {
        Add => left + right,
        Sub => left - right,
        Mul => left * right,
        Div => left / right,
    }
}

fn unwind_with_ast(op_stack: &mut Vec<(usize, Op, i32)>, ast: AST) -> Result<AST, ParseError> {
    println!(
        "unwind_with_ast called with op_stack={:?} ast={:?}",
        op_stack, ast
    );
    use Side::*;
    if let Some((_op_depth, op, operand)) = op_stack.pop() {
        unwind_with_ast(
            op_stack,
            AST {
                operation: op,
                left: Leaf(operand),
                right: Node(Box::new(ast)),
            },
        )
    } else {
        Ok(ast)
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::{Op::*, Side::*, Token::*, *};

    #[test]
    fn lexing() {
        assert!(lex("420 + 69").is_ok());
        assert!(lex("mmmm, brains").is_err());

        assert_eq!(lex(""), Ok(vec![]));
        assert_eq!(lex("2+2"), Ok(vec![Number(2), Operation(Add), Number(2)]));
        assert_eq!(
            lex("(2+(3+4)*5)/6 - 1"),
            Ok(vec![
                LeftParen,
                Number(2),
                Operation(Add),
                LeftParen,
                Number(3),
                Operation(Add),
                Number(4),
                RightParen,
                Operation(Mul),
                Number(5),
                RightParen,
                Operation(Div),
                Number(6),
                Operation(Sub),
                Number(1)
            ])
        );
    }

    #[test]
    fn parsing() {
        assert_eq!(
            parse(&mut vec![Number(2), Operation(Add), Number(2)]),
            Ok(4)
        );

        assert_eq!(
            parse(&mut vec![
                Number(5),
                Operation(Add),
                LeftParen,
                Number(4),
                Operation(Sub),
                Number(3),
                RightParen
            ]),
            Ok(6)
        );
    }
}
