use rustler::{Env, Term, Encoder, Binary};
rustler::init!("Elixir.Pvm.Native");

use crate::init_program::initialize_program;
mod consts;
mod memory;
mod vm;
mod init_program;



#[rustler::nif]
fn execute<'a>(
    env: Env<'a>,
    program: Binary,
    counter: u64,
    gas: u64,
    args: Binary,
) -> Term<'a> {
    // For now: just allocate a VM, don't run
    let vm = initialize_program(&program, &args);

    // Return {:ok, %{pc: ..., gas: ...}}
    let result = (
        "ok",
        vec![("pc", vm.pc as u64), ("gas", gas), ("counter", counter)],
    );

    result.encode(env)
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
