#![feature(map_first_last)]
#![deny(unsafe_code)]

use rhai::def_package;
use rhai::plugin::*;

pub mod memory;
pub mod native;
pub mod os;
pub mod process;

use crate::memory::memory_functions;
use crate::os::os_functions;
use crate::process::process_functions;

def_package! {
    /// Package for memory introspection with memflow
    pub MemflowPackage(lib) {
        lib.set_custom_type::<process::SharedProcess>("Process");
        combine_with_exported_module!(lib, "rhai_memflow_native", native::export_mod);
        combine_with_exported_module!(lib, "rhai_memflow_memory", memory_functions);
        combine_with_exported_module!(lib, "rhai_memflow_os", os_functions);
        combine_with_exported_module!(lib, "rhai_memflow_process", process_functions);
    } |> |engine| {
        native::register_native_syntax(engine);
    }
}
