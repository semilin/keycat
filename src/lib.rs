pub mod analysis;
pub mod corpus;
pub mod layout;
#[cfg(feature = "opt")]
pub mod opt;
pub use corpus::{Corpus, CorpusChar, NgramType};
pub use layout::{Layout, Nstroke, Pos, Swap};
