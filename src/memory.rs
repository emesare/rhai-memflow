use std::ops::Add;
use std::str::FromStr;

use memflow::prelude::MemoryView;
use memflow::types::Address;

use rhai::plugin::*;

use super::native::Type;

/*
    When reading i32, u32, u8, u16, u64 you get back an i64 right now,
    this is just to insure you can actually use the numbers (i64 is the system INT for rhai),
    weirdly rhai seems to support operations between numbers, are we doing something wrong?
*/

pub type NativePointer = (Box<Type>, Address);

/// Memory functions.
#[export_module]
#[allow(dead_code)]
#[warn(missing_docs)]
pub mod memory_functions {
    pub mod address_functions {
        use memflow::types::Address;

        pub fn addr(num: rhai::INT) -> Address {
            Address::from(num.abs())
        }

        #[rhai_fn(pure, global, name = "to_string", name = "to_debug")]
        pub fn to_string(addr: &mut Address) -> String {
            format!("{:?}", addr)
        }

        /// Return `true` if two addresses are equal.
        #[rhai_fn(pure, global, name = "==")]
        pub fn eq(addr: &mut Address, addr2: Address) -> bool {
            *addr == addr2
        }

        /// Return `true` if address equals the `num` number.
        #[rhai_fn(pure, global, name = "==")]
        pub fn eq_num(addr: &mut Address, num: rhai::INT) -> bool {
            addr.to_umem() == num.unsigned_abs()
        }

        /// Return an address which is offset from the original address.
        #[rhai_fn(pure, global, name = "+")]
        pub fn add_num(addr: &mut Address, offset: rhai::INT) -> Address {
            addr.add(offset.abs())
        }
    }

    pub mod pointer_functions {
        pub fn ptr(ty: Type, addr: Address) -> NativePointer {
            (Box::new(ty), addr)
        }

        #[rhai_fn(get = "addr")]
        pub fn get_addr((_, addr): NativePointer) -> Address {
            addr
        }

        #[rhai_fn(pure, global, name = "to_string", name = "to_debug")]
        pub fn to_string(ptr: &mut NativePointer) -> String {
            format!("{:?}", ptr)
        }

        /// Return `true` if two pointers are equal.
        #[rhai_fn(pure, global, name = "==")]
        pub fn eq((ty, addr): &mut NativePointer, (ty2, addr2): NativePointer) -> bool {
            *addr == addr2 && *ty == ty2
        }

        /// Return `true` if pointer's address equals the `addr` address.
        #[rhai_fn(pure, global, name = "==")]
        pub fn eq_addr((_, addr): &mut NativePointer, addr2: Address) -> bool {
            *addr == addr2
        }

        /// Return `true` if pointer's address equals the `num` number.
        #[rhai_fn(pure, global, name = "==")]
        pub fn eq_num_addr((_, addr): &mut NativePointer, num: rhai::INT) -> bool {
            addr.to_umem() == num.unsigned_abs()
        }

        /// Return `true` if pointer equals the `ty2` type.
        #[rhai_fn(pure, global, name = "==")]
        pub fn eq_ty((ty, _): &mut NativePointer, ty2: Type) -> bool {
            **ty == ty2
        }

        /// Return a pointer which is offset from the original pointer.
        #[rhai_fn(global, name = "+")]
        pub fn add_num((ty, addr): NativePointer, offset: rhai::INT) -> NativePointer {
            (ty, addr.add(offset.abs()))
        }
    }
}

// TODO: Write `From` helpers to these fns

pub fn read_to_dyn(
    mem: &mut impl MemoryView,
    ty: &Type,
    addr: Address,
) -> Result<Dynamic, Box<EvalAltResult>> {
    match ty {
        Type::UInt8 => match mem.read::<u8>(addr) {
            Ok(uint) => Ok(Dynamic::from_int(uint as rhai::INT)),
            Err(e) => Err(e.as_str().into()),
        },
        Type::UInt16 => match mem.read::<u16>(addr) {
            Ok(uint) => Ok(Dynamic::from_int(uint as rhai::INT)),
            Err(e) => Err(e.as_str().into()),
        },
        Type::Int32 => match mem.read::<i32>(addr) {
            Ok(int) => Ok(Dynamic::from_int(int as rhai::INT)),
            Err(e) => Err(e.as_str().into()),
        },
        Type::UInt32 => match mem.read::<u32>(addr) {
            Ok(uint) => Ok(Dynamic::from_int(uint as rhai::INT)),
            Err(e) => Err(e.as_str().into()),
        },
        Type::Fp32 => match mem.read::<f32>(addr) {
            Ok(fp) => Ok(Dynamic::from_float(fp as f64)),
            Err(e) => Err(e.as_str().into()),
        },
        Type::Address32 => match mem.read_addr32(addr) {
            Ok(addr) => Ok(Dynamic::from(addr)),
            Err(e) => Err(e.as_str().into()),
        },
        Type::Pointer32(ty) => match mem.read_addr32(addr) {
            Ok(addr) => Ok(Dynamic::from((ty.clone(), addr))),
            Err(e) => Err(e.as_str().into()),
        },
        Type::Int64 => match mem.read::<i64>(addr) {
            Ok(int) => Ok(Dynamic::from_int(int)),
            Err(e) => Err(e.as_str().into()),
        },
        // TODO: u64 -> i64 is very bad if the u64 num sets the sign bit, fix!
        Type::UInt64 => match mem.read::<u64>(addr) {
            Ok(uint) => Ok(Dynamic::from_int(uint as rhai::INT)),
            Err(e) => Err(e.as_str().into()),
        },
        Type::Fp64 => match mem.read::<f64>(addr) {
            Ok(fp) => Ok(Dynamic::from_float(fp)),
            Err(e) => Err(e.as_str().into()),
        },
        Type::Address64 => match mem.read_addr64(addr) {
            Ok(addr) => Ok(Dynamic::from(addr)),
            Err(e) => Err(e.as_str().into()),
        },
        Type::Pointer64(ty) => match mem.read_addr64(addr) {
            Ok(addr) => Ok(Dynamic::from((ty.clone(), addr))),
            Err(e) => Err(e.as_str().into()),
        },
        Type::String(len) => match mem.read_char_array(addr, *len as usize) {
            Ok(strn) => match Dynamic::from_str(&strn) {
                Ok(dystr) => Ok(dystr),
                Err(_) => Err("parsing string from shared_mem.borrow()ory failed".into()),
            },
            Err(e) => Err(e.as_str().into()),
        },
        Type::WideString(_) => todo!("Support wide strings"),
        Type::Struct(n) => {
            let mut map = rhai::Map::new();

            for (offset, nf) in n.clone() {
                // TODO: We are doing seperate read calls for each item, we instead should read up to each padding jump.
                let field_val = read_to_dyn(mem, &nf.ty, addr + offset)?;
                map.insert(nf.name.into(), field_val);
            }

            Ok(Dynamic::from_map(map))
        }
        Type::Collection(ty, num) => {
            let mut arr = rhai::Array::with_capacity(*num as usize);
            let size = ty.size();

            let mut current = 0;
            while current < *num {
                let item_addr = addr + (current * size);
                current += 1;
                // TODO: We are doing seperate read calls for each item, we instead should read the entire list and then iterate inside of it.
                arr.push(read_to_dyn(mem, ty, item_addr)?);
            }

            Ok(Dynamic::from_array(arr))
        }
    }
}

pub fn write_from_dyn(
    mem: &mut impl MemoryView,
    ty: &Type,
    addr: Address,
    val: Dynamic,
) -> Result<(), Box<EvalAltResult>> {
    // TODO: Add special logic to write `Address` and other non numerical types.
    match ty {
        Type::UInt8 => mem
            .write(addr, &(val.as_int().unwrap() as u8))
            .map_err(|e| Box::new(e.as_str().into())),
        Type::UInt16 => mem
            .write(addr, &(val.as_int().unwrap() as u16))
            .map_err(|e| Box::new(e.as_str().into())),
        Type::Int32 => mem
            .write(addr, &(val.as_int().unwrap() as i32))
            .map_err(|e| Box::new(e.as_str().into())),
        Type::UInt32 | Type::Address32 => mem
            .write(addr, &(val.as_int().unwrap() as u32))
            .map_err(|e| Box::new(e.as_str().into())),
        Type::Fp32 => mem
            .write(addr, &(val.as_int().unwrap() as f32))
            .map_err(|e| Box::new(e.as_str().into())),
        Type::Pointer32(pty) => match mem.read_addr32(addr) {
            Ok(ptr) => write_from_dyn(mem, pty, ptr, val),
            Err(e) => Err(format!("read pointer to write: {}", e).into()),
        },
        Type::Int64 => mem
            .write(addr, &(val.as_int().unwrap() as rhai::INT))
            .map_err(|e| Box::new(e.as_str().into())),
        // TODO: u64 -> i64 is very bad if the u64 num sets the sign bit, fix!
        Type::UInt64 | Type::Address64 => mem
            .write(addr, &(val.as_int().unwrap() as u64))
            .map_err(|e| Box::new(e.as_str().into())),
        Type::Fp64 => mem
            .write(addr, &(val.as_int().unwrap() as f64))
            .map_err(|e| Box::new(e.as_str().into())),
        Type::Pointer64(pty) => match mem.read_addr64(addr) {
            Ok(ptr) => write_from_dyn(mem, pty, ptr, val),
            Err(e) => Err(format!("read pointer to write: {}", e).into()),
        },
        Type::String(len) => match val.into_string() {
            Ok(str) => {
                if (*len as usize) < str.len() {
                    if str.is_ascii() {
                        mem.write(addr, str.as_bytes())
                            .map_err(|e| Box::new(e.as_str().into()))
                    } else {
                        // i mean we COULD allow writing but this would confuse users (able to write utf8, no utf8 reading)
                        todo!("return error, not ascii")
                    }
                } else {
                    todo!("return error, string too long")
                }
            }
            Err(_) => todo!("return error"),
        },
        Type::WideString(_) => todo!("Support wide strings"),
        Type::Struct(n) => {
            if let Some(map) = val.try_cast::<rhai::Map>() {
                // TODO: Wasteful clone due to ref.
                for (offset, nf) in n.clone() {
                    match map.get(nf.name.as_str()) {
                        Some(val) => write_from_dyn(mem, &nf.ty, addr + offset, val.clone())?,
                        None => todo!("return error"),
                    }
                }
            } else {
                todo!("return error")
            }

            Ok(())
        }
        Type::Collection(ty, _) => {
            let arr = val.into_array().unwrap();

            // TODO: Err if vec length mismatch (more or less of what the native type expects)
            let size = ty.size();
            arr.into_iter().enumerate().for_each(|(current, val)| {
                let item_addr = addr + (current as u32 * size);
                write_from_dyn(mem, ty, item_addr, val).unwrap();
            });

            Ok(())
        }
    }
}
