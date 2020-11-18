/// Memory management
// Hold the registers 
#[derive(Debug)]
struct Registers{
    common: [i32; 32],
    sp: i32,
}

impl Registers {
    pub fn new() -> Registers {
        Registers{
            common: [0; 32],
            sp: 0
        }
    }
}

//Hold the memory
#[derive(Debug)]
struct Memory {
}

//Manage memory
impl Memory{
    //Return a new memory with all null data
    pub fn new() -> Memory {
        Memory {
        }
    }
}

#[derive(Debug)]
struct RType{
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
struct IType{
    imm: i32,
    rs1: usize,
    funct3: u8,
    rd: usize,
    opcode: u8
}

impl IType {
    pub fn parse(instruction: u32) -> IType {
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
struct UType{
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
struct JType{
    imm: u32,
    rd: usize,
    opcode: u8
}

impl JType{
    pub fn parse(instruction:u32) -> JType{
        JType{
            imm: (((instruction >> 12) & 0b1111_1111_1111_1111_1111) << 12) as u32,
            rd:     ((instruction >> 7) & 0b11111) as usize,
            opcode: (instruction & 0b111_1111) as u8
        }
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
        let instr = IType::parse(instr);

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
}



fn main() {
    let mut cpu = CPU::new();

    //ADDI
    let instr = 0b011011001110_01110_000_10010_0010011 as u32;

    cpu.exec_itype(instr);
    println!("After: {:x?}", cpu);
}
