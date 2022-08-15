.DEFAULT_GOAL = all

shinonome-0.9.11p1.tar.bz2:
	curl -sSL http://openlab.ring.gr.jp/efont/dist/shinonome/shinonome-0.9.11p1.tar.bz2 > ./shinonome-0.9.11p1.tar.bz2

shinonome-0.9.11: shinonome-0.9.11p1.tar.bz2
	tar -xf ./shinonome-0.9.11p1.tar.bz2

shnm8x16a.bdf: shinonome-0.9.11
	cp ./shinonome-0.9.11/bdf/shnm8x16a.bdf ./

.PHONY: all
all: shnm8x16a.bdf
