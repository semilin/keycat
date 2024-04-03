use crate::Layout;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents an index in a `Corpus` for bigrams, skipgrams, and
/// trigrams.
pub type CorpusIndex = usize;
/// Represents a character in the `Corpus`.
pub type CorpusChar = CorpusIndex;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
pub enum NgramType {
    Monogram,
    Bigram,
    Skipgram,
    Trigram,
}

/// Structure for storing text ngram frequencies.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Corpus {
    char_map: HashMap<char, CorpusChar>,
    pub char_list: Vec<Vec<char>>,
    pub chars: Vec<u32>,
    pub bigrams: Vec<u32>,
    pub skipgrams: Vec<u32>,
    pub trigrams: Vec<u32>,
}

impl Corpus {
    /// Produces a new `Corpus` with the specified list of
    /// characters. Any characters that are not in the list will be
    /// ignored when counting frequencies.

    /// ```rust
    /// use keycat::Corpus;
    /// let mut corpus = Corpus::with_char_list(
    ///     "abcdefghijklmnopqrstuvwxyz"
    ///         .chars()
    ///         .map(|c| vec![c, c.to_uppercase().next().unwrap()])
    ///         .collect::<Vec<Vec<char>>>()
    /// );
    /// ```
    #[must_use]
    pub fn with_char_list(mut char_list: Vec<Vec<char>>) -> Self {
        let char_list = {
            let mut vec = vec![vec!['\0']];
            vec.append(&mut char_list);
            vec
        };
        let mut c = Corpus {
            char_map: HashMap::new(),
            char_list: char_list.clone(),
            chars: vec![0; char_list.len()],
            bigrams: vec![0; char_list.len() * char_list.len()],
            skipgrams: vec![0; char_list.len() * char_list.len()],
            trigrams: vec![0; char_list.len() * char_list.len() * char_list.len()],
        };
        for (i, chars) in c.char_list.iter().enumerate() {
            for ch in chars {
                c.char_map.insert(*ch, i);
            }
        }
        c
    }
    /// Converts a `CorpusIndex` back into its original `char`. This
    /// doesn't start at 0, but 1. Index 0 is reserved as a null
    /// character for positions that may not exist on the layout.
    ///
    /// ```rust
    /// use keycat::Corpus;
    /// let mut corpus = Corpus::with_char_list("abcdefghijklmnopqrstuvwxyz"
    ///                                           .chars()
    ///                                           .map(|c| vec![c])
    ///                                           .collect());
    /// assert_eq!('a', corpus.uncorpus_unigram(1));
    /// assert_eq!('b', corpus.uncorpus_unigram(2));
    /// assert_eq!('z', corpus.uncorpus_unigram(26));
    /// ```
    #[must_use]
    pub fn uncorpus_unigram(&self, unigram: CorpusIndex) -> char {
        self.char_list[unigram][0]
    }
    #[must_use]
    pub fn uncorpus_bigram(&self, bigram: CorpusIndex) -> Vec<char> {
        let len = self.char_list.len();
        let c1 = bigram / len;
        let c2 = bigram % len;
        vec![self.char_list[c1][0], self.char_list[c2][0]]
    }
    #[must_use]
    pub fn uncorpus_trigram(&self, trigram: CorpusIndex) -> Vec<char> {
        let len = self.char_list.len();
        let c1 = trigram / len.pow(2);
        let c2 = trigram % len.pow(2) / len;
        let c3 = trigram % len;
        vec![
            self.char_list[c1][0],
            self.char_list[c2][0],
            self.char_list[c3][0],
        ]
    }
    /// Converts a `char` to its corresponding index in the `Corpus`.
    #[must_use]
    pub fn corpus_char(&self, c: char) -> CorpusChar {
        *self.char_map.get(&c).unwrap_or(&0)
    }
    #[must_use]
    pub fn corpus_bigram(&self, chars: &[char; 2]) -> CorpusIndex {
        self.bigram_idx(self.corpus_char(chars[0]), self.corpus_char(chars[1]))
    }
    #[must_use]
    pub fn corpus_trigram(&self, chars: &[char; 3]) -> CorpusIndex {
        self.trigram_idx(
            self.corpus_char(chars[0]),
            self.corpus_char(chars[1]),
            self.corpus_char(chars[2]),
        )
    }
    #[must_use]
    pub fn bigram_idx(&self, c1: CorpusChar, c2: CorpusChar) -> CorpusIndex {
        let len = self.char_list.len();
        (c1 * len) + c2
    }
    #[must_use]
    pub fn trigram_idx(&self, c1: CorpusChar, c2: CorpusChar, c3: CorpusChar) -> CorpusIndex {
        let len = self.char_list.len();
        (c1 * len * len) + (c2 * len) + c3
    }
    /// Processes a string and adds its ngram frequencies to the
    /// `Corpus`.
    pub fn add_str(&mut self, s: &str) {
        let iter = s.chars().map(|c| self.char_map.get(&c));
        // extremely gross fix later
        let mut trigram: Vec<Option<&CorpusChar>> = vec![None, None, None];
        for c in iter {
            trigram.rotate_left(1);
            trigram[2] = c;
            if let Some(c3) = trigram[2] {
                self.chars[*c3] += 1;
                if let Some(c2) = trigram[1] {
                    let bg = self.bigram_idx(*c2, *c3);
                    self.bigrams[bg] += 1;
                    if let Some(c1) = trigram[0] {
                        let tg = self.trigram_idx(*c1, *c2, *c3);
                        self.trigrams[tg] += 1;
                        let sg = self.bigram_idx(*c1, *c3);
                        self.skipgrams[sg] += 1;
                    }
                } else if let Some(c1) = trigram[0] {
                    let sg = self.bigram_idx(*c1, *c3);
                    self.skipgrams[sg] += 1;
                }
            }
        }
    }
    /// Reads a file line by line and adds the ngrams from each line
    /// to the `Corpus` totals.
    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let lines = BufReader::new(file).lines();
        lines.map_while(Result::ok).for_each(|l| self.add_str(&l));
        Ok(())
    }
    #[must_use]
    pub fn layout_from_str(&self, s: &str) -> Layout {
        Layout {
            matrix: s.chars().map(|c| self.corpus_char(c)).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_corpus() {
        let mut corpus = Corpus::with_char_list(
            "abcdefghijklmnopqrstuvwxyz"
                .chars()
                .map(|c| vec![c])
                .collect(),
        );
        corpus.add_str("the quick brown fox jumps over the lazy dog");

        assert_eq!(corpus.trigrams[corpus.corpus_trigram(&['t', 'h', 'e'])], 2);
        assert_eq!(corpus.trigrams[corpus.corpus_trigram(&['q', 'u', 'i'])], 1);
        assert_eq!(corpus.trigrams[corpus.corpus_trigram(&['z', 'y', ' '])], 0);
        assert_eq!(corpus.trigrams[corpus.corpus_trigram(&['a', 'b', 'c'])], 0);
        assert_eq!(corpus.bigrams[corpus.corpus_bigram(&['t', 'h'])], 2);
        assert_eq!(corpus.bigrams[corpus.corpus_bigram(&['v', 'e'])], 1);
        assert_eq!(corpus.skipgrams[corpus.corpus_bigram(&['v', 'e'])], 0);
        assert_eq!(corpus.skipgrams[corpus.corpus_bigram(&['f', 'x'])], 1);
        assert_eq!(corpus.skipgrams[corpus.corpus_bigram(&['t', 'e'])], 2);
        assert_eq!(
            corpus.skipgrams[corpus.corpus_bigram(&['e', 'l'])],
            1,
            "skipgrams should be counted across invalid characters"
        );
    }
}
