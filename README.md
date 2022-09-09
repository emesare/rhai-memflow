# About `rhai-memflow`

This crate provides the memory introspection library [Memflow] to [Rhai], an embedded scripting language and evaluation engine for [Rust].

[Memflow] is a library that allows live memory introspection of running systems and their snapshots.

## Usage

-----

### `Cargo.toml`

```toml
[dependencies]
rhai-memflow = "0.1"
# `rhai-memflow` uses version `^0.2.0-beta`, this .
memflow = { version = "^0.2.0-beta", features = ["plugins"] }
```

### [Rhai] script

```js
let calc_proc = OS.process("CalculatorApp.exe");
let mod_base = calc_proc.mod("CalculatorApp.dll").base;

// Our native!
native COFFHeader {
    ^ 6, // Pad 6 bytes.
    sections: UInt16,
    timestamp: UInt32
};

let coff_header = calc_proc.read(COFFHeader, mod_base + 0x40);
print(coff_header);
```

### Rust source

```rust
use memflow::prelude::v1::*;
use rhai::{packages::Package, Engine, Scope};

// Create our inventory and OS.
let inventory = Inventory::scan();
let os = inventory.builder().os_chain(chain).build()?;

// Register our memflow package.
let mut engine = Engine::new();
let package = MemflowPackage::new();
package.register_into_engine(&mut engine);

// Add our OS to rhai scope.
let mut scope = Scope::new();
let shared_os: SharedOs = RefCell::new(os);
scope.push_constant("OS", shared_os);

// Run our script.
engine
    .eval_with_scope::<()>(&mut scope, include_str!("script.rhai"))
    .expect("eval failed");
```

[Memflow]: https://memflow.github.io/
[Rhai]: https://rhai.rs/
[Rust]: https://www.rust-lang.org/
