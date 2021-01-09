use super::cpu::CPU;
use super::elf_reader;
use super::fuzzer::Fuzzer;

use std::path::PathBuf;
use std::rc::Rc;
use std::str;
use std::cell::RefCell;

pub struct Emu{
    cpu: CPU,
    fuzzer: Rc<RefCell<Fuzzer>>,
}

impl Emu{
    pub fn new() -> Emu{
        Emu{
            cpu: CPU::new(true),
            fuzzer: Rc::new(RefCell::new(Fuzzer::new())),
        }
    }

    /// OS Emulator part, should be in a different structure but i just cant
    /// get it working
    pub fn exec_elf(&mut self, path: &PathBuf) {
        let elf = match elf::File::open_path(&path) {
            Ok(f) => f,
            Err(e) => panic!("Error {:?}", e)
        };
    
        let entrypoint = elf.ehdr.entry;
        println!("Entry point: {:#X}", entrypoint);
    
        let mut symtab: Option<elf::Section> = None;
        let mut strtab: Option<elf::Section> = None;
        
        println!("Mapping memory sections:");
        for s in elf.sections{
            if (s.shdr.flags.0 & elf::types::SHF_ALLOC.0) != 0 {
                self.cpu.memory.allocate(s.shdr.addr, s.shdr.size, &s.data);

                println!("  * {:}({}b): {:08X} -> {:08X}", 
                    s.shdr.name, s.shdr.size, s.shdr.addr, s.shdr.addr + s.shdr.size);
            }
    
            match s.shdr.name.as_ref() {
                ".symtab" => {
                    symtab = Some(s); 
                },
                ".strtab" => {
                    strtab = Some(s);
                }
                _ => {},
            }
        }
        
        let symtab = symtab.expect("Symtab memory region not found in ELF");
        let strtab = strtab.expect("Strtab memory region not found in ELF");
    
        let symbols =  elf_reader::read_symbols_list(symtab, strtab);
    
        //Test3 is the state from where we want to restart the execution,
        //set a breakpoint on it and save a snapshot
        if let Some(addr) = symbols.get("main"){
            println!("Breakpoint set at main ({:#8X})", addr);
            self.cpu.set_breakpoint(*addr, Self::bp_save_state);
        }
        else{
            panic!("Couldnt find main in exported symbols");
        }
    
        //The success symbol represents the end of the execution, from
        //there we want to reset to the initial state reached at Test3
        if let Some(addr) = symbols.get("exit"){
            println!("Breakpoint set at exit ({:#8X})", addr);
            self.cpu.set_breakpoint(*addr, Self::reset_to_snapshot);
        }
        else{
            panic!("Couldnt find pass in exported symbols");
        }
        self.cpu.execute(entrypoint, Rc::clone(&self.fuzzer));
    }
    
    fn bp_save_state(cpu: &mut CPU){
        println!("State saved:");
        println!("{:?}", cpu);
        cpu.save_as_initial_state();
    }
    
    fn reset_to_snapshot(cpu: &mut CPU){
        let coverage = cpu.reset_to_initial_state();
        println!("Coverage : {:X?}", coverage.len());
        if (cpu.nbr_exec != 0) && ((cpu.nbr_exec % 10) == 0){
            println!("State reset: {:}", cpu.nbr_exec);
            cpu.exit = true;
        }
    }
}