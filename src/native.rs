use std::{any::TypeId, collections::BTreeMap};

use rhai::{plugin::*, EvalContext, Expression};

pub fn register_native_syntax(engine: &mut Engine) {
    // Used to define a `Native`, (i.e. `native MonoString { field_1: Int32, str: WideString(255) };`).
    engine.register_custom_syntax_raw("native", parse_native, true, implement_native);
}

#[export_module]
#[allow(non_snake_case, non_upper_case_globals)]
#[warn(missing_docs)]
pub mod export_mod {
    use super::Type;
    use rhai::plugin::*;

    // Constructors for 'NativeType' variants
    pub const UInt8: Type = Type::UInt8;
    pub const UInt16: Type = Type::UInt16;
    pub const Int32: Type = Type::Int32;
    pub const UInt32: Type = Type::UInt32;
    pub const Fp32: Type = Type::Fp32;
    pub const Address32: Type = Type::Address32;
    pub const Int64: Type = Type::Int64;
    pub const UInt64: Type = Type::UInt64;
    pub const Fp64: Type = Type::Fp64;
    pub const Address64: Type = Type::Address64;

    pub fn String(value: rhai::INT) -> Type {
        Type::String(value as u32)
    }

    pub fn WideString(value: rhai::INT) -> Type {
        Type::WideString(value as u32)
    }

    pub fn Pointer32(ty: Type) -> Type {
        Type::Pointer32(Box::new(ty))
    }

    pub fn Pointer64(ty: Type) -> Type {
        Type::Pointer64(Box::new(ty))
    }

    pub fn Struct(native: self::Struct) -> Type {
        Type::Struct(native)
    }

    pub fn Collection(ty: Type, size: rhai::INT) -> Type {
        Type::Collection(Box::new(ty), size as u32)
    }

    #[rhai_fn(pure, global, get = "enum_type")]
    pub fn get_type(native_ty: &mut Type) -> String {
        match native_ty {
            Type::UInt8 => "UInt8".to_string(),
            Type::UInt16 => "UInt16".to_string(),
            Type::Int32 => "Int32".to_string(),
            Type::UInt32 => "UInt32".to_string(),
            Type::Fp32 => "Fp32".to_string(),
            Type::Address32 => "Address32".to_string(),
            Type::Pointer32(_) => "Pointer32".to_string(),
            Type::Int64 => "Int64".to_string(),
            Type::UInt64 => "UInt64".to_string(),
            Type::Fp64 => "Fp64".to_string(),
            Type::Address64 => "Address64".to_string(),
            Type::Pointer64(_) => "Pointer64".to_string(),
            Type::String(_) => "String".to_string(),
            Type::WideString(_) => "WideString".to_string(),
            Type::Struct(_) => "Struct".to_string(),
            Type::Collection(_, _) => "Collection".to_string(),
        }
    }

    // Access to fields
    #[rhai_fn(pure, global, get = "size")]
    pub fn get_size(native_ty: &mut Type) -> Dynamic {
        Dynamic::from_int(native_ty.size().into())
    }

    #[rhai_fn(pure, global, get = "native")]
    pub fn get_native_struct(native_ty: &mut Type) -> Dynamic {
        // Clones the native struct and returns it as a custom type.
        match native_ty {
            Type::Struct(ns) => Dynamic::from(ns.clone()),
            _ => Dynamic::UNIT,
        }
    }

    // Printing
    #[rhai_fn(pure, global, name = "to_string", name = "to_debug")]
    pub fn to_string(native_ty: &mut Type) -> String {
        format!("{:?}", native_ty)
    }

    // '==' and '!=' operators
    #[rhai_fn(pure, global, name = "==")]
    pub fn eq(native_ty: &mut Type, native_ty2: Type) -> bool {
        native_ty == &native_ty2
    }

    #[rhai_fn(pure, global, name = "!=")]
    pub fn neq(native_ty: &mut Type, native_ty2: Type) -> bool {
        native_ty != &native_ty2
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Type {
    UInt8,
    UInt16,
    Int32,
    UInt32,
    Fp32,
    Address32,
    Pointer32(Box<Type>),
    Int64,
    UInt64,
    Fp64,
    Address64,
    Pointer64(Box<Type>),
    // TODO: Consolidate strings and add encoding to enum `String`.
    String(u32),
    WideString(u32),
    Struct(Struct),
    Collection(Box<Type>, u32),
}

impl Type {
    /// Size in bytes
    pub fn size(&self) -> u32 {
        match self {
            Self::UInt8 => 1,
            Self::UInt16 => 2,
            Self::Int32 | Self::UInt32 | Self::Fp32 | Self::Address32 | Self::Pointer32(_) => 4,
            Self::Int64 | Self::UInt64 | Self::Fp64 | Self::Address64 | Self::Pointer64(_) => 8,
            Self::String(len) | Type::WideString(len) => *len,
            Self::Struct(u) => u.size(),
            Self::Collection(u, size) => size * u.size(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

impl Field {
    pub fn new(name: String, ty: Type) -> Self {
        Self { name, ty }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Struct(pub BTreeMap<u32, Field>);

impl Struct {
    pub fn new(fields: BTreeMap<u32, Field>) -> Self {
        Self(fields)
    }

    pub fn size(&self) -> u32 {
        match self.0.last_key_value() {
            // Adds the last offset + the fields type size to get the max size of the `NativeType::User`
            Some(last_field) => last_field.0 + last_field.1.ty.size(),
            None => 0,
        }
    }

    pub fn get_field(&self, offset: u32) -> Option<&Field> {
        self.0.get(&offset)
    }

    pub fn get_field_from_name(&self, field_name: &str) -> Option<&Field> {
        self.0
            .iter()
            .find_map(|(_, nf)| field_name.eq(&nf.name).then_some(nf))
    }
}

impl IntoIterator for Struct {
    type Item = (u32, Field);

    type IntoIter = std::collections::btree_map::IntoIter<u32, Field>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

fn parse_native(
    symbols: &[ImmutableString],
    look_ahead: &str,
) -> Result<Option<ImmutableString>, rhai::ParseError> {
    match (symbols.len(), look_ahead) {
        (1, _) => Ok(Some("$ident$".into())),
        (2, _) => Ok(Some("{".into())),
        (x, lh) if x >= 2 => {
            // Get the previously parsed field symbols.
            let mut field_symbols: Vec<&ImmutableString> = symbols
                .iter()
                .rev()
                .take_while(|s| **s != "{" && **s != ",")
                .collect();

            // Fix direction to get: <keyword> <param1> <param2>.
            field_symbols.reverse();

            // TODO: The below code hardcodes the different field types, regular and padding (denoted by `^`), if we ever want to add more it might be a good idea to transform the symbols into an enum and match against that instead.

            let res = match field_symbols.first() {
                Some(kw) => {
                    let keyword = kw.as_str();
                    match (field_symbols.len(), lh) {
                        (1, _) => match keyword {
                            "^" => Some("$int$"),
                            _ => Some(":"),
                        },
                        (2, _) => match keyword {
                            "^" => None,
                            _ => Some("$expr$"),
                        },
                        _ => None,
                    }
                }
                None => {
                    // We have yet to start a new field, we could expect a number of keywords (such as padding keyword `^`).
                    match lh {
                        "^" => Some("$symbol$"),
                        _ => Some("$ident$"),
                    }
                }
            };

            match res {
                Some(expected) => Ok(Some(expected.into())),
                // Expect field ending.
                None => match lh {
                    // TODO: We are letting the last field include the `}`, this might make us vulnerable to some nasty parsing issues.
                    "}" => Ok(Some("}".into())),
                    ";" => Ok(None),
                    _ => Ok(Some(",".into())),
                },
            }
        }
        _ => unreachable!(),
    }
}

fn implement_native(
    context: &mut EvalContext,
    inputs: &[Expression],
) -> Result<Dynamic, Box<EvalAltResult>> {
    let native_name = inputs[0].get_string_value().unwrap();
    let mut native = Struct::new(BTreeMap::new());

    let mut offset = 0u32;
    let mut expr_iter = inputs.iter().skip(1);
    while let Some(expr) = expr_iter.next() {
        match expr.get_string_value() {
            Some(keyword) => match keyword {
                // Padding.
                "^" => {
                    // Get the pad size from next expression and add it to the current offset.
                    if let Some(padding_offset) = expr_iter.next().and_then(|expr| {
                        expr.get_literal_value::<rhai::INT>()
                            .map(|i| i.abs() as u32)
                    }) {
                        offset += padding_offset;
                    } else {
                        return Err("padding must be a constant literal".into());
                    }
                }
                // Regular field.
                _ => {
                    if let Some(field_type) = expr_iter
                        .next()
                        .and_then(|expr| context.eval_expression_tree(expr).ok())
                    {
                        if field_type.is_variant() {
                            // TODO: Refactor this please.
                            if field_type.type_id() == TypeId::of::<Struct>() {
                                // Implicit `Struct(*)`.
                                let native_struct: Struct = field_type.cast();
                                native.0.insert(
                                    offset,
                                    Field::new(
                                        keyword.to_string(),
                                        Type::Struct(native_struct.clone()),
                                    ),
                                );
                                offset += native_struct.size();
                            } else {
                                // Explicit `Struct(*)` and other types.
                                let native_type: Type = field_type.cast();
                                native.0.insert(
                                    offset,
                                    Field::new(keyword.to_string(), native_type.clone()),
                                );
                                offset += native_type.size();
                            }
                        } else {
                            return Err(format!(
                                "cannot cast field_type from the primitive `{}`",
                                field_type.type_name()
                            )
                            .into());
                        }
                    } else {
                        return Err(format!("failed to retrieve type for `{}`", keyword).into());
                    }
                }
            },
            None => {
                return Err(format!(
                    "unhandled expression found in native block `{}`",
                    native_name
                )
                .into());
            }
        }
    }

    // TODO: Maybe instead return the type?
    context
        .scope_mut()
        .push_constant(native_name, Type::Struct(native));

    Ok(Dynamic::UNIT)
}
