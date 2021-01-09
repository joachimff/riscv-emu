extern crate rand;

use rand::{thread_rng, Rng};
use std::str;

use super::cpu::CPU;

pub enum SpecialFD{
    Stdin = 0,
    Stdout = 1,
    Stderr = 2,
}

/// Generate fuzzed inputs, everything is store in memory for speed.
/// Starting from no corpus
pub struct Fuzzer{
    /// Contains entries that lead to unique code execution path, when generating
    /// a new input one of them is selected then random bytes are flipped if the
    /// code execution is unique it will be added to this array 
    corpus: Vec<Vec<u8>>,

    mutated_input: Vec<Vec<u8>>,
}

impl Fuzzer{
    pub fn new() -> Self{
        let mut f = Fuzzer{
            corpus: Vec::new(),
            mutated_input: Vec::new(),
        };
        f.corpus.push(vec![0x42,0x4e,0x45,0x0a]);
        f.corpus.push(vec![12,12,12,0x0a]);
        f.corpus.push(vec![12,12,12,0x0a]);
    
        println!("{:?}", f.corpus);
        f
    }

    // Returns the last element 
    pub fn get_fuzz_input(&mut self) -> Vec<u8> {
        let corpus = self.corpus.pop();
        
        return corpus.unwrap();
    }

    pub fn syscall(&mut self, cpu: &mut CPU){
        let syscall_nbr = cpu.registers.common[17]; //a7
        
        println!("Syscall: {:X}, {:X}, {:X}, {:X}", 
            cpu.registers.common[10], cpu.registers.common[11],
            cpu.registers.common[12], cpu.registers.common[13]);
        
        match syscall_nbr{
            // Read
            63 => {
                println!("Read");
                let fp = cpu.registers.common[10];
                let ptr = cpu.registers.common[11];
                let len = cpu.registers.common[12];
                
                if fp != SpecialFD::Stdin as u64{
                    panic!("Reading from files is not supported");
                }
                
                let buf = self.get_fuzz_input();
    
                println!("Pulled: {:X?} from fuzz queue", buf);
                cpu.memory.write(ptr, &buf);
                
                cpu.registers.common[10] = buf.len() as u64;
            },
            // Write
            64 => {
                println!("Write");
                let fp = cpu.registers.common[10];
                let ptr = cpu.registers.common[11];
                let len = cpu.registers.common[12];
                
                if fp != SpecialFD::Stdout as u64{
                    panic!("Writing to files is not supported");
                }
                
                if cpu.redirect_stdout{
                    let mut buf = vec![0 as u8; len as usize];
                    cpu.memory.read(ptr, &mut buf);
                    
                    let message = str::from_utf8(&buf).unwrap();
                    println!("STDOUT: {}", message);
                    
                    // Returns the number of bytes written
                    cpu.registers.common[10] = len;
                }
            },
            // Fstat
            80 => {
                println!("Fstate");
            }
            // Brk
            214 => {
                println!("Brk");
            }
            _ => {
                println!("Unknown syscall: {:?}", syscall_nbr);
            }
        }
        println!();
    }
}