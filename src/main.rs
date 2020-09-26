use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let buffer = std::env::args().nth(2);
    let buffer = if let Some(buffer) = buffer {
        buffer
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    let raw_idx = std::env::args().nth(1).expect("no col specified");

    let lex_tokens = parse_idx(raw_idx);

    let table = create_table(buffer);

    //write_table(&table, lex_tokens);

    Ok(())
}

fn create_table(buffer: String) -> Vec<Vec<String>> {
    let mut table: Vec<Vec<String>> = Default::default();
    let mut tmp_row: Vec<String> = Default::default();
    let mut tmp_str: String = Default::default();

    let mut buffer = buffer.chars().peekable();
    loop {
        let c = buffer.next();
        match c {
            Some(' ') => {
                tmp_row.push(tmp_str.drain(..).collect());
                while buffer.peek() == Some(&' ') {
                    buffer.next();
                }
            }
            Some('\n') => {
                tmp_row.push(tmp_str.drain(..).collect());
                table.push(tmp_row.drain(..).collect());
            }
            Some(c) => {
                tmp_str.push(c);
            }
            None => break,
        }
    }

    //leftover
    if !tmp_str.is_empty() {
        tmp_row.push(tmp_str.drain(..).collect());
    }
    if !tmp_row.is_empty() {
        table.push(tmp_row.drain(..).collect());
    }
    table
}

//
// Parsing
//

#[derive(Debug, PartialEq)]
enum Token {
    Any,
    Num(isize),
    Range,
    Comma,
}

#[derive(Debug)]
enum LexToken {
    Num(isize),
    Range((Limit, Limit)),
    Any,
}

#[derive(Debug)]
enum Limit {
    Limited(isize),
    Unlimited,
}

// row;col
// 1;2
// 1;_
// 1,2,3;-4~6
// 1~2,8;_
fn parse_idx(raw_idx: String) -> (Vec<LexToken>, Vec<LexToken>) {
    let mut idx = raw_idx.split(';');
    let row = idx.next().unwrap();
    let col = idx.next().unwrap();

    fn parse(raw: &str) -> Vec<Token> {
        let mut tokens = vec![];
        let mut num = String::new();
        for c in raw.chars() {
            match c {
                c if c == '-' || c.is_numeric() => {
                    num.push(c);
                }
                ',' => {
                    assert!(!num.is_empty());
                    tokens.push(Token::Num(
                        num.drain(..).collect::<String>().parse().unwrap(),
                    ));
                    tokens.push(Token::Comma);
                }
                '~' => {
                    if !num.is_empty() {
                        tokens.push(Token::Num(
                            num.drain(..).collect::<String>().parse().unwrap(),
                        ));
                    }
                    tokens.push(Token::Range)
                }
                '_' => {
                    tokens.push(Token::Any);
                    break;
                }
                _ => unreachable!(),
            }
        }
        if !num.is_empty() {
            dbg!(&num);
            tokens.push(Token::Num(
                num.drain(..).collect::<String>().parse().unwrap(),
            ));
        }
        tokens
    }

    fn lex(tokens: Vec<Token>) -> Vec<LexToken> {
        let mut res = vec![];

        let mut tokens = tokens.into_iter().peekable();

        loop {
            let token = tokens.next();
            match token {
                Some(Token::Num(n)) => {
                    if tokens.peek() != Some(&Token::Range) {
                        res.push(LexToken::Num(n));
                    } else {
                        // Range
                        tokens.next();
                        // Comma or Num
                        match tokens.peek() {
                            Some(Token::Comma) | None => {
                                let _ = tokens.next();
                                res.push(LexToken::Range((Limit::Limited(n), Limit::Unlimited)));
                            }
                            Some(Token::Num(upper_limit)) => {
                                let upper_limit = *upper_limit;
                                let _ = tokens.next();
                                res.push(LexToken::Range((
                                    Limit::Limited(n),
                                    Limit::Limited(upper_limit),
                                )));
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                Some(Token::Comma) => {}
                Some(Token::Any) => {
                    res.clear();
                    res.push(LexToken::Any);
                    break;
                }
                Some(Token::Range) => {
                    // this handle this case:
                    // ~3
                    if let Some(Token::Num(upper_limit)) = tokens.next() {
                        res.push(LexToken::Range((
                            Limit::Unlimited,
                            Limit::Limited(upper_limit),
                        )))
                    } else {
                        unreachable!()
                    }
                }
                None => break,
            }
        }
        res
    }

    (lex(parse(row)), lex(parse(col)))
}
