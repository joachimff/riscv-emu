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

    //Execute one instruction
    pub fn exec_instruction(&mut self, instr: u32){
        let opcode = instr & 0b111_1111;

        match opcode{
            //LUI
            0b011_0111 => {
                let instr = UType::from(instr);
                self.registers.common[instr.rd] = (instr.imm as i32) << 12; 
            },
            //AUIPC TODO: double check
            0b001_0111 => {
                let instr = UType::from(instr);

                let addr = (instr.imm as i32) << 12;
                self.registers.common[instr.rd] = self.registers.pc.wrapping_add(addr);
            },
            //JAL
            0b110_1111 => {
                let instr = JType::from(instr);

                self.registers.common[2] = self.registers.pc.wrapping_add(4);
                self.registers.pc = self.registers.common[instr.rd].wrapping_add(instr.imm);
            },
            //JALR
            0b110_0111 => {
                let instr = IType::from(instr);

                self.registers.common[2] = self.registers.pc.wrapping_add(4);
                self.registers.pc = (instr.rs1 as i32).wrapping_add(instr.imm);
            },
            //Conditional Branches
            0b110_0011 => {
                let instr = BType::from(instr);

                match instr.func3 {
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
                    _ => {unreachable!()},
                }
            },
            //LOAD 
            0b000_0011 => {
                let instr = IType::from(instr);
                let addr = (instr.rs1 as i32).wrapping_add(instr.imm) as usize;

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
                let addr = (instr.rs1 as i32).wrapping_add(instr.imm) as usize;

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
                
                match instr.funct3{
                    //ADDI
                    0b000 => { self.registers.common[instr.rd] = self.registers.common[instr.rs1] + instr.imm; },
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
                        if ((instr.imm >> 11) & 0b1) == 1{
                            self.registers.common[instr.rd] = (self.registers.common[instr.rs1] as i32) >> shamt;
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
                                self.registers.common[instr.rs1] + self.registers.common[instr.rs2];
                        }
                        //SUB
                        else{
                            self.registers.common[instr.rd] =
                                self.registers.common[instr.rs1] - self.registers.common[instr.rs2];
                        }
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
                    0b101 => {
                        //SRL
                        if instr.funct7 == 0{
                            self.registers.common[instr.rd] =
                                ((self.registers.common[instr.rs1] as u32) >> (instr.rs2 & 0b1_1111)) as i32;
                        }
                        //SRA
                        else {
                            self.registers.common[instr.rd] =
                                self.registers.common[instr.rs1] >> (instr.rs2 & 0b1_1111);
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
            0b111_0011 => { panic!("ECALL/EBREAK NYI"); },
            
            _ => unreachable!()
        }
    }
}


fn main() {
    let mut cpu = CPU::new();

    //ADDI
    let instr = 0b011011001110_01110_000_10010_0010011 as u32;

    cpu.exec_instruction(instr);
    println!("After: {:x?}", cpu);
}
