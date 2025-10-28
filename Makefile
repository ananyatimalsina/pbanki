PBSDK ?= ./SDK-B300-6.8/usr
LIB = $(PBSDK)/lib
CC = LD_LIBRARY_PATH=$(LIB) $(PBSDK)/bin/arm-obreey-linux-gnueabi-gcc
STRIP = $(PBSDK)/bin/arm-obreey-linux-gnueabi-strip
PBRES = $(PBSDK)/bin/pbres

GIT_VERSION := "0.0.1-nightly"

CFLAGS = -DCOMBINED -std=c99 -DNDEBUG -fsigned-char -fomit-frame-pointer -fPIC -O2 -march=armv7-a -mtune=cortex-a8 -mfpu=neon -mfloat-abi=softfp -linkview -lfreetype -lm -D_XOPEN_SOURCE=632 -DVERSION=\"$(GIT_VERSION)\"

FRONTENDSRCS := $(wildcard frontend/*.c)
FRONTENDOBJS := $(FRONTENDSRCS:%.c=%.o)
FRONTENDHDRS := $(wildcard include/frontend/*.h)

HEADERS := $(wildcard include /*.h)

all: build/pbAnki.app

build/pbAnki.app: $(FRONTENDOBJS)
	mkdir -p ./build
	LC_ALL=C $(CC) -s $(FRONTENDOBJS) -I./include -o ./build/pbAnki.app $(CFLAGS)
	LC_ALL=C $(STRIP) ./build/pbAnki.app

frontend/%.o: frontend/%.c $(HEADERS) $(FRONTENDHDRS)
	LC_ALL=C $(CC) -c $< -o $@ -I./include $(CFLAGS)

clean:
	rm -f build/pbAnki.app frontend/*.o
