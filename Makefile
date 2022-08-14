mnt:
	mkdir -p ./mnt
	hdiutil mount -mountpoint ./mnt ./disk.img

target/aarch64-unknown-uefi/bootx64.efi:
	cargo build --target aarch64-unknown-uefi

disk.img: target/aarch64-unknown-uefi/bootx64.efi
	rm -f disk.img || true
	qemu-img create -f raw ./disk.img 200M
	mkfs.fat -n 'MIKAN' -s 2 -f 2 -R 32 -F 32 ./disk.img
	$(MAKE) mnt
	mkdir -p ./mnt/EFI/BOOT
	cp ./target/aarch64-unknown-uefi/debug/bootx64.efi ./mnt/EFI/BOOT/BOOTAA64.EFI
	$(MAKE) umount

aavmf:
	$(MAKE) -C aavmf all

.PHONY: mount
mount:
	$(MAKE) -B mnt

.PHONY: umount
umount:
	umount ./mnt

.PHONY: build
build: disk.img aavmf

.PHONY: rebuild
rebuild:
	$(MAKE) -B build

.PHONY: boot
boot: build
	qemu-system-aarch64 \
		-machine virt \
		-cpu cortex-a57 \
		-m 512 \
		-bios ./aavmf/QEMU_EFI.fd \
		-drive 'if=virtio,file=./disk.img,format=raw'

.PHONY: reboot
reboot:
	$(MAKE) -B boot
