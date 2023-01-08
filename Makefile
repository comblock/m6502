cc65:
	cc65 -t none -Oi -o bin/main.s src/cc65/main.c
	ca65 bin/main.s
	ca65 src/cc65/crt0.s -o bin/crt0.o
	ar65 a src/cc65/.lib bin/crt0.o
	ld65 -C src/cc65/ld65.cfg bin/main.o src/cc65/.lib -o "${OUT_DIR}/program"

