#[derive(Debug)]
pub struct RType{
    funct7: u8,
    rs2: usize,
    rs1: usize,
    funct3: u8,
    rd: usize,
    opcode: u8
}

impl RType{
    pub fn parse(instruction:u32) -> RType{
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
            imm:    (instruction >> 20) as i32,
            rs1:    ((instruction >> 15)  & 0b11111) as usize,
            funct3: ((instruction >> 12) & 0b111) as u8,
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

impl UType{
    pub fn parse(instruction:u32) -> UType{
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

impl JType{
    pub fn parse(instruction:u32) -> JType{
        JType{
            imm: (((instruction >> 21) & 0b1_1111_1111) << 1 |
                ((instruction >> 20) & 0b1) << 11 |
                ((instruction >> 12) & 0b1111_1111) << 12 |
                //Sign extended
                (instruction >> 30) << 20) as i32,
            rd:     ((instruction >> 7) & 0b11111) as usize,
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
                ((instruction >> 7) & 0b1) << 11 |
                //Sign extended
                (instruction >> 30) << 12) as i32,
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
            imm: (((instruction >> 7) & 0b11111) << 1 |
                //Sign extended
                (instruction >> 25) << 5) as i32,
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
        Registers{
            common: [0; 32],
            pc: 0
        }
    }
}

//Hold the memory
#[derive(Debug)]
struct Memory {
    data: [u8; 2048]
}

//Manage memory
impl Memory{
    //Return a new memory with all null data
    pub fn new() -> Memory {
        Memory {
            data: [0; 2048]
        }
    }

    pub fn read(&self, at: usize, buf: &mut [u8]) {
        buf.copy_from_slice(&self.data[at..buf.len()]);
    }

    pub fn write(&mut self, at: usize, buf: &[u8]){
        self.data[at..buf.len()].copy_from_slice(buf);
    }
}

#[derive(Debug)]
struct CPU{
    memory: Memory,
    registers: Registers,
}

impl CPU{
    //Return a new CPU with null memory
    pub fn new() -> CPU {
        CPU{
            memory: Memory::new(),
            registers: Registers::new(),
        }
    }

    //Execute one instruction RType
    pub fn exec_itype(&mut self, instr: u32){
        let instr = IType::from(instr);

        println!("New IType instruction: {:x?}", instr);
        match instr.opcode{
            0b001_0011 => {
                match instr.funct3{
                    //ADDI
                    0b000 => {
                        //ADDI x0, x0, 0 <=> NOP

                        self.registers.common[instr.rd] = self.registers.common[instr.rs1] + instr.imm;
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
                        if instr.imm == 0{
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
                    0b100 => {
                        self.registers.common[instr.rd] = self.registers.common[instr.rs1] ^ instr.imm; 
                    },
                    //ORI
                    0b110 => {
                        self.registers.common[instr.rd] = self.registers.common[instr.rs1] | instr.imm; 
                    },
                    //ANDI
                    0b111 => {
                        self.registers.common[instr.rd] = self.registers.common[instr.rs1] & instr.imm; 
                    },
                    _ => {
                        panic!("Invalid funct3({:#b}) value for opcode {:#b}", instr.funct3, instr.opcode)
                    }
                }
            },
            0b1100111 => {
                match instr.funct3 {
                    //JALR
                    0b000 => {
                        self.registers.common[2] = self.registers.pc.wrapping_add(4);
                        self.registers.pc = (instr.rs1 as i32).wrapping_add(instr.imm);
                    },
                    _ => { panic!("Impossible"); }
                }
            }
            _ => {
                panic!("NYI OpCode: {:#b}", instr.opcode)
            }
        }
    }

    pub fn exec_rtype(&mut self, instr: u32){
        let instr = RType::parse(instr);

        println!("New RType instruction: {:x?}", instr);
        match instr.opcode{
            0b011_0011 => {
                match instr.funct7 {
                    0b000_0000 => {
                        match instr.funct3 {
                            //ADD
                            0b000 =>{
                                self.registers.common[instr.rd] =
                                    self.registers.common[instr.rs1] + self.registers.common[instr.rs2];
                            },
                            //SLL
                            0b001 => {
                                self.registers.common[instr.rd] =
                                    self.registers.common[instr.rs1] << (instr.rs2 & 0b1_1111);
                            },
                            //SLT
                            0b010 => {
                                self.registers.common[instr.rd] = if instr.rs1 < instr.rs2 {1} else {0};
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
                                        if (instr.rs1 as u32) < (instr.rs2 as u32) {1} else {0};
                                }
                            },
                            //XOR
                            0b100 => {
                                self.registers.common[instr.rd] = 
                                    self.registers.common[instr.rs1] ^ self.registers.common[instr.rs2]; 
                            },
                            //SRL
                            0b101 => {
                                self.registers.common[instr.rd] =
                                    ((self.registers.common[instr.rs1] as u32) >> (instr.rs2 & 0b1_1111)) as i32;
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
                            _ => { panic!("Unreachable") }
                        }
                    },
                    0b100_000 => {
                        match instr.funct3 {
                            //SUB
                            0b000 => {
                                self.registers.common[instr.rd] =
                                    self.registers.common[instr.rs1] - self.registers.common[instr.rs2];
                            },
                            //SRA
                            0b101 => {
                                self.registers.common[instr.rd] =
                                    self.registers.common[instr.rs1] >> (instr.rs2 & 0b1_1111);
                            },
                            _ => {
                                panic!("Invalid funct3({:#b}) for opcode {:#b} and funct7 {:#b}", 
                                    instr.funct3, instr.opcode, instr.funct7)
                            }
                        }
                    },
                    _ => {
                        panic!("Invalid funct7({:#b}) for opcode {:#b}", instr.funct7, instr.opcode)            
                    }
                }
            }
            _ => {
                panic!("Unknown opcode for RType: {:#b}", instr.opcode)
            }
        }
    }

    fn exec_jtype(&mut self, instr: u32){
        println!("New JType instruction: {:x?}", instr);
        
        let instr = JType::parse(instr);
        match instr.opcode{
            //JAL
            0b110_1111 => {
                //Return address
                self.registers.common[2] = self.registers.pc.wrapping_add(4);
                self.registers.pc = self.registers.common[instr.rd].wrapping_add(instr.imm);
            },
            _ => { panic!("PANIC") }
        }
    }

    fn exec_btype(&mut self, instr: u32){
        println!("New JType instruction: {:x?}", instr);

        let instr = BType::from(instr);
        match instr.opcode{
            0b110_0011 => {
                match instr.func3{
                    //BEQ
                    0b000 => {
                        if self.registers.common[instr.rs1] == self.registers.common[instr.rs2]{
                            self.registers.pc = self.registers.pc.wrapping_add(instr.imm);
                        }
                    },
                    //BNE
                    0b001 => {
                        if self.registers.common[instr.rs1] != self.registers.common[instr.rs2]{
                            self.registers.pc = self.registers.pc.wrapping_add(instr.imm);
                        }
                    },
                    //BLT
                    0b100 => {
                        if self.registers.common[instr.rs1] < self.registers.common[instr.rs2]{
                            self.registers.pc = self.registers.pc.wrapping_add(instr.imm);
                        }
                    },
                    //BGE
                    0b101 => {
                        if self.registers.common[instr.rs1] > self.registers.common[instr.rs2]{
                            self.registers.pc = self.registers.pc.wrapping_add(instr.imm);
                        }
                    }
                    //BLTU
                    0b110 => {
                        if (self.registers.common[instr.rs1] as u32) < (self.registers.common[instr.rs2] as u32){
                            self.registers.pc = self.registers.pc.wrapping_add(instr.imm);
                        }
                    }
                    //BGEU
                    0b111 => {
                        if (self.registers.common[instr.rs1] as u32) > (self.registers.common[instr.rs2] as u32){
                            self.registers.pc = self.registers.pc.wrapping_add(instr.imm);
                        }
                    },
                    _ => panic!("") 
                }
            },
            _ => panic!("")
        }
    }

    fn load_instruction(&mut self, instr: u32){
        let instr = IType::from(instr);

        match instr.opcode{
            0b000_0000 => {
                let addr = (instr.rs1 as i32).wrapping_add(instr.imm) as usize;
                
                match instr.funct3 {
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
                    _ => { panic!(""); }
                }
            },
            _ => { panic!("") }
        }
    }

    fn store_instruction(&mut self, instr: u32){
        let instr = SType::from(instr);

        match instr.opcode{
            0b010_0011 => {
                let addr = (instr.rs1 as i32).wrapping_add(instr.imm) as usize;

                match instr.funct3 {
                    //SB
                    0b000 => {
                        self.memory.write(addr, &(self.registers.common[instr.rs2] as u8).to_le_bytes());
                    },
                    //SH
                    0b001 => {
                        self.memory.write(addr, &(self.registers.common[instr.rs2] as u16).to_le_bytes());
                    },
                    //SW
                    0b010 => {
                        self.memory.write(addr, &self.registers.common[instr.rs2].to_le_bytes());
                    },
                    _ => { panic!(""); }
                }
            },
            _ => { panic!("") }
        }
    }

    //LUI
    pub fn LUI(&mut self, instr: u32){
        let instr = UType::parse(instr);

        self.registers.common[instr.rd] = (instr.imm as i32) << 12;
    }

    pub fn AUIPC(&mut self, instr: u32){
        let instr = UType::parse(instr);

        let addr = (instr.imm as i32) << 12;
        self.registers.common[instr.rd] = self.registers.pc.wrapping_add(addr);
    }

    //FENCE => unreachable, used for I/O dont care
    //EBREAK
    //ECALL
}


fn main() {
    let mut cpu = CPU::new();

    //ADDI
    let instr = 0b011011001110_01110_000_10010_0010011 as u32;

    cpu.exec_itype(instr);
    println!("After: {:x?}", cpu);
}
