#![allow(dead_code)]

use std::collections::HashSet;

type EdgeIndex = usize;
type NodeIndex = usize;

// Custom data structure to hold dictionary and support operations required for scrabble.
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
    children_edges: HashSet<EdgeIndex>,
    parent_edges: HashSet<EdgeIndex>,
    is_terminal: bool,
}

impl Node {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

#[derive(Debug)]
struct Edge {
    letter: char,
    child: NodeIndex,
    parent: NodeIndex,
    id: EdgeIndex,
}

impl Dict {
    fn new() -> Self {
        Self {
            nodes: vec![Node::new()],
            edges: Vec::new(),
        }
    }

    fn from_iter<'a, I>(iter: I) -> Result<Self, &'static str>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut dict = Self::new();
        let start_node: NodeIndex = 0;
        let mut previous_word = "";
        for word in iter {
            if word < previous_word {
                return Err("Words not provided in lexiographical order.");
            }
            let last_node = dict.insert(start_node, &word);
        }
        Ok(dict)
    }

    fn insert(&mut self, start_node: NodeIndex, word: &str) -> NodeIndex {
        // if word < previous_word {
        //     return Err("Input stream not lexiographically ordered.");
        // }

        // let cut_word = Self::cut_off_matching_prefix(word, previous_word);

        let letters: Vec<char> = word.to_ascii_uppercase().chars().collect();
        // let mut idx: NodeIndex = if cut_word.len() < word.len() {
        //     start_node
        // } else {
        //     Self::root_index_of(&letters[0])
        // };
        let mut idx = start_node;
        'letter_loop: for letter in letters {
            for &edge in &self.nodes[idx].children_edges {
                if self.edges[edge].letter == letter {
                    idx = self.edges[edge].child;
                    continue 'letter_loop;
                }
            }
            let child_idx = self.add_node();
            self.add_edge(child_idx, idx, letter);
            idx = child_idx;
        }
        self.nodes[idx].is_terminal = true;

        idx
    }

    fn cut_off_matching_prefix(to_cut: &str, other: &str) -> String {
        to_cut
            .chars()
            .zip(other.chars().chain(std::iter::repeat('?')))
            .skip_while(|(a, b)| a == b)
            .map(|(a, _)| a)
            .collect()
    }

    pub fn contains(&self, word: &str) -> bool {
        if word.is_empty() {
            return false;
        }

        let mut idx: NodeIndex = 0;
        'letter_loop: for letter in word.to_ascii_uppercase().chars() {
            for &child in &self.nodes[idx].children_edges {
                if self.edges[child].letter == letter {
                    idx = self.edges[child].child;
                    continue 'letter_loop;
                }
                return false;
            }
        }
        self.nodes[idx].is_terminal
    }

    fn add_node(&mut self) -> NodeIndex {
        let index = self.nodes.len();
        self.nodes.push(Node::new());
        index
    }

    fn add_edge(&mut self, child: NodeIndex, parent: NodeIndex, letter: char) {
        // first we create an edge from the child to the parent
        let id = self.edges.len();
        let child_node = &mut self.nodes[child];
        child_node.parent_edges.insert(id);
        let parent_node = &mut self.nodes[parent];
        self.edges.push(Edge {
            letter,
            child,
            parent,
            id,
        });
        parent_node.children_edges.insert(id);
    }

    fn root_index_of(letter: &char) -> NodeIndex {
        (letter.to_ascii_uppercase() as usize) - ('A' as usize)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn contain_finds_contained() {
        let dict = Dict::from_iter(vec!["contained", "contained_also"]).unwrap();

        assert!(dict.contains("contained"));
        assert!(dict.contains("contained_also"));
    }

    #[test]
    fn contain_doesnt_find_not_contained() {
        let mut dict = Dict::new();

        dict.insert(0, "contained");
        assert!(!dict.contains("not_contained"));
    }

    #[test]
    fn cuts_off_matching_prefix_correctly() {
        assert_eq!(
            Dict::cut_off_matching_prefix("test_accepted", "test"),
            "_accepted".to_string()
        );

        assert_eq!(
            Dict::cut_off_matching_prefix("test_accepted", "test_also_accepted"),
            "ccepted".to_string()
        );
        assert_eq!(
            Dict::cut_off_matching_prefix("test_a", "test_b"),
            "a".to_string()
        );
        assert_eq!(
            Dict::cut_off_matching_prefix("Aword", "B"),
            "Aword".to_string()
        );
    }
}
