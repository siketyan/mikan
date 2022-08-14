mnt:
	mkdir -p ./mnt
	hdiutil mount -mountpoint ./mnt ./disk.img

target/aarch64-unknown-uefi/bootx64.efi:
	cd efi && cargo build

target/aarch64-unknown-elf/kernel.elf:
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

.PHONY: build
build: disk.img

.PHONY: rebuild
rebuild:
	$(MAKE) -B build

.PHONY: boot
boot: build aavmf
	qemu-system-aarch64 \
		-machine virt \
		-cpu cortex-a57 \
		-m 512 \
		-bios ./aavmf/QEMU_EFI.fd \
		-drive 'if=virtio,file=./disk.img,format=raw'

.PHONY: reboot
reboot:
	$(MAKE) -B boot
