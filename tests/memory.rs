use memflow::prelude::phys_mem::PhysicalMemoryView;
use memflow::types::{size, Address};
use memflow::{
    dummy::*,
    prelude::{MemoryView, PhysicalMemory},
};
use rhai::packages::Package;
use rhai::{Dynamic, Engine, EvalAltResult, ImmutableString, Scope};
use rhai_memflow::memory::{read_to_dyn, write_from_dyn};
use rhai_memflow::native::Type;
use rhai_memflow::MemflowPackage;
use widestring::U16String;

#[test]
fn test_memory() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    // Register our memflow package.
    let package = MemflowPackage::new();
    package.register_into_engine(&mut engine);

    // Register `TestMemory` type for `DummyMemory`.
    type TestMemory = PhysicalMemoryView<DummyMemory>;
    engine
        .register_type::<TestMemory>()
        .register_result_fn(
            "read",
            |mem: &mut TestMemory,
             ty: Type,
             addr: Address|
             -> Result<Dynamic, Box<EvalAltResult>> { read_to_dyn(mem, &ty, addr) },
        )
        .register_result_fn(
            "write",
            |mem: &mut TestMemory,
             ty: Type,
             addr: Address,
             val: Dynamic|
             -> Result<(), Box<EvalAltResult>> { write_from_dyn(mem, &ty, addr, val) },
        );

    // `into_phys_view` before `trait_obj` gets us a downcastable memory view.
    let mut mem = DummyMemory::new(size::mb(64)).into_phys_view();
    mem.write::<i32>(0.into(), &16).unwrap();
    mem.write::<i32>(16.into(), &420).unwrap();
    mem.write::<[u8]>(32.into(), "test".as_bytes()).unwrap();
    mem.write::<[u16]>(64.into(), U16String::from_str("test").as_slice())
        .unwrap();

    let mut scope = Scope::new();
    scope.push_constant("MEMORY", mem);

    // Read works
    assert_eq!(
        engine.eval_with_scope::<rhai::INT>(
            &mut scope,
            r#"let test_addr = MEMORY.read(Address32, addr(0)); MEMORY.read(Int32, test_addr)"#
        )?,
        420
    );

    // Read native works
    assert_eq!(
        engine.eval_with_scope::<rhai::INT>(
            &mut scope,
            r#"native Test { ^ 16, num: Int32 }; MEMORY.read(Test, addr(0)).num"#
        )?,
        420
    );

    // Read string works
    assert_eq!(
        engine.eval_with_scope::<ImmutableString>(
            &mut scope,
            r#"MEMORY.read(String(4), addr(32))"#
        )?,
        "test"
    );

    // Read widestring works
    assert_eq!(
        engine.eval_with_scope::<ImmutableString>(
            &mut scope,
            r#"MEMORY.read(WideString(4), addr(64))"#
        )?,
        "test"
    );

    // Write works
    engine.eval_with_scope::<()>(&mut scope, r#"MEMORY.write(UInt32, addr(0), 66)"#)?;
    assert!(
        scope
            .get("MEMORY")
            .unwrap()
            .clone_cast::<TestMemory>()
            .read(0.into())
            == Ok(66 as rhai::INT)
    );

    // TODO: Add testing for the type casts.

    Ok(())
}
