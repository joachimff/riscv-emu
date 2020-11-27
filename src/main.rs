pub mod memory;
pub mod elf_reader;

extern crate elf;

use std::fmt;
use std::path::PathBuf;
use std::collections::HashMap;
use std::fs;

use memory::{Memory, MEMORY_SIZE};

#[derive(Debug)]
pub struct RType{
    funct7: u8,
    rs2: usize,
    rs1: usize,
    funct3: u8,
    rd: usize,
    opcode: u8
}

impl From<u32> for RType{
    fn from(instruction:u32) -> Self{
        RType{
            funct7: (instruction >> 25) as u8,
            rs2:    ((instruction >> 20) & 0b11111) as usize,
            rs1:    ((instruction >> 15) & 0b11111) as usize,
            funct3: ((instruction >> 12) & 0b111) as u8,
            rd:     ((instruction >> 7) & 0b11111) as usize,
            opcode: (instruction & 0b111_1111) as u8
        }
    }
}

#[derive(Debug)]
pub struct IType{
    imm: i32,
    rs1: usize,
    funct3: u8,
    rd: usize,
    opcode: u8
}

impl From<u32> for IType {
    fn from(instruction: u32) -> Self {
        IType{
            imm:    (instruction as i32) >> 20,
            rs1:    ((instruction >> 15)  & 0b11111) as usize,
            funct3: ((instruction >> 12)  & 0b111) as u8,
            rd:     ((instruction >> 7)   & 0b11111) as usize,
            opcode: (instruction          & 0b1111_111) as u8,
        }
    }
}

#[derive(Debug)]
pub struct UType{
    imm: u32,
    rd: usize,
    opcode: u8
}

impl From<u32> for UType{
    fn from(instruction:u32) -> Self{
        UType{
            imm: ((instruction >> 12) & 0b1111_1111_1111_1111_1111) as u32,
            rd:     ((instruction >> 7) & 0b11111) as usize,
            opcode: (instruction & 0b111_1111) as u8
        }
    }
}

#[derive(Debug)]
pub struct JType{
    imm: i32,
    rd: usize,
    opcode: u8
}

impl From<u32> for JType{
    fn from(instruction:u32) -> Self{
        JType{
            imm: (((instruction >> 21) & 0b1_1111_1111) << 1 |
                ((instruction >> 20) & 0b1) << 11 |
                ((instruction >> 12) & 0b1111_1111) << 12) as i32 |
                //Sign extended
                (((instruction as i32) >> 30) << 20) as i32,
            rd:     ((instruction >> 7) & 0b1_1111) as usize,
            opcode: (instruction & 0b111_1111) as u8
        }
    }
}

#[derive(Debug)]
pub struct BType{
    imm: i32,
    rs1: usize,
    rs2: usize,
    func3: u8,
    opcode: u8
}

impl From<u32> for BType{
    fn from(instruction:u32) -> Self{
        BType{
            imm: (((instruction >> 8) & 0b1111) << 1 |
                ((instruction >> 25) & 0b11_1111) << 5 |
                ((instruction >> 7) & 0b1) << 11) as i32 |
                //Sign extended
                (((instruction as i32) >> 31) << 12),
            rs1: ((instruction >> 15) & 0b11111) as usize,
            rs2: ((instruction >> 20) & 0b11111) as usize,
            func3: ((instruction >> 12) & 0b111) as u8,
            opcode: (instruction  & 0b111_1111) as u8,
        }
    }
}

#[derive(Debug)]
pub struct SType{
    imm: i32,
    rs1: usize,
    rs2: usize,
    funct3: u8,
    opcode: u8
}

impl From<u32> for SType{
    fn from(instruction:u32) -> Self{
        SType{
            imm: ((instruction >> 7) & 0b1_1111) as i32|
                //Sign extended
                (((instruction as i32) >> 25) << 5),
            rs1: ((instruction >> 15) & 0b11111) as usize,
            rs2: ((instruction >> 20) & 0b11111) as usize,
            funct3: ((instruction >> 12) & 0b111) as u8,
            opcode: (instruction  & 0b111_1111) as u8,
        }
    }
}

/// Memory management
// Hold the registers 
#[derive(Debug)]
struct Registers{
    common: [i32; 32],
    pc: i32,
}

impl Registers {
    pub fn new() -> Registers {
        let mut common = [0; 32];
        common[2] = MEMORY_SIZE as i32;

        Registers{
            common: common,
            pc: 0
        }
    }
}

struct CPU{
    memory: Memory,
    registers: Registers,
    exit: bool, //Exit the execution
    breakpoints: HashMap<usize, fn(&mut CPU)>,
}

impl CPU{
    //Return a new CPU with null memory
    pub fn new() -> CPU {
        CPU{
            memory: Memory::new(),
            registers: Registers::new(),
            exit: false,
            breakpoints: HashMap::new(),
        }
    }

    //Execute one instruction
    pub fn exec_instruction(&mut self, instr: u32){
        let opcode = instr & 0b111_1111;
        let mut should_incr_pc = true;

        match opcode{
            //LUI
            0b011_0111 => {
                let instr = UType::from(instr);
                self.registers.common[instr.rd] = (instr.imm as i32) << 12; 
            },
            //AUIPC
            0b001_0111 => {
                let instr = UType::from(instr);
                println!("{:?}", instr);
                let addr = (instr.imm << 12) as i32;
                println!("{:X?}", addr);
                self.registers.common[instr.rd] = self.registers.pc.wrapping_add(addr);
            },
            //JAL
            0b110_1111 => {
                let instr = JType::from(instr);
                println!("JAL: {:?}", instr);
                should_incr_pc = false;

                //plain unconditionnal jump are encoded with rd=x0
                if instr.rd != 0{
                    self.registers.common[instr.rd] = self.registers.pc.wrapping_add(4);
                }
                self.registers.pc = self.registers.pc.wrapping_add(instr.imm);
            },
            //JALR
            0b110_0111 => {
                let instr = IType::from(instr);
                should_incr_pc = false;

                let target = self.registers.common[instr.rs1].wrapping_add(instr.imm);
                
                if instr.rd != 0{
                    self.registers.common[instr.rd] = self.registers.pc.wrapping_add(4);
                }
                self.registers.pc = target;
            },
            //Conditional Branches
            0b110_0011 => {
                let instr = BType::from(instr);
                //println!("{:?}", instr);

                match instr.func3 {
                    //BEQ
                    0b000 => {
                        if self.registers.common[instr.rs1] == self.registers.common[instr.rs2]{
                            self.registers.pc = self.registers.pc.wrapping_add(instr.imm);
                            should_incr_pc = false;
                        }
                    },
                    //BNE
                    0b001 => {
                        if self.registers.common[instr.rs1] != self.registers.common[instr.rs2]{
                            self.registers.pc = self.registers.pc.wrapping_add(instr.imm);
                            should_incr_pc = false;
                        }
                    },
                    //BLT
                    0b100 => {
                        if self.registers.common[instr.rs1] < self.registers.common[instr.rs2]{
                            self.registers.pc = self.registers.pc.wrapping_add(instr.imm);
                            should_incr_pc = false;
                        }
                    },
                    //BGE
                    0b101 => {
                        if self.registers.common[instr.rs1] >= self.registers.common[instr.rs2]{
                            self.registers.pc = self.registers.pc.wrapping_add(instr.imm);
                            should_incr_pc = false;
                        }
                    }
                    //BLTU
                    0b110 => {
                        if (self.registers.common[instr.rs1] as u32) < (self.registers.common[instr.rs2] as u32){
                            self.registers.pc = self.registers.pc.wrapping_add(instr.imm);
                            should_incr_pc = false;
                        }
                    }
                    //BGEU
                    0b111 => {
                        if (self.registers.common[instr.rs1] as u32) >= (self.registers.common[instr.rs2] as u32){
                            self.registers.pc = self.registers.pc.wrapping_add(instr.imm);
                            should_incr_pc = false;
                        }
                    },
                    _ => {unreachable!()},
                }
            },
            //LOAD 
            0b000_0011 => {
                let instr = IType::from(instr);
                let addr = self.registers.common[instr.rs1].wrapping_add(instr.imm) as usize;

                println!("==>{:?}, addr:{:#X}", instr, addr);
                match instr.funct3{
                    //LB
                    0b000 => {
                        let mut buf = [0u8; 1];
                        self.memory.read(addr, &mut buf);
    
                        self.registers.common[instr.rd] = i8::from_le_bytes(buf) as i32;
                    },
                    //LH
                    0b001 => {
                        let mut buf = [0u8; 2];
                        self.memory.read(addr, &mut buf);
    
                        self.registers.common[instr.rd] = i16::from_le_bytes(buf) as i32;
                    },
                    //LW
                    0b010 => {
                        let mut buf = [0u8; 4];
                        self.memory.read(addr, &mut buf);
    
                        self.registers.common[instr.rd] = i32::from_le_bytes(buf);
                    },
                    //LBU
                    0b100 => {
                        let mut buf = [0u8; 1];
                        self.memory.read(addr, &mut buf);
    
                        self.registers.common[instr.rd] = u8::from_le_bytes(buf) as i32;
                    },
                    //LHU
                    0b101 => {
                        let mut buf = [0u8; 2];
                        self.memory.read(addr, &mut buf);
    
                        self.registers.common[instr.rd] = u16::from_le_bytes(buf) as i32;
                    },
                    _ => {unreachable!()}
                }
            },
            //STORE
            0b010_0011 => {
                let instr = SType::from(instr);
                let addr = self.registers.common[instr.rs1].wrapping_add(instr.imm) as usize;

                //println!("==>{:?}, addr:{:#x}", instr, addr);
                match instr.funct3 {
                    //SB
                    0b000 => { self.memory.write(addr, &(self.registers.common[instr.rs2] as u8).to_le_bytes()); },
                    //SH
                    0b001 => { self.memory.write(addr, &(self.registers.common[instr.rs2] as u16).to_le_bytes()); },
                    //SW
                    0b010 => { self.memory.write(addr, &self.registers.common[instr.rs2].to_le_bytes()); },
                    _ => { unreachable!(); }
                }
            },
            //Integer register-immediate instructions
            0b001_0011 => {
                let instr = IType::from(instr);
                println!("==>{:?}", instr);

                match instr.funct3{
                    //ADDI
                    0b000 => { 
                        self.registers.common[instr.rd] = self.registers.common[instr.rs1].wrapping_add(instr.imm); 
                    },

                    //SLTI
                    0b010 => {
                        if self.registers.common[instr.rs1] < instr.imm{
                            self.registers.common[instr.rd] = 1;
                        }
                        else{
                            self.registers.common[instr.rd] = 0;
                        }
                    },
                    //SLTIU
                    0b011 => {
                        //Special case
                        if instr.imm == 1{
                            self.registers.common[instr.rd] = 
                                if self.registers.common[instr.rs1] == 0 {1} else {0};
                        }

                        else{
                            if (self.registers.common[instr.rs1] as u32) < (instr.imm as u32){
                                self.registers.common[instr.rd] = 1;
                            }
                            else{
                                self.registers.common[instr.rd] = 0;
                            }
                        }
                    },
                    //XORI
                    0b100 => { self.registers.common[instr.rd] = self.registers.common[instr.rs1] ^ instr.imm; },
                    //ORI
                    0b110 => { self.registers.common[instr.rd] = self.registers.common[instr.rs1] | instr.imm; },
                    //ANDI
                    0b111 => { self.registers.common[instr.rd] = self.registers.common[instr.rs1] & instr.imm; },
                    
                    //These three instructions have a special encoding
                    //SLLI
                    0b001 => {
                        let shamt = instr.imm & 0b1_1111;
                        self.registers.common[instr.rd] = self.registers.common[instr.rs1] << shamt; 
                    }
                    0b101 => {
                        let shamt = instr.imm & 0b1_1111;
                        
                        //SRAI
                        if ((instr.imm >> 10) & 0b1) == 1{
                            self.registers.common[instr.rd] = self.registers.common[instr.rs1] >> shamt;
                        }
                        //SRLI
                        else{
                            self.registers.common[instr.rd] = ((self.registers.common[instr.rs1] as u32) >> shamt as u32) as i32; 
                        }
                    },
                    _ => { unreachable!() }
                }
            },
            0b011_0011 => {
                let instr = RType::from(instr);

                match instr.funct3 {
                    0b000 =>{
                        //ADD
                        if instr.funct7 == 0{
                            self.registers.common[instr.rd] =
                                self.registers.common[instr.rs1].wrapping_add(self.registers.common[instr.rs2]);
                        }
                        //SUB
                        else{
                            self.registers.common[instr.rd] =
                                self.registers.common[instr.rs1].wrapping_sub(self.registers.common[instr.rs2]);
                        } 
                    },
                    //SLL
                    0b001 => {
                        self.registers.common[instr.rd] =
                            self.registers.common[instr.rs1] << (self.registers.common[instr.rs2] & 0b1_1111);
                    },
                    //SLT
                    0b010 => {
                        self.registers.common[instr.rd] = if self.registers.common[instr.rs1] < self.registers.common[instr.rs2] {1} else {0};
                    },
                    //SLTU
                    0b011 => {
                        //Special case
                        if instr.rs1 == 0{
                            self.registers.common[instr.rd] =
                                if self.registers.common[instr.rs2] != 0 {1} else {0};
                        }
                        else {
                            self.registers.common[instr.rd] = 
                                if (self.registers.common[instr.rs1] as u32) < (self.registers.common[instr.rs2] as u32) {1} else {0};
                        }
                    },
                    //XOR
                    0b100 => {
                        self.registers.common[instr.rd] = 
                            self.registers.common[instr.rs1] ^ self.registers.common[instr.rs2]; 
                    },
                    0b101 => {
                        //SRL
                        if instr.funct7 == 0{
                            self.registers.common[instr.rd] =
                                ((self.registers.common[instr.rs1] as u32) >> ((self.registers.common[instr.rs2] as u32) & 0b1_1111)) as i32;
                        }
                        //SRA
                        else {
                            self.registers.common[instr.rd] =
                                self.registers.common[instr.rs1] >> (self.registers.common[instr.rs2] & 0b1_1111);
                        }
                    }
                    //OR
                    0b110 => {
                        self.registers.common[instr.rd] =
                            self.registers.common[instr.rs1] | self.registers.common[instr.rs2];
                    }
                    //AND
                    0b111 => {
                        self.registers.common[instr.rd] = 
                            self.registers.common[instr.rs1] & self.registers.common[instr.rs2];
                    },
                    _ => { unreachable!() }
                }
            },
            //FENCE
            0b000_1111 => { panic!("FENCE NYI"); },
            //ECALL EBREAK
            0b111_0011 => { panic!("ECALL/EBREAK NYI, pc={:X?}", self.registers.pc); },
            
            _ => unreachable!()
        }

        if should_incr_pc {
            self.registers.pc = self.registers.pc.wrapping_add(4);
        }
    }

    fn execute(&mut self, entrypoint: usize){
        self.registers.pc = entrypoint as i32;

        loop {
            let mut instr = [0 as u8; 4];
            self.memory.read(self.registers.pc as usize, &mut instr);

            let instr = u32::from_le_bytes(instr);

            println!("[PC:{:#8X}]=>{:#x}", self.registers.pc, instr);

            if let Some(b) = self.breakpoints.get(&(self.registers.pc as usize & 0xFFFF_FFFF)){
                println!("<========>BREAKPOINT HIT<=========>");
                b(self);
            }

            if self.exit{
                break;
            }

            self.exec_instruction(instr);
            println!("{:?}", self);
            //println!();
        }
    }

    fn set_breakpoint(&mut self, at: usize, handler: fn(&mut CPU)){
        self.breakpoints.insert(at, handler);
    }
}

fn bp_end_of_test(cpu: &mut CPU){
    println!("Tests OK");
    cpu.exit = true;
}

fn bp_test_error(cpu: &mut CPU){
    println!("{:?}", cpu);
    panic!("Fail on test: {:#8?}", cpu.registers.common[3]);
}

impl fmt::Debug for CPU{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "PC: {:#8X}", self.registers.pc);
        writeln!(f, "-----------------------------------------------Registers------------------------------------------------");
    
        writeln!(f, "x0:{:#8X}, ra:{:#8X},  sp:{:#8X},  gp:{:#8X}, tp:{:#8X}, t0:{:#8X}, t1:{:#8X}, t2:{:#8X}, ", 
            self.registers.common[0], self.registers.common[1], self.registers.common[2], self.registers.common[3],
            self.registers.common[4], self.registers.common[5], self.registers.common[6], self.registers.common[7]);
        writeln!(f, "s0:{:#8X}, s1:{:#8X},  a0:{:#8X},  a1:{:#8X}, a2:{:#8X}, a3:{:#8X}, a4:{:#8X}, a5:{:#8X}, ", 
            self.registers.common[8], self.registers.common[9], self.registers.common[10], self.registers.common[11],
            self.registers.common[12], self.registers.common[13], self.registers.common[14], self.registers.common[15]);
        writeln!(f, "a6:{:#8X}, a7:{:#8X},  s2:{:#8X},  s3:{:#8X}, s4:{:#8X}, s5:{:#8X}, s6:{:#8X}, s7:{:#8X}, ", 
            self.registers.common[16], self.registers.common[17], self.registers.common[18], self.registers.common[19],
            self.registers.common[20], self.registers.common[21], self.registers.common[22], self.registers.common[23]);
        writeln!(f, "s8:{:#8X}, s9:{:#8X}, s10:{:#8X}, s11:{:#8X}, t3:{:#8X}, t4:{:#8X}, t5:{:#8X}, t6:{:#8X}, ", 
            self.registers.common[24], self.registers.common[25], self.registers.common[26], self.registers.common[27],
            self.registers.common[28], self.registers.common[29], self.registers.common[30], self.registers.common[31]);
        write!(f, "{:?}", self.memory)
    }
}

fn start_elf(path: &PathBuf) -> Result<(), Box<dyn std::error::Error + 'static>>{
    let mut cpu: CPU = CPU::new();

    //Read instructions
    //let path: PathBuf = From::from(path);
    let elf = match elf::File::open_path(&path) {
        Ok(f) => f,
        Err(e) => panic!("Error {:?}", e)
    };

    let entry_point = elf.ehdr.entry;
    println!("Entry point: {:#X}", entry_point);

    let mut symtab: Option<elf::Section> = None;
    let mut strtab: Option<elf::Section> = None;

    for s in elf.sections{
        match s.shdr.name.as_ref() {
            ".text.init" => {
                cpu.memory.allocate(&s.data);
            },
            ".symtab" => {
                symtab = Some(s); 
            },
            ".strtab" => {
                strtab = Some(s);
            }
            _ => {},
        }
    }

    match symtab{
        Some(symtab) => {
            match strtab{
                Some(strtab) => {
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
                },
                _ => {}
            }
        },
        _ => {}
    }

    Ok(())
}

fn main(){
    let paths = fs::read_dir("test/riscv-tests/").unwrap();
    let mut i =0;
    for p in paths{
        println!("Executing test: {:?}({:})", p, i);
        start_elf(&p.unwrap().path());

        i += 1;
    }
    println!("{:} tests passed", i);
}
