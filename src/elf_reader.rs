use std::collections::HashMap;
use core::convert::TryInto;
use std::str;

#[derive(Debug)]
pub struct Symbol{
    name: u32,
    info: u8,
    other: u8,
    shndx: u16,
    value: u64,
    size: u64,
}

impl Symbol{
    fn read_symbol(data: &[u8]) -> Self{
        Self{
            name: u32::from_le_bytes(data[0..4].try_into().unwrap()),
            info: data[4],
            other: data[5],
            shndx: u16::from_le_bytes(data[6..8].try_into().unwrap()),
            value: u64::from_le_bytes(data[8..16].try_into().unwrap()),
            size: u64::from_le_bytes(data[16..24].try_into().unwrap()),
        }
    }
}

pub fn read_symbols_list(symtab: elf::Section, strtab: elf::Section) -> HashMap<String, usize>{
    let mut ret = HashMap::new();

    for i in (0..symtab.data.len()).step_by(24){
        let s = Symbol::read_symbol(&symtab.data[i..]);
    
        let name = str::from_utf8(&strtab.data[(s.name as usize)..]);
        if let Ok(name) = name{
            let name_end = name.find("\0");
            if let Some(name_end) = name_end{
                ret.insert(String::from(&name[0..name_end]), s.value as usize);
            }
            else{
                println!("Error reading end of name for symbol: {:#?}", s);
            }
        }
        else{
            println!("Error reading name for symbol: {:#?}", s);
        } 
    }
    println!("{:?}", ret);
    ret
}