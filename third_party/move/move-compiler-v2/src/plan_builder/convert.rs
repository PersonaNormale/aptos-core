// Parts of the file are Copyright (c) The Diem Core Contributors
// Parts of the file are Copyright (c) The Move Contributors
// Parts of the file are Copyright (c) Aptos Foundation
// All Aptos Foundation code and content is licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

//! Converts a `#[test(...)]` attribute value into the `MoveValue` a declared parameter type
//! expects: scalar/vector dispatch and the leaf primitive conversions.

use move_core_types::{account_address::AccountAddress, value::MoveValue};
use move_model::{
    ast::{Address, AttributeValue, ModuleName, Value},
    model::{GlobalEnv, ModuleEnv, NodeId, QualifiedId, StructId},
    symbol::Symbol,
    ty::{PrimitiveType, Type},
};
use num::{BigInt, ToPrimitive};

/// Resolves an optional explicit module name to the `ModuleEnv` it names, falling back to
/// `current_module` when `opt_module_name` is `None` (an unqualified reference). Returns `None`
/// without emitting a diagnostic; callers phrase their own "not found" message, since an
/// unqualified name failing to resolve and a qualified name failing to resolve want different
/// wording (`resolve_named_constant`'s "cannot find module" vs. the struct-Pack path's
/// "undeclared struct").
pub(super) fn resolve_module_env<'env>(
    env: &'env GlobalEnv,
    current_module: &ModuleName,
    opt_module_name: &Option<ModuleName>,
) -> Option<ModuleEnv<'env>> {
    match opt_module_name {
        Some(module_name) => env.find_module(module_name),
        None => env.find_module(current_module),
    }
}

/// Why `to_move_value` could not produce a `MoveValue` for a given parameter type. Carries
/// enough detail for the caller to phrase a specific diagnostic; `to_move_value` and its
/// helpers never emit diagnostics themselves, since only the caller knows the parameter's own
/// location to label.
pub(super) enum ConversionError {
    NotANumber,
    NotAnAddress,
    NotABool,
    TypeMismatch {
        declared: Type,
    },
    OutOfRange {
        min: BigInt,
        max: BigInt,
    },
    UnsupportedParameterType,
    UnknownStruct,
    VariantOnNonEnum {
        struct_id: QualifiedId<StructId>,
        variant: Symbol,
    },
    VariantRequired {
        struct_id: QualifiedId<StructId>,
    },
    UnknownVariant {
        struct_id: QualifiedId<StructId>,
        variant: Symbol,
    },
    StructNotConstructible {
        struct_id: QualifiedId<StructId>,
    },
    ConstructorMismatch {
        expected_positional: bool,
    },
    MissingFields(Vec<Symbol>),
    UnknownField(Symbol),
    FieldCountMismatch {
        expected: usize,
        found: usize,
    },
    InvalidUtf8 {
        node_id: NodeId,
    },
    OptionVecTooLong {
        node_id: NodeId,
    },
    InvalidAscii {
        node_id: NodeId,
    },
}

/// Converts a `#[test(...)]` attribute value into the `MoveValue` a parameter of type `target`
/// expects. `value` carries its own `NodeId` at every nesting level, so a suffixed scalar's suffix
/// check works identically whether `value` is the whole attribute assignment or an element nested
/// inside a vector. `AttributeValue::Vector` carries its own explicit element type directly
/// (`None` when the literal had no `vector<T>[...]` annotation), rather than through its `NodeId`.
///
/// Not `std::convert::TryFrom`: resolving a symbolic address alias needs `env`, and `TryFrom`'s
/// signature has no room for it. Emits no diagnostic; the caller owns the parameter's location and
/// reports the failure itself.
pub(super) fn to_move_value(
    value: &AttributeValue,
    target: &Type,
    current_module: &ModuleName,
    env: &GlobalEnv,
) -> Result<MoveValue, ConversionError> {
    match (value, target) {
        (AttributeValue::Value(node_id, val), Type::Primitive(p)) => {
            to_move_scalar(val, *node_id, *p, env)
        },
        (AttributeValue::Vector(_node_id, explicit_ty, elems), Type::Vector(inner)) => {
            if let Some(declared) = explicit_ty {
                if *declared != **inner {
                    return Err(ConversionError::TypeMismatch {
                        declared: declared.clone(),
                    });
                }
            }
            let converted = elems
                .iter()
                .map(|elem| to_move_value(elem, inner, current_module, env))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(MoveValue::Vector(converted))
        },
        (
            AttributeValue::Pack(_node_id, opt_module, name, variant, opt_type_args, fields),
            Type::Struct(target_mid, target_sid, target_args),
        ) => super::struct_conversion::to_move_struct(
            opt_module,
            *name,
            variant,
            opt_type_args,
            fields,
            *target_mid,
            *target_sid,
            target_args,
            current_module,
            env,
        ),
        (AttributeValue::Value(_, Value::ByteArray(bytes)), Type::Vector(inner))
            if **inner == Type::Primitive(PrimitiveType::U8) =>
        {
            Ok(MoveValue::Vector(
                bytes.iter().map(|b| MoveValue::U8(*b)).collect(),
            ))
        },
        (
            AttributeValue::Value(node_id, _)
            | AttributeValue::Vector(node_id, ..)
            | AttributeValue::Pack(node_id, ..),
            _,
        ) => Err(ConversionError::TypeMismatch {
            declared: env.get_node_type(*node_id),
        }),
        (AttributeValue::Name(..), _) => Err(ConversionError::UnsupportedParameterType),
    }
}

/// The scalar leaf of `to_move_value`: per-primitive conversion, dispatched from the top level or
/// from the `Vector` arm for each element.
fn to_move_scalar(
    value: &Value,
    node_id: NodeId,
    target: PrimitiveType,
    env: &GlobalEnv,
) -> Result<MoveValue, ConversionError> {
    match target {
        PrimitiveType::Address => expect_address(value, env).map(MoveValue::Address),
        PrimitiveType::Signer => expect_address(value, env).map(MoveValue::Signer),
        PrimitiveType::Bool => expect_bool(value).map(MoveValue::Bool),
        PrimitiveType::U8 => expect_bounded_number(value, node_id, PrimitiveType::U8, env)
            .map(|n| MoveValue::U8(n.to_u8().expect("bounds already checked"))),
        PrimitiveType::U16 => expect_bounded_number(value, node_id, PrimitiveType::U16, env)
            .map(|n| MoveValue::U16(n.to_u16().expect("bounds already checked"))),
        PrimitiveType::U32 => expect_bounded_number(value, node_id, PrimitiveType::U32, env)
            .map(|n| MoveValue::U32(n.to_u32().expect("bounds already checked"))),
        PrimitiveType::U64 => expect_bounded_number(value, node_id, PrimitiveType::U64, env)
            .map(|n| MoveValue::U64(n.to_u64().expect("bounds already checked"))),
        PrimitiveType::U128 => expect_bounded_number(value, node_id, PrimitiveType::U128, env)
            .map(|n| MoveValue::U128(n.to_u128().expect("bounds already checked"))),
        PrimitiveType::I8 => expect_bounded_number(value, node_id, PrimitiveType::I8, env)
            .map(|n| MoveValue::I8(n.to_i8().expect("bounds already checked"))),
        PrimitiveType::I16 => expect_bounded_number(value, node_id, PrimitiveType::I16, env)
            .map(|n| MoveValue::I16(n.to_i16().expect("bounds already checked"))),
        PrimitiveType::I32 => expect_bounded_number(value, node_id, PrimitiveType::I32, env)
            .map(|n| MoveValue::I32(n.to_i32().expect("bounds already checked"))),
        PrimitiveType::I64 => expect_bounded_number(value, node_id, PrimitiveType::I64, env)
            .map(|n| MoveValue::I64(n.to_i64().expect("bounds already checked"))),
        PrimitiveType::I128 => expect_bounded_number(value, node_id, PrimitiveType::I128, env)
            .map(|n| MoveValue::I128(n.to_i128().expect("bounds already checked"))),
        PrimitiveType::U256 => expect_bounded_number(value, node_id, PrimitiveType::U256, env)
            .map(|n| MoveValue::U256(n.clone().try_into().expect("bounds already checked"))),
        PrimitiveType::I256 => expect_bounded_number(value, node_id, PrimitiveType::I256, env)
            .map(|n| MoveValue::I256(n.clone().try_into().expect("bounds already checked"))),
        PrimitiveType::Num | PrimitiveType::Range | PrimitiveType::EventStore => {
            Err(ConversionError::UnsupportedParameterType)
        },
    }
}

/// Resolves a `Value::Address`, following a symbolic alias if needed. Used for both `address`
/// and `signer` parameters, which differ only in which `MoveValue` variant wraps the address.
fn expect_address(value: &Value, env: &GlobalEnv) -> Result<AccountAddress, ConversionError> {
    let Value::Address(addr) = value else {
        return Err(ConversionError::NotAnAddress);
    };
    match addr {
        Address::Numerical(addr) => Ok(*addr),
        Address::Symbolic(sym) => env
            .resolve_address_alias(*sym)
            .ok_or(ConversionError::NotAnAddress),
    }
}

/// Resolves a `Value::Bool`. Unlike a numeric literal, `true`/`false` never needs a suffix check:
/// the model builder already resolves a bool literal to a fully concrete `Type::Primitive(Bool)`
/// with no unsuffixed-default ambiguity, so there is nothing left to verify here beyond the value
/// kind itself.
fn expect_bool(value: &Value) -> Result<bool, ConversionError> {
    let Value::Bool(b) = value else {
        return Err(ConversionError::NotABool);
    };
    Ok(*b)
}

/// Resolves a `Value::Number`, checking it against `target`'s bounds. If the literal carried an
/// explicit suffix (`env.get_node_type(node_id)` is a concrete `Type::Primitive`), that suffix
/// must agree with `target` first. An unsuffixed literal's node type is an unresolved
/// `Type::Var`, since the throwaway `ExpTranslator` that typed it never runs the finalization
/// pass that would default it to `u64`; that case skips the suffix check entirely, the same
/// way an unsuffixed literal in an ordinary function call is free to take on its target type.
fn expect_bounded_number<'a>(
    value: &'a Value,
    node_id: NodeId,
    target: PrimitiveType,
    env: &GlobalEnv,
) -> Result<&'a BigInt, ConversionError> {
    let Value::Number(n) = value else {
        return Err(ConversionError::NotANumber);
    };
    if let Type::Primitive(declared) = env.get_node_type(node_id) {
        if declared != target {
            return Err(ConversionError::TypeMismatch {
                declared: Type::Primitive(declared),
            });
        }
    }
    let min = target.get_min_value().expect("numeric target has a min");
    let max = target.get_max_value().expect("numeric target has a max");
    if n < &min || n > &max {
        return Err(ConversionError::OutOfRange { min, max });
    }
    Ok(n)
}
