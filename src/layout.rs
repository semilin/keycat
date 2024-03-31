use crate::{Corpus, CorpusChar, NgramType};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type Pos = usize;

/// Flat keyboard layout structure.
#[derive(Clone)]
pub struct Layout {
    /// The actual details of each position is irrelevant to Keycat,
    /// so it doesn't need to be more complicated than just a list of
    /// CorpusChars.
    pub matrix: Vec<CorpusChar>,
}

impl Layout {
    #[must_use]
    pub fn nstroke_chars(&self, ns: &Nstroke) -> Vec<CorpusChar> {
        match ns {
            Nstroke::Monostroke(idx) => vec![self.matrix[*idx]],
            Nstroke::Bistroke(idx) => idx.iter().map(|p| self.matrix[*p]).collect(),
            Nstroke::Tristroke(idx) => idx.iter().map(|p| self.matrix[*p]).collect(),
        }
    }
    #[must_use]
    pub fn frequency(&self, corpus: &Corpus, ns: &Nstroke, ng: Option<NgramType>) -> u32 {
        match ns {
            Nstroke::Monostroke(idx) => corpus.chars[self.matrix[*idx]],
            Nstroke::Bistroke(idx) => {
                let idx = corpus.bigram_idx(self.matrix[idx[0]], self.matrix[idx[1]]);
                match ng {
                    Some(NgramType::Skipgram) => corpus.skipgrams[idx],
                    _ => corpus.bigrams[idx],
                }
            }
            Nstroke::Tristroke(idx) => {
                corpus.trigrams[corpus.trigram_idx(
                    self.matrix[idx[0]],
                    self.matrix[idx[1]],
                    self.matrix[idx[2]],
                )]
            }
        }
    }
    #[must_use]
    pub fn total_char_count(&self, corpus: &Corpus) -> u32 {
        self.matrix.iter().map(|c| corpus.chars[*c]).sum()
    }
    pub fn swap(&mut self, s: &Swap) {
        self.matrix.swap(s.a, s.b);
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(untagged))]
#[derive(Debug)]
pub enum Nstroke {
    Monostroke(usize),
    Bistroke([usize; 2]),
    Tristroke([usize; 3]),
}

impl Nstroke {
    #[must_use]
    pub fn to_vec(&self) -> Vec<usize> {
        match self {
            Nstroke::Monostroke(u) => vec![*u],
            Nstroke::Bistroke(a) => a.to_vec(),
            Nstroke::Tristroke(a) => a.to_vec(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Swap {
    pub a: usize,
    pub b: usize,
}

impl Swap {
    #[must_use]
    pub fn new(a: usize, b: usize) -> Self {
        Self { a, b }
    }
}