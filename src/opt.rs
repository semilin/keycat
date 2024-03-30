use crate::{analysis::Analyzer, Layout, Swap};
use rand::prelude::*;
use rand::thread_rng;
use rayon::prelude::*;

pub struct Scoring {
    pub weights: Vec<(usize, f32)>,
}

impl Scoring {
    pub fn score(&self, diff: &[f32]) -> f32 {
        self.weights
            .iter()
            .fold(0.0, |acc, (i, weight)| acc + (weight * diff[*i]))
    }
}

pub trait Optimizer {
    /// Prepares the optimizer for running.
    fn setup(&mut self, l: Layout);
    fn pin(self, pins: Vec<usize>) -> Self;
    /// Runs the optimization for as long as needed.
    fn run(&mut self, analyzer: &Analyzer, scoring: &Scoring) -> Vec<(Layout, f32)>;
}

/// An Optimizer that runs simulated annealing on a pool of size
/// population_size.
pub struct AnnealingOptimizer {
    layouts: Vec<Layout>,
    pins: Vec<usize>,
    /// The number of layouts to be optimized in parallel.
    pub population_size: usize,
    /// The amount the temperature should decrease per step.
    pub temp_decrement: f64,
}

impl AnnealingOptimizer {
    pub fn new(population_size: usize, temp_decrement: f64) -> Self {
        Self {
            layouts: Vec::with_capacity(population_size),
            pins: vec![],
            population_size,
            temp_decrement,
        }
    }
}

impl Optimizer for AnnealingOptimizer {
    fn setup(&mut self, l: Layout) {
        let mut rng = thread_rng();
        self.layouts.resize(self.population_size, l);
    }

    fn pin(mut self, pins: Vec<usize>) -> Self {
        self.pins = pins;
        self
    }

    fn run(&mut self, analyzer: &Analyzer, scoring: &Scoring) -> Vec<(Layout, f32)> {
        self.layouts.par_iter_mut().for_each(|l| {
            let mut diffs = vec![0.0; analyzer.data.metrics.len()];
            let mut rng = rand::thread_rng();
            let possible_swaps: Vec<Swap> = (0..l.matrix.len())
                .flat_map(|a| (0..l.matrix.len()).map(move |b| Swap::new(a, b)))
                .filter(|swap| !self.pins.iter().any(|p| *p == swap.a || *p == swap.b))
                .collect();
            let mut temp: f64 = 1.0;
            while temp >= 0.0 {
                let swap = possible_swaps.choose(&mut rng).unwrap();
                diffs = analyzer.swap_diff(diffs, l, swap);
                let diff = scoring.score(&diffs);
                if diff < 0.0 || rng.gen::<f64>() < temp {
                    l.swap(swap);
                }
                for val in &mut diffs {
                    *val = 0.0;
                }
                temp += self.temp_decrement;
            }
        });
        let mut layouts: Vec<(Layout, f32)> = self
            .layouts
            .par_iter()
            .map(|l| {
                (
                    l.clone(),
                    scoring.score(&analyzer.calc_stats(vec![0.0; analyzer.data.metrics.len()], l)),
                )
            })
            .collect();
        layouts.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        layouts
    }
}
