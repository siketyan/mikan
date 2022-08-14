source.deb:
	curl -sSL http://ftp.jp.debian.org/debian/pool/main/e/edk2/qemu-efi-aarch64_2022.05-2_all.deb > source.deb

data.tar.xz: source.deb
	ar -x ./source.deb

usr: data.tar.xz
	tar -xf ./data.tar.xz

AAVMF_CODE.fd: usr
	cp ./usr/share/AAVMF/AAVMF_CODE.fd ./AAVMF_CODE.fd

AAVMF_VARS.fd: usr
	cp ./usr/share/AAVMF/AAVMF_VARS.fd ./AAVMF_VARS.fd

QEMU_EFI.fd: usr
	cp ./usr/share/qemu-efi-aarch64/QEMU_EFI.fd ./QEMU_EFI.fd

all: AAVMF_CODE.fd AAVMF_VARS.fd QEMU_EFI.fd
