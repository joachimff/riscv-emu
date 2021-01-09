pub mod cpu;

use cpu::emu::Emu;
use std::path::PathBuf;

fn main(){
    let mut emu = Emu::new();
    emu.exec_elf(&PathBuf::from("test/real/main"));
}   