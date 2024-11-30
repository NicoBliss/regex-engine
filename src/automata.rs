mod automata {
    use crate::graph::graph::{Graph, NodeIndex};
    use crate::parser::parser::{CharClass, CharCost};

    struct MatchData {
        matched_string: String,
        location: usize
    }

    fn run_automata(automata: Graph<CharCost>, code: String) -> Vec<MatchData> {
        todo!()
    } 
}