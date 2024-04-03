use crate::{analysis::Analyzer, Layout, Swap};
use rand::prelude::*;
use rayon::prelude::*;

/// Trait for objective functions, used in optimization.
pub trait Objective {
    /// Returns how well the stats meet the objective. Lower values
    /// should mean a better fit to the objective.
    fn score(&self, stats: &[f32]) -> f32;
}

pub struct Weight {
    metric: usize,
    weight: f32,
}

/// The most basic kind of objective function. Each metric is
/// associated with a multiplicative weight to create a single
/// composite value.
pub struct WeightsObjective {
    pub weights: Vec<Weight>,
}

impl WeightsObjective {
    pub fn new(weights: Vec<Weight>) -> Self {
        WeightsObjective { weights }
    }
}

impl Objective for WeightsObjective {
    #[must_use]
    fn score(&self, stats: &[f32]) -> f32 {
        self.weights
            .iter()
            .fold(0.0, |acc, Weight { metric, weight}| {
                acc + (weight * stats[*metric])
            })
    }
}

pub trait Optimizer {
    /// Prepares the optimizer for running.
    fn setup(&mut self, l: Layout);
    #[must_use]
    fn pin(self, pins: Vec<usize>) -> Self;
    /// Runs the optimization for as long as needed.
    fn run(
        &mut self,
        analyzer: &Analyzer,
        objective: &(dyn Objective + Send + Sync),
    ) -> Vec<(Layout, f32)>;
}

/// An `Optimizer` that runs simulated annealing on a pool of size
/// `population_size`.
pub struct AnnealingOptimizer {
    layouts: Vec<Layout>,
    pins: Vec<usize>,
    /// The number of layouts to be optimized in parallel.
    pub population_size: usize,
    /// The amount the temperature should decrease per step.
    pub temp_decrement: f64,
}

impl AnnealingOptimizer {
    #[must_use]
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
        self.layouts.resize(self.population_size, l);
    }

    fn pin(mut self, pins: Vec<usize>) -> Self {
        self.pins = pins;
        self
    }

    fn run(
        &mut self,
        analyzer: &Analyzer,
        objective: &(dyn Objective + Send + Sync),
    ) -> Vec<(Layout, f32)> {
        self.layouts.par_iter_mut().for_each(|l| {
            let mut diffs = vec![0.0; analyzer.data.metrics.len()];
            let mut rng = rand::thread_rng();
            let possible_swaps: Vec<Swap> = (0..l.matrix.len())
                .flat_map(|a| (0..l.matrix.len()).map(move |b| Swap::new(a, b)))
                .filter(|swap| !self.pins.iter().any(|p| *p == swap.a || *p == swap.b))
                .collect();
            if possible_swaps.is_empty() {
                return;
            }
            let mut temp: f64 = 1.0;
            while temp >= 0.0 {
                let swap = possible_swaps
                    .choose(&mut rng)
                    .expect("possible_swaps should not be empty");
                analyzer.swap_diff(&mut diffs, l, swap);
                let diff = objective.score(&diffs);
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
            .map(|l| (l.clone(), objective.score(&analyzer.calc_stats(l))))
            .collect();
        layouts.sort_by(|a, b| a.1.partial_cmp(&b.1).expect("score should never be NaN"));
        layouts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        analysis::{Analyzer, MetricAmount, MetricData, NstrokeData},
        Corpus, NgramType, Nstroke,
    };
    #[test]
    fn test_optimization() {
        let mut corpus = Corpus::with_char_list(
            "abcdefghijklmnopqrstuvwxyz,./;"
                .chars()
                .map(|c| vec![c])
                .collect(),
        );

        let text = "the quick brown fox jumps over the lazy dog";
        corpus.add_str(&text);

        let qwerty = corpus.layout_from_str("qazwsxedcrfvtgbyhnujmik,lo.p;/");

        let metrics = vec![NgramType::Bigram];
        let mut strokes: Vec<NstrokeData> = vec![];
        // bigram alternation
        for a in 0..30 {
            for b in 0..30 {
                if (a < 15) == (b < 15) {
                    strokes.push(NstrokeData::new(
                        Nstroke::Bistroke([a, b]),
                        vec![MetricAmount::new(0, 1.0)],
                    ));
                }
            }
        }
        let data = MetricData::from(metrics, strokes, 30);
        let analyzer = Analyzer::from(data, corpus);
        let mut optimizer = AnnealingOptimizer::new(8, -0.01).pin(vec![0]);
        let objective = WeightsObjective::new(vec![Weight {
            metric: 0,
            weight: 1.0,
        }]);
        let start = objective.score(&analyzer.calc_stats(&qwerty));
        optimizer.setup(qwerty.clone());
        let optimized = optimizer.run(&analyzer, &objective);
        let end = &optimized[0].1;
        assert!(*end < start, "optimized should be lower score than unoptimized");
    }
}
