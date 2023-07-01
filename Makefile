UNAME := $(shell uname)

ifeq ($(UNAME), Darwin)
	MOUNT := hdiutil mount -mountpoint ./mnt ./disk.img
else
	MOUNT := mount -o loop ./disk.img ./mnt
endif

mnt:
	mkdir -p ./mnt
	$(MOUNT)

resources/fonts/shinonome/shnm8x16a.bdf:
	$(MAKE) -C resources

.PHONY: resources
resources: resources/fonts/shinonome/shnm8x16a.bdf

target/aarch64-unknown-uefi/bootx64.efi:
	cd efi && cargo build

target/aarch64-unknown-elf/kernel.elf: resources
	cd kernel && cargo build

disk.img: target/aarch64-unknown-uefi/bootx64.efi target/aarch64-unknown-elf/kernel.elf
	rm -f disk.img || true
	qemu-img create -f raw ./disk.img 200M
	mkfs.fat -n 'MIKAN' -s 2 -f 2 -R 32 -F 32 ./disk.img
	$(MAKE) mount
	mkdir -p ./mnt/EFI/BOOT
	cp ./target/aarch64-unknown-uefi/debug/bootx64.efi ./mnt/EFI/BOOT/BOOTAA64.EFI
	cp ./target/aarch64-unknown-elf/debug/kernel.elf ./mnt/kernel.elf
	$(MAKE) umount

aavmf:
	$(MAKE) -C aavmf all

.PHONY: mount
mount:
	$(MAKE) -B mnt

.PHONY: umount
umount:
	umount ./mnt
	rm -rf ./mnt

.PHONY: check
check:
	cd efi && cargo check
	cd kernel && cargo check

.PHONY: clippy
clippy:
	cd efi && cargo clippy
	cd kernel && cargo clippy

.PHONY: build
build: disk.img

.PHONY: rebuild
rebuild:
	$(MAKE) -B build

.PHONY: boot
boot: build aavmf
	qemu-system-aarch64 \
		-s -S \
		-machine virt \
		-cpu cortex-a57 \
		-m 512 \
		-bios ./aavmf/QEMU_EFI.fd \
		-drive 'if=virtio,file=./disk.img,format=raw' \
		-device ramfb \
		-device qemu-xhci \
		-device usb-mouse \
		-trace 'usb_*,file=usb.log'

.PHONY: reboot
reboot:
	$(MAKE) -B boot
