use keycat::{Corpus, CorpusChar};

pub fn main() {
    let mut corpus = Corpus::with_char_list(
        "abcdefghijklmnopqrstuvwxyz"
            .chars()
            .map(|c| (c, c.to_uppercase().next().unwrap()))
            .collect::<Vec<(char, char)>>()
            .as_slice(),
    );
    println!("{:?}", corpus.char_list);
    corpus.add_str("hello");
    for (i, v) in corpus.bigrams.iter().enumerate() {
	if *v != 0 {
	    let first = i / corpus.char_list.len();
	    let second = i % corpus.char_list.len();
	    println!("{} = {} + {} ({:?} {:?})", i, first, second, corpus.char_list[first], corpus.char_list[second]);
	}
    }
}
