use keycat::{Nstroke, NgramType, NstrokeData, MetricAmount, MetricData, Corpus, Analyzer, Layout};
use std::fs::File;
use std::io::{self, BufRead};

pub fn main() {
    let mut corpus = Corpus::with_char_list(
        "abcdefghijklmnopqrstuvwxyz,./'"
            .chars()
            .map(|c| vec![c, c.to_uppercase().next().unwrap()])
	    .collect::<Vec<Vec<char>>>()
    );
    println!("{:?}", corpus.char_list);
    let file = File::open("./tr.txt").unwrap();
    let lines = io::BufReader::new(file).lines();

    lines.flatten().for_each(|l| corpus.add_str(&l));

    for (i, v) in corpus.bigrams.iter().enumerate() {
	if *v >= 20000 {
	    let s: String = corpus.uncorpus_bigram(i).iter().collect();
	    println!("{}. '{}' {}", i, s, v);
	}
    }

    let metrics = vec![NgramType::Bigram];
    let strokes = vec![
        // NstrokeData::new(Nstroke::Bistroke([0, 2]), vec![MetricAmount::new(0, 0.0)]),
        // NstrokeData::new(Nstroke::Bistroke([3, 4]), vec![MetricAmount::new(0, 0.0)]),
        NstrokeData::new(Nstroke::Bistroke([0, 1]), vec![MetricAmount::new(0, 1.0)])];
    let data = MetricData::from(metrics, strokes, 2);
    println!("{:?}", data.position_strokes);

    let layout = Layout {
	matrix: "flhvz'wuoysrntkcdeaixjbmqpg,./".chars().map(|c| *corpus.corpus_char(c).unwrap()).collect()
    };
    
    let analyzer = Analyzer::from(&data, &corpus, layout);

    println!("{:?}", analyzer.stats);
}
