use keycat::{Nstroke, NgramType, NstrokeData, MetricAmount, MetricData, Corpus, Analyzer, Layout};
use std::fs::File;
use std::io::{self, BufRead};

pub fn main() {
    let mut corpus = {
        let mut char_list = "abcdefghijklmnopqrstuvwxyz"
            .chars()
            .map(|c| vec![c, c.to_uppercase().next().unwrap()])
            .collect::<Vec<Vec<char>>>();
        char_list.extend(vec![vec![',', '<'],
			      vec!['.', '>'],
			      vec!['/', '?'],
			      vec!['\'', '\"']]);
        Corpus::with_char_list(char_list)
    };

    println!("{:?}", corpus.char_list);

    corpus.add_file("./tr.txt").unwrap();

    for (i, v) in corpus.bigrams.iter().enumerate() {
	if *v >= 533 {
	    let s: String = corpus.uncorpus_bigram(i).iter().collect();
	    println!("{}. '{}' {}", i, s, v);
	}
    }

    let metrics = vec![NgramType::Bigram, NgramType::Skipgram];
    let strokes = vec![
        NstrokeData::new(Nstroke::Bistroke([0, 13]), vec![MetricAmount::new(0, 1.0)]),
        NstrokeData::new(Nstroke::Bistroke([0, 13]), vec![MetricAmount::new(1, 1.0)])];
    let data = MetricData::from(metrics, strokes, 30);
    println!("{:?}", data.position_strokes);

    let layout = Layout {
	matrix: "flhvz'wuoysrntkcdeaixjbmqpg,./".chars().map(|c| *corpus.corpus_char(c).unwrap()).collect()
    };
    
    let analyzer = Analyzer::from(&data, &corpus, layout);

    println!("{:?}", analyzer.stats);
}
