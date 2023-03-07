use std::collections::HashSet;

pub struct Pos {
    row: usize,
    col: usize,
}

pub type CorpusChar = u16;

pub struct Corpus {
    pub char_list: HashSet<char>,
    pub chars: Vec<u64>,
    pub bigrams: Vec<u64>,
    pub skipgrams: Vec<u64>,
    pub trigrams: Vec<u64>,
}

impl Corpus {
    pub fn with_char_list(char_list: &[char]) -> Self {
	Corpus {
	    char_list: char_list.iter().map(|c| *c).collect(),
	    chars: vec!(),
	    bigrams: vec!(),
	    trigrams: vec!(),
	    skipgrams: vec!()
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
