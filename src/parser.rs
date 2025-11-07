use std::error::Error;

use crate::lexer::{Lexer, Token};

/// 構文木の頂点
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum Node {
    Character(char),
    Empty,
    Star(Box<Node>),
    Union(Box<Node>, Box<Node>),
    Concat(Box<Node>, Box<Node>),
}

/// パーサ
pub(crate) struct Parser<'a> {
    lexer: Lexer<'a>,
    look: Token,
}

impl Parser<'_> {
    pub fn new(mut lexer: Lexer) -> Parser {
        let look: Token = lexer.scan();
        Parser { lexer, look }
    }

    pub fn parse(&mut self) -> ParseResult<Node> {
        self.expression()
    }

    fn match_next(&mut self, token: Token) -> ParseResult<()> {
        match &self.look {
            look if *look == token => {
                self.look = self.lexer.scan();
                Ok(())
            }
            other => Err(ParseError::new(&[token], *other)),
        }
    }

    // --- 文法規則 ---

    /// <expression> ::= <sub_expression> Token::End
    fn expression(&mut self) -> ParseResult<Node> {
        let expression: Node = self.sub_expression()?;
        self.match_next(Token::End)?;
        Ok(expression)
    }

    /// <sub_expression> ::= <sequence> '|' <sub_expression> | <sequence>
    fn sub_expression(&mut self) -> ParseResult<Node> {
        let sequence: Node = self.sequence()?;
        Ok(match &self.look {
            Token::UnionOp => {
                self.match_next(Token::UnionOp)?;
                Node::Union(Box::new(sequence), Box::new(self.sub_expression()?))
            }
            _ => sequence,
        })
    }

    /// <sequence> ::= <sub_sequence> | ''
    fn sequence(&mut self) -> ParseResult<Node> {
        match &self.look {
            Token::LeftParen | Token::Character(_) => self.sub_sequence(),
            _ => Ok(Node::Empty),
        }
    }

    /// <sub_sequence> ::= <star sub_sequence> | <star>
    fn sub_sequence(&mut self) -> ParseResult<Node> {
        let star: Node = self.star()?;
        Ok(match &self.look {
            Token::LeftParen | Token::Character(_) => {
                Node::Concat(Box::new(star), Box::new(self.sub_sequence()?))
            }
            _ => star,
        })
    }

    /// <star> ::= <factor> '*' | <factor>
    fn star(&mut self) -> ParseResult<Node> {
        let factor: Node = self.factor()?;
        Ok(match &self.look {
            Token::StarOp => {
                self.match_next(Token::StarOp)?;
                Node::Star(Box::new(factor))
            }
            _ => factor,
        })
    }

    /// <factor> ::= '(' <sub_expression> ')' | Token::Character
    fn factor(&mut self) -> ParseResult<Node> {
        match &self.look {
            Token::LeftParen => {
                self.match_next(Token::LeftParen)?;
                let result: ParseResult<Node> = self.sub_expression();
                self.match_next(Token::RightParen)?;
                result
            }
            Token::Character(c) => {
                let node: Node = Node::Character(*c);
                self.match_next(Token::Character(*c))?;
                Ok(node)
            }
            other => Err(ParseError::new(
                &[Token::LeftParen, Token::Character('_')],
                *other,
            )),
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Token::Character(_) => "Character",
            Token::UnionOp => "|",
            Token::StarOp => "*",
            Token::LeftParen => "(",
            Token::RightParen => ")",
            Token::End => "EOF",
        };
        write!(f, "{}", str)
    }
}

type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub struct ParseError {
    expected: Vec<Token>,
    actual: Token,
}
impl ParseError {
    fn new(expected: &[Token], actual: Token) -> Self {
        let expected = expected.iter().map(|token| *token).collect::<Vec<_>>();
        return ParseError { expected, actual };
    }
}
impl Error for ParseError {}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let expected = self
            .expected
            .iter()
            .map(|token| format!("{}", token))
            .collect::<Vec<_>>()
            .join(", ");
        let actual = match self.actual {
            Token::Character(c) => format!("'{}'", c),
            actual => format!("'{}'", actual),
        };
        write!(f, "Expected one of [{}], found {}", expected, actual)
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::*;
    use crate::parser::*;

    #[test]
    fn expression() {
        let mut parser = Parser::new(Lexer::new(r"a|(bc)*"));
        assert_eq!(
            parser.expression().unwrap(),
            Node::Union(
                Box::new(Node::Character('a')),
                Box::new(Node::Star(Box::new(Node::Concat(
                    Box::new(Node::Character('b')),
                    Box::new(Node::Character('c'))
                ))))
            )
        );
    }

    #[test]
    fn expression2() {
        let mut parser = Parser::new(Lexer::new(r"a|"));
        assert_eq!(
            parser.expression().unwrap(),
            Node::Union(Box::new(Node::Character('a')), Box::new(Node::Empty))
        );
    }

    #[test]
    fn fail() {
        let mut parser1 = Parser::new(Lexer::new(r"a("));
        let mut parser2 = Parser::new(Lexer::new(r"a)"));
        assert!(parser1.expression().is_err());
        assert!(parser2.expression().is_err());
    }
}
