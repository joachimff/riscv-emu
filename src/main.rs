pub mod cpu;

use cpu::cpu::CPU;
use cpu::elf_reader;

use std::path::PathBuf;
use std::fs::read_dir;

fn bp_end_of_test(cpu: &mut CPU){
    println!("Tests OK");
    cpu.exit = true;
}

fn bp_test_error(cpu: &mut CPU){
    println!("{:?}", cpu);
    panic!("Fail on test: {:#8?}", cpu.registers.common[3]);
}

fn start_test_elf(path: &PathBuf){
    let mut cpu: CPU = CPU::new(true);

    let elf = match elf::File::open_path(&path) {
        Ok(f) => f,
        Err(e) => panic!("Error {:?}", e)
    };

    let entry_point = elf.ehdr.entry;
    println!("Entry point: {:#X}", entry_point);

    let mut symtab: Option<elf::Section> = None;
    let mut strtab: Option<elf::Section> = None;
    
    println!("Mapping memory:");
    for s in elf.sections{
        if (s.shdr.flags.0 & elf::types::SHF_ALLOC.0) != 0 {
            cpu.memory.allocate(s.shdr.addr, s.shdr.size, &s.data);
            println!("  * {:}", s.shdr.name);
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
    
    //Set a breakpoint on the success function of the test
    if let Some(addr) = symbols.get("pass"){
        println!("Breakpoint set at pass ({:#8X})", addr);
        cpu.set_breakpoint(*addr, bp_end_of_test);
    }
    else{
        println!("Couldnt find pass in exported symbols");
    }
    
    //Set a breakpoint on the failure function of the test
    if let Some(addr) = symbols.get("fail"){
        println!("Breakpoint set at fail ({:#8X})", addr);
        cpu.set_breakpoint(*addr, bp_test_error);
    }
    else{
        println!("Couldnt find pass in exported symbols");
    }

    //We start from the first test in order to skip the init part which requires 
    //csrc extension
    if let Some(entrypoint) = symbols.get("test_2"){
        println!("Test_2 found at {:#8X}", entrypoint);
        cpu.execute(*entrypoint);
    }
    else{
        panic!("Couldnt find Test_2 in exported symbols");
    }
}

fn run_all_tests(){
    let paths = read_dir("test/riscv-tests/").unwrap();
    let mut i = 0;
    for p in paths{
        println!("Executing test: {:?}({:})", p, i);
        start_test_elf(&p.unwrap().path());

        i += 1;
    }
    println!("{:} tests passed", i);
}

fn exec_elf(path: &PathBuf) {
    let mut cpu: CPU = CPU::new(true);

    let elf = match elf::File::open_path(&path) {
        Ok(f) => f,
        Err(e) => panic!("Error {:?}", e)
    };

    let entrypoint = elf.ehdr.entry;
    println!("Entry point: {:#X}", entrypoint);

    let mut symtab: Option<elf::Section> = None;
    let mut strtab: Option<elf::Section> = None;
    
    println!("Mapping memory:");
    for s in elf.sections{
        if (s.shdr.flags.0 & elf::types::SHF_ALLOC.0) != 0 {
            cpu.memory.allocate(s.shdr.addr, s.shdr.size, &s.data);
            println!("  * {:}", s.shdr.name);
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

    //Used to set breakpoint
    //let symbols =  elf_reader::read_symbols_list(symtab, strtab);
    
    cpu.execute(entrypoint);
}


fn main(){
    let f = PathBuf::from("test/riscv-tests/rv64ui-p-addiw");
    exec_elf(&f);
    
}