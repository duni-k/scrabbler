// Credit to https://github.com/amedeedaboville/fst-gaddag,
// where this code was stolen from (and then cut down and rewritten)

pub use fst::raw::{CompiledAddr, Node as FstNode};
use fst::{Result, Set};
use std::iter;

pub static SEP: u8 = b'+';
pub static STR_SEP: &str = "+";

// newtype compiledaddr, type alias not enough to
// stop users from misusing api by sending random usize
// as a "node"
#[derive(Clone, Copy)]
pub struct Node {
    addr: CompiledAddr,
}

impl Node {
    fn new(addr: CompiledAddr) -> Self {
        Self { addr }
    }
}

#[derive(Clone)]
pub struct Gaddag {
    set: fst::Set<Vec<u8>>,
}

impl Gaddag {
    pub fn contains(&self, input: &str) -> bool {
        self.set.contains(input.as_bytes())
    }

    pub fn root(&self) -> Node {
        Node::new(self.set.as_fst().root().addr())
    }

    ///Takes a Fst::Set and returns a Gaddag.
    pub fn from_fst(set: Set<Vec<u8>>) -> Self {
        Self { set }
    }

    ///Builds a Gaddag from its byte representation.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(Self::from_fst(Set::new(bytes)?))
    }

    ///Builds a Gaddag from an input list of words.
    pub fn from_words(input: impl IntoIterator<Item = String>) -> Self {
        Self::from_fst(Set::from_iter(Gaddag::build_entries(input)).unwrap())
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
        let mut current_node: FstNode = self.set.as_fst().root();
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
     * CARES
     * ERAC+S
     * RAC+ES
     * AC+RES
     * C+ARES
     */
    fn build_entries(input: impl IntoIterator<Item = String>) -> impl IntoIterator<Item = Vec<u8>> {
        // seriously doubt this is idiomatic but it SHOULD be better to return an iterator
        // so we can lazily evaluate the input, because if input is buffered (which it is in our case),
        // we never have to hold the entire input in memory.
        input.into_iter().flat_map(|word| {
            vec![
                word.as_bytes().iter().cloned().collect(),
                (1..word.len())
                    .flat_map(|n| {
                        word.as_bytes()
                            .iter()
                            .take(n)
                            .rev()
                            .chain(iter::once(&SEP))
                            .chain(word.as_bytes().iter().skip(n))
                            .cloned()
                    })
                    .collect(),
            ]
        })
    }
}
