extern crate elf;

use std::fmt;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;
use std::rc::Rc;
use std::cell::RefCell;

use super::memory::{Memory, STACK_SIZE};
use super::instr_type::{*};
use super::fuzzer::{Fuzzer};

/// Memory management
// Hold the registers 
#[derive(Debug, Clone)]
pub struct Registers{
    pub common: [u64; 32],
    pub pc: u64,
}

impl Registers {
    pub fn new() -> Registers {
        let mut common = [0; 32];
        common[2] = STACK_SIZE as u64;

        Registers{
            common: common,
            pc: 0
        }
    }
}

pub struct CPU{
    pub memory: Memory,
    pub registers: Registers,
    pub exit: bool, //Exit the execution
    pub breakpoints: HashMap<u64, fn(&mut CPU)>,
    pub redirect_stdout: bool,

    /// Everytime an instruction is executed its address is added to this set
    /// this slow down the execution a lot as a set lookup is required at each
    /// instruction execution
    pub coverage_enabled: bool,
    pub coverage: HashSet<u64>, 

    /// This is from this state that the delta for dirty pages will be calculed
    /// at that time only one snapshot is supported, a call must be made to
    /// save_as_initial_state before usage
    pub nbr_exec: u64,
    saved_state: Option<CpuSnapshot>,
}

/// This structure is used to store the state of memory and registers
/// at a given time
struct CpuSnapshot{
    pub registers: Registers,
    pub coverage: Option<HashSet<u64>>,
}

impl CPU{
    //Return a new CPU
    pub fn new(coverage_enabled: bool) -> CPU {
        CPU{
            memory: Memory::new(),
            registers: Registers::new(),
            exit: false,
            breakpoints: HashMap::new(),
            redirect_stdout: true,
            coverage_enabled: coverage_enabled,
            coverage: HashSet::new(),
            saved_state: None,
            nbr_exec: 0,
        }
    }

    //Execute one instruction
    fn exec_instruction(&mut self, instr: u32, fuzzer: Rc<RefCell<Fuzzer>>){
        //The hash of the origin and the destination of a branch is recorded for code coverage calculation
        let mut branch_dest = 0;

        let opcode = instr & 0b111_1111;
        let mut take_branch = false;

        match opcode{
            //LUI
            0b011_0111 => {
                let instr = UType::from(instr);
                self.registers.common[instr.rd] = (instr.imm << 12) as i32 as i64 as u64; 
            },
            //AUIPC
            0b001_0111 => {
                let instr = UType::from(instr);

                let addr = (instr.imm << 12) as i32 as i64 as u64;
                self.registers.common[instr.rd] = self.registers.pc.wrapping_add(addr);
            },
            //JAL
            0b110_1111 => {
                let instr = JType::from(instr);
                take_branch = true;

                //plain unconditionnal jump are encoded with rd=x0
                if instr.rd != 0{
                    self.registers.common[instr.rd] = self.registers.pc.wrapping_add(4);
                }
                branch_dest = self.registers.pc.wrapping_add(instr.imm as u64);
            },
            //JALR
            0b110_0111 => {
                let instr = IType::from(instr);
                take_branch = true;

                branch_dest = self.registers.common[instr.rs1].wrapping_add(instr.imm as u64);
                
                if instr.rd != 0{
                    self.registers.common[instr.rd] = self.registers.pc.wrapping_add(4);
                }
            },
            //Conditional Branches
            0b110_0011 => {
                let instr = BType::from(instr);
                //println!("{:?}", instr);

                match instr.func3 {
                    //BEQ
                    0b000 => {
                        if self.registers.common[instr.rs1] == self.registers.common[instr.rs2]{
                            branch_dest = self.registers.pc.wrapping_add(instr.imm as u64);
                            take_branch = true;
                        }
                    },
                    //BNE
                    0b001 => {
                        if self.registers.common[instr.rs1] != self.registers.common[instr.rs2]{
                            branch_dest = self.registers.pc.wrapping_add(instr.imm as u64);
                            take_branch = true;
                        }
                    },
                    //BLT
                    0b100 => {
                        if (self.registers.common[instr.rs1] as i32) < (self.registers.common[instr.rs2] as i32){
                            branch_dest = self.registers.pc.wrapping_add(instr.imm as u64);
                            take_branch = true;
                        }
                    },
                    //BGE
                    0b101 => {
                        if (self.registers.common[instr.rs1] as i32) >= (self.registers.common[instr.rs2] as i32){
                            branch_dest = self.registers.pc.wrapping_add(instr.imm as u64);
                            take_branch = true;
                        }
                    }
                    //BLTU
                    0b110 => {
                        if (self.registers.common[instr.rs1] as u32) < (self.registers.common[instr.rs2] as u32){
                            branch_dest = self.registers.pc.wrapping_add(instr.imm as u64);
                            take_branch = true;
                        }
                    }
                    //BGEU
                    0b111 => {
                        if (self.registers.common[instr.rs1] as u32) >= (self.registers.common[instr.rs2] as u32){
                            branch_dest = self.registers.pc.wrapping_add(instr.imm as u64);
                            take_branch = true;
                        }
                    },
                    _ => {unreachable!()},
                }

            },
            //LOAD 
            0b000_0011 => {
                let instr = IType::from(instr);
                let addr = self.registers.common[instr.rs1].wrapping_add(instr.imm as u64);
                //println!("{:X?} => addr:{:X?}", instr, addr);
                match instr.funct3{
                    //LB
                    0b000 => {
                        let mut buf = [0u8; 1];
                        self.memory.read(addr, &mut buf);
    
                        self.registers.common[instr.rd] = i8::from_le_bytes(buf) as i64 as u64;
                    },
                    //LH
                    0b001 => {
                        let mut buf = [0u8; 2];
                        self.memory.read(addr, &mut buf);
    
                        self.registers.common[instr.rd] = i16::from_le_bytes(buf) as i64 as u64;
                    },
                    //LW
                    0b010 => {
                        let mut buf = [0u8; 4];
                        self.memory.read(addr, &mut buf);
    
                        self.registers.common[instr.rd] = i32::from_le_bytes(buf) as i64 as u64;
                    },
                    //LBU
                    0b100 => {
                        let mut buf = [0u8; 1];
                        self.memory.read(addr, &mut buf);
    
                        self.registers.common[instr.rd] = u8::from_le_bytes(buf) as u64;
                    },
                    //LHU
                    0b101 => {
                        let mut buf = [0u8; 2];
                        self.memory.read(addr, &mut buf);
    
                        self.registers.common[instr.rd] = u16::from_le_bytes(buf) as u64;
                    },
                    //LD
                    0b011 => {
                        let mut buf = [0u8; 8];
                        self.memory.read(addr, &mut buf);

                        self.registers.common[instr.rd] = u64::from_le_bytes(buf);

                    }
                    //LWU
                    0b110 => {
                        let mut buf = [0u8; 4];
                        self.memory.read(addr, &mut buf);
    
                        self.registers.common[instr.rd] = u32::from_le_bytes(buf) as u64;
                    }
                    _ => {unreachable!()}
                }
            },
            //STORE
            0b010_0011 => {
                let instr = SType::from(instr);
                let addr = self.registers.common[instr.rs1].wrapping_add(instr.imm as u64);

                //println!("==>{:?}, addr:{:#x}, base{:#X}", instr, addr, self.registers.common[instr.rs1]);
                match instr.funct3 {
                    //SB
                    0b000 => { self.memory.write(addr, &(self.registers.common[instr.rs2] as u8).to_le_bytes()); },
                    //SH
                    0b001 => { self.memory.write(addr, &(self.registers.common[instr.rs2] as u16).to_le_bytes()); },
                    //SW
                    0b010 => { self.memory.write(addr, &(self.registers.common[instr.rs2] as u32).to_le_bytes()); },
                    //SD
                    0b011 => { self.memory.write(addr, &self.registers.common[instr.rs2].to_le_bytes()); },
                    _ => { unreachable!(); }
                }
            },
            //Integer register-immediate instructions
            0b001_0011 => {
                let instr = IType::from(instr);
                //println!("==>{:?}", instr);

                match instr.funct3{
                    //ADDI
                    0b000 => { 
                        self.registers.common[instr.rd] = self.registers.common[instr.rs1].wrapping_add(instr.imm as u64); 
                    },

                    //SLTI
                    0b010 => {
                        if (self.registers.common[instr.rs1] as i32) < instr.imm{
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
                    0b100 => { self.registers.common[instr.rd] = self.registers.common[instr.rs1] ^ (instr.imm as u64); },
                    //ORI
                    0b110 => { self.registers.common[instr.rd] = self.registers.common[instr.rs1] | (instr.imm as u64); },
                    //ANDI
                    0b111 => { self.registers.common[instr.rd] = self.registers.common[instr.rs1] & (instr.imm as u64); },
                    
                    //These three instructions have a special encoding
                    //SLLI
                    0b001 => {
                        let shamt = instr.imm & 0b11_1111;
                        self.registers.common[instr.rd] = self.registers.common[instr.rs1] << shamt; 
                    }
                    0b101 => {
                        let shamt = instr.imm & 0b11_1111;
                        
                        //SRAI
                        if ((instr.imm >> 10) & 0b1) == 1{
                            self.registers.common[instr.rd] = ((self.registers.common[instr.rs1] as i64) >> shamt) as u64;
                        }
                        //SRLI
                        else{
                            self.registers.common[instr.rd] = self.registers.common[instr.rs1] >> shamt; 
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
                            self.registers.common[instr.rs1] << (self.registers.common[instr.rs2] & 0b11_1111);
                    },
                    //SLT
                    0b010 => {
                        self.registers.common[instr.rd] = 
                            if (self.registers.common[instr.rs1] as i32) < (self.registers.common[instr.rs2] as i32) {1} else {0};
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
                                self.registers.common[instr.rs1] >> (self.registers.common[instr.rs2] & 0b11_1111);
                        }
                        //SRA
                        else {
                            self.registers.common[instr.rd] =
                                ((self.registers.common[instr.rs1] as i64) >> (self.registers.common[instr.rs2] & 0b11_1111)) as u64;
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
            0b111_0011 => { 
                fuzzer.borrow_mut().syscall(self);
            },
            
            //RV64I specific instructions
            0b001_1011 =>{
                let instr = IType::from(instr);

                match instr.funct3{
                    //ADDIW
                    0b000 => {  
                        self.registers.common[instr.rd] = (self.registers.common[instr.rs1] as i32).wrapping_add(instr.imm) as i64 as u64;
                    },
                    //SLLIW
                    0b001 => {
                        let shamt = instr.imm & 0b11_1111;
                        self.registers.common[instr.rd] = (self.registers.common[instr.rs1] << shamt) as i32 as i64 as u64; 
                    }
                    //SRLIW / SRAIW
                    0b101 => {
                        let shamt = instr.imm & 0b11_1111;

                        //SRAIW
                        if ((instr.imm >> 10) & 0b1) == 1{
                            self.registers.common[instr.rd] = ((self.registers.common[instr.rs1] as i32) >> shamt) as i32 as u64;
                        }
                        //SRLIW
                        else{
                            self.registers.common[instr.rd] = ((self.registers.common[instr.rs1] as u32) >> shamt) as i32 as u64; 
                        }
                    },
                    _ => {unreachable!()}
                }
            },
            0b011_1011 =>{
                let instr = RType::from(instr);

                match instr.funct3{
                    0b000 =>{
                        //ADDW
                        if instr.funct7 == 0{
                            self.registers.common[instr.rd] =
                                (self.registers.common[instr.rs1] as i32).wrapping_add(self.registers.common[instr.rs2] as i32) as i64 as u64;
                        }
                        //SUBW
                        else{
                            self.registers.common[instr.rd] =
                                (self.registers.common[instr.rs1] as i32).wrapping_sub(self.registers.common[instr.rs2] as i32) as i64 as u64;
                        }
                    },
                    //SLLW
                    0b001 => {
                        self.registers.common[instr.rd] =
                            ((self.registers.common[instr.rs1] as i32) << (self.registers.common[instr.rs2] & 0b1_1111)) as i64 as u64;
                    },
                    0b101 => {
                        //SRLW
                        if instr.funct7 == 0{
                            self.registers.common[instr.rd] =
                                ((self.registers.common[instr.rs1] as u32) >> (self.registers.common[instr.rs2] & 0b1_1111)) as i32 as u64;
                        }
                        //SRAW
                        else{
                            self.registers.common[instr.rd] =
                                ((self.registers.common[instr.rs1] as i32) >> (self.registers.common[instr.rs2] & 0b1_1111)) as i64 as u64;
                        }
                    },
                    _ => {unreachable!()}
                }
            }

            _ => unreachable!("{:b}", opcode)
        }

        //We branched
        if take_branch{
            if branch_dest == 0{
                panic!("Branching to a non set destination");
            }
            //Record the xor of the origin and the destination
            self.coverage.insert(self.registers.pc ^ branch_dest);
            self.registers.pc = branch_dest;
        }
        else {
            self.registers.pc = self.registers.pc.wrapping_add(4);
        }
    }

    /// Store a copy of the current CPU state
    pub fn save_as_initial_state(&mut self){
        self.saved_state = Some(CpuSnapshot{
            registers: self.registers.clone(),
            coverage:{
                if self.coverage_enabled{ Some(self.coverage.clone()) }
                else{ None }
            }
        });
        self.memory.save_state();
    }

    /// Reset to the state snapshot saved thourgh save_as_initial_state
    /// here only dirty pages are reseted, returns the coverage of the last run 
    pub fn reset_to_initial_state(&mut self) -> HashSet<u64>{
        let initial_state = self.saved_state.as_ref()
            .expect("Trying to reset but no initial state has been saved");

        self.registers = initial_state.registers.clone();
        self.memory.reset_to_saved_state();

        self.nbr_exec = self.nbr_exec.wrapping_add(1);

        self.coverage.clone()
    }

    pub fn execute(&mut self, entrypoint: u64, fuzzer: Rc<RefCell<Fuzzer>>){
        self.registers.pc = entrypoint;
        let start_t = Instant::now();

        loop {
            if let Some(b) = self.breakpoints.get(&(self.registers.pc)){
                //println!("<========>BREAKPOINT HIT:{:X}<=========>", self.registers.pc);
                b(self);
            }

            if self.exit{
                if self.coverage_enabled{
                    println!("CC: {:}", self.coverage.len());
                }
                
                println!("Time elapsed (ms): {:}", start_t.elapsed().as_millis());
                break;
            }

            let mut instr = [0 as u8; 4];
            self.memory.read(self.registers.pc, &mut instr);

            let instr = u32::from_le_bytes(instr);

            //println!("{:08X}", self.registers.pc);
            self.exec_instruction(instr, Rc::clone(&fuzzer));
            //println!("{:?}", self);
        }
    }

    pub fn set_breakpoint(&mut self, at: u64, handler: fn(&mut CPU)){
        self.breakpoints.insert(at, handler);
    }
}

impl fmt::Debug for CPU{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "PC: {:#8X}", self.registers.pc);
        writeln!(f, "-----------------------------------------------Registers------------------------------------------------");
    
        writeln!(f, "x0:{:#8X}, ra:{:#8X},  sp:{:#8X},  gp:{:#8X}, tp:{:#8X}, t0:{:#8X}, t1:{:#8X}, t2:{:#8X}, ", 
            self.registers.common[0], self.registers.common[1], self.registers.common[2], self.registers.common[3],
            self.registers.common[4], self.registers.common[5], self.registers.common[6], self.registers.common[7])
            .expect("");
        writeln!(f, "s0:{:#8X}, s1:{:#8X},  a0:{:#8X},  a1:{:#8X}, a2:{:#8X}, a3:{:#8X}, a4:{:#8X}, a5:{:#8X}, ", 
            self.registers.common[8], self.registers.common[9], self.registers.common[10], self.registers.common[11],
            self.registers.common[12], self.registers.common[13], self.registers.common[14], self.registers.common[15])
            .expect("");
        writeln!(f, "a6:{:#8X}, a7:{:#8X},  s2:{:#8X},  s3:{:#8X}, s4:{:#8X}, s5:{:#8X}, s6:{:#8X}, s7:{:#8X}, ", 
            self.registers.common[16], self.registers.common[17], self.registers.common[18], self.registers.common[19],
            self.registers.common[20], self.registers.common[21], self.registers.common[22], self.registers.common[23])
            .expect("");
        writeln!(f, "s8:{:#8X}, s9:{:#8X}, s10:{:#8X}, s11:{:#8X}, t3:{:#8X}, t4:{:#8X}, t5:{:#8X}, t6:{:#8X}, ", 
            self.registers.common[24], self.registers.common[25], self.registers.common[26], self.registers.common[27],
            self.registers.common[28], self.registers.common[29], self.registers.common[30], self.registers.common[31])
            .expect("");
        write!(f, "{:?}", self.memory)
    }
}