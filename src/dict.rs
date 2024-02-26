#![allow(dead_code)]

type EdgeIndex = usize;
type NodeIndex = usize;

// custom data structure to hold dictionary and support operations required for scrabble
//
// In short we construct a dawg-like tree where the first 26 nodes in the arena make up an implicit root
// We can do this because we know the scrabble dictionary will include words for each initial letter
#[derive(Debug)]
pub struct Dict {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(Debug, Default)]
struct Node {
    letter: char,
    first_child: Option<EdgeIndex>,
    first_parent: Option<EdgeIndex>,
    is_terminal: bool,
}

impl Node {
    pub fn new(letter: char) -> Self {
        Self {
            letter,
            ..Default::default()
        }
    }
}

#[derive(Debug)]
struct Edge {
    target: NodeIndex,
    next_outgoing_edge: Option<EdgeIndex>,
}

impl Dict {
    fn new() -> Self {
        Self {
            nodes: Vec::from_iter(('A'..='Z').map(|letter| Node::new(letter))),
            edges: Vec::new(),
        }
    }

    fn insert(&mut self, word: &str) {
        if word.is_empty() {
            return;
        }


        let letters: Vec<char> = word.to_ascii_uppercase().chars().collect();
        let mut idx: NodeIndex = Self::root_index_of(&letters[0]);
        for l_idx in 1..letters.len() {
            let mut found_child = false;
            for target in self.children(idx) {
                if self.nodes[target].letter == letters[l_idx] {
                    idx = target;
                    found_child = true;
                    break;
                }
            }
            if !found_child {
                let child_idx = self.add_node(letters[l_idx]);
                self.add_edge(child_idx, idx);
                idx = child_idx;
            }
        }
        self.nodes[idx].is_terminal = true;

    }

    fn cut_off_matching_prefix(to_cut: &str, other: &str) -> String {
        if to_cut.len() < other.len() {
            return to_cut.into();
        }
        let to_cut: Vec<char> = to_cut.chars().collect();
        let other: Vec<char> = other.chars().collect();
        let mut i = 0;
        while i < other.len() && to_cut[i] == other[i] {
            i += 1;
        }

        to_cut[i..to_cut.len()].iter().collect()
    }
    pub fn contains(&self, word: &str) -> bool {
        if word.is_empty() {
            return false;
        }

        let letters: Vec<char> = word.to_ascii_uppercase().chars().collect();
        let mut idx: NodeIndex = Self::root_index_of(&letters[0]);
        // navigate through the dawg and check if the leaf node is terminal
        for l_idx in 1..letters.len() {
            let mut found_child = false;
            for child_idx in self.children(idx) {
                if self.nodes[child_idx].letter == letters[l_idx] {
                    idx = child_idx;
                    found_child = true;
                    break;
                }
            }
            if !found_child {
                return false;
            }
        }
        self.nodes[idx].is_terminal
    }

    fn add_node(&mut self, letter: char) -> NodeIndex {
        let index = self.nodes.len();
        self.nodes.push(Node::new(letter));
        index
    }

    fn add_edge(&mut self, child: NodeIndex, parent: NodeIndex) {
        // first we create an edge from the child to the parent
        let edge_index = self.edges.len();
        let node = &mut self.nodes[child];
        self.edges.push(Edge {
            target: parent,
            next_outgoing_edge: node.first_parent,
        });
        node.first_parent = Some(edge_index);
        // and then the edge from parent to child
        let edge_index = self.edges.len();
        let node = &mut self.nodes[parent];
        self.edges.push(Edge {
            target: child,
            next_outgoing_edge: node.first_child,
        });
        node.first_child = Some(edge_index);
    }

    fn root_index_of(letter: &char) -> NodeIndex {
        (letter.to_ascii_uppercase() as usize) - ('A' as usize)
    }

    fn parents(&self, source: NodeIndex) -> Successors {
        let first_outgoing_edge = self.nodes[source].first_parent;
        self.successors(first_outgoing_edge)
    }

    fn children(&self, source: NodeIndex) -> Successors {
        let first_outgoing_edge = self.nodes[source].first_child;
        self.successors(first_outgoing_edge)
    }

    fn successors(&self, first_outgoing_edge: Option<EdgeIndex>) -> Successors {
        Successors {
            graph: self,
            current_edge_index: first_outgoing_edge,
        }
    }
}

pub struct Successors<'dict> {
    graph: &'dict Dict,
    current_edge_index: Option<EdgeIndex>,
}

impl<'dict> Iterator for Successors<'dict> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<NodeIndex> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let edge = &self.graph.edges[edge_num];
                self.current_edge_index = edge.next_outgoing_edge;
                Some(edge.target)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn contain_finds_contained() -> Result<(), &'static str> {
        let mut dict = Dict::new();

        dict.insert("test", "")?;
        dict.insert("tests", "test")?;

        assert!(dict.contains("test"));
        Ok(assert!(dict.contains("tests")))
    }

    #[test]
    fn contain_doesnt_find_not_contained() -> Result<(), &'static str> {
        let mut dict = Dict::new();

        dict.insert("tests", "")?;

        Ok(assert!(!dict.contains("test")))
    }

    #[test]
    fn cuts_off_matching_prefix_correctly() {
        assert_eq!(
            Dict::cut_off_matching_prefix("test_accepted", "test"),
            "_accepted".to_string()
        );
    }
}
