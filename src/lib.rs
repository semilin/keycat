#[cfg(feature = "opt")]
pub mod opt;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
// use rayon::prelude::*;

pub type Pos = usize;

pub type CorpusIndex = usize;
pub type CorpusChar = CorpusIndex;

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
    /// ```rust
    /// use keycat::Corpus;
    /// let mut corpus = Corpus::with_char_list(
    ///     &mut "abcdefghijklmnopqrstuvwxyz"
    ///         .chars()
    ///         .map(|c| vec![c, c.to_uppercase().next().unwrap()])
    ///         .collect::<Vec<Vec<char>>>()
    /// );
    /// ```
    pub fn with_char_list(char_list: &mut Vec<Vec<char>>) -> Self {
        let char_list = {
            let mut vec = vec![vec!['\0']];
            vec.append(char_list);
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
        vec![
            self.char_list[c1][0],
            self.char_list[c2][0],
            self.char_list[c3][0],
        ]
    }
    pub fn corpus_char(&self, c: char) -> &CorpusChar {
        self.char_map.get(&c).unwrap_or(&0)
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
    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let lines = BufReader::new(file).lines();
        lines.flatten().for_each(|l| self.add_str(&l));
        Ok(())
    }
}

/// Flat keyboard layout structure.
#[derive(Clone)]
pub struct Layout {
    /// The actual details of each position is irrelevant to Keycat,
    /// so it doesn't need to be more complicated than just a list of
    /// CorpusChars.
    pub matrix: Vec<CorpusChar>,
}

impl Layout {
    pub fn nstroke_chars(&self, ns: &Nstroke) -> Vec<CorpusChar> {
        match ns {
            Nstroke::Monostroke(idx) => vec![self.matrix[*idx]],
            Nstroke::Bistroke(idx) => idx.iter().map(|p| self.matrix[*p]).collect(),
            Nstroke::Tristroke(idx) => idx.iter().map(|p| self.matrix[*p]).collect(),
        }
    }
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
    pub fn to_vec(&self) -> Vec<usize> {
        match self {
            Nstroke::Monostroke(u) => vec![*u],
            Nstroke::Bistroke(a) => a.to_vec(),
            Nstroke::Tristroke(a) => a.to_vec(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
pub enum NgramType {
    Monogram,
    Bigram,
    Skipgram,
    Trigram,
}

pub type MetricIndex = usize;
pub type NstrokeIndex = usize;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct MetricAmount {
    #[cfg_attr(feature = "serde", serde(rename = "met"))]
    pub metric: MetricIndex,
    #[cfg_attr(feature = "serde", serde(rename = "amt"))]
    pub amount: f32,
}

impl MetricAmount {
    pub fn new(metric: MetricIndex, amount: f32) -> Self {
        Self { metric, amount }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct NstrokeData {
    #[cfg_attr(feature = "serde", serde(rename = "ns"))]
    pub nstroke: Nstroke,
    #[cfg_attr(feature = "serde", serde(rename = "ams"))]
    pub amounts: Vec<MetricAmount>,
}

impl NstrokeData {
    pub fn new(nstroke: Nstroke, amounts: Vec<MetricAmount>) -> Self {
        Self { nstroke, amounts }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
/// Structure for storing metric data and performing analysis on layouts.
pub struct MetricData {
    /// The list of metrics. Not much data about the metric is needed,
    /// so just the NgramType is stored.
    pub metrics: Vec<NgramType>,
    /// The list of strokes needed for analysis.
    pub strokes: Vec<NstrokeData>,
    /// Maps a position to all of the strokes that contain it.
    pub position_strokes: Vec<Vec<NstrokeIndex>>,
}

impl MetricData {
    /// ```rust
    /// use keycat::{NgramType, Nstroke, NstrokeData, MetricAmount, MetricData};
    /// let metrics = vec![NgramType::Bigram];
    /// let strokes = vec![NstrokeData::new(Nstroke::Bistroke([0, 1]),
    ///                                     vec![MetricAmount::new(0, 0.0)])];
    /// let data = MetricData::from(metrics, strokes, 2);
    /// ```
    pub fn from(metrics: Vec<NgramType>, strokes: Vec<NstrokeData>, num_positions: usize) -> Self {
        let mut position_strokes: Vec<Vec<NstrokeIndex>> = vec![vec![]; num_positions];
        for (i, stroke) in strokes.iter().map(|s| &s.nstroke).enumerate() {
            for pos in stroke.to_vec() {
                position_strokes[pos].push(i);
            }
        }
        Self {
            metrics,
            strokes,
            position_strokes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Swap {
    a: usize,
    b: usize,
}

impl Swap {
    pub fn new(a: usize, b: usize) -> Self {
        Self { a, b }
    }
}

pub struct Analyzer {
    pub data: MetricData,
    pub corpus: Corpus,
}

impl Analyzer {
    pub fn from(data: MetricData, corpus: Corpus) -> Self {
	let analyzer = Self {
            data,
            corpus,
	};

	analyzer
    }
    pub fn calc_stats(&self, mut stats: Vec<f32>, l: &Layout) -> Vec<f32> {
	for stroke in &self.data.strokes {
            let ns = &stroke.nstroke;
            let basefreq = l.frequency(&self.corpus, ns, None);
            let skipfreq = match ns {
                Nstroke::Bistroke(_) => l.frequency(&self.corpus, ns, Some(NgramType::Skipgram)),
                _ => 0,
            };

            for amount in &stroke.amounts {
                let freq = if let NgramType::Skipgram = &self.data.metrics[amount.metric] {
                    skipfreq
                } else {
                    basefreq
                };
                stats[amount.metric] += freq as f32 * amount.amount;
            }
        }
	stats
    }

    /// Calculates the diff for a swap.
    pub fn swap_diff(&self, mut diffs: Vec<f32>, l: &Layout, swap: &Swap) -> Vec<f32> {
	let corpus = &self.corpus;
	let c_a = l.matrix[swap.a];
        let c_b = l.matrix[swap.b];
        let it1 = &mut self.data.position_strokes[swap.a].iter();
        let it2 = &mut self.data.position_strokes[swap.b].iter();
        let mut stroke_a = None;
        let mut stroke_b = None;
        loop {
            match (stroke_a, stroke_b) {
                (None, None) => {
                    stroke_a = it1.next();
                    stroke_b = it2.next();
                }
                (Some(_), None) => {
                    stroke_a = it1.next();
                }
                (None, Some(_)) => {
                    stroke_b = it2.next();
                }
                (Some(a), Some(b)) => {
                    if a < b {
                        stroke_a = it1.next();
                    } else if b < a {
                        stroke_b = it2.next();
                    } else {
                        stroke_a = it1.next();
                        stroke_b = it2.next();
                    }
                }
            };

            let stroke = match (stroke_a, stroke_b) {
                (None, None) => {
                    break;
                }
                (Some(a), None) => a,
                (None, Some(b)) => b,
                (Some(a), Some(b)) => a.min(b),
            };

            // println!("{:?} {:?} -> stroke {}", stroke_a, stroke_b, stroke);

            let data = &self.data.strokes[*stroke];
            let ns = &data.nstroke;
            let basefreqs: [i32; 2] = [
                l.frequency(&self.corpus, ns, None) as i32,
                match ns {
                    Nstroke::Monostroke(a) => {
                        corpus.chars[if *a == swap.a {
                            c_b
                        } else if *a == swap.b {
                            c_a
                        } else {
                            l.matrix[*a]
                        }]
                    }
                    Nstroke::Bistroke(arr) => {
                        let [a, b]: [usize; 2] = arr.map(|p| {
                            if p == swap.a {
                                c_b
                            } else if p == swap.b {
                                c_a
                            } else {
                                l.matrix[p]
                            }
                        });
                        // println!("{:?}", b);
                        // println!("{} | {}{}", arr.map(|p| corpus.uncorpus_unigram(l.matrix[p])).iter().collect::<String>(), corpus.uncorpus_unigram(a), corpus.uncorpus_unigram(b));
                        corpus.bigrams[corpus.bigram_idx(a, b)]
                    }
                    Nstroke::Tristroke(arr) => {
                        let [a, b, c]: [usize; 3] = arr.map(|p| {
                            if p == swap.a {
                                c_b
                            } else if p == swap.b {
                                c_a
                            } else {
                                l.matrix[p]
                            }
                        });
                        corpus.trigrams[corpus.trigram_idx(a, b, c)]
                    }
                } as i32,
            ];
            let skipfreqs: [i32; 2] = match ns {
                Nstroke::Bistroke(arr) => [
                    l.frequency(&corpus, ns, Some(NgramType::Skipgram)) as i32,
                    {
                        let [a, b]: [usize; 2] = arr.map(|p| {
                            if p == swap.a {
                                c_b
                            } else if p == swap.b {
                                c_a
                            } else {
                                l.matrix[p]
                            }
                        });
                        corpus.skipgrams[corpus.bigram_idx(a, b)] as i32
                    },
                ],
                _ => [0, 0],
            };

            for amount in &data.amounts {
                let diff: f32 = if let NgramType::Skipgram = self.data.metrics[amount.metric] {
                    skipfreqs[1] - skipfreqs[0]
                } else {
                    basefreqs[1] - basefreqs[0]
                } as f32;
                // println!("{: >8}", diff);
                let real_diff = amount.amount * diff;
                diffs[amount.metric] += real_diff;
            }
        }
	diffs
    }
}
