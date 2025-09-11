use rustler::{Binary, Encoder, Env, Term};

use crate::decoder::{deblob};
use crate::init_program::initialize_program;
use crate::vm::Vm;
mod consts;
mod init_program;
mod memory;
mod vm;
mod decoder;
mod encoder;

rustler::init!("Elixir.Pvm.Native", load = load);

fn load(_env: Env, _info: Term) -> bool {
    true
}

#[rustler::nif]
fn execute<'a>(env: Env<'a>, program: Binary, counter: u64, gas: u64, args: Binary) -> Term<'a> {
    // Try to initialize the program
    let (code, registers, memory) = match initialize_program(&program, &args) {
        None => {
            // Return {0, :panic, context} equivalent - for now just return {0, :panic}
            return (0u64, "panic").encode(env)
        }
        Some(v) => v,
    };

    let deblob = deblob(code);

    return registers.encode(env);
}

#[cfg(test)]
mod tests {
    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
