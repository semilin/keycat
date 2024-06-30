use crate::{Corpus, CorpusChar, NgramType};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type Pos = usize;

/// Flat keyboard layout structure. The actual details of each
/// position is irrelevant to Keycat, so it doesn't need to be more
/// complicated than just a list of CorpusChars.
#[derive(Clone)]
pub struct Layout(pub Vec<CorpusChar>);

impl Layout {
    #[must_use]
    pub fn nstroke_chars(&self, ns: &Nstroke) -> Vec<CorpusChar> {
        match ns {
            Nstroke::Monostroke(idx) => vec![self.0[*idx]],
            Nstroke::Bistroke(idx) => idx.iter().map(|p| self.0[*p]).collect(),
            Nstroke::Tristroke(idx) => idx.iter().map(|p| self.0[*p]).collect(),
        }
    }
    #[must_use]
    pub fn frequency(&self, corpus: &Corpus, ns: &Nstroke, ng: Option<NgramType>) -> u32 {
        match ns {
            Nstroke::Monostroke(idx) => corpus.chars[self.0[*idx]],
            Nstroke::Bistroke(idx) => {
                let idx = corpus.bigram_idx(self.0[idx[0]], self.0[idx[1]]);
                match ng {
                    Some(NgramType::Skipgram) => corpus.skipgrams[idx],
                    _ => corpus.bigrams[idx],
                }
            }
            Nstroke::Tristroke(idx) => {
                corpus.trigrams[corpus.trigram_idx(self.0[idx[0]], self.0[idx[1]], self.0[idx[2]])]
            }
        }
    }
    #[must_use]
    pub fn total_char_count(&self, corpus: &Corpus) -> u32 {
        self.0.iter().map(|c| corpus.chars[*c]).sum()
    }
    fn bi_count(&self, corpus: &Corpus, frequencies: &[u32]) -> u32 {
        self.0
            .iter()
            .flat_map(|a| (self.0.iter().map(move |b| (a, b))))
            .map(|(a, b)| frequencies[corpus.bigram_idx(*a, *b)])
            .sum()
    }
    #[must_use]
    pub fn total_bigram_count(&self, corpus: &Corpus) -> u32 {
        self.bi_count(corpus, &corpus.bigrams)
    }
    #[must_use]
    pub fn total_skipgram_count(&self, corpus: &Corpus) -> u32 {
        self.bi_count(corpus, &corpus.skipgrams)
    }
    #[must_use]
    pub fn total_trigram_count(&self, corpus: &Corpus) -> u32 {
        self.0
            .iter()
            .flat_map(|a| (self.0.iter().map(move |b| (a, b))))
            .flat_map(|(a, b)| (self.0.iter().map(move |c| (a, b, c))))
            .map(|(a, b, c)| corpus.trigrams[corpus.trigram_idx(*a, *b, *c)])
            .sum()
    }
    #[must_use]
    pub fn totals(&self, corpus: &Corpus) -> LayoutTotals {
        LayoutTotals {
            chars: self.total_char_count(&corpus),
            bigrams: self.total_bigram_count(&corpus),
            skipgrams: self.total_skipgram_count(&corpus),
            trigrams: self.total_trigram_count(&corpus),
        }
    }
    pub fn swap(&mut self, s: &Swap) {
        self.0.swap(s.a, s.b);
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

#[derive(Debug)]
pub struct LayoutTotals {
    chars: u32,
    bigrams: u32,
    skipgrams: u32,
    trigrams: u32,
}

impl LayoutTotals {
    pub fn percentage(&self, freq: f32, kind: NgramType) -> f32 {
        let denom = match kind {
            NgramType::Monogram => self.chars,
            NgramType::Bigram => self.bigrams,
            NgramType::Skipgram => self.skipgrams,
            NgramType::Trigram => self.trigrams,
        } as f32;
        100. * freq / denom
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_layout() {
        let mut corpus = Corpus::with_char_list(
            "abcdefghijklmnopqrstuvwxyz,./;"
                .chars()
                .map(|c| vec![c])
                .collect(),
        );

        let text = "the quick brown fox jumps over the lazy dog";
        corpus.add_str(&text);

        let mut qwerty = corpus.layout_from_str("qazwsxedcrfvtgbyhnujmik,lo.p;/");

        assert_eq!(
            1,
            qwerty.frequency(&corpus, &Nstroke::Monostroke(0), None),
            "q occurs once"
        );
        assert_eq!(
            3,
            qwerty.frequency(&corpus, &Nstroke::Monostroke(6), None),
            "e occurs 3 times"
        );

        assert_eq!(
            2,
            qwerty.frequency(&corpus, &Nstroke::Bistroke([12, 16]), None),
            "th occurs twice"
        );

        assert_eq!(
            2,
            qwerty.frequency(&corpus, &Nstroke::Tristroke([12, 16, 6]), None),
            "the occurs twice"
        );

        assert_eq!(corpus.corpus_char('q'), qwerty.0[0]);
        assert_eq!(corpus.corpus_char('a'), qwerty.0[1]);
        qwerty.swap(&Swap::new(0, 1));
        assert_eq!(corpus.corpus_char('a'), qwerty.0[0]);
        assert_eq!(corpus.corpus_char('q'), qwerty.0[1]);

        assert_eq!(
            text.chars().filter(|c| *c != ' ').collect::<Vec<_>>().len() as u32,
            qwerty.total_char_count(&corpus)
        );
    }
}
