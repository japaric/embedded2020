set -euxo pipefail

main() {
    local pkg_name=semidap

    arm-none-eabi-as -march=armv7e-m asm.s -o bin/$pkg_name.o
    ar crs bin/thumbv7em-none-eabi.a bin/$pkg_name.o

    rm bin/*.o
}

main
