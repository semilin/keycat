#[cfg(feature = "opt")]
pub mod opt;
pub mod layout;
pub mod corpus;
pub mod analysis;
pub use layout::{Layout, Pos, Nstroke, Swap};
pub use corpus::{Corpus, CorpusChar, NgramType};
