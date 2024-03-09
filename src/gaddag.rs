use std::{collections::BTreeSet, iter};

use fst::{raw::CompiledAddr, Result};

static SEP: u8 = b'+';

// newtype compiledaddr to stop misuse
// (compiledaddr is just a type alias for usize)
#[derive(Clone, Copy)]
pub struct Node {
    addr: CompiledAddr,
}

impl Node {
    fn new(addr: CompiledAddr) -> Self {
        Self { addr }
    }
}

/// https://en.wikipedia.org/wiki/GADDAG
#[derive(Clone)]
pub struct Gaddag {
    set: fst::Set<Vec<u8>>,
}

impl Gaddag {
    pub fn accepts(&self, input: &str) -> bool {
        self.set
            .contains(input.as_bytes().iter().rev().cloned().collect::<Vec<u8>>())
    }

    pub fn root(&self) -> Node {
        Node::new(self.set.as_fst().root().addr())
    }

    pub fn from_fst(set: fst::Set<Vec<u8>>) -> Self {
        Self { set }
    }

    ///Builds a Gaddag from its byte representation.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(Self::from_fst(fst::Set::new(bytes)?))
    }

    ///Builds a Gaddag from an input list of words.
    pub fn from_words(input: impl IntoIterator<Item = String>) -> Self {
        Self::from_fst(fst::Set::from_iter(Gaddag::build_entries(input)).unwrap())
    }

    ///Returns the byte representation of the Gaddag.
    pub fn as_bytes(&self) -> &[u8] {
        self.set.as_fst().as_bytes()
    }

    /// Returns the node address for a prefix in the dictionary.
    /// This means the input doesn't have to be a full word, but has to be a prefix
    /// of a word in the dictionary. Will return None if the word doesn't exist in the
    /// dictionary.
    pub fn node_for_prefix(&self, prefix: &str) -> Option<Node> {
        let mut current_node = self.set.as_fst().root();
        for ch in prefix.chars() {
            if let Some(transition_idx) = current_node.find_input(ch as u8) {
                let next_node = self
                    .set
                    .as_fst()
                    .node(current_node.transition_addr(transition_idx));
                current_node = next_node;
            } else {
                return None;
            }
        }
        Some(Node::new(current_node.addr()))
    }

    /// Attempts to follow the node in the GADDAG, and returns the next node.
    pub fn next_node(&self, node: &Node, next: char) -> Option<Node> {
        let current_node = self.set.as_fst().node(node.addr);
        current_node
            .find_input(next as u8)
            .map(|i| Node::new(current_node.transition_addr(i)))
    }

    pub fn is_final(&self, node: &Node) -> bool {
        self.set.as_fst().node(node.addr).is_final()
    }

    /*
     * CARES becomes:
     * ERAC+S
     * RAC+ES
     * AC+RES
     * C+ARES
     * ECARES
     */
    fn build_entries(input: impl IntoIterator<Item = String>) -> BTreeSet<Vec<u8>> {
        let mut entries = BTreeSet::new();
        for word in input {
            for n in 1..word.len() {
                entries.insert(
                    word.as_bytes()
                        .iter()
                        .take(n)
                        .rev()
                        .chain(iter::once(&SEP))
                        .chain(word.as_bytes().iter().skip(n))
                        .cloned()
                        .collect(),
                );
                entries.insert(word.as_bytes().iter().rev().cloned().collect());
            }
        }
        entries
    }
}
