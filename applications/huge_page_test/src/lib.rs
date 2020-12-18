#![no_std]
// #![feature(plugin)]
// #![plugin(application_main_fn)]


extern crate alloc;
extern crate memory;
extern crate memory_structs;
// #[macro_use] extern crate log;
#[macro_use] extern crate terminal_print;

use alloc::vec::Vec;
use alloc::string::String;
use memory::{create_mapping, create_huge_mapping, EntryFlags};
use memory_structs::HugePageSize;

pub fn main(_args: Vec<String>) -> isize {
    println!("Testing huge page mappings");
    // Get 2M pages
    let page_size = HugePageSize::new(2*1024*1024).unwrap();
    let bytes = 2*1024*1024;

    //create normal mapping
    match create_mapping(bytes, EntryFlags::PRESENT | EntryFlags::WRITABLE){
        Ok(_m) => {
            println!("Normal mapping Successful");
        },
        Err(e) => {
            println!("ERROR : Normal MAPPING FAILED {}",e);
        }
    }

    // create huge mapping
    match create_huge_mapping(bytes, EntryFlags::PRESENT | EntryFlags::WRITABLE, page_size){
        Ok(_m) => {
            println!("Huge mapping successful");
        },
        Err(e) => {
            println!("ERROR : Huge MAPPING FAILED {}",e);
        }
    }
    0
}
