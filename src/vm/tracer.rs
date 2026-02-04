use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::vm::VmState;
use crate::vm::instructions::opcodes::opcode_to_name;

#[derive(Debug)]
pub struct Tracer {
    file: Mutex<File>,
    counter: AtomicU64,
}

impl Tracer {
    pub fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();
        let file_name = format!("pvm_trace_{}.log", timestamp);
        let file = File::create(file_name).expect("Unable to create trace file");

        Self {
            file: Mutex::new(file),
            counter: AtomicU64::new(0),
        }
    }

    pub fn trace(&self, pc: usize, opcode: u8, state: &VmState) {
        let mut file = self.file.lock().unwrap();
        let counter = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
        
        let counter_str = format!("{:>5}", counter);
        let pc_str = format!("{:>5}", pc);
        
        let instruction_str = opcode_to_name(opcode); 
        let instruction_str_padded = format!("{:<20}", instruction_str);

        let gas = state.initial_gas - state.spent_gas;
        let regs = state.registers.data.iter().map(|r| r.to_string()).collect::<Vec<_>>().join(" ");
        
        // line = "#{counter_str}: PC #{pc_str} #{instruction_str(opcode)} g=#{post_state.gas} reg=[#{regs}]"
        writeln!(file, "{}: PC {} {} g={} reg=[{}]", 
            counter_str, pc_str, instruction_str_padded, gas, regs).unwrap();
    }
}
