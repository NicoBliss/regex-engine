pub mod parser {
    use std::{fmt::Error, ops::Range};

    use crate::graph::graph::{Graph, NodeIndex};
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CharClass {
        chars: Vec<char>,
        ranges: Vec<Range<char>>
    }

    #[derive(PartialEq, Eq, Debug)]
    pub enum CharCost {
        Singleton(char),
        Dot,
        Class(CharClass)
    }

    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    enum Lexeme {
        Literal(char),
        OpenParen, CloseParen,
        OpenBracket, CloseBracket,
        Star, Question, Plus, Dot, Bar,
        Builtin(char), Range(char, char)
    }

    impl Lexeme {
        fn match_char(character: char) -> Self {
            match character {
                '(' => Lexeme::OpenParen,
                ')' => Lexeme::CloseParen,
                '[' => Lexeme::OpenBracket,
                ']' => Lexeme::CloseBracket,
                '*' => Lexeme::Star,
                '?' => Lexeme::Question,
                '+' => Lexeme::Plus,
                '.' => Lexeme::Dot,
                '|' => Lexeme::Bar,
                a => Lexeme::Literal(a)
            }
        }

        fn lexeme_to_char(self) -> char {
            match self {
                Lexeme::Bar => '|',
                Lexeme::Builtin(a) => a,
                Lexeme::CloseBracket => ']',
                Lexeme::CloseParen => ')',
                Lexeme::Dot => '.',
                Lexeme::Literal(a) => a,
                Lexeme::OpenBracket => '[',
                Lexeme::OpenParen => '(',
                Lexeme::Plus => '+',
                Lexeme::Question => '?',
                Lexeme::Star => '*',
                Lexeme::Range(_, _) => '-'
            }
        }

        fn set(&mut self, new_lex: Lexeme) {
            *self = new_lex;
        }
    }

    impl CharCost {
        fn fromchar(singleton: char) -> Self {
            CharCost::Singleton(singleton)
        }
    }

    impl CharClass {
        fn is_in(&self, letter: char) -> bool {
            self.chars.contains(&letter) || self.ranges.iter().fold(false, |a,x| a | x.contains(&letter) )
        }

        fn new() -> Self {
            CharClass {
                chars: vec![],
                ranges: vec![]
            }
        }

        fn plus_literal(&mut self, new_char: char) {
            self.chars.push(new_char);
        }

        fn plus_range(&mut self, start_char: char, end_char: char) {
            self.ranges.push(Range {start: start_char, end: end_char})
        }
    }

    // TODO: implement real errors
    fn lexer(regex: String) -> Result<Vec<Lexeme>,Error> {
        let mut lex_string = Vec::new();
        let mut chars = regex.chars();
        let mut in_class = false;

        while let Some(character) = chars.next() {
            // some extra logic required to escape the reserved characters
            if in_class && character != ']' && character != '-' {
                lex_string.push(Lexeme::Literal(character));
                continue;
            } else if in_class && character == '-' {
                if let Some(last) = lex_string.pop() {
                    match last {
                        Lexeme::OpenBracket => return Err(Error),
                        Lexeme::Range(_, _) => return Err(Error),
                        _ => {}
                    }
                    if let Some(next) = chars.next() {
                        if next == ']' {
                            return Err(Error)
                        }
                        lex_string.push(Lexeme::Range(last.lexeme_to_char(), next));
                        continue;
                    } else {
                        return Err(Error)
                    }
                } else {
                    return Err(Error) // Should *never* happen!
                }
            } else if in_class && character == ']' {
                in_class = false;            
            } else if character == '[' {
                in_class = true;
            } else if character == '\\' {
                if let Some(next) = chars.next() {
                    match Lexeme::match_char(next) {
                        Lexeme::Literal(_) => {
                            lex_string.push(Lexeme::Builtin(next));
                        },
                        _ => lex_string.push(Lexeme::Literal(next))

                    }
                }
                continue;
            }
            lex_string.push(Lexeme::match_char(character));
        }

        Ok(lex_string)
    }

    enum ParserState {
        OutOfClassWithoutQual,
        InClass(NodeIndex, CharClass),
        QualWithoutClass(NodeIndex)
    }

    impl ParserState {
        fn add_cost(&mut self, new_char: char) {
            match self {
                ParserState::InClass(_, a) => {a.plus_literal(new_char);}
                _ => {}
            }
        }

        fn add_cost_range(&mut self, start_char: char, end_char: char) {
            match self {
                ParserState::InClass(_, a) => {a.plus_range(start_char, end_char);}
                _ => {}
            }
        }
    }

    pub fn parser(regex: String) -> Result<Graph<CharCost>, Error> {
        let mut group_starts: Vec<NodeIndex> = vec![];
        let mut state = ParserState::OutOfClassWithoutQual;
        let mut graph = Graph::new();
        let lex_string;
        if let Ok(lexs) = lexer(regex) {
            lex_string = lexs;
        } else {
            return Err(Error);
        }

        for lexeme in lex_string {
            match (lexeme, &mut state) {
                (Lexeme::Bar, ParserState::OutOfClassWithoutQual) | (Lexeme::Bar, ParserState::QualWithoutClass(_)) => {
                    graph.add_junction(*group_starts.last().unwrap_or(&0));
                }
                (Lexeme::OpenParen, ParserState::OutOfClassWithoutQual) | (Lexeme::OpenParen, ParserState::QualWithoutClass(_)) => {
                    group_starts.push(graph.active);
                }
                (Lexeme::OpenBracket, ParserState::OutOfClassWithoutQual) | (Lexeme::OpenBracket, ParserState::QualWithoutClass(_)) => {
                    state = ParserState::InClass(graph.active, CharClass::new());
                }
                (Lexeme::CloseParen, ParserState::OutOfClassWithoutQual) | (Lexeme::CloseParen, ParserState::QualWithoutClass(_)) => {
                    if let Some(start) = group_starts.pop() {
                        graph.close_junction(start);
                        state = ParserState::QualWithoutClass(start)
                    } else {
                        return Err(Error)
                    }
                }
                (Lexeme::Literal(character), ParserState::OutOfClassWithoutQual) | (Lexeme::Literal(character), ParserState::QualWithoutClass(_)) => {
                    graph.add_cost(CharCost::fromchar(character));
                    state = ParserState::QualWithoutClass(graph.active);
                }
                (Lexeme::Dot, ParserState::OutOfClassWithoutQual) | (Lexeme::Dot, ParserState::QualWithoutClass(_)) => {
                    graph.add_cost(CharCost::Dot);
                    state = ParserState::QualWithoutClass(graph.active);
                }
                (Lexeme::Builtin(char), ParserState::QualWithoutClass(_)) | (Lexeme::Builtin(char), ParserState::OutOfClassWithoutQual) => {
                    // TODO!!!! Do builtins
                }
                (_, ParserState::OutOfClassWithoutQual) => {
                    return Err(Error);
                }
                (Lexeme::CloseBracket, ParserState::InClass(_, class)) => {
                    graph.add_cost(CharCost::Class(class.clone()));
                    state = ParserState::OutOfClassWithoutQual;
                }
                (Lexeme::Literal(new_char), ParserState::InClass(_, _)) => {
                    state.add_cost(new_char);
                }
                (Lexeme::Range(start_char, end_char), ParserState::InClass(_, _)) => {
                    state.add_cost_range(start_char,end_char);
                    state.add_cost(end_char);
                }
                (_, ParserState::InClass(_, _)) => {
                    return Err(Error)
                }
                (Lexeme::Plus, ParserState::QualWithoutClass(start)) => {
                    graph.one_or_more(*start);
                    state = ParserState::OutOfClassWithoutQual;
                }
                (Lexeme::Question, ParserState::QualWithoutClass(start)) => {
                    graph.zero_or_one(*start);
                    state = ParserState::OutOfClassWithoutQual;
                }
                (Lexeme::Star, ParserState::QualWithoutClass(start)) => {
                    graph.zero_or_more(*start);
                    state = ParserState::OutOfClassWithoutQual;
                }
                (_, ParserState::QualWithoutClass(_)) => {
                    return Err(Error)
                }
            }
        }
        
        Ok(graph)
    }


    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_charclass_is_in() {
            let char_range = Range {
                start: 'a',
                end: 'z'
            };
            let class = CharClass {
                chars: vec!['z'],
                ranges: vec![char_range]
            };
            for letter in 'a'..='z' {
                assert!(class.is_in(letter));
            }
        }

        #[test]
        fn test_lexer() {
            let string = "(ac\\||[ab])?".to_string();
            let lex_string = lexer(string).ok().unwrap();
            let goal = vec![
                Lexeme::OpenParen,
                Lexeme::Literal('a'),
                Lexeme::Literal('c'),
                Lexeme::Literal('|'),
                Lexeme::Bar,
                Lexeme::OpenBracket,
                Lexeme::Literal('a'),
                Lexeme::Literal('b'),
                Lexeme::CloseBracket,
                Lexeme::CloseParen,
                Lexeme::Question
            ];
            assert_eq!(goal, lex_string);
        }

        #[test]
        fn test_lexer_class() {
            let string = "(()[a?b[])".to_string();
            let lex_string = lexer(string).ok().unwrap();
            let goal = vec![
                Lexeme::OpenParen,
                Lexeme::OpenParen,
                Lexeme::CloseParen,
                Lexeme::OpenBracket,
                Lexeme::Literal('a'),
                Lexeme::Literal('?'),
                Lexeme::Literal('b'),
                Lexeme::Literal('['),
                Lexeme::CloseBracket,
                Lexeme::CloseParen
            ];
            assert_eq!(goal, lex_string);
        }

        #[test]
        fn test_lexer_in_class_ranges() {
            let string = "[][a-zssA-)]".to_string();
            let lex_string = lexer(string).ok().unwrap();
            let goal = vec![
                Lexeme::OpenBracket,
                Lexeme::CloseBracket,
                Lexeme::OpenBracket,
                Lexeme::Range('a', 'z'),
                Lexeme::Literal('s'),
                Lexeme::Literal('s'),
                Lexeme::Range('A', ')'),
                Lexeme::CloseBracket
            ];
            assert_eq!(goal, lex_string)
        }

        #[test]
        fn test_parser() {
            let regex = "([abcd]|a|b|c|d)+".to_string();
            let graph = parser(regex).ok().unwrap();
            let mut goal = Graph::new();
            goal.add_cost(CharCost::Class(CharClass {chars: vec!['a','b','c','d'], ranges: vec![]}));
            goal.add_junction(0);
            goal.add_cost(CharCost::Singleton('a'));
            goal.add_junction(0);
            goal.add_cost(CharCost::Singleton('b'));
            goal.add_junction(0);
            goal.add_cost(CharCost::Singleton('c'));
            goal.add_junction(0);
            goal.add_cost(CharCost::Singleton('d'));
            goal.close_junction(0);
            goal.one_or_more(0);

            assert_eq!(goal, graph);
        }
    }

}