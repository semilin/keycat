use crate::{Corpus, Layout, NgramType, Nstroke, Swap};
use std::cmp::Ordering;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
    #[must_use]
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
    #[must_use]
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
    /// use keycat::{NgramType, Nstroke};
    /// use keycat::analysis::{NstrokeData, MetricAmount, MetricData};
    /// let metrics = vec![NgramType::Bigram];
    /// let strokes = vec![NstrokeData::new(Nstroke::Bistroke([0, 1]),
    ///                                     vec![MetricAmount::new(0, 0.0)])];
    /// let data = MetricData::from(metrics, strokes, 2);
    /// ```
    #[must_use]
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

pub struct Analyzer {
    pub data: MetricData,
    pub corpus: Corpus,
}

#[allow(clippy::cast_precision_loss)]
impl Analyzer {
    #[allow(clippy::cast_possible_wrap)]
    fn diff_freqs(a: u32, b: u32) -> i32 {
        match a.cmp(&b) {
            Ordering::Greater => (a - b) as i32,
            Ordering::Less => -((b - a) as i32),
            Ordering::Equal => 0,
        }
    }

    #[must_use]
    pub fn from(data: MetricData, corpus: Corpus) -> Self {
        Self { data, corpus }
    }
    #[must_use]
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

    /// Calculates the diff for a `Swap`.
    #[must_use]
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
                (Some(a), Some(b)) => match a.cmp(b) {
                    Ordering::Less => {
                        stroke_a = it1.next();
                    }
                    Ordering::Greater => {
                        stroke_b = it2.next();
                    }
                    Ordering::Equal => {
                        stroke_a = it1.next();
                        stroke_b = it2.next();
                    }
                },
            };

            let stroke = match (stroke_a, stroke_b) {
                (None, None) => {
                    break;
                }
                (Some(a), None) => a,
                (None, Some(b)) => b,
                (Some(a), Some(b)) => a.min(b),
            };

            let data = &self.data.strokes[*stroke];
            let ns = &data.nstroke;
            let basefreqs: [u32; 2] = [
                l.frequency(&self.corpus, ns, None),
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
                },
            ];
            let skipfreqs: [u32; 2] = match ns {
                Nstroke::Bistroke(arr) => [l.frequency(corpus, ns, Some(NgramType::Skipgram)), {
                    let [a, b]: [usize; 2] = arr.map(|p| {
                        if p == swap.a {
                            c_b
                        } else if p == swap.b {
                            c_a
                        } else {
                            l.matrix[p]
                        }
                    });
                    corpus.skipgrams[corpus.bigram_idx(a, b)]
                }],
                _ => [0, 0],
            };

            for amount in &data.amounts {
                let diff: f32 = if let NgramType::Skipgram = self.data.metrics[amount.metric] {
                    Analyzer::diff_freqs(skipfreqs[1], skipfreqs[0])
                } else {
                    Analyzer::diff_freqs(basefreqs[1], basefreqs[0])
                } as f32;

                let real_diff = amount.amount * diff;
                diffs[amount.metric] += real_diff;
            }
        }
        diffs
    }
}
