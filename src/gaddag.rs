// Credit to https://github.com/amedeedaboville/fst-gaddag,
// where this code was stolen from (and then cut down and rewritten)

pub use fst::raw::{CompiledAddr, Node};
use fst::{Result, Set};
use std::{collections::BTreeSet, iter};

pub static SEP: u8 = b'+';
pub static STR_SEP: &str = "+";

/*
 * CARES:
 * SERAC
 * ERAC+S
 * RAC+ES
 * AC+RES
 * C+ARES
 *
*/

#[derive(Clone)]
pub struct Gaddag {
    set: fst::Set<Vec<u8>>,
}

impl Gaddag {
    /// Returns true if the given word is in the dictionary.
    /// Searches for `^input.rev()$`.
    pub fn contains(&self, input: &str) -> bool {
        self.set
            .contains(input.chars().rev().map(|ch| ch as u8).collect::<Vec<u8>>())
    }

    ///Takes a Fst::Set and returns a Gaddag.
    pub fn from_fst(set: Set<Vec<u8>>) -> Self {
        Self { set }
    }

    ///Builds a Gaddag from its byte representation.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        let fst_set = Set::new(bytes)?;
        Ok(Self::from_fst(fst_set))
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
    pub fn node_for_prefix(&self, prefix: &str) -> Option<CompiledAddr> {
        let mut current_node: Node = self.set.as_fst().root();
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
        Some(current_node.addr())
    }
    /// Attempts to follow the node in the GADDAG, and returns the next node.
    pub fn can_next(&self, node_addr: CompiledAddr, next: char) -> Option<CompiledAddr> {
        let current_node = self.set.as_fst().node(node_addr);
        current_node
            .find_input(next as u8)
            .map(|i| current_node.transition(i).addr)
    }

    fn build_entries(input: impl IntoIterator<Item = String>) -> BTreeSet<Vec<u8>> {
        let mut entries: BTreeSet<Vec<u8>> = BTreeSet::new();
        for word in input {
            // we can skip reversing 0 elements because it's the same as reversing all elements
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
                entries.insert(word.as_bytes().into_iter().rev().cloned().collect());
            }
        }
        entries
    }
}
