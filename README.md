# QEMU support using RustSBI

Compile and run with:

```shell
cargo qemu
```

When running `cargo qemu`, the test kernel will build and run. Expected output should be:


```shell
xtask: mode: Debug
   Compiling rustsbi-qemu v0.1.0 (D:\RustProjects\rustsbi-qemu\rustsbi-qemu)
    Finished dev [unoptimized + debuginfo] target(s) in 1.09s
    Finished dev [unoptimized + debuginfo] target(s) in 0.23s
[rustsbi] RustSBI version 0.2.0-alpha.3
.______       __    __      _______.___________.  _______..______   __
|   _  \     |  |  |  |    /       |           | /       ||   _  \ |  |
|  |_)  |    |  |  |  |   |   (----`---|  |----`|   (----`|  |_)  ||  |
|      /     |  |  |  |    \   \       |  |      \   \    |   _  < |  |
|  |\  \----.|  `--'  |.----)   |      |  |  .----)   |   |  |_)  ||  |
| _| `._____| \______/ |_______/       |__|  |_______/    |______/ |__|

[rustsbi] Implementation: RustSBI-QEMU Version 0.1.0
[rustsbi-dtb] Hart count: cluster0 with 1 cores
[rustsbi] misa: RV64ACDFIMSU
[rustsbi] mideleg: 0x222
[rustsbi] medeleg: 0xb1ab
[rustsbi] enter supervisor 0x80200000
<< Test-kernel: Hart id = 0, DTB physical address = 0x87e00000
>> Test-kernel: Testing base extension
<< Test-kernel: Base extension version: 1
<< Test-kernel: SBI specification version: 2
<< Test-kernel: SBI implementation Id: 4
<< Test-kernel: SBI implementation version: 200
<< Test-kernel: Device mvendorid: 0
<< Test-kernel: Device marchid: 0
<< Test-kernel: Device mimpid: 0
>> Test-kernel: Testing SBI instruction emulation
<< Test-kernel: Current time: 289cc0
>> Test-kernel: Trigger illegal exception
<< Test-kernel: Value of scause: Exception(IllegalInstruction)
<< Test-kernel: Illegal exception delegate success
<< Test-kernel: SBI test SUCCESS, shutdown
```
