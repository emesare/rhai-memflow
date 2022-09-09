use std::cell::RefCell;

use memflow::{
    dummy::*,
    os::OsInner,
    prelude::IntoProcessInstance,
    types::{size, Address},
};
// Used for trait_obj
use cglue::arc::CArc;
use cglue::*;

use rhai::{packages::Package, Engine, EvalAltResult, Scope};
use rhai_memflow::{process::SharedProcess, MemflowPackage};

#[test]
fn test_process() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    // Register our memflow package.
    let package = MemflowPackage::new();
    package.register_into_engine(&mut engine);

    // Create dummy process to test.
    let mem = DummyMemory::new(size::mb(64));
    let mut os = DummyOs::new(mem);
    let pid = os.alloc_process(size::mb(60), &[]);
    let mut prc = os.into_process_by_pid(pid).unwrap();
    prc.proc.add_modules(10, size::kb(1));

    let module_addr = prc.proc.modules.get(0).unwrap().base;

    // Push process (and by extension the moved kernel) to the scope.
    let mut scope = Scope::new();
    // TODO: Write some helper function for Arc-less process'
    let ref_to_count: CArc<cglue::trait_group::c_void> = CArc::default();
    let shared_process: SharedProcess =
        RefCell::new(group_obj!((prc, ref_to_count) as IntoProcessInstance));
    scope.push_constant("PROCESS", shared_process);

    assert_eq!(
        engine.eval_with_scope::<Address>(&mut scope, r#"PROCESS.mod(addr(0)).base"#)?,
        module_addr
    );

    Ok(())
}
