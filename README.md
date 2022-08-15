# 🍊 mikan
[![Rust](https://github.com/siketyan/mikan/actions/workflows/rust.yml/badge.svg)](https://github.com/siketyan/mikan/actions/workflows/rust.yml)
[![Image](https://github.com/siketyan/mikan/actions/workflows/image.yml/badge.svg)](https://github.com/siketyan/mikan/actions/workflows/image.yml)

Yet another implementation of MikanOS for aarch64 CPUs, written in Rust.

MikanOS ([uchan-nos/mikanos](https://github.com/uchan-nos/mikanos)) was originally created by @uchan-nos,
who is author of the book [ゼロからの OS 自作入門](https://zero.osdev.jp/) by Mynavi Publishing Corporation.
I tried to port this OS to aarch64 CPUs, written in Rust, built on macOS.
For details of this OS and C++ implementation, please refer the original repository.

Note that this repository aims to implement all features of MikanOS, but their design and implementation is customised
and optimised for writing in Rust.

## Features
- Supports aarch64 (ARM64) CPUs
- Written in Rust (no_std)
- Built on macOS

## Prerequisites
- Rust Toolchain (1.65-nightly+)
- QEMU
- dosfstools (macOS only)

## Building
Builds a disk image and boots them on QEMU by calling only one command: 

```shell
make boot
```

## Roadmap
- [x] Day 1: Hello world
- [x] Day 2: Memory map
- [x] Day 3: Bootloader and framebuffer
- [x] Day 4: Pixel drawing
- [ ] Day 5: Text rendering and console
- [ ] Day 6: Mouse input and PCI
- [ ] Day 7: Interruption and FIFO
- [ ] Day 8: Memory management
- [ ] Day 9: Super-positioning
- [ ] Day 10: Windows
- [ ] Day 11: Timer and ACPI
- [ ] Day 12: Key inputs
- [ ] Day 13: Multi-tasking (1)
- [ ] Day 14: Multi-tasking (2)
- [ ] Day 15: Terminal
- [ ] Day 16: Commands
- [ ] Day 17: Filesystem
- [ ] Day 18: Applications
- [ ] Day 19: Paging
- [ ] Day 20: System calls
- [ ] Day 21: Windows in application
- [ ] Day 22: Graphics and events (1)
- [ ] Day 23: Graphics and events (2)
- [ ] Day 24: Multiple terminals
- [ ] Day 25: Loading files into app
- [ ] Day 26: Writing files from app
- [ ] Day 27: Memory management for apps
- [ ] Day 28: Japanese (CJK) support and redirecting
- [ ] Day 29: Inter-application communication
- [ ] Day 30: Misc applications
