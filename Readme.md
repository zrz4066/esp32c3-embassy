a project to run embassy on an ESP32C3



### Installing the Rust Compiler Target

The compilation target for this device is officially supported by the mainline Rust compiler and can be installed using [rustup](https://rustup.rs/):

```
rustup target add riscv32imc-unknown-none-elf
```



### Build demo

```
cargo build --release
```


