use std::cell::RefCell;

use memflow::prelude::{OsInner, OsInstanceArcBox};

use rhai::plugin::*;

use crate::process::SharedProcess;

pub type SharedOs<'a> = RefCell<OsInstanceArcBox<'a>>;

#[export_module]
#[allow(dead_code)]
#[warn(missing_docs)]
pub mod os_functions {
    #[rhai_fn(pure, return_raw)]
    pub fn process_list(os: &mut SharedOs) -> Result<rhai::Array, Box<EvalAltResult>> {
        match os.borrow_mut().process_info_list() {
            // TODO: Cloning every process info, ouch!
            Ok(mut pi) => Ok(pi.iter_mut().map(|pi| Dynamic::from(pi.clone())).collect()),
            Err(e) => Err(format!("get process failed with {}", &e).into()),
        }
    }

    #[rhai_fn(pure, return_raw, name = "process")]
    pub fn get_process_from_name(
        os: &mut SharedOs,
        name: &str,
    ) -> Result<SharedProcess<'static>, Box<EvalAltResult>> {
        // TODO: This clones the kernel for each process.
        match os.borrow_mut().clone().into_process_by_name(name) {
            Ok(pi) => Ok(RefCell::new(pi)),
            Err(e) => Err(format!("get process failed with {}", &e).into()),
        }
    }

    #[rhai_fn(pure, return_raw, name = "process")]
    pub fn get_process_from_id(
        os: &mut SharedOs,
        id: rhai::INT,
    ) -> Result<SharedProcess<'static>, Box<EvalAltResult>> {
        // TODO: This clones the kernel for each process.
        match os.borrow_mut().clone().into_process_by_pid(id as u32) {
            Ok(pi) => Ok(RefCell::new(pi)),
            Err(e) => Err(format!("get process failed with {}", &e).into()),
        }
    }
}
