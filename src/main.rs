use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let file_path = std::env::args().nth(2);
    let buffer = if let Some(file_path) = file_path {
        std::fs::read_to_string(file_path).expect("Error opening file")
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    let raw_idx = std::env::args().nth(1).expect("no idx specified");

    let lex_tokens = parse_idx(raw_idx);

    let table = create_table(buffer);

    let filtered_table = filter_table(&table, lex_tokens);

    write_table(filtered_table).expect("Error while writing table to stdout");

    Ok(())
}

fn write_table(table: Vec<Vec<String>>) -> Result<(), io::Error> {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    let col_len = table
        .get(0)
        .expect("Table should always have at least one element")
        .len();

    // find column unified width
    use std::collections::HashMap;
    let mut col_widths: HashMap<usize, usize> = HashMap::new();

    for row in 0..col_len {
        for (col, _) in table.iter().enumerate() {
            col_widths.insert(
                col,
                std::cmp::max(*col_widths.get(&col).unwrap_or(&0), table[col][row].len()),
            );
        }
    }

    // print while transposing
    for row in 0..col_len {
        for col in 0..table.len() {
            write!(stdout, "{:1$}", table[col][row], col_widths[&col])?;
            write!(stdout, " ")?;
        }
        writeln!(stdout)?;
    }
    Ok(())
}

fn filter_table(table: &[Vec<String>], tokens: (Vec<LexToken>, Vec<LexToken>)) -> Vec<Vec<String>> {
    fn adjust_idx(idx: isize, len: usize) -> usize {
        if idx == 0 {
            panic!("0 is an invalid idx");
        }
        if idx > 0 {
            (idx - 1) as usize
        } else {
            let adjust_idx = idx + len as isize;
            if adjust_idx < 0 {
                panic!("Idx is out of range, idx: {} limit: {}", idx, len)
            } else {
                adjust_idx as usize
            }
        }
    }

    // filter rows
    let mut filtered_rows: Vec<&Vec<String>> = vec![];
    if let LexToken::Any = tokens.0[0] {
        filtered_rows = table.iter().collect();
    } else {
        let row_number = table.len();
        for row in tokens.0.into_iter() {
            match row {
                LexToken::Num(n) => {
                    let n = adjust_idx(n, row_number);
                    filtered_rows.push(&table[n]);
                }
                LexToken::Range((lower_limit, upper_limit)) => match (lower_limit, upper_limit) {
                    (Limit::Limited(ll), Limit::Limited(ul)) => {
                        let ll = adjust_idx(ll, row_number);
                        let ul = adjust_idx(ul, row_number);
                        filtered_rows.extend(table[ll..ul + 1].iter());
                    }
                    (Limit::Unlimited, Limit::Limited(ul)) => {
                        let ul = adjust_idx(ul, row_number);
                        filtered_rows.extend(table[..ul + 1].iter());
                    }
                    (Limit::Limited(ll), Limit::Unlimited) => {
                        let ll = adjust_idx(ll, row_number);
                        filtered_rows.extend(table[ll..].iter());
                    }
                    // caught by parser
                    // ~
                    _ => unreachable!(),
                },
                // Any caught above
                _ => unreachable!(),
            }
        }
    }

    // filter cols
    fn transpose(rows: Vec<&Vec<String>>) -> Vec<Vec<String>> {
        let mut new_rows = vec![];
        let row_len = rows
            .get(0)
            .expect("Table should always have atleast one elemnt")
            .len();

        let mut tmp_row: Vec<String> = vec![];

        for col in 0..row_len {
            for row in 0..rows.len() {
                tmp_row.push(
                    rows.get(row)
                        .map(|r| r.get(col))
                        .unwrap_or_default()
                        .map(ToOwned::to_owned)
                        .unwrap_or_else(|| "_".to_string()),
                );
            }
            new_rows.push(tmp_row.drain(..).collect());
        }
        assert!(tmp_row.is_empty());
        new_rows
    }

    let filtered_rows = transpose(filtered_rows);
    let col_number = filtered_rows.len();

    let mut filtered = vec![];
    if let LexToken::Any = tokens.1[0] {
        filtered = filtered_rows.iter().collect();
    } else {
        for col in tokens.1.into_iter() {
            match col {
                LexToken::Num(n) => {
                    let n = adjust_idx(n, col_number);
                    filtered.push(&filtered_rows[n]);
                }
                LexToken::Range((lower_limit, upper_limit)) => match (lower_limit, upper_limit) {
                    (Limit::Limited(ll), Limit::Limited(ul)) => {
                        let ll = adjust_idx(ll, col_number);
                        let ul = adjust_idx(ul, col_number);
                        filtered.extend(filtered_rows[ll..ul + 1].iter());
                    }
                    (Limit::Unlimited, Limit::Limited(ul)) => {
                        let ul = adjust_idx(ul, col_number);
                        filtered.extend(filtered_rows[..ul + 1].iter());
                    }
                    (Limit::Limited(ll), Limit::Unlimited) => {
                        let ll = adjust_idx(ll, col_number);
                        filtered.extend(filtered_rows[ll..].iter());
                    }
                    // caught by parse
                    _ => unreachable!(),
                },
                // Any handeled above
                _ => unreachable!(),
            }
        }
    }

    filtered.into_iter().map(ToOwned::to_owned).collect()
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
    let row = idx.next().expect("no row specified");
    let col = idx.next().expect("no col specified");

    fn parse(raw: &str) -> Vec<Token> {
        let mut tokens = vec![];
        let mut num = String::new();
        for c in raw.chars() {
            match c {
                c if c == '-' || c.is_numeric() => {
                    num.push(c);
                }
                ',' => {
                    if !num.is_empty() {
                        tokens.push(Token::Num(
                            num.drain(..).collect::<String>().parse().unwrap(),
                        ));
                    }
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
                _ => panic!("Invalid idx format"),
            }
        }
        if !num.is_empty() {
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
                            _ => panic!("Invalid idx format"),
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
                        panic!("Invalid idx format")
                    }
                }
                None => break,
            }
        }
        res
    }

    (lex(parse(row)), lex(parse(col)))
}
