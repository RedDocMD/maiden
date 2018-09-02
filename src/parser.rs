use common::*;
use nom;
use nom::types::CompleteStr;
use regex::Regex;
use std::collections::HashMap;
use std::ops::IndexMut;

fn is_space(chr: char) -> bool {
    chr == ' ' || chr == '\t' || chr == '.'
}

fn is_newline(chr: char) -> bool {
    chr == '\r' || chr == '\n'
}

fn is_quote(chr: char) -> bool {
    chr == '\"' || chr == '\''
}

fn string_character(chr: char) -> bool {
    !is_newline(chr) && !is_quote(chr)
}

fn word_character(chr: char) -> bool {
    (chr >= 'A' && chr <= 'Z') || (chr >= 'a' && chr <= 'z')
}

named!(title_case<Span, String>,
    do_parse!(
        not!(keyword) >> // to shortcut the "Until Counter" case
        first: take_while_m_n!(1, 1, char::is_uppercase) >>
        rest: take_while1!(word_character) >>
        (format!("{}{}", first.fragment, rest.fragment))
    ));

named!(proper_variable<Span, String>,
    do_parse!(
        first: title_case >>
        rest: many0!(do_parse!(
            take_while1!(is_space) >>
            word: title_case >>
            (word)
        )) >>
        (format!("{}{}{}", first, if rest.is_empty() {""} else {" "}, rest.join(" ")))
    ));

named!(variable<Span, String>, alt_complete!(
    do_parse!(
        keyword: alt_complete!(
            tag_no_case!("a") |
            tag_no_case!("an") |
            tag_no_case!("the") |
            tag_no_case!("my") |
            tag_no_case!("your")
        ) >>
        take_while1!(is_space) >>
        word: take_while1!(char::is_lowercase) >>
        (format!("{} {}", keyword.fragment, word.fragment))
    ) => {|s| s } |
    proper_variable => {|s| s }
));

named!(keyword<Span, Span>, // single-words only
    alt_complete!(
        tag_no_case!("and") |
        tag_no_case!("build") |
        tag_no_case!("end") |
        tag_no_case!("else") |
        tag_no_case!("if") |
        tag_no_case!("into") |
        tag_no_case!("is") |
        tag_no_case!("minus") |
        tag_no_case!("put") |
        tag_no_case!("say") |
        tag_no_case!("scream") |
        tag_no_case!("shout") |
        tag_no_case!("takes") |
        tag_no_case!("until") |
        tag_no_case!("up") |
        tag_no_case!("was") |
        tag_no_case!("while") |
        tag_no_case!("whisper") |
        tag_no_case!("with") |
        tag_no_case!("without")
    )
);

named!(word(Span) -> SymbolType,
    alt_complete!(
        do_parse!(
            tag_no_case!("is") >>
            take_while1!(is_space) >>
            tag_no_case!("as") >>
            take_while1!(is_space) >>
            alt_complete!(
                tag_no_case!("high") | tag_no_case!("strong") | tag_no_case!("big")
            ) >>
            take_while1!(is_space) >>
            tag_no_case!("as") >>
            (())
        ) => {|_| SymbolType::GreaterThanOrEqual} |
        do_parse!(
            tag_no_case!("is") >>
            take_while1!(is_space) >>
            alt_complete!(
                tag_no_case!("less") | tag_no_case!("weaker") | tag_no_case!("lower") | tag_no_case!("smaller")
            ) >>
            take_while1!(is_space) >>
            tag_no_case!("than") >>
            (())
        ) => {|_| SymbolType::LessThan} |
        do_parse!(
            tag_no_case!("is") >>
            take_while1!(is_space) >>
            alt_complete!(
                tag_no_case!("higher") | tag_no_case!("stronger") | tag_no_case!("bigger") | tag_no_case!("greater")
            ) >>
            take_while1!(is_space) >>
            tag_no_case!("than") >>
            (())
        ) => {|_| SymbolType::GreaterThan} |
        alt_complete!(
            tag_no_case!("is") | tag_no_case!("was")
        ) => {|_| SymbolType::Is} |
        tag_no_case!("if") => {|_| SymbolType::If} |
        tag_no_case!("build") => {|_| SymbolType::Build} |
        tag_no_case!("up") => {|_| SymbolType::Up} |
        tag_no_case!("knock") => {|_| SymbolType::Knock} |
        tag_no_case!("down") => {|_| SymbolType::Down} |
        tag_no_case!("aint") => {|_| SymbolType::Aint} |
        alt_complete!(
            tag_no_case!("say") | tag_no_case!("shout") | tag_no_case!("whisper") | tag_no_case!("scream")
        ) => {|_| SymbolType::Say} |
        tag_no_case!("and") => {|_| SymbolType::And} |
        tag_no_case!("while") => {|_| SymbolType::While} |
        tag_no_case!("until") => {|_| SymbolType::Until} |
        alt_complete!(
            tag_no_case!("end") | tag_no_case!("around we go")
        ) => {|_| SymbolType::Next} |
        alt_complete!(
            tag_no_case!("take it to the top") | tag_no_case!("continue")
        ) => {|_| SymbolType::Continue} |
        tag_no_case!("give back") => {|_| SymbolType::Return} |
        tag_no_case!("takes") => {|_| SymbolType::Takes} |
        alt_complete!(
            tag_no_case!("without") | tag_no_case!("minus")
        ) => {|_| SymbolType::Subtract} |
        alt_complete!(
            tag_no_case!("with") | tag_no_case!("plus")
        ) => {|_| SymbolType::Add} |
        alt_complete!(
            tag_no_case!("times") | tag_no_case!("of")
        ) => {|_| SymbolType::Times } |
        tag_no_case!("into") => {|_| SymbolType::Where} |
        tag_no_case!("put") => {|_| SymbolType::Put} |
        tag_no_case!("else") => {|_| SymbolType::Else} |
        tag_no_case!("nothing") => {|_| SymbolType::Integer("0".to_string()) } |
        do_parse!(
            target: variable >>
            take_while1!(is_space) >>
            tag_no_case!("taking") >>
            take_while1!(is_space) >>
            first_arg: variable >>
            other_args: many0!(
                do_parse!(
                    take_while!(is_space) >>
                    alt!(tag!(",") | tag_no_case!("and")) >>
                    take_while!(is_space) >>
                    var: variable >>
                    (var)
                )) >>
            (target, first_arg, other_args)
        ) => {|(target, first_arg, mut other_args): (String, String, Vec<String>)| {
            other_args.insert(0, first_arg);
            SymbolType::Taking{target, args: other_args}
        }} |
        take_while1!(char::is_numeric) => {|n: Span| SymbolType::Integer(n.fragment.to_string())} |
        variable => {|s| SymbolType::Variable(s) } |
        do_parse!(
            tag!("\"") >>
            phrase: take_while!(string_character) >>
            tag!("\"") >>
            (phrase)
        ) => {|p: Span| SymbolType::String(p.to_string())} |
        do_parse!(
            tag!("(") >>
            take_until!(")") >>
            tag!(")") >>
            ()
        ) => {|_| SymbolType::Comment } |
        take_while1!(word_character) => {|word: Span| SymbolType::Words(vec![word.to_string()])}
    ));

named!(poetic_number_literal_core<Span, (u32, String, Vec<Span>)>,
    do_parse!(
        pv: variable >>
        take_while1!(is_space) >>
        tag!("is") >>
        position: position!() >>
        words: many1!(
            do_parse!(
                take_while1!(is_space) >>
                word: take_while1!(word_character) >>
                (word)
            )
        ) >>
        (position.line, pv, words)
    )
);

fn poetic_number_literal(input: Span) -> nom::IResult<Span, Vec<Token>> {
    let (rest, (line, target, words)) = poetic_number_literal_core(input)?;
    let literal = SymbolType::Words(words.iter().map(|s| s.to_string()).collect());
    return Ok((
        rest,
        vec![SymbolType::Variable(target), SymbolType::Is, literal]
            .into_iter()
            .map(|x| {
                Token {
                    line: line,
                    symbol: x,
                }
            })
            .collect(),
    ));
}

named!(pub line<Span, Vec<Token>>, alt_complete!(
    poetic_number_literal => {|s| s } |
    do_parse!(
        position: position!() >>
        first_word: word >>
        other_words: many0!(
            alt_complete!(
                tag!(",") => {|_| Token{line: position.line, symbol: SymbolType::Comma}} |
                do_parse!(
                    take_while1!(is_space) >>
                    word: word >>
                    (Token{line: position.line, symbol: word})
                ) => {|t| t}
        )) >>
        (Token{line: position.line, symbol: first_word}, other_words)
    ) => {|(first, mut other):(Token, Vec<Token>)| {
        other.insert(0, first);
        other
         }}
));

named!(blank_line<Span, Vec<Token>>,
    do_parse!(
        pos: position!() >>
        take_while!(is_space) >>
        alt!(tag!("\n") | tag!("\r")) >>
        take_while!(is_space) >>
        (vec![Token{line: pos.line, symbol: SymbolType::Newline}])
    )
);

named!(lines_core<Span, (Vec<Token>, Vec<Vec<Token>>)>,
    do_parse!(
        many0!(blank_line) >>
        first_line: line >>
        other_lines: many0!(
            alt_complete!(
                do_parse!(
                    take_while!(is_space) >>
                    alt!(tag!("\n") | tag!("\r")) >>
                    take_while!(is_space) >>
                    a_line: line >>
                    (a_line)
                ) => {|l| l } |
                blank_line => {|b| b }
            )
        ) >>
        (first_line, other_lines)
    )
);

fn lines(input: &str) -> nom::IResult<Span, Vec<Vec<Token>>> {
    let cs = CompleteStr(&input);
    let complete: Span = Span::new(cs);
    return match lines_core(complete) {
        Ok((rest, (first, mut others))) => {
            others.insert(0, first);
            Ok((rest, others))
        }
        Err(err) => Err(err),
    };
}

fn compact_words(line: Vec<Token>) -> Vec<Token> {
    let mut symbols: Vec<Token> = Vec::new();
    let mut words = Vec::new();
    let pos = line[0].line;
    for word in line {
        match word.symbol {
            SymbolType::Words(other) => {
                words.extend_from_slice(&other);
            }
            _ => {
                if !words.is_empty() {
                    symbols.push(Token {
                        line: word.line,
                        symbol: SymbolType::Words(words),
                    });
                    words = Vec::new();
                }
                symbols.push(word);
            }
        }
    }
    if !words.is_empty() {
        symbols.push(Token {
            line: pos,
            symbol: SymbolType::Words(words),
        });
    }
    return symbols;
}

fn evaluate(value: &SymbolType, line: u32) -> Result<Expression> {
    match value {
        SymbolType::Words(words) => {
            if words.len() == 1 {
                if words[0] == "nothing" {
                    return Ok(Expression::Integer(0));
                }
                let as_int = words[0].parse::<i128>();
                if let Ok(int) = as_int {
                    return Ok(Expression::Integer(int));
                }
            }
            let mut number = 0i128;
            for word in words {
                number *= 10;
                let len: i128 = (word.len() % 10) as i128;
                number += len;
            }
            return Ok(Expression::Integer(number));
        }
        SymbolType::String(phrase) => {
            return Ok(Expression::String(phrase.to_string()));
        }
        _ => {
            bail!(ErrorKind::Unimplemented(format!("Evaluate: '{:?}'", value), line));
        }
    }
}

fn next_operator<'a>(items: &Vec<&'a SymbolType>, mut index: usize) -> Option<(&'a SymbolType, usize)> {
    loop {
        let item_poss = items.get(index);
        if item_poss.is_none() {
            return None;
        }
        let item = item_poss.unwrap();
        match item {
            &SymbolType::Is |
            &SymbolType::Aint |
            &SymbolType::GreaterThanOrEqual |
            &SymbolType::GreaterThan |
            &SymbolType::LessThan |
            &SymbolType::Add |
            &SymbolType::Subtract |
            &SymbolType::Times |
            &SymbolType::And => {
                return Some((item, index));
            }
            _ => {}
        }
        index += 1;
    }
}

fn single_symbol_to_expression(sym: &SymbolType, line: u32) -> Result<Expression> {
    return match sym {
        &SymbolType::Words(_) => evaluate(sym, line),
        &SymbolType::Variable(ref name) => Ok(Expression::Variable(name.clone())),
        &SymbolType::String(ref phrase) => Ok(Expression::String(phrase.clone())),
        &SymbolType::Integer(ref val) => {
            return match val.parse::<i128>() {
                Ok(i) => Ok(Expression::Integer(i)),
                Err(_) => bail!(ErrorKind::ParseIntError(val.to_string(), line)),
            };
        }
        &SymbolType::Taking {
            ref target,
            ref args,
        } => {
            Ok(Expression::Call(
                target.to_string(),
                args.iter()
                    .map(|s| Expression::Variable(s.to_string()))
                    .collect(),
            ))
        }
        _ => {
            bail!(ErrorKind::Unimplemented(format!("Single symbol to expression: {:?}", sym), line));
        }
    };
}

fn parse_expression(items: Vec<&SymbolType>, line: u32) -> Result<Expression> {
    // based off of https://en.wikipedia.org/wiki/Operator-precedence_parser#Pseudo-code
    let describe = format!("{:?}", items);
    if items.len() == 0 {
        bail!(ErrorKind::UnbalancedExpression(describe, line));
    }
    debug!("Begin parse: {}", describe);
    let res = parse_expression_1(
        &items,
        0,
        single_symbol_to_expression(items[0], line)?,
        &LOWEST_PRECDENCE,
        line,
    )?;
    if res.1 != items.len() - 1 {
        bail!(ErrorKind::UnbalancedExpression(describe, line));
    }
    return Ok(res.0);
}

fn parse_expression_1(
    items: &Vec<&SymbolType>,
    mut index: usize,
    mut lhs: Expression,
    precedence: &SymbolType,
    line: u32,
) -> Result<(Expression, usize)> {
    debug!("index: {}, lhs: {:?} precedence: {:?}", index, lhs, precedence);
    let mut lookahead = next_operator(items, index);
    while lookahead.is_some() && lookahead.unwrap().0 >= precedence {
        debug!("lookahead: {:?}", lookahead.unwrap());
        let op = lookahead.unwrap().0;
        index = if lookahead.is_some() {
            lookahead.unwrap().1 + 1
        } else {
            index
        };
        if index >= items.len() {
            bail!(ErrorKind::UnbalancedExpression(format!("{:?}", items), line));
        }
        let mut rhs = single_symbol_to_expression(items[index], line)?;
        lookahead = next_operator(items, index);
        while lookahead.is_some() && lookahead.unwrap().0 > op {
            let l = lookahead.unwrap().1;
            if l >= items.len() {
                bail!(ErrorKind::UnbalancedExpression(format!("{:?}", items), line));
            }
            let res = parse_expression_1(items, index, rhs, &items[l], line)?;
            rhs = res.0;
            index = res.1;
            lookahead = next_operator(items, index);
        }
        lhs = match op {
            &SymbolType::Is => Expression::Is(Box::new(lhs.clone()), Box::new(rhs)),
            &SymbolType::Aint => Expression::Aint(Box::new(lhs.clone()), Box::new(rhs)),
            &SymbolType::GreaterThanOrEqual => Expression::GreaterThanOrEqual(Box::new(lhs.clone()), Box::new(rhs)),
            &SymbolType::GreaterThan => Expression::GreaterThan(Box::new(lhs.clone()), Box::new(rhs)),
            &SymbolType::LessThan => Expression::LessThan(Box::new(lhs.clone()), Box::new(rhs)),
            &SymbolType::Add => Expression::Add(Box::new(lhs.clone()), Box::new(rhs)),
            &SymbolType::Subtract => Expression::Subtract(Box::new(lhs.clone()), Box::new(rhs)),
            &SymbolType::Times => Expression::Times(Box::new(lhs.clone()), Box::new(rhs)),
            &SymbolType::And => Expression::And(Box::new(lhs.clone()), Box::new(rhs)),
            _ => {
                bail!(ErrorKind::Unimplemented(format!("No operation for {:?}", op), line));
            }
        }
    }
    return Ok((lhs.clone(), index));
}

fn build_next(commands: &mut Vec<CommandLine>, loop_starts: &mut Vec<usize>) -> Command {
    let loop_start = loop_starts.pop().expect("loop_starts");
    let loop_len = commands.len();
    match commands.index_mut(loop_start).cmd {
        Command::Until {
            ref mut loop_end,
            expression: _,
        } => {
            loop_end.get_or_insert(loop_len);
        }
        Command::While {
            ref mut loop_end,
            expression: _,
        } => {
            loop_end.get_or_insert(loop_len);
        }
        _ => {
            panic!("loop to non-loop command");
        }
    }
    return Command::Next { loop_start: loop_start };
}

pub fn parse(input: &str) -> Result<Program> {
    let re = Regex::new(r"'s\W+").unwrap();
    let fixed_input = re.replace_all(input, " is ").replace("'", "");
    let raw_lines = lines(&fixed_input)?;
    if !raw_lines.0.fragment.is_empty() && raw_lines.0.fragment.chars().any(|c| !c.is_whitespace()) {
        // ignore empty and all-whitespace blocks
        let pos = raw_lines.0;
        bail!(ErrorKind::UnparsedText(pos.fragment.to_string(), pos.line));
    }
    debug!("{:?}", raw_lines);
    let mut functions: HashMap<String, Function> = HashMap::new();
    let mut commands: Vec<CommandLine> = Vec::new();
    let mut loop_starts: Vec<usize> = Vec::new();
    let mut func_starts: Vec<usize> = Vec::new();
    let mut if_starts: Vec<usize> = Vec::new();
    let mut last_line = 0;
    for raw_symbols in raw_lines.1 {
        debug!("raw_symbols: {:?}", raw_symbols);
        let mut symbols = compact_words(raw_symbols);
        if symbols[0].symbol == SymbolType::And {
            symbols.remove(0);
        }
        if symbols[symbols.len() - 1].symbol == SymbolType::Comma {
            symbols.pop();
        }
        debug!("symbols: {:?}", symbols);
        if symbols.is_empty() {
            bail!(ErrorKind::NoSymbols(last_line+1));
        }
        let current_line = symbols.first().unwrap().line;
        last_line = current_line;
        let symbols: Vec<SymbolType> = symbols.into_iter().map(|t| t.symbol).collect();
        match symbols.as_slice() {
            [SymbolType::Build, SymbolType::Variable(target), SymbolType::Up] => {
                commands.push(CommandLine {
                    cmd: Command::Increment { target: target.to_string() },
                    line: current_line,
                });
            }
            [SymbolType::Knock, SymbolType::Variable(target), SymbolType::Down] => {
                commands.push(CommandLine {
                    cmd: Command::Decrement { target: target.to_string() },
                    line: current_line,
                });
            }
            [SymbolType::Next] => {
                let command = build_next(&mut commands, &mut loop_starts);
                commands.push(CommandLine {
                    cmd: command,
                    line: current_line,
                });
            }
            [SymbolType::Continue] => {
                let loop_start = loop_starts.last().expect("loop_starts");
                commands.push(CommandLine {
                    cmd: Command::Continue { loop_start: *loop_start },
                    line: current_line,
                });
            }
            [SymbolType::Newline] |
            [SymbolType::Comment] => {
                // Comment on it's own is newline-equivalent
                if !if_starts.is_empty() {
                    let if_start = if_starts.pop().expect("if_starts");
                    let if_len = commands.len();
                    match commands.index_mut(if_start) {
                        CommandLine {
                            cmd: Command::If {
                                expression: _,
                                ref mut if_end,
                            },
                            line: _,
                        } => {
                            if_end.get_or_insert(if_len);
                        }
                        _ => {
                            panic!("return to non-if command");
                        }
                    }
                    commands.push(CommandLine {
                        cmd: Command::EndIf,
                        line: if symbols[0] == SymbolType::Newline {
                            current_line + 1 // Newline line is the one before this
                        } else {
                            current_line
                        },
                    });
                } else if !loop_starts.is_empty() {
                    let command = build_next(&mut commands, &mut loop_starts);
                    commands.push(CommandLine {
                        cmd: command,
                        line: if symbols[0] == SymbolType::Newline {
                            current_line + 1 // Newline line is the one before this
                        } else {
                            current_line
                        },
                    });
                } else if !func_starts.is_empty() {
                    let func_start = func_starts.pop().expect("func_starts");
                    let func_len = commands.len();
                    match commands.index_mut(func_start) {
                        CommandLine {
                            cmd: Command::FunctionDeclaration {
                                name: _,
                                args: _,
                                ref mut func_end,
                            },
                            line: _,
                        } => {
                            func_end.get_or_insert(func_len);
                        }
                        _ => {
                            panic!("return to non-func command");
                        }
                    }
                    commands.push(CommandLine {
                        cmd: Command::EndFunction,
                        line: if symbols[0] == SymbolType::Newline {
                            current_line + 1 // Newline line is the one before this
                        } else {
                            current_line
                        },
                    });
                } else {
                    debug!("Double newline that doesn't end anything");
                }
            }
            [SymbolType::Taking { target, args }] => {
                commands.push(CommandLine {
                    cmd: Command::Call {
                        name: target.to_string(),
                        args: args.iter()
                            .map(|a| Expression::Variable(a.to_string()))
                            .collect(),
                    },
                    line: current_line,
                });
            }
            _ => {
                // Better done with slice patterns once they stabilise
                // (see https://github.com/rust-lang/rust/issues/23121)
                if symbols[0] == SymbolType::Say && symbols.len() > 1 {
                    let expression_seq: Vec<&SymbolType> = symbols.iter().skip(1).collect();
                    let expression = parse_expression(expression_seq, current_line)?;
                    commands.push(CommandLine {
                        cmd: Command::Say { value: expression },
                        line: current_line,
                    });
                } else if symbols.len() > 1 && symbols[1] == SymbolType::Is {
                    if let SymbolType::Variable(ref target) = symbols[0] {
                        let expression_seq: Vec<&SymbolType> = symbols.iter().skip(2).collect();
                        let expression = parse_expression(expression_seq, current_line)?;
                        commands.push(CommandLine {
                            cmd: Command::Assignment {
                                target: target.to_string(),
                                value: expression,
                            },
                            line: current_line,
                        });
                    } else {
                        bail!(ErrorKind::BadIs(symbols.to_vec(), current_line));
                    }
                } else if symbols[0] == SymbolType::Until && symbols.len() > 1 {
                    loop_starts.push(commands.len());
                    let expression_seq: Vec<&SymbolType> = symbols.iter().skip(1).collect();
                    let expression = parse_expression(expression_seq, current_line)?;
                    commands.push(CommandLine {
                        cmd: Command::Until {
                            expression: expression,
                            loop_end: None,
                        },
                        line: current_line,
                    });
                } else if symbols[0] == SymbolType::While && symbols.len() > 1 {
                    loop_starts.push(commands.len());
                    let expression_seq: Vec<&SymbolType> = symbols.iter().skip(1).collect();
                    let expression = parse_expression(expression_seq, current_line)?;
                    commands.push(CommandLine {
                        cmd: Command::While {
                            expression: expression,
                            loop_end: None,
                        },
                        line: current_line,
                    });
                } else if symbols[0] == SymbolType::If && symbols.len() > 1 {
                    if_starts.push(commands.len());
                    let expression_seq: Vec<&SymbolType> = symbols.iter().skip(1).collect();
                    let expression = parse_expression(expression_seq, current_line)?;
                    commands.push(CommandLine {
                        cmd: Command::If {
                            expression: expression,
                            if_end: None,
                        },
                        line: current_line,
                    });
                } else if symbols.len() > 3 && symbols[0] == SymbolType::Put &&
                           symbols[symbols.len() - 2] == SymbolType::Where
                {
                    if let SymbolType::Variable(ref target) = symbols[symbols.len() - 1] {
                        let expression_seq: Vec<&SymbolType> = symbols.iter().skip(1).take(symbols.len() - 3).collect();
                        let expression = parse_expression(expression_seq, current_line)?;
                        commands.push(CommandLine {
                            cmd: Command::Assignment {
                                target: target.to_string(),
                                value: expression,
                            },
                            line: current_line,
                        });
                    } else {
                        bail!(ErrorKind::BadPut(symbols.to_vec(), current_line));
                    }
                } else if symbols.len() > 2 && symbols[1] == SymbolType::Takes {
                    if let SymbolType::Variable(ref name) = symbols[0] {
                        let mut var_seq = symbols.iter().skip(2);
                        let mut args = vec![];
                        loop {
                            if let Some(SymbolType::Variable(ref arg)) = var_seq.next() {
                                args.push(arg.to_string());
                                match var_seq.next() {
                                    Some(sym) => {
                                        if sym != &SymbolType::And {
                                            bail!(ErrorKind::BadFunctionDeclaration(symbols.to_vec(), current_line));
                                        }
                                    }
                                    None => {
                                        break;
                                    }
                                }
                            } else {
                                bail!(ErrorKind::BadFunctionDeclaration(symbols.to_vec(), current_line));
                            }
                        }
                        func_starts.push(commands.len());
                        functions.insert(
                            name.to_string(),
                            Function {
                                location: commands.len(),
                                args: args.clone(),
                            },
                        );
                        commands.push(CommandLine {
                            cmd: Command::FunctionDeclaration {
                                name: name.to_string(),
                                args,
                                func_end: None,
                            },
                            line: current_line,
                        });
                    } else {
                        bail!(ErrorKind::BadFunctionDeclaration(symbols.to_vec(), current_line));
                    }
                } else if symbols[0] == SymbolType::Return && symbols.len() > 1 {
                    let expression_seq: Vec<&SymbolType> = symbols.iter().skip(1).collect();
                    let expression = parse_expression(expression_seq, current_line)?;
                    commands.push(CommandLine {
                        cmd: Command::Return { return_value: expression },
                        line: current_line,
                    });
                } else {
                    bail!(ErrorKind::BadCommandSequence(symbols.to_vec(), current_line));
                }
            }
        }
    }
    return Ok(Program {
        commands,
        functions,
    });
}

#[cfg(any(target_arch = "wasm32", test))]
fn print_command(command: &Command) -> String {
    format!("{:?}", command)
}

#[cfg(any(target_arch = "wasm32", test))]
pub fn print_program(program: &Program) -> String {
    let mut res = String::new();
    let mut indent = 0;
    let mut last_line = 0;
    for command in &program.commands {
        match command.cmd {
            Command::EndFunction |
            Command::EndIf |
            Command::Next { loop_start: _ } => {
                indent -= 1;
            }
            _ => {}
        }
        while last_line < command.line - 1 {
            last_line += 1;
            res += &format!("{}:\n", last_line);
        }
        last_line = command.line;
        res += &format!("{}: ", command.line);
        for _ in 0..indent {
            res += "  ";
        }
        res += &(print_command(&command.cmd) + "\n");
        match command.cmd {
            Command::FunctionDeclaration {
                name: _,
                args: _,
                func_end: _,
            } |
            Command::If {
                expression: _,
                if_end: _,
            } |
            Command::While {
                expression: _,
                loop_end: _,
            } |
            Command::Until {
                expression: _,
                loop_end: _,
            } => {
                indent += 1;
            }
            _ => {}
        }
    }
    return res;
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_env_logger;

    #[test]
    fn multi_word_quote_parse() {
        let (span, tokens) = line(Span::new(CompleteStr("say \"shout let it all out\""))).unwrap();
        assert_eq!(CompleteStr(""), span.fragment);
        assert_eq!(vec![SymbolType::Say, SymbolType::String("shout let it all out".to_string())],
            tokens.into_iter().map(|t| t.symbol).collect::<Vec<_>>());
    }

    #[test]
    fn check_evaluate() {
        pretty_env_logger::try_init().unwrap_or(());
        assert_eq!(
            evaluate(&SymbolType::Words(
                vec!["a".to_string(), "lovestruck".to_string(), "ladykiller".to_string()]), 0).unwrap(),
            Expression::Integer(100)
        );
        assert_eq!(evaluate(&SymbolType::Words(vec!["nothing".to_string()]), 0).unwrap(), Expression::Integer(0));
    }

    #[test]
    fn check_full_expression_parse() {
        pretty_env_logger::try_init().unwrap_or(());
        let expression = Expression::And(
            Box::new(Expression::Is(
                Box::new(Expression::Call(
                    "Midnight".to_string(),
                    vec![Expression::Variable("my world".to_string()), Expression::Variable("Fire".to_string())],
                )),
                Box::new(Expression::Integer(0)),
            )),
            Box::new(Expression::Is(
                Box::new(Expression::Call(
                    "Midnight".to_string(),
                    vec![Expression::Variable("my world".to_string()), Expression::Variable("Hate".to_string())],
                )),
                Box::new(Expression::Integer(0)),
            )),
        );
        let commands = vec![CommandLine{cmd: Command::If{expression: expression, if_end: None}, line: 1}];
        let functions = HashMap::new();
        assert_eq!(
            parse("If Midnight taking my world, Fire is nothing and Midnight taking my world, Hate is nothing")
                .unwrap(),
            Program{commands, functions}
        );
    }

    fn lines_tokens_check(input: &str, tokens: Vec<SymbolType>) {
        pretty_env_logger::try_init().unwrap_or(());
        let mut raw_lines = lines(input).unwrap();
        assert_eq!(raw_lines.0.fragment, CompleteStr(""));
        assert_eq!(raw_lines.1.len(), 1, "{:?}", raw_lines.1);
        assert_eq!(raw_lines.1.remove(0).into_iter().map(|t| t.symbol).collect::<Vec<_>>(), tokens);
    }

    #[test]
    fn check_expression_parse() {
        lines_tokens_check(
            "If Midnight taking my world, Fire is nothing and Midnight taking my world, Hate is nothing",
            vec![
                SymbolType::If,
                SymbolType::Taking {
                    target: "Midnight".to_string(),
                    args: vec!["my world".to_string(), "Fire".to_string()] },
                SymbolType::Is,
                SymbolType::Integer("0".to_string()),
                SymbolType::And,
                SymbolType::Taking {
                    target: "Midnight".to_string(),
                    args: vec!["my world".to_string(), "Hate".to_string()] },
                SymbolType::Is, SymbolType::Integer("0".to_string())],
        );
    }

    #[test]
    fn comment_parsing() {
        lines_tokens_check("(foo bar baz)", vec![SymbolType::Comment]);
    }

    #[test]
    fn apostrophe_parsing() {
        let commands = vec![CommandLine{
            cmd: Command::Assignment{
                target: "Bar".to_string(),
                value: Expression::Integer(4)
            },
            line: 1}];
        let functions = HashMap::new();
        assert_eq!(
            parse("Bar is foo'd")
                .unwrap(),
            Program{commands, functions}
        );
    }

    #[test]
    fn multi_word_proper_variable() {
        lines_tokens_check(
            "Liftin High takes the spirit and greatness",
            vec![SymbolType::Variable("Liftin High".to_string()),
                SymbolType::Takes, SymbolType::Variable("the spirit".to_string()),
                SymbolType::And, SymbolType::Words(vec!["greatness".to_string()])],
        );
    }

    #[test]
    fn not_proper_variable() {
        lines_tokens_check(
            "Until Counter is Limit",
            vec![
                SymbolType::Until, SymbolType::Variable("Counter".to_string()),
                SymbolType::Is, SymbolType::Variable("Limit".to_string())],
        );
    }

    #[test]
    fn split_nothing() {
        pretty_env_logger::try_init().unwrap_or(());
        let mut raw_lines = lines("If a thought is greater than nothinggggggggg").unwrap();
        assert_eq!(raw_lines.0.fragment, CompleteStr("gggggggg"));
        assert_eq!(raw_lines.1.len(), 1, "{:?}", raw_lines.1);
        assert_eq!(raw_lines.1.remove(0).into_iter().map(|t| t.symbol).collect::<Vec<_>>(),
            vec![
                SymbolType::If, SymbolType::Variable("a thought".to_string()),
                SymbolType::GreaterThan, SymbolType::Integer("0".to_string())
            ]);
    }

    #[test]
    fn great_davy() {
        pretty_env_logger::try_init().unwrap_or(());
        let expression = Expression::Aint(
            Box::new(Expression::Variable("Davy".to_string())),
            Box::new(Expression::Variable("Greatness".to_string())),
        );
        let commands = vec![CommandLine{cmd: Command::While{ expression, loop_end: None}, line:1}];
        let functions = HashMap::new();
        assert_eq!(
            parse("While Davy ain't Greatness")
                .unwrap(),
            Program{commands, functions}
        );
    }

    #[test]
    fn pretty_print() {
        assert_eq!(
            print_program(&parse("Absolute takes a thought")
                .unwrap()),
            "1: FunctionDeclaration { name: \"Absolute\", args: [\"a thought\"], func_end: None }\n"
        )
    }

    #[test]
    fn bad_fragment() {
        pretty_env_logger::try_init().unwrap_or(());
        let err = parse("test is a").err().unwrap().0;
        if let ErrorKind::BadIs(symbols, line) = err {
            assert_eq!(symbols, vec![
                SymbolType::Words(vec!["test".to_string()]),
                SymbolType::Is,
                SymbolType::Words(vec!["a".to_string()])]);
            assert_eq!(line, 1);
        } else {
            assert!(false, err);
        }
    }

    #[test]
    fn too_long_int() {
        pretty_env_logger::try_init().unwrap_or(());
        let err = parse("the loneliest is 340282366920938463463374607431768211455")
            .err()
            .unwrap()
            .0;
        if let ErrorKind::ParseIntError(val, line) = err {
            assert_eq!(val, "340282366920938463463374607431768211455");
            assert_eq!(line, 1);
        } else {
            assert!(false, err);
        }
    }

    #[test]
    fn bad_expression() {
        pretty_env_logger::try_init().unwrap_or(());
        let err = parse("if t is").err().unwrap().0;
        if let ErrorKind::UnbalancedExpression(name, line) = err {
            assert_eq!(name, "[Words([\"t\"]), Is]");
            assert_eq!(line, 1);
        } else {
            assert!(false, err);
        }
    }

    #[test]
    fn bad_put() {
        pretty_env_logger::try_init().unwrap_or(());
        let err = parse("put foo into bar").err().unwrap().0;
        if let ErrorKind::BadPut(expression, line) = err {
            assert_eq!(expression, vec![
                SymbolType::Put,
                SymbolType::Words(vec!["foo".to_string()]),
                SymbolType::Where,
                SymbolType::Words(vec!["bar".to_string()])]);
            assert_eq!(line, 1);
        } else {
            assert!(false, err);
        }
    }
}
