use std::io::Error as IoError;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("life is full of surprises")]
    Unexpected,

    #[error("current directory doesn't contain get repository")]
    NotAGetRepo,

    #[error("repository already exist")]
    RepoAlreadyExist,

    #[error("only utf-8 is supported")]
    UnsupportedEncoding,

    #[error("io error {0}")]
    IoError(#[from] IoError),

    #[error("no such commit")]
    CommitNotFound,
}
