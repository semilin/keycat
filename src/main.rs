use keycat::{Corpus};
use std::fs::File;
use std::io::{self, BufRead};

pub fn main() {
    let mut corpus = Corpus::with_char_list(
        "abcdefghijklmnopqrstuvwxyz"
            .chars()
            .map(|c| (c, c.to_uppercase().next().unwrap()))
            .collect::<Vec<(char, char)>>()
            .as_slice(),
    );
    println!("{:?}", corpus.char_list);
    let file = File::open("./tr_quotes.txt").unwrap();
    let lines = io::BufReader::new(file).lines();

    lines.flatten().for_each(|l| corpus.add_str(&l));

    for (i, v) in corpus.trigrams.iter().enumerate() {
	if *v != 0 {
	    let s: String = corpus.uncorpus_trigram(i).iter().collect();
	    println!("{}. '{}' {}", i, s, v);
	}
    }
}
