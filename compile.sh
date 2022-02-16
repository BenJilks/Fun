#!/usr/bin/env bash

echo "Compiling" &&\
cargo run test.fun > test.asm &&\
echo "Assembling" &&\
nasm -felf32 test.asm &&\
echo "Linking" &&\
gcc -m32 test.o -o test &&\
rm -f test.o &&\
echo "Running" &&\
./test &&\
echo '' &&\
rm -f test

