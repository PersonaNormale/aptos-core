// Parts of the file are Copyright (c) The Diem Core Contributors
// Parts of the file are Copyright (c) The Move Contributors
// Parts of the file are Copyright (c) Aptos Foundation
// All Aptos Foundation code and content is licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

//! Converts test arguments to `MoveValue`s of the declared parameter types.

use move_core_types::{account_address::AccountAddress, value::MoveValue};
use move_model::{
    ast::{Address, AttributeValue, ModuleName, Value},
    model::{GlobalEnv, NodeId, QualifiedId, StructId},
    symbol::Symbol,
    ty::{PrimitiveType, Type},
};
use num::{BigInt, ToPrimitive};

/// Conversion failure details for the caller to report at the parameter's location.
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
    UnknownModule {
        module: ModuleName,
    },
    UnknownConstant {
        opt_module: Option<ModuleName>,
        name: Symbol,
    },
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
}

/// Converts an attribute value recursively. Scalar node types retain numeric suffixes;
/// vector literals carry their optional explicit element type separately.
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
                        declared: Type::Vector(Box::new(declared.clone())),
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
        (AttributeValue::Name(node_id, opt_module, name), _) => {
            let (value, resolved_ty) = super::constant_resolution::resolve_test_constant(
                env,
                current_module,
                opt_module,
                *name,
            )?;
            if resolved_ty != *target {
                return Err(ConversionError::TypeMismatch {
                    declared: resolved_ty,
                });
            }
            to_move_value(
                &AttributeValue::Value(*node_id, value),
                target,
                current_module,
                env,
            )
        },
    }
}

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

fn expect_bool(value: &Value) -> Result<bool, ConversionError> {
    let Value::Bool(b) = value else {
        return Err(ConversionError::NotABool);
    };
    Ok(*b)
}

/// Resolves a `Value::Number`, checking it against `target`'s bounds. A concrete node type that
/// disagrees with `target` is a suffix mismatch only if `n` could have been more than one
/// primitive type (`PrimitiveType::possible_int_types`, the same check `translate_number` uses
/// to decide ambiguity). If `n` only fits one type at all (`i256`/`u256`), that's not a suffix,
/// it's just out of `target`'s range.
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
        if declared != target && PrimitiveType::possible_int_types(n.clone()).len() > 1 {
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
