use std::collections::HashMap;

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
    pub fn with_char_list(char_list: Vec<(char, char)>) -> Self {
	let mut c = Corpus {
	    char_map: HashMap::new(),
	    char_list,
	    chars: vec!(),
	    bigrams: vec!(),
	    trigrams: vec!(),
	    skipgrams: vec!()
	};
	for (i, (c1, c2)) in c.char_list.iter().enumerate() {
	    c.char_map.insert(*c1, i);
	    c.char_map.insert(*c2, i);
	}
	c
    }
    pub fn bigram_idx(&self, c1: CorpusChar, c2: CorpusChar) -> usize {
	let len = self.char_list.len();
	(c1*len)+c2
    }
    pub fn trigram_idx(&self, c1: CorpusChar, c2: CorpusChar, c3: CorpusChar) -> usize {
	let len = self.char_list.len();
	(c1*len*len)+(c2*len)+c3
    }
    pub fn add_str(&mut self, s: &str) {
	let mut iter = s.chars().map(|c| self.char_map.get(&c));
	// extremely gross fix later
	if let (Some(Some(c1)), Some(Some(c2))) = (iter.next(), iter.next()) {
	    for c3 in iter {
		if let Some(c3) = c3  {
		    self.chars[*c1] += 1;
		    let bg = self.bigram_idx(*c1, *c2);
		    self.bigrams[bg] += 1;
		    self.skipgrams[bg] += 1;
		    let tg = self.trigram_idx(*c1, *c2, *c3);
		    self.trigrams[tg] += 1;
		}
	    }
	    self.chars[*c1] += 1;
	    self.chars[*c2] += 1;
	    let bg = self.bigram_idx(*c1, *c2);
	    self.bigrams[bg] += 1;
	}
    }
}

pub trait Layout {
    fn pos_to_key(&self, p: Pos) -> CorpusChar;
}

pub struct StandardLayout {
    pub matrix: [CorpusChar; 30] 
}

impl Layout for StandardLayout {
    fn pos_to_key(&self, p: Pos) -> CorpusChar {
        self.matrix[p.row*10 + p.col]
    }
}

pub struct FlexibleLayout {
    pub matrix: Vec<Vec<CorpusChar>>
}

impl Layout for FlexibleLayout {
    fn pos_to_key(&self, p: Pos) -> CorpusChar {
	self.matrix[p.row][p.col]
    }
}
