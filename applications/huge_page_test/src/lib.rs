//! A very simple application to test huge page mappings of various sizes.
//! Creates mappings of 4KB 2MB and 1GB pages providing architecture support

#![no_std]
// #![feature(plugin)]
// #![plugin(application_main_fn)]


extern crate alloc;
extern crate memory;
extern crate memory_structs;
// extern crate page_allocator;
#[macro_use] extern crate log;
#[macro_use] extern crate terminal_print;

use alloc::vec::Vec;
use alloc::string::String;
use memory::{create_mapping, create_huge_mapping, EntryFlags};
use memory_structs::HugePageSize;

pub fn main(_args: Vec<String>) -> isize {
    println!("Testing huge page mappings");

    // page_allocator::dump_page_allocator_state();

    let bytes = 4*1024*1024;
    //create normal mapping
    match create_mapping(bytes, EntryFlags::PRESENT | EntryFlags::WRITABLE){
        Ok(m) => {
            debug!("{:?}", m);
            println!("Normal mapping Successful");
        },
        Err(e) => {
            println!("ERROR : Normal MAPPING FAILED {}",e);
        }
    }

    // create 2M mapping
    match HugePageSize::new(2*1024*1024) {
        Ok(page_size) => {
            match create_huge_mapping(bytes, EntryFlags::PRESENT | EntryFlags::WRITABLE, page_size){
                Ok(m) => {
                    debug!("{:?}", m);
                    println!("2M mapping successful");
                },
                Err(e) => {
                    println!("ERROR : 2M MAPPING FAILED {}",e);
                }
            }
        },
        Err(e) => {
            println!("Err {}",e);
        },
    }

    // create 1G mapping
    match HugePageSize::new(1024*1024*1024) {
        Ok(page_size) => {
            match create_huge_mapping(bytes, EntryFlags::PRESENT | EntryFlags::WRITABLE, page_size){
                Ok(m) => {
                    debug!("{:?}", m);
                    println!("1G mapping successful");
                },
                Err(e) => {
                    println!("ERROR : 1G MAPPING FAILED {}",e);
                }
            }
        },
        Err(e) => {
            println!("Err {}",e);
        },
    }

    0
}
