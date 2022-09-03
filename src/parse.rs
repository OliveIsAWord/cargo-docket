#[derive(Debug, Eq, PartialEq)]
pub enum TokenKind<'a> {
    Text(&'a str),
    Newline(Newline),
    Heading { level: usize, text: &'a str },
    Comment(MultilineText<'a>),
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

#[derive(Debug, Default, Eq, PartialEq)]
pub struct MultilineText<'a> {
    t: Vec<Result<&'a str, Newline>>,
}

impl<'a> MultilineText<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { t: vec![Ok(text)] }
    }
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
                        let mut level = 0;
                        // Don't advance
                        let mut try_iter = source[i + 1..].chars().peekable();
                        while try_iter.next_if_eq(&'#').is_some() {
                            level += 1;
                        }
                        let level = level; // cease mutability
                                           // Ignore any hash characters if there is no space following it.
                        if try_iter.next() == Some(' ') {
                            let (start_index, skip_char) = iter
                                .nth(level)
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
                            // TODO: should we unconditionally advance the main iter?
                            // if level > 0 {
                            //     let _ = iter
                            //         .nth(usize::from(level - 1))
                            //         .expect("should not reach end of source");
                            // }
                            None
                        }
                    }
                    _ => None,
                }
            }
            '<' => {
                let mut is_comment = true;
                for cc in "!--".chars() {
                    if iter.next_if(|&(_, c)| c == cc).is_none() {
                        is_comment = false;
                        break;
                    }
                }
                if is_comment {
                    let mut start = None;
                    let mut end = 0;
                    let mut exhausted = true;
                    for (i, c) in iter.by_ref() {
                        if start.is_none() {
                            start = Some(i);
                        }
                        end = i;
                        // TODO: This is awful, fix
                        if c == '>' {
                            exhausted = false;
                            break;
                        }
                    }
                    if exhausted {
                        None
                    } else {
                        Some(Kind::Comment(
                            start
                                .map(|s| MultilineText::new(&source[s..end - 2]))
                                .unwrap_or_default(),
                        ))
                    }
                } else {
                    None
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

    #[test]
    fn subheadings() {
        assert_eq!(
            parse("## hi"),
            vec![Kind::Heading {
                level: 1,
                text: "hi"
            },]
        );
    }

    #[test]
    fn comment() {
        assert_eq!(
            parse("text1<!--text2\ntext3-->text4"),
            vec![
                Kind::Text("text1"),
                Kind::Comment(MultilineText {
                    t: vec![Ok("text2\ntext3")]
                }),
                Kind::Text("text4")
            ]
        );
    }
}
