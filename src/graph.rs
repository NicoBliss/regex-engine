pub mod graph {
    #[derive(Debug, PartialEq)]
    pub struct Graph<T> {
        pub arena: Vec<Option<Node<T>>>,
        pub start: NodeIndex,
        pub active: NodeIndex
    }

    pub type NodeIndex = usize;

    #[derive(Debug, PartialEq)]
    pub struct Node<T> {
        pub edges: Vec<(NodeIndex, Option<T>)>,
        endlinked: bool
    }

    impl<T> Node<T> {
        fn new(edges: Vec<(NodeIndex, Option<T>)>) -> Self {
            Node {
                edges,
                endlinked: false
            }
        }
    }

    impl<T> Graph<T> {
        // ensures "monotonicity" of node numbers
        fn add_node(&mut self, node: Node<T>) -> NodeIndex {
            self.arena.push(Some(node));
            let index = self.arena.len()-1;
            index
        }

        pub fn new() -> Self {
            let mut graph = Graph {
                arena: Vec::new(),
                start: 0,
                active: 0
            };
            let start = Node::new(vec!());
            assert_eq!(graph.add_node(start), 0);
            graph.set_active(0);
            graph
        }

        fn set_active(&mut self, new_active: NodeIndex) {
            self.arena[new_active].as_mut().unwrap().endlinked = true;
            self.active = new_active;
        }

        fn bump_endlinked(&mut self, endlinked: NodeIndex, new: NodeIndex, cost: Option<T>) {
            assert!(self.arena.len() >= endlinked && self.arena.len() >= new);
            assert!(self.arena[endlinked].as_ref().is_some());
            assert!(self.arena[endlinked].as_ref().unwrap().endlinked);

            let bumped_node = self.arena[endlinked].as_mut().unwrap();
            bumped_node.endlinked = false;
            bumped_node.edges.push((new, cost));
        }

        pub fn add_cost(&mut self, cost: T) {
            let new_active_node = Node::new(vec![]);
            let new_active_node_index = self.add_node(new_active_node);

            self.bump_endlinked(self.active, new_active_node_index, Some(cost));
            self.set_active(new_active_node_index);
        }

        pub fn add_junction(&mut self, start: NodeIndex) {
            // check that it's a valid starting node
            assert!(start < self.arena.len());
            assert!(self.arena[start].is_some());

            self.set_active(start);
        }

        pub fn close_junction(&mut self, start: NodeIndex) {
            // we will use a very strong property of the way we've made this structure. 
            // if start is actually a junction, then we know that everything it points to is _after_ it, and before "now"
            let mut dangling_nodes: Vec<NodeIndex> = Vec::new();
            for node_index in start..self.arena.len() {
                if self.arena[node_index].is_none() {
                    continue;
                }
                if self.arena[node_index].as_ref().unwrap().endlinked {
                    dangling_nodes.push(node_index);
                }
            }
            if dangling_nodes.len() == 0 {
                return
            }
            let new_active_node = Node::new(vec![]);
            let new_active_node_index = self.add_node(new_active_node);

            for node_index in dangling_nodes {
                self.bump_endlinked(node_index, new_active_node_index, None);
            }

            self.set_active(new_active_node_index);
        }

        pub fn zero_or_one(&mut self, start: NodeIndex) {
            self.add_junction(start);
            self.close_junction(start);
        }

        pub fn one_or_more(&mut self, start: NodeIndex) {
            self.arena[self.active].as_mut().unwrap().edges.push((start, None));
        }

        pub fn zero_or_more(&mut self, start: NodeIndex) {
            self.arena[self.active].as_mut().unwrap().edges.push((start, None));
            self.add_junction(start);
        }

        pub fn compile(self) -> Self {
            todo!()
        }
    }

    #[cfg(test)]
mod tests {
    use super::{Graph, Node};

    #[test]
    fn basic_addition() {
        let mut graph = Graph::new();
        graph.add_cost('a');
        graph.add_cost('b');
        let goal = Graph {
            arena: vec![
                Some(Node {
                    edges: vec![(1, Some('a'))],
                    endlinked: false
                }), Some(Node {
                    edges: vec![(2, Some('b'))],
                    endlinked: false
                }), Some(Node {
                    edges: vec![],
                    endlinked: true
                }) ],
            start: 0,
            active: 2
        };

        assert_eq!(graph, goal)
    }

    #[test]
    fn ripped_graph() {
        let mut graph = Graph::new();
        graph.add_cost('N');
        graph.add_cost('3');
        graph.add_junction(0);
        graph.add_cost('T');
        graph.add_cost('R');
        graph.add_cost('A');
        graph.add_junction(0);
        graph.add_cost('N');
        graph.add_cost('7');
        graph.close_junction(0);
        graph.one_or_more(0);

        let goal = Graph {
            arena: vec![
                Some(Node {
                    edges: vec![(1, Some('N')),(3, Some('T')),(6, Some('N'))],
                    endlinked: false
                }), Some(Node {
                    edges: vec![(2, Some('3'))],
                    endlinked: false
                }), Some(Node {
                    edges: vec![(8, None)],
                    endlinked: false
                }), Some(Node {
                    edges: vec![(4, Some('R'))],
                    endlinked: false
                }), Some(Node {
                    edges: vec![(5, Some('A'))],
                    endlinked: false
                }), Some(Node {
                    edges: vec![(8, None)],
                    endlinked: false
                }), Some(Node {
                    edges: vec![(7, Some('7'))],
                    endlinked: false
                }), Some(Node {
                    edges: vec![(8, None)],
                    endlinked: false
                }), Some(Node {
                    edges: vec![(0, None)],
                    endlinked: true
                })
                
            ],
            start: 0,
            active: 8
        };

        assert_eq!(goal, graph);
    }
}
}