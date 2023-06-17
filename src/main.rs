use keycat::{Nstroke, NgramType, NstrokeAmount, MetricTable, Corpus};
use std::fs::File;
use std::io::{self, BufRead};

pub fn main() {
    let mut corpus = Corpus::with_char_list(
        "abcdefghijklmnopqrstuvwxyz,./"
            .chars()
            .map(|c| vec![c, c.to_uppercase().next().unwrap()])
	    .collect::<Vec<Vec<char>>>()
    );
    println!("{:?}", corpus.char_list);
    let file = File::open("./tr.txt").unwrap();
    let lines = io::BufReader::new(file).lines();

    lines.flatten().for_each(|l| corpus.add_str(&l));

    for (i, v) in corpus.bigrams.iter().enumerate() {
	if *v >= 10000 {
	    let s: String = corpus.uncorpus_bigram(i).iter().collect();
	    println!("{}. '{}' {}", i, s, v);
	}
    }

    let metrics = vec![NgramType::Bigram];
    let strokes = vec![
        NstrokeAmount::new(0, Nstroke::Bistroke([0, 2]), 0.0),
        NstrokeAmount::new(0, Nstroke::Bistroke([3, 4]), 0.0),
        NstrokeAmount::new(0, Nstroke::Bistroke([3, 1]), 0.0)];
    let table = MetricTable::from(metrics, strokes, 6);
    println!("{:?}", table.position_strokes);
}
