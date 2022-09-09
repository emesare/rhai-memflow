use std::collections::BTreeMap;

use rhai::{packages::Package, Engine, EvalAltResult};

use rhai_memflow::{
    native::{Field, Struct, Type},
    MemflowPackage,
};

#[test]
fn test_native_syntax() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    // Register our memflow package.
    let package = MemflowPackage::new();
    package.register_into_engine(&mut engine);

    // it works
    assert_eq!(
        engine.eval::<Type>(r#"native Test { ^ 30, field: Int32, field2: UInt16 }; Test"#)?,
        {
            let mut fields: BTreeMap<u32, Field> = BTreeMap::new();

            fields.insert(
                30,
                Field {
                    name: "field".to_string(),
                    ty: Type::Int32,
                },
            );

            fields.insert(
                34,
                Field {
                    name: "field2".to_string(),
                    ty: Type::UInt16,
                },
            );

            Type::Struct(Struct::new(fields))
        }
    );

    // Make sure we don't panic and instead throw errors.
    assert!(engine
        .eval::<()>(r#"native Test { asdsd: 234234 }"#)
        .is_err());
    assert!(engine.eval::<()>(r#"native Test { 234234 }"#).is_err());
    assert!(engine.eval::<()>(r#"native Test { asdsd }"#).is_err());
    assert!(engine.eval::<()>(r#"native Test { asdsd }"#).is_err());
    assert!(engine.eval::<()>(r#"native Test {}"#).is_err());
    // Missing token `:`.
    assert!(engine
        .eval::<()>(r#"native Test { asdsd ^: ^ asdasd }"#)
        .is_err());

    Ok(())
}

#[test]
fn test_native_size() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    // Register our memflow package.
    let package = MemflowPackage::new();
    package.register_into_engine(&mut engine);

    assert_eq!(
        engine
            .eval::<Type>(r#"native Test { ^ 30, field: Int32 }; Test"#)?
            .size(),
        34
    );

    // Test implicit Struct(*)
    assert_eq!(
        engine
            .eval::<Type>(
                r#"native Custom { f: Fp32 }; native Test { ^ 30, field: Custom }; Test"#
            )?
            .size(),
        34
    );

    // Test explicit Struct(*)
    // TODO: BROKEN
    // assert_eq!(
    //     engine
    //         .eval::<Type>(
    //             r#"native Custom { f: Fp32 }; native Test { ^ 30, field: Struct(Custom) }; Test"#
    //         )?
    //         .size(),
    //     34
    // );

    // TODO: Add one to check pointer field

    Ok(())
}

#[test]
fn test_custom_native() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    // Register our memflow package.
    let package = MemflowPackage::new();
    package.register_into_engine(&mut engine);

    match engine
        .eval::<Type>(r#"native Custom { f: Fp32 }; native Test { ^ 30, custom: Custom }; Test"#)?
    {
        Type::Struct(n) => assert_eq!(n.get_field(30).unwrap(), {
            let mut custom_native = Struct::new(BTreeMap::new());

            custom_native.0.insert(
                0,
                Field {
                    name: "f".into(),
                    ty: Type::Fp32,
                },
            );

            &Field::new("custom".to_string(), Type::Struct(custom_native))
        }),
        _ => panic!("Malformed return of `NativeType::User`"),
    }

    Ok(())
}

#[test]
fn test_native_type() -> Result<(), Box<EvalAltResult>> {
    let mut engine = Engine::new();

    // Register our memflow package.
    let package = MemflowPackage::new();
    package.register_into_engine(&mut engine);

    assert_eq!(engine.eval::<Type>("Int32")?, Type::Int32);
    assert_eq!(engine.eval::<Type>("Int64")?, Type::Int64);
    assert_eq!(engine.eval::<Type>("UInt32")?, Type::UInt32);
    assert_eq!(engine.eval::<Type>("UInt64")?, Type::UInt64);

    Ok(())
}
