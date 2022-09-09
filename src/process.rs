use std::cell::RefCell;

use memflow::{
    prelude::{IntoProcessInstanceArcBox, ModuleInfo, Process, ProcessInfo},
    types::Address,
};

use rhai::plugin::*;

use crate::{
    memory::{read_to_dyn, NativePointer},
    native::Type,
};

pub type SharedProcess<'a> = RefCell<IntoProcessInstanceArcBox<'a>>;

#[export_module]
#[allow(dead_code)]
#[warn(missing_docs)]
pub mod process_functions {
    #[rhai_fn(pure, return_raw, name = "mod")]
    pub fn get_module_from_name(
        proc: &mut SharedProcess,
        name: &str,
    ) -> Result<ModuleInfo, Box<EvalAltResult>> {
        proc.borrow_mut()
            .module_by_name(name)
            .map_err(|e| e.as_str().into())
    }

    // Originally named module however that is a reserved keyword.
    #[rhai_fn(pure, return_raw, name = "mod")]
    pub fn get_module_from_addr(
        proc: &mut SharedProcess,
        addr: Address,
    ) -> Result<ModuleInfo, Box<EvalAltResult>> {
        let mut prc = proc.borrow_mut();
        let arch = prc.info().sys_arch;
        prc.module_by_address(addr, arch)
            .map_err(|e| e.as_str().into())
    }

    #[rhai_fn(pure, return_raw, get = "addr")]
    pub fn get_addr(proc: &mut SharedProcess) -> Result<Address, Box<EvalAltResult>> {
        proc.borrow_mut()
            .primary_module_address()
            .map_err(|e| e.as_str().into())
    }

    #[rhai_fn(pure, return_raw, name = "read")]
    pub fn read(
        proc: &mut SharedProcess,
        ty: Type,
        addr: Address,
    ) -> Result<Dynamic, Box<EvalAltResult>> {
        read_to_dyn(proc.get_mut(), &ty, addr)
    }

    #[rhai_fn(pure, return_raw, name = "read")]
    pub fn read_ptr(
        proc: &mut SharedProcess,
        ptr: NativePointer,
    ) -> Result<Dynamic, Box<EvalAltResult>> {
        read_to_dyn(proc.get_mut(), &ptr.0, ptr.1)
    }

    #[rhai_fn(pure, get = "info")]
    pub fn get_info(proc: &mut SharedProcess) -> ProcessInfo {
        proc.borrow_mut().info().clone()
    }

    pub mod module_info_functions {
        #[rhai_fn(pure, get = "header")]
        pub fn get_header_addr(mi: &mut ModuleInfo) -> Address {
            mi.address
        }

        #[rhai_fn(pure, get = "base")]
        pub fn get_base_addr(mi: &mut ModuleInfo) -> Address {
            mi.base
        }

        #[rhai_fn(pure, get = "size")]
        pub fn get_size(mi: &mut ModuleInfo) -> rhai::INT {
            mi.size as rhai::INT
        }

        #[rhai_fn(pure, get = "name")]
        pub fn get_name(mi: &mut ModuleInfo) -> String {
            mi.name.to_string()
        }

        #[rhai_fn(pure, get = "path")]
        pub fn get_path(mi: &mut ModuleInfo) -> String {
            mi.path.to_string()
        }
    }

    pub mod process_info_functions {
        #[rhai_fn(pure, get = "addr")]
        pub fn get_addr(pi: &mut ProcessInfo) -> Address {
            pi.address
        }

        #[rhai_fn(pure, get = "pid")]
        pub fn get_pid(pi: &mut ProcessInfo) -> rhai::INT {
            pi.pid as rhai::INT
        }

        #[rhai_fn(pure, get = "name")]
        pub fn get_name(pi: &mut ProcessInfo) -> String {
            pi.name.to_string()
        }

        #[rhai_fn(pure, get = "path")]
        pub fn get_path(pi: &mut ProcessInfo) -> String {
            pi.path.to_string()
        }

        #[rhai_fn(pure, get = "cmdline")]
        pub fn get_cmdline(pi: &mut ProcessInfo) -> String {
            pi.command_line.to_string()
        }
    }
}
