#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm)]
#![feature(generator_trait)]
#![feature(default_alloc_error_handler)]

mod clint;
mod ns16550a;
mod test_device;
mod execute;
mod runtime;
mod count_harts;
mod feature;

use core::panic::PanicInfo;
use buddy_system_allocator::LockedHeap;

use rustsbi::{print, println};

const SBI_HEAP_SIZE: usize = 64 * 1024;
static mut HEAP_SPACE: [u8; SBI_HEAP_SIZE] = [0; SBI_HEAP_SIZE];
#[global_allocator]
static HEAP: LockedHeap<32> = LockedHeap::empty();

#[cfg_attr(not(test), panic_handler)]
#[allow(unused)]
fn panic(info: &PanicInfo) -> ! {
    let hart_id = riscv::register::mhartid::read();
    // 输出的信息大概是“[rustsbi-panic] hart 0 panicked at ...”
    println!("[rustsbi-panic] hart {} {}", hart_id, info);
    println!("[rustsbi-panic] system shutdown scheduled due to RustSBI panic");
    use rustsbi::Reset;
    test_device::Reset.system_reset(
        rustsbi::reset::RESET_TYPE_SHUTDOWN,
        rustsbi::reset::RESET_REASON_SYSTEM_FAILURE
    );
    loop { }
}

extern "C" fn rust_main(hartid: usize, dtb_pa: usize) -> ! {
    runtime::init();
    if hartid == 0 {
        init_heap();
        init_legacy_stdio();
        init_clint();
        init_test_device();
        println!("[rustsbi] RustSBI version {}", rustsbi::VERSION);
        println!("{}", rustsbi::LOGO);
        println!("[rustsbi] Implementation: RustSBI-QEMU Version {}", env!("CARGO_PKG_VERSION"));
        unsafe { count_harts::init_hart_count(dtb_pa) };
    }
    // 把S的中断全部委托给S层
    unsafe {
        use riscv::register::{mideleg, medeleg, mie};
        mideleg::set_sext();
        mideleg::set_stimer();
        mideleg::set_ssoft();
        medeleg::set_instruction_misaligned();
        medeleg::set_breakpoint();
        medeleg::set_user_env_call();
        medeleg::set_instruction_page_fault();
        medeleg::set_load_page_fault();
        medeleg::set_store_page_fault();
        medeleg::set_instruction_fault();
        medeleg::set_load_fault();
        medeleg::set_store_fault();
        mie::set_mext();
        // 不打开mie::set_mtimer
        mie::set_msoft();
    }
    if hartid == 0 {
        print_misa_medeleg_mideleg();
        println!("[rustsbi] enter supervisor 0x80200000");
    }
    execute::execute_supervisor(0x80200000, hartid, dtb_pa);
}

fn init_heap() {
    unsafe {
        HEAP.lock().init(
            HEAP_SPACE.as_ptr() as usize, SBI_HEAP_SIZE
        )
    }
}

fn init_legacy_stdio() {
    let serial = ns16550a::Ns16550a::new(0x10000000, 0, 11_059_200, 115200);
    use rustsbi::legacy_stdio::init_legacy_stdio_embedded_hal;
    init_legacy_stdio_embedded_hal(serial);
}

fn init_clint() {
    let clint = clint::Clint::new(0x2000000 as *mut u8);
    use rustsbi::init_ipi;
    init_ipi(clint);
    let clint = clint::Clint::new(0x2000000 as *mut u8);
    use rustsbi::init_timer;
    init_timer(clint);
}

fn init_test_device() {
    use rustsbi::init_reset;
    init_reset(test_device::Reset);
}

fn print_misa_medeleg_mideleg() {
    use riscv::register::{misa::{self, MXL}, medeleg, mideleg};
    let isa = misa::read();
    if let Some(isa) = isa {
        let mxl_str = match isa.mxl() {
            MXL::XLEN32 => "RV32",
            MXL::XLEN64 => "RV64",
            MXL::XLEN128 => "RV128",
        };
        print!("[rustsbi] misa: {}", mxl_str);
        for ext in 'A'..='Z' {
            if isa.has_extension(ext) {
                print!("{}", ext);
            }
        }
        println!("");
    }
    println!("[rustsbi] mideleg: {:#x}", mideleg::read().bits());
    println!("[rustsbi] medeleg: {:#x}", medeleg::read().bits());
}

const BOOT_STACK_SIZE: usize = 4096 * 4 * 8;

#[link_section = ".bss.stack"]
static mut BOOT_STACK: [u8; BOOT_STACK_SIZE] = [0; BOOT_STACK_SIZE];

#[naked]
#[link_section = ".text.entry"] 
#[export_name = "_start"]
unsafe extern "C" fn entry() -> ! {
    asm!("
    # 1. set sp
    # sp = bootstack + (hartid + 1) * 0x10000
    add     t0, a0, 1
    slli    t0, t0, 14
1:  auipc   sp, %pcrel_hi({boot_stack})
    addi    sp, sp, %pcrel_lo(1b)
    add     sp, sp, t0

    # 2. jump to rust_main (absolute address)
1:  auipc   t0, %pcrel_hi({rust_main})
    addi    t0, t0, %pcrel_lo(1b)
    jr      t0
    ", 
    boot_stack = sym BOOT_STACK, 
    rust_main = sym rust_main,
    options(noreturn))
}
