//! Tokenization library for SutraWorks
//!
//! Supports multiple tokenization algorithms:
//! - BPE (Byte Pair Encoding) - Used by GPT-2, GPT-3
//! - WordPiece - Used by BERT
//! - Unigram - Used by SentencePiece/mT5
//!
//! Features:
//! - Fast tokenization with caching
//! - Vocabulary management
//! - Special token handling
//! - Encoding/decoding with offsets

pub mod bpe;
pub mod error;
pub mod normalizer;
pub mod pretokenizer;
pub mod tokenizer;
pub mod unigram;
pub mod vocab;
pub mod wordpiece;

pub use bpe::{BpeConfig, BpeTokenizer};
pub use error::{Result, TokenizerError};
pub use tokenizer::{Encoding, Tokenizer, TokenizerConfig};
pub use unigram::{UnigramConfig, UnigramTokenizer};
pub use vocab::{Vocab, VocabBuilder};
pub use wordpiece::{WordPieceConfig, WordPieceTokenizer};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{BpeConfig, BpeTokenizer};
    pub use crate::{Encoding, Tokenizer, TokenizerConfig};
    pub use crate::{Result, TokenizerError};
    pub use crate::{UnigramConfig, UnigramTokenizer};
    pub use crate::{Vocab, VocabBuilder};
    pub use crate::{WordPieceConfig, WordPieceTokenizer};
}
