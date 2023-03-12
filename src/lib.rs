use std::{collections::HashMap};

#[derive(Hash, PartialEq, Eq)]
pub struct Pos {
    row: usize,
    col: usize,
}

pub type CorpusChar = usize;

pub struct Corpus {
    char_map: HashMap<char, CorpusChar>,
    pub char_list: Vec<(char, char)>,
    pub chars: Vec<u64>,
    pub bigrams: Vec<u64>,
    pub skipgrams: Vec<u64>,
    pub trigrams: Vec<u64>,
}

impl Corpus {
    pub fn with_char_list(char_list: &[(char, char)]) -> Self {
        let mut c = Corpus {
            char_map: HashMap::new(),
            char_list: char_list.to_vec(),
            chars: vec![0; char_list.len()],
            bigrams: vec![0; char_list.len()*char_list.len()],
            skipgrams: vec![0; char_list.len()*char_list.len()],
            trigrams: vec![0; char_list.len()*char_list.len()*char_list.len()],
        };
        for (i, (c1, c2)) in c.char_list.iter().enumerate() {
            c.char_map.insert(*c1, i);
            c.char_map.insert(*c2, i);
        }
        c
    }
    pub fn uncorpus_unigram(&self, unigram: usize) -> char {
	self.char_list[unigram].0
    }
    pub fn uncorpus_bigram(&self, bigram: usize) -> Vec<char> {
	let len = self.char_list.len();
	let c1 = bigram / len;
	let c2 = bigram % len;
	vec![self.char_list[c1].0, self.char_list[c2].0]
    }
    pub fn uncorpus_trigram(&self, trigram: usize) -> Vec<char> {
	let len = self.char_list.len();
	let c1 = trigram / len.pow(2);
	let c2 = trigram % len.pow(2) / len;
	let c3 = trigram % len;
	vec![self.char_list[c1].0, self.char_list[c2].0, self.char_list[c3].0]
    }
    pub fn corpus_char(&self, c: char) -> Option<&CorpusChar> {
	self.char_map.get(&c)
    }
    pub fn bigram_idx(&self, c1: CorpusChar, c2: CorpusChar) -> usize {
        let len = self.char_list.len();
        (c1 * len) + c2
    }
    pub fn trigram_idx(&self, c1: CorpusChar, c2: CorpusChar, c3: CorpusChar) -> usize {
        let len = self.char_list.len();
        (c1 * len * len) + (c2 * len) + c3
    }
    pub fn add_str(&mut self, s: &str) {
        let mut iter = s.chars().map(|c| self.char_map.get(&c));
        // extremely gross fix later
	let mut trigram: Vec<Option<&usize>> = vec![None, None, None];
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

pub trait Layout {
    fn pos_to_key(&self, p: Pos) -> Option<&CorpusChar>;
    fn pos_to_key_unchecked(&self, p: Pos) -> CorpusChar;
    fn swap(&mut self, a: Pos, b: Pos);
}

pub struct StandardLayout {
    pub matrix: [CorpusChar; 30],
}

impl Layout for StandardLayout {
    fn pos_to_key(&self, p: Pos) -> Option<&CorpusChar> {
        self.matrix.get(p.row * 10 + p.col)
    }
    fn pos_to_key_unchecked(&self, p: Pos) -> CorpusChar {
	self.matrix[p.row * 10 + p.col]
    }
    fn swap(&mut self, a: Pos, b: Pos) {
	self.matrix.swap(a.row*10 + a.col, b.row*10 + b.col);
    }
}

pub struct FlexibleLayout {
    pub matrix: HashMap<Pos, CorpusChar>,
}

impl Layout for FlexibleLayout {
    fn pos_to_key(&self, p: Pos) -> Option<&CorpusChar> {
        self.matrix.get(&p)
    }
    fn pos_to_key_unchecked(&self, p: Pos) -> CorpusChar {
        *self.matrix.get(&p).unwrap()
    }
    fn swap(&mut self, a: Pos, b: Pos) {
	let x = self.matrix.get_mut(&a).unwrap() as *mut CorpusChar;
	let y = self.matrix.get_mut(&b).unwrap() as *mut CorpusChar;
	unsafe {
	    std::ptr::swap(x, y);
	}
    }
}
