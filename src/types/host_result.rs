use rustler::{Atom, NifStruct, NifTaggedEnum};

#[derive(Debug, Clone, NifTaggedEnum)]

pub enum HostOutput {
    Bytes(Vec<u8>),
    Atom(Atom),
}

#[derive(Debug, Clone, NifStruct)]
#[module = "ExecuteResult"]
pub struct ExecuteResult {
    pub used_gas: u64,
    pub output: HostOutput,
}
