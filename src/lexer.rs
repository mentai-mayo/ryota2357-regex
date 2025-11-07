use std::str::Chars;

/// トークン
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Token {
    /// 文字
    Character(char),
    /// 和集合演算子 |
    UnionOp,
    /// 繰り返し演算子 *
    StarOp,
    /// 左括弧 (
    LeftParen,
    /// 右括弧 )
    RightParen,
    /// 文末
    End,
}

pub(crate) struct Lexer<'a> {
    src: Chars<'a>,
}

impl Lexer<'_> {
    /// create Lexer
    pub fn new(src: &str) -> Lexer {
        Lexer { src: src.chars() }
    }
    /// scan next character
    pub fn scan(&mut self) -> Token {
        match self.src.next() {
            Some('\\') => Token::Character(self.src.next().expect("EOF detected after '\\'.")),
            Some('|') => Token::UnionOp,
            Some('(') => Token::LeftParen,
            Some(')') => Token::RightParen,
            Some('*') => Token::StarOp,
            Some(c) => Token::Character(c),
            None => Token::End,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer;

    use super::*;

    #[test]
    fn scan() {
        let mut lexer = Lexer::new(r"a|(bc)*");
        assert_eq!(lexer.scan(), Token::Character('a'));
        assert_eq!(lexer.scan(), Token::UnionOp);
        assert_eq!(lexer.scan(), Token::LeftParen);
        assert_eq!(lexer.scan(), Token::Character('b'));
        assert_eq!(lexer.scan(), Token::Character('c'));
        assert_eq!(lexer.scan(), Token::RightParen);
        assert_eq!(lexer.scan(), Token::StarOp);
        assert_eq!(lexer.scan(), Token::End);
    }

    #[test]
    fn scan_with_esc() {
        let mut lexer = Lexer::new(r"a|\|\\(\)");
        assert_eq!(lexer.scan(), Token::Character('a'));
        assert_eq!(lexer.scan(), Token::UnionOp);
        assert_eq!(lexer.scan(), Token::Character('|'));
        assert_eq!(lexer.scan(), Token::Character('\\'));
        assert_eq!(lexer.scan(), Token::LeftParen);
        assert_eq!(lexer.scan(), Token::Character(')'));
        assert_eq!(lexer.scan(), Token::End);
    }

    #[test]
    fn with_empty() {
        let mut lexer = Lexer::new(r#""#);
        assert_eq!(lexer.scan(), Token::End);
    }
}
