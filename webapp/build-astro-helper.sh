#!/bin/sh
set -eu
commit="61dc07020aaa6885d2c7f688a4d82beaf6edb9ef"
curl --fail --silent --show-error --location "https://raw.githubusercontent.com/cosinekitty/astronomy/${commit}/source/c/astronomy.c" -o astronomy.c
curl --fail --silent --show-error --location "https://raw.githubusercontent.com/cosinekitty/astronomy/${commit}/source/c/astronomy.h" -o astronomy.h
cc -Os -ffunction-sections -fdata-sections -Wl,--gc-sections -s -o astro-helper astro_helper.c astronomy.c -lm
