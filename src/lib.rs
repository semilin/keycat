use std::collections::HashMap;

pub type Pos = usize;

pub type CorpusChar = usize;
pub type CorpusIndex = usize;

pub struct Corpus {
    char_map: HashMap<char, CorpusChar>,
    pub char_list: Vec<Vec<char>>,
    pub chars: Vec<CorpusIndex>,
    pub bigrams: Vec<CorpusIndex>,
    pub skipgrams: Vec<CorpusIndex>,
    pub trigrams: Vec<CorpusIndex>,
}

impl Corpus {
    pub fn with_char_list(char_list: Vec<Vec<char>>) -> Self {
        let mut c = Corpus {
            char_map: HashMap::new(),
            char_list: char_list.clone(),
            chars: vec![0; char_list.len()],
            bigrams: vec![0; char_list.len()*char_list.len()],
            skipgrams: vec![0; char_list.len()*char_list.len()],
            trigrams: vec![0; char_list.len()*char_list.len()*char_list.len()],
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
	vec![self.char_list[c1][0], self.char_list[c2][0], self.char_list[c3][0]]
    }
    pub fn corpus_char(&self, c: char) -> Option<&CorpusChar> {
	self.char_map.get(&c)
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
	let mut trigram: Vec<Option<&CorpusIndex>> = vec![None, None, None];
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
}

pub type Layout = Vec<CorpusChar>;
