disk.img:
	rm -f disk.img || true
	qemu-img create -f raw ./disk.img 200M
	mkfs.fat -n 'MIKAN' -s 2 -f 2 -R 32 -F 32 ./disk.img
	hdiutil mount -mountpoint ./mnt ./disk.img
	mkdir -p ./mnt/EFI/BOOT
	cp ./target/aarch64-unknown-uefi/debug/bootx64.efi ./mnt/EFI/BOOT/BOOTAA64.EFI
	umount ./mnt

aavmf:
	$(MAKE) -C aavmf all

.PHONY: boot
boot: disk.img aavmf
	qemu-system-aarch64 \
		-machine virt \
		-cpu cortex-a57 \
		-m 512 \
		-bios ./aavmf/QEMU_EFI.fd \
		-drive 'if=virtio,file=./disk.img,format=raw'
