
pub type CptmlResult<T> = Result<T, CptmlError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CptmlError {
    FauxPanic(String),
    NotImplemented,
}
