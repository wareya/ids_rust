use std::fs::File;
use std::io::Read;
use std::collections::HashSet;

fn load_to_string(fname : &str) -> std::io::Result<String>
{
    let mut file = File::open(fname)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    return Ok(string);
}

fn is_hanzi(c : u32) -> bool
{
    (c >= 0x4E00 && c <= 0x9FFF)
    || (c >= 0x3400 && c <= 0x4DBF)
    || (c >= 0x20000 && c <= 0x2A6DF)
    || (c >= 0x2A700 && c <= 0x2B73F)
    || (c >= 0x2B740 && c <= 0x2B81F)
    || (c >= 0x2B820 && c <= 0x2CEAF)
    || (c >= 0x2CEB0 && c <= 0x2EBEF)
    || (c >= 0xF900 && c <= 0xFAFF)
}

fn main() -> Result<(), std::io::Error>
{
    let kanjidic = load_to_string("joyoplus.txt")?;
    
    let mut set = HashSet::<char>::new();
    
    for c in kanjidic.chars()
    {
        if is_hanzi(c as u32)
        {
            set.insert(c);
        }
    }
    
    for c in set
    {
        println!("{}", c);
    }
    
    Ok(())
}