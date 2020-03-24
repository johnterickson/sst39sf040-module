#!/usr/bin/env bash
gdb-multiarch -q target/thumbv7em-none-eabihf/debug/sst39sf040-flasher --nx --ex "target remote :3333" --ex "monitor tpiu config internal itm.txt uart off 8000000" --ex "monitor itm port 0 on"
