#[derive(Debug, Eq, PartialEq)]
pub enum TokenKind<'a> {
    Text(&'a str),
    Newline(Newline),
    Heading { level: u8, text: &'a str },
    // Comment(&'a str),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Newline {
    // The character `\n` not immediately preceded by `\r`.
    Unix,
    // The character sequence `\r\n`.
    Windows,
    // The character `\r` not immediately followed by `\n`.
    Mac,
}

pub fn parse(source: &str) -> Vec<TokenKind<'_>> {
    use TokenKind as Kind;
    let mut iter = source.char_indices().peekable();
    let mut tokens = vec![];
    let mut text_start = 0;
    let mut just_parsed = true;
    while let Some((i, c)) = iter.next() {
        println!("-> {:?}", (i, c));
        if just_parsed {
            text_start = i;
        }
        just_parsed = false;
        let token_kind = match c {
            '\n' => Some(Kind::Newline(Newline::Unix)),
            '\r' => {
                let nl = if iter.next_if(|&(_, next)| next == '\n').is_some() {
                    Newline::Windows
                } else {
                    Newline::Mac
                };
                Some(Kind::Newline(nl))
            }
            '#' => {
                dbg!(tokens.last());
                dbg!(text_start);
                dbg!(i);
                match tokens.last() {
                    None | Some(Kind::Newline(_)) if text_start == i => {
                        let mut level: u8 = 0;
                        // Don't advance
                        let mut try_iter = source[i + 1..].chars().peekable();
                        while try_iter.next_if_eq(&'#').is_some() {
                            level += 1;
                        }
                        let level = level; // end mutability
                        // Ignore any hash characters if there is no space following it.
                        if try_iter.next() == Some(' ') {
                            let (start_index, skip_char) = iter
                                .nth(usize::from(level))
                                .expect("should not reach end of source");
                            debug_assert_eq!(skip_char, ' ');
                            let mut end_index = start_index;
                            while let Some((i, _)) = iter.next_if(|&(_, c)| c != '\n' && c != '\r')
                            {
                                end_index = i;
                            }
                            let text = &source[start_index + 1..=end_index];
                            text_start = end_index + 1;
                            Some(Kind::Heading { level, text })
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        };
        if let Some(kind) = token_kind {
            just_parsed = true;
            // debug_assert!(
            //     i >= text_start,
            //     "character at byte {}, somehow less than {}.",
            //     i,
            //     text_start
            // );
            if i > text_start {
                tokens.push(Kind::Text(&source[text_start..i]));
                text_start = i;
            }
            tokens.push(kind);
        }
    }
    let last_text = &source[text_start..];
    if !last_text.is_empty() {
        tokens.push(Kind::Text(last_text));
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use Newline::*;
    use TokenKind as Kind;
    #[test]
    fn uwu() {
        assert_eq!(
            parse("meow! # uwu\n#awa"),
            vec![
                Kind::Text("meow! # uwu"),
                Kind::Newline(Unix),
                Kind::Text("#awa")
            ]
        );
    }
    #[test]
    fn uwu2() {
        assert_eq!(
            parse("meow! #uwu\n#  awa  \rcool"),
            vec![
                Kind::Text("meow! #uwu"),
                Kind::Newline(Unix),
                Kind::Heading {
                    level: 0,
                    text: " awa  "
                },
                Kind::Newline(Mac),
                Kind::Text("cool")
            ]
        );
    }
}
