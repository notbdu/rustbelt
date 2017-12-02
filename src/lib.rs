#![feature(lang_items)]
#![feature(unique)]
#![feature(const_unique_new)]
#![feature(const_fn)]
#![no_std]
#![allow(dead_code)]
extern crate rlibc;
extern crate volatile;
extern crate spin;
extern crate multiboot2;
#[macro_use]
extern crate bitflags;
extern crate x86_64;

#[macro_use]
mod vga_buffer;
mod memory;

use memory::FrameAllocator;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    vga_buffer::clear_screen();

	let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
	let memory_map_tag = boot_info.memory_map_tag()
		.expect("Memory map tag required");

	println!("memory areas:");
	for area in memory_map_tag.memory_areas() {
		println!("    start: 0x{:x}, length: 0x{:x}",
			area.base_addr, area.length);
	}

    let elf_sections_tag = boot_info.elf_sections_tag()
        .expect("Elf-sections tag required");

	//println!("kernel sections:");
	//for section in elf_sections_tag.sections() {
	//	println!("    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
	//		section.addr, section.size, section.flags);
	//}

	let kernel_start = elf_sections_tag.sections().map(|s| s.addr)
		.min().unwrap();
	let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size)
		.max().unwrap();

	let multiboot_start = multiboot_information_address;
	let multiboot_end = multiboot_start + (boot_info.total_size as usize);

    println!("kernel start: 0x{:x}, kernel end: 0x{:x}", kernel_start, kernel_end);
    println!("multiboot start: 0x{:x}, multiboot end: 0x{:x}", multiboot_start, multiboot_end);
    let mut allocator = memory::Allocator::new(kernel_end as usize, multiboot_start as usize,
                                               multiboot_end as usize);
    println!("{:?}", &allocator as *const _);
    println!("{:?}", allocator.allocate(1));
    println!("{:?}", allocator.allocate(1));
    println!("{:?}", allocator.allocate(2));
    println!("{:?}", allocator.allocate(2));
    memory::test_paging(&mut allocator);

    loop{}
}

#[no_mangle]
#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}

#[no_mangle]
#[lang = "panic_fmt"]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("\n\nPANIC in {} at line {}:", file, line);
    println!("    {}", fmt);
    loop{}
}
