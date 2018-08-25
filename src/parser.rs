use nom::types::CompleteStr;
use std::ops::IndexMut;
use common::*;
use nom;
use std::collections::HashMap;
use regex::Regex;

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
        rest: take_while1!(char::is_lowercase) >>
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
        tag_no_case!("nothing") => {|_| SymbolType::Integer(0) } |
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
        take_while1!(char::is_numeric) => {|n: Span| SymbolType::Integer(n.fragment.parse::<u32>().unwrap())} |
        variable => {|s| SymbolType::Variable(s) } |
        tag!(",") => {|_| SymbolType::Comma} |
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

named!(poetic_number_literal_core<Span, (Span, String, Vec<Span>)>,
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
        (position, pv, words)
    )
);

fn poetic_number_literal(input: Span) -> nom::IResult<Span, Vec<Token>> {
    let (rest, (position, target, words)) = poetic_number_literal_core(input)?;
    let literal = SymbolType::Words(words.iter().map(|s| s.to_string()).collect());
    return Ok((
        rest,
        vec![SymbolType::Variable(target), SymbolType::Is, literal]
            .into_iter()
            .map(|x| {
                Token {
                    position,
                    symbol: x,
                }
            })
            .collect(),
    ));
}

named!(pub line<Span, Vec<Token>>, alt_complete!(
    poetic_number_literal => {|s| s } |
    many1!(do_parse!(
        position: position!() >>
        word: word >>
        take_while!(is_space) >>
        (Token{position: position, symbol:word})
    )) => {|s| s }
));

named!(lines_core<Span, Vec<Vec<Token>>>, many0!(
    alt!(
        do_parse!(
            alt!(tag!("\n") | tag!("\r")) >>
            pos: position!() >>
            take_while!(is_space) >>
            (pos)
        ) => {|pos| vec![Token{position: pos, symbol: SymbolType::Newline}]} |
        do_parse!(
            a_line: line >>
            opt!(alt!(tag!("\n") | tag!("\r"))) >>
            take_while!(is_space) >>
            (a_line)
        ) => {|l| l }
    )));

fn lines(input: &str) -> nom::IResult<Span, Vec<Vec<Token>>> {
    let cs = CompleteStr(&input);
    let complete: Span = Span::new(cs);
    return match lines_core(complete) {
        Ok(ret) => Ok(ret),
        Err(_) => {
            unimplemented!();
        }
    };
}

fn compact_words(line: Vec<Token>) -> Vec<Token> {
    let mut symbols: Vec<Token> = Vec::new();
    let mut words = Vec::new();
    let pos = line[0].position;
    for word in line {
        match word.symbol {
            SymbolType::Words(other) => {
                words.extend_from_slice(&other);
            }
            _ => {
                if !words.is_empty() {
                    symbols.push(Token {
                        position: word.position,
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
            position: pos,
            symbol: SymbolType::Words(words),
        });
    }
    return symbols;
}

fn evaluate(value: &SymbolType) -> Result<Expression> {
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
            warn!("Evaluate: '{:?}'", value);
            unimplemented!();
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

fn single_symbol_to_expression(sym: &SymbolType) -> Result<Expression> {
    return match sym {
        &SymbolType::Words(_) => evaluate(sym),
        &SymbolType::Variable(ref name) => Ok(Expression::Variable(name.clone())),
        &SymbolType::String(ref phrase) => Ok(Expression::String(phrase.clone())),
        &SymbolType::Integer(ref val) => Ok(Expression::Integer(*val as i128)),
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
            unimplemented!("single symbol: {:?}", sym);
        }
    };
}

fn parse_expression(items: Vec<&SymbolType>, line: u32) -> Result<Expression> {
    // based off of https://en.wikipedia.org/wiki/Operator-precedence_parser#Pseudo-code
    let describe = format!("{:?}", items);
    debug!("Begin parse: {}", describe);
    let res = parse_expression_1(
        &items,
        0,
        single_symbol_to_expression(items[0])?,
        &LOWEST_PRECDENCE,
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
        let mut rhs = single_symbol_to_expression(items[index])?;
        lookahead = next_operator(items, index);
        while lookahead.is_some() && lookahead.unwrap().0 > op {
            let res = parse_expression_1(items, index, rhs, &items[lookahead.unwrap().1])?;
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
                unimplemented!("No operation for {:?}", op);
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
            loop_end: ref mut loop_end,
            expression: _,
        } => {
            loop_end.get_or_insert(loop_len);
        }
        Command::While {
            loop_end: ref mut loop_end,
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
    let raw_lines = lines(&fixed_input).unwrap();
    if raw_lines.0.fragment.len() > 0 {
        let pos = raw_lines.0;
        bail!(ErrorKind::UnparsedText(pos.fragment.to_string(), pos.line));
    }
    debug!("{:?}", raw_lines);
    let mut functions: HashMap<String, Function> = HashMap::new();
    let mut commands: Vec<CommandLine> = Vec::new();
    let mut loop_starts: Vec<usize> = Vec::new();
    let mut func_starts: Vec<usize> = Vec::new();
    let mut if_starts: Vec<usize> = Vec::new();
    for raw_symbols in raw_lines.1 {
        let mut symbols = compact_words(raw_symbols);
        if symbols[0].symbol == SymbolType::And {
            symbols.remove(0);
        }
        if symbols[symbols.len() - 1].symbol == SymbolType::Comma {
            symbols.pop();
        }
        debug!("{:?}", symbols);
        let current_line = symbols.first().unwrap().position.line;
        let symbols: Vec<SymbolType> = symbols.into_iter().map(|t| t.symbol).collect();
        match symbols.as_slice() {
            [SymbolType::Build, SymbolType::Variable(target), SymbolType::Up] => {
                commands.push(CommandLine{cmd:Command::Increment { target: target.to_string()}, line:current_line});
            }
            [SymbolType::Knock, SymbolType::Variable(target), SymbolType::Down] => {
                commands.push(CommandLine{cmd:Command::Decrement { target: target.to_string() }, line:current_line});
            }
            [SymbolType::Next] => {
                let command = build_next(&mut commands, &mut loop_starts);
                commands.push(CommandLine{cmd: command, line:current_line});
            }
            [SymbolType::Continue] => {
                let loop_start = loop_starts.last().expect("loop_starts");
                commands.push(CommandLine{cmd:Command::Next { loop_start: *loop_start }, line:current_line});
            }
            [SymbolType::Newline] |
            [SymbolType::Comment] => {
                // Comment on it's own is newline-equivalent
                if !if_starts.is_empty() {
                    let if_start = if_starts.pop().expect("if_starts");
                    let if_len = commands.len();
                    match commands.index_mut(if_start) {
                        CommandLine{cmd:Command::If {
                            expression: _,
                            if_end: ref mut if_end,
                        }, line:_} => {
                            if_end.get_or_insert(if_len - 1); // because there's not a real next to jump over
                        }
                        _ => {
                            panic!("return to non-if command");
                        }
                    }
                } else if !loop_starts.is_empty() {
                    let command = build_next(&mut commands, &mut loop_starts);
                    commands.push(CommandLine{cmd: command, line:current_line});
                } else if !func_starts.is_empty() {
                    let func_start = func_starts.pop().expect("func_starts");
                    let func_len = commands.len();
                    match commands.index_mut(func_start) {
                        CommandLine{cmd: Command::FunctionDeclaration {
                            name: _,
                            args: _,
                            func_end: ref mut func_end,
                        }, line: _} => {
                            func_end.get_or_insert(func_len);
                        }
                        _ => {
                            panic!("return to non-func command");
                        }
                    }
                    commands.push(CommandLine{cmd:Command::EndFunction { return_value: Expression::Nothing }, line:current_line});
                } else {
                    debug!("Double newline that doesn't end anything");
                }
            }
            [SymbolType::Taking { target, args }] => {
                commands.push(CommandLine{cmd:Command::Call {
                    name: target.to_string(),
                    args: args.iter()
                        .map(|a| Expression::Variable(a.to_string()))
                        .collect(),
                }, line:current_line});
            }
            _ => {
                // Better done with slice patterns once they stabilise
                // (see https://github.com/rust-lang/rust/issues/23121)
                if symbols[0] == SymbolType::Say && symbols.len() > 1 {
                    let expression_seq: Vec<&SymbolType> = symbols.iter().skip(1).collect();
                    let expression = parse_expression(expression_seq, current_line)?;
                    commands.push(CommandLine{cmd:Command::Say { value: expression }, line:current_line});
                } else if symbols.len() > 1 && symbols[1] == SymbolType::Is {
                    if let SymbolType::Variable(ref target) = symbols[0] {
                        let expression_seq: Vec<&SymbolType> = symbols.iter().skip(2).collect();
                        let expression = parse_expression(expression_seq, current_line)?;
                        commands.push(CommandLine{cmd:Command::Assignment {
                            target: target.to_string(),
                            value: expression,
                        }, line:current_line});
                    } else {
                        error!("Bad 'is' section: {:?}", symbols);
                    }
                } else if symbols[0] == SymbolType::Until && symbols.len() > 1 {
                    loop_starts.push(commands.len());
                    let expression_seq: Vec<&SymbolType> = symbols.iter().skip(1).collect();
                    let expression = parse_expression(expression_seq, current_line)?;
                    commands.push(CommandLine{cmd:Command::Until {
                        expression: expression,
                        loop_end: None,
                    }, line:current_line});
                } else if symbols[0] == SymbolType::While && symbols.len() > 1 {
                    loop_starts.push(commands.len());
                    let expression_seq: Vec<&SymbolType> = symbols.iter().skip(1).collect();
                    let expression = parse_expression(expression_seq, current_line)?;
                    commands.push(CommandLine{cmd:Command::While {
                        expression: expression,
                        loop_end: None,
                    }, line:current_line});
                } else if symbols[0] == SymbolType::If && symbols.len() > 1 {
                    if_starts.push(commands.len());
                    let expression_seq: Vec<&SymbolType> = symbols.iter().skip(1).collect();
                    let expression = parse_expression(expression_seq, current_line)?;
                    commands.push(CommandLine{cmd:Command::If {
                        expression: expression,
                        if_end: None,
                    }, line:current_line});
                } else if symbols.len() > 3 && symbols[0] == SymbolType::Put &&
                           symbols[symbols.len() - 2] == SymbolType::Where
                {
                    if let SymbolType::Variable(ref target) = symbols[symbols.len() - 1] {
                        let expression_seq: Vec<&SymbolType> = symbols.iter().skip(1).take(symbols.len() - 3).collect();
                        let expression = parse_expression(expression_seq, current_line)?;
                        commands.push(CommandLine{cmd:Command::Assignment {
                            target: target.to_string(),
                            value: expression,
                        }, line:current_line});
                    } else {
                        error!("Bad 'put' section: {:?}", symbols);
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
                                            error!("Bad 'function declaration' section: {:?} {:?}", sym, symbols);
                                            break;
                                        }
                                    }
                                    None => {
                                        break;
                                    }
                                }
                            } else {
                                error!("Bad 'function declaration' section: {:?}", symbols);
                                break;
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
                        commands.push(CommandLine{cmd:Command::FunctionDeclaration {
                            name: name.to_string(),
                            args,
                            func_end: None,
                        }, line: current_line});
                    } else {
                        error!("Bad 'function declaration' section: {:?}", symbols);
                    }
                } else if symbols[0] == SymbolType::Return && symbols.len() > 1 {
                    let expression_seq: Vec<&SymbolType> = symbols.iter().skip(1).collect();
                    let expression = parse_expression(expression_seq, current_line)?;
                    commands.push(CommandLine{cmd:Command::EndFunction { return_value: expression }, line:current_line});
                } else {
                    panic!("Don't recognise command sequence {:?}", symbols);
                }
            }
        }
    }
    return Ok(Program {
        commands,
        functions,
    });
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
                vec!["a".to_string(), "lovestruck".to_string(), "ladykiller".to_string()])).unwrap(),
            Expression::Integer(100)
        );
        assert_eq!(evaluate(&SymbolType::Words(vec!["nothing".to_string()])).unwrap(), Expression::Integer(0));
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
                SymbolType::Integer(0),
                SymbolType::And,
                SymbolType::Taking {
                    target: "Midnight".to_string(),
                    args: vec!["my world".to_string(), "Hate".to_string()] },
                SymbolType::Is, SymbolType::Integer(0)],
        );
    }

    #[test]
    fn comment_parsing() {
        lines_tokens_check("(foo bar baz)", vec![SymbolType::Comment]);
    }

    #[test]
    fn apostrophe_parsing() {
        let commands = vec![CommandLine{cmd: Command::Assignment{ target: "Bar".to_string(), value: Expression::Integer(4)}, line:1}];
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
}
