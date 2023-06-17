use std::collections::HashMap;
// use rayon::prelude::*;

pub type Pos = usize;

pub type CorpusIndex = usize;
pub type CorpusChar = CorpusIndex;

/// Structure for storing text ngram frequencies. 
pub struct Corpus {
    char_map: HashMap<char, CorpusChar>,
    pub char_list: Vec<Vec<char>>,
    pub chars: Vec<u32>,
    pub bigrams: Vec<u32>,
    pub skipgrams: Vec<u32>,
    pub trigrams: Vec<u32>,
}

impl Corpus {
    /// ```rust
    /// use keycat::Corpus;
    /// let mut corpus = Corpus::with_char_list(
    ///     "abcdefghijklmnopqrstuvwxyz"
    ///         .chars()
    ///         .map(|c| vec![c, c.to_uppercase().next().unwrap()])
    ///         .collect::<Vec<Vec<char>>>()
    /// );
    /// ```
    pub fn with_char_list(char_list: Vec<Vec<char>>) -> Self {
        let mut c = Corpus {
            char_map: HashMap::new(),
            char_list: char_list.clone(),
            chars: vec![0; char_list.len()],
            bigrams: vec![0; char_list.len()*char_list.len()],
            skipgrams: vec![0; char_list.len()*char_list.len()],
            trigrams: vec![0; char_list.len()*char_list.len()*char_list.len()],
        };
        for (i, chars) in c.char_list.iter().enumerate() {
	    for ch in chars.iter() {
		c.char_map.insert(*ch, i);
	    }
        }
        c
    }
    pub fn uncorpus_unigram(&self, unigram: CorpusIndex) -> char {
	self.char_list[unigram][0]
    }
    pub fn uncorpus_bigram(&self, bigram: CorpusIndex) -> Vec<char> {
	let len = self.char_list.len();
	let c1 = bigram / len;
	let c2 = bigram % len;
	vec![self.char_list[c1][0], self.char_list[c2][0]]
    }
    pub fn uncorpus_trigram(&self, trigram: CorpusIndex) -> Vec<char> {
	let len = self.char_list.len();
	let c1 = trigram / len.pow(2);
	let c2 = trigram % len.pow(2) / len;
	let c3 = trigram % len;
	vec![self.char_list[c1][0], self.char_list[c2][0], self.char_list[c3][0]]
    }
    pub fn corpus_char(&self, c: char) -> Option<&CorpusChar> {
	self.char_map.get(&c)
    }
    pub fn bigram_idx(&self, c1: CorpusChar, c2: CorpusChar) -> CorpusIndex {
        let len = self.char_list.len();
        (c1 * len) + c2
    }
    pub fn trigram_idx(&self, c1: CorpusChar, c2: CorpusChar, c3: CorpusChar) -> CorpusIndex {
        let len = self.char_list.len();
        (c1 * len * len) + (c2 * len) + c3
    }
    pub fn add_str(&mut self, s: &str) {
        let mut iter = s.chars().map(|c| self.char_map.get(&c));
        // extremely gross fix later
	let mut trigram: Vec<Option<&CorpusChar>> = vec![None, None, None];
	while let Some(c) = iter.next() {
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
		}
	    }
	}
    }
}

/// Flat keyboard layout structure.
pub struct Layout {
    /// The actual details of each position is irrelevant to Keycat,
    /// so it doesn't need to be more complicated than just a list of
    /// CorpusChars.
    matrix: Vec<CorpusChar>,
}

impl Layout {
    pub fn frequency(self, corpus: Corpus, ns: Nstroke) -> u32 {
        match ns {
            Nstroke::Monostroke(idx) => corpus.chars[self.matrix[idx]],
            Nstroke::Bistroke(idx) => corpus.bigrams[corpus.bigram_idx(idx[0], idx[1])],
            Nstroke::Tristroke(idx) => corpus.bigrams[corpus.trigram_idx(idx[0], idx[1], idx[2])],
        }
    }
}

pub enum Nstroke {
    Monostroke(usize),
    Bistroke([usize; 2]),
    Tristroke([usize; 3]),
}

pub enum NgramType {
    Monogram,
    Bigram,
    Skipgram,
    Trigram,
}

pub type MetricIndex = usize;
pub type NstrokeIndex = usize;

pub struct NstrokeAmount {
    metric: MetricIndex,
    nstroke: Nstroke,
    amount: f32,
}

impl NstrokeAmount {
    pub fn new(metric: MetricIndex, nstroke: Nstroke, amount: f32) -> Self {
        Self {
            metric, nstroke, amount
        }
    }
}

/// Structure for storing metric data and performing analysis on layouts.
pub struct MetricTable {
    /// The list of metrics. Not much data about the metric is needed,
    /// so just the NgramType is stored.
    pub metrics: Vec<NgramType>,
    /// The list of strokes needed for analysis.
    pub strokes: Vec<NstrokeAmount>,
    /// Maps a position to all of the strokes that contain it.
    pub position_strokes: Vec<Vec<NstrokeIndex>> ,
}

impl MetricTable {
    /// ```rust
    /// use keycat::{NgramType, Nstroke, NstrokeAmount, MetricTable};
    /// let metrics = vec![NgramType::Bigram];
    /// let strokes = vec![NstrokeAmount::new(0, Nstroke::Bistroke([0, 2]), 0.0)];
    /// let table = MetricTable::from(metrics, strokes, 3);
    /// ```
    pub fn from(metrics: Vec<NgramType>, strokes: Vec<NstrokeAmount>, num_positions: usize) -> Self {
        let mut position_strokes: Vec<Vec<NstrokeIndex>> = vec![vec![]; num_positions];
        for (i, stroke) in strokes.iter().map(|s| &s.nstroke).enumerate() {
            for pos in match stroke {
                Nstroke::Monostroke(v) => vec![*v],
                Nstroke::Bistroke(a) => a.to_vec(),
                Nstroke::Tristroke(a) => a.to_vec(),
            } {
                position_strokes[pos].push(i);
            }
        }
        Self {
            metrics, strokes, position_strokes
        }
    }
}
