use std::cell::RefCell;

use memflow::{
    dummy::*,
    os::OsInner,
    prelude::OsInstance,
    types::{size, Address},
};
// Used for trait_obj
use cglue::arc::CArc;
use cglue::*;

use rhai::{packages::Package, Engine, EvalAltResult, Scope};
use rhai_memflow::{os::SharedOs, MemflowPackage};

#[test]
fn test_os() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    // Register our memflow package.
    let package = MemflowPackage::new();
    package.register_into_engine(&mut engine);

    // Create dummy process to test.
    let mem = DummyMemory::new(size::mb(64));
    let mut os = DummyOs::new(mem);
    let pid = os.alloc_process(size::mb(60), &[]);
    let mut prc = os.process_by_pid(pid).unwrap();
    prc.proc.add_modules(10, size::kb(1));
    let prc_addr = prc.proc.info.address;

    // Push kernel to the scope.
    let mut scope = Scope::new();
    // TODO: Write some helper function for Arc-less os'
    let ref_to_count: CArc<cglue::trait_group::c_void> = CArc::default();
    let shared_kernel: SharedOs = RefCell::new(group_obj!((os, ref_to_count) as OsInstance));
    scope.push_constant("KERNEL", shared_kernel);

    assert_eq!(
        engine.eval_with_scope::<Address>(&mut scope, "KERNEL.process_list()[0].addr")?,
        prc_addr
    );

    Ok(())
}
