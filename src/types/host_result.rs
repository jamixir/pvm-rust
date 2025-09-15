use rustler::{Atom, NifTuple, NifUntaggedEnum};

#[derive(Debug, Clone, NifUntaggedEnum)]
pub enum HostOutput {
    Bytes(Vec<u8>),
    Atom(Atom),
}

#[derive(Debug, Clone, NifTuple)]
pub struct ExecuteResult {
    pub used_gas: u64,
    pub output: HostOutput,
}
