# [#0](https://ryota2357.com/blog/2023/dfa-regex-with-rust-0/)

## 参考

- [オートマトンは正規表現の夢を見るか(見るし、夢というかそのものですらある)](https://zenn.dev/canalun/articles/regexp_and_automaton)
- [正規表現エンジンを作ろう 1 ~ 6](https://codezine.jp/article/detail/3039)

# [#1](https://ryota2357.com/blog/2023/dfa-regex-with-rust-1/)

LexerとParserの実装

## Lexer

字句解析(src -> Token[])する.

```rust
enum Token {
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
```

## Parser

トークン列から構文木を生成する.

```bnf
<expression>     ::= <sub_expression> Token::End
<sub_expression> ::= <sequence> '|' <sub_expression> | <sequence>
<sequence>       ::= <sub_sequence> | ''
<sub_sequence>   ::= <star sub_sequence> | <star>
<star>           ::= <factor> '*' | <factor>
<factor>         ::= '(' <sub_expression> ')' | Token::Character
```

# [#2](https://ryota2357.com/blog/2023/dfa-regex-with-rust-2/)

構文木 -> NFA -> DFA と構築し, Regexを作成する.

## NFA (非決定性有限オートマトン)

## DFA (決定性有限オートマトン)
