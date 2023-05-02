use keycat::{Corpus};
use std::fs::File;
use std::io::{self, BufRead};

pub fn main() {
    let mut corpus = Corpus::with_char_list(
        "abcdefghijklmnopqrstuvwxyz"
            .chars()
            .map(|c| vec![c, c.to_uppercase().next().unwrap()])
	    .collect::<Vec<Vec<char>>>()
    );
    println!("{:?}", corpus.char_list);
    let file = File::open("./tr_quotes.txt").unwrap();
    let lines = io::BufReader::new(file).lines();

    lines.flatten().for_each(|l| corpus.add_str(&l));

    for (i, v) in corpus.bigrams.iter().enumerate() {
	if *v != 0 {
	    let s: String = corpus.uncorpus_bigram(i).iter().collect();
	    println!("{}. '{}' {}", i, s, v);
	}
    }
}
