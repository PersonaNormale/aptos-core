// Parts of the file are Copyright (c) The Diem Core Contributors
// Parts of the file are Copyright (c) The Move Contributors
// Parts of the file are Copyright (c) Aptos Foundation
// All Aptos Foundation code and content is licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

//! Value conversion helpers for `#[expected_failure(...)]` and `#[test(...)]` attribute
//! payloads.

use super::error::{Checked, ErrorReported};
use codespan_reporting::diagnostic::Severity;
use legacy_move_compiler::shared::known_attributes::TestingAttribute;
use move_binary_format::file_format::Visibility;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::ModuleId,
    value::{MoveStruct, MoveValue},
};
use move_model::{
    ast::{Address, Attribute, AttributeValue, ModuleName, PackFields, Value},
    model::{
        GlobalEnv, Loc, ModuleEnv, ModuleId as ModelModuleId, NodeId, QualifiedId, StructEnv,
        StructId,
    },
    symbol::Symbol,
    ty::{PrimitiveType, Type},
};
use num::{BigInt, ToPrimitive};
use std::collections::BTreeMap;

/// One variant per `TestingAttribute::expected_failure_cases()` entry.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ExpectedFailureKind {
    AbortCode,
    ArithmeticError,
    OutOfGas,
    VectorError,
    MajorStatus,
}

impl ExpectedFailureKind {
    const ALL: [ExpectedFailureKind; 5] = [
        ExpectedFailureKind::AbortCode,
        ExpectedFailureKind::ArithmeticError,
        ExpectedFailureKind::OutOfGas,
        ExpectedFailureKind::VectorError,
        ExpectedFailureKind::MajorStatus,
    ];

    pub(super) fn attr_name(self) -> &'static str {
        match self {
            ExpectedFailureKind::AbortCode => TestingAttribute::ABORT_CODE_NAME,
            ExpectedFailureKind::ArithmeticError => TestingAttribute::ARITHMETIC_ERROR_NAME,
            ExpectedFailureKind::OutOfGas => TestingAttribute::OUT_OF_GAS_NAME,
            ExpectedFailureKind::VectorError => TestingAttribute::VECTOR_ERROR_NAME,
            ExpectedFailureKind::MajorStatus => TestingAttribute::MAJOR_STATUS_NAME,
        }
    }
}

/// Picks the single failure-kind sub-attribute out of `attrs`, erroring if the count isn't
/// exactly 1.
pub(super) fn resolve_expected_failure_kind<'a>(
    env: &GlobalEnv,
    attr_loc: Loc,
    attrs: &mut BTreeMap<String, &'a Attribute>,
) -> Checked<(ExpectedFailureKind, &'a Attribute)> {
    let mut matches: Vec<(ExpectedFailureKind, &'a Attribute)> = ExpectedFailureKind::ALL
        .iter()
        .filter_map(|kind| {
            let attr = attrs.remove(kind.attr_name())?;
            Some((*kind, attr))
        })
        .collect();
    if matches.len() != 1 {
        let msg = format!(
            "`#[expected_failure(...)]` must specify exactly one failure kind, found {}",
            matches.len()
        );
        let note = format!(
            "expected one of: {}",
            TestingAttribute::expected_failure_cases()
                .to_vec()
                .join(", ")
        );
        env.diag_with_notes(Severity::Error, &attr_loc, &msg, vec![note]);
        return Err(ErrorReported);
    }
    Ok(matches.pop().expect("matches.len() == 1 checked above"))
}

/// Errors if `attr` carries any `(...)` parameters. Some failure kinds, such as `out_of_gas`,
/// are markers and take none.
pub(super) fn ensure_no_attribute_params(
    env: &GlobalEnv,
    kind: &str,
    attr: &Attribute,
) -> Checked<()> {
    match attr {
        Attribute::Apply {
            node_id,
            name,
            attrs: params,
            ..
        } => {
            assert!(env.symbol_pool().string(*name).to_string() == kind);
            if params.is_empty() {
                Ok(())
            } else {
                let loc = env.get_node_loc(*node_id);
                env.error(&loc, &format!("`{}` does not accept parameters", kind));
                Err(ErrorReported)
            }
        },
        Attribute::Assign { node_id, name, .. } => {
            assert!(env.symbol_pool().string(*name).to_string() == kind);
            let loc = env.get_node_loc(*node_id);
            env.error(&loc, &format!("`{}` does not take a value", kind));
            Err(ErrorReported)
        },
    }
}

/// Errors unless `attr` is `kind = <value>`, otherwise returns the assignment's location and
/// value.
pub(super) fn expect_assigned_value(
    env: &GlobalEnv,
    kind: &str,
    attr: &Attribute,
) -> Checked<(Loc, AttributeValue)> {
    match attr {
        Attribute::Assign {
            node_id,
            name,
            value,
            ..
        } => {
            assert!(env.symbol_pool().string(*name).to_string() == kind);
            let loc = env.get_node_loc(*node_id);
            Ok((loc, value.clone()))
        },
        Attribute::Apply { node_id, .. } => {
            let loc = env.get_node_loc(*node_id);
            env.error(&loc, &format!("expected `{}=...`", kind));
            Err(ErrorReported)
        },
    }
}

/// Resolves `location = <module>` into the `ModuleId` it names.
pub(super) fn resolve_location(env: &GlobalEnv, attr: &Attribute) -> Checked<ModuleId> {
    let (loc, value) = expect_assigned_value(env, TestingAttribute::ERROR_LOCATION, attr)?;
    match value {
        AttributeValue::Name(id, opt_module_name, sym) => {
            let vloc = env.get_node_loc(id);
            let module_id_opt = resolve_module_id(env, vloc.clone(), opt_module_name);
            if !sym.display(env.symbol_pool()).to_string().is_empty() || module_id_opt.is_none() {
                env.error_with_labels(&loc, "invalid attribute value", vec![(
                    vloc,
                    "expected a module identifier, e.g. `std::vector`".to_string(),
                )]);
            }
            module_id_opt.ok_or(ErrorReported)
        },
        AttributeValue::Value(id, _val) => {
            let vloc = env.get_node_loc(id);
            env.error_with_labels(&loc, "invalid attribute value", vec![(
                vloc,
                "expected a module identifier, e.g. `std::vector`".to_string(),
            )]);
            Err(ErrorReported)
        },
        AttributeValue::Vector(id, _elems) => {
            let vloc = env.get_node_loc(id);
            env.error_with_labels(&loc, "invalid attribute value", vec![(
                vloc,
                "expected a module identifier, e.g. `std::vector`".to_string(),
            )]);
            Err(ErrorReported)
        },
        AttributeValue::Pack(id, ..) => {
            let vloc = env.get_node_loc(id);
            env.error_with_labels(&loc, "invalid attribute value", vec![(
                vloc,
                "expected a module identifier, e.g. `std::vector`".to_string(),
            )]);
            Err(ErrorReported)
        },
    }
}

/// Resolves an `abort_code`/`minor_status` payload: either a `u64` literal, or a path to a
/// `u64` module constant.
pub(super) fn resolve_u64_constant_or_literal(
    env: &GlobalEnv,
    current_module: &ModuleName,
    value: &AttributeValue,
) -> Checked<(Loc, Option<ModuleId>, u64)> {
    match value {
        AttributeValue::Value(id, val) => {
            let loc = env.get_node_loc(*id);
            let (vloc, u) = literal_value_to_u64(env, loc, val)?;
            Ok((vloc, None, u))
        },
        AttributeValue::Name(id, opt_module_name, member) => {
            let vloc = env.get_node_loc(*id);
            let (module_name, ty, value) =
                resolve_named_constant(env, current_module, opt_module_name, *member, &vloc)?;
            let mod_id = resolve_module_id(env, vloc.clone(), opt_module_name.clone());
            let u = constant_value_to_u64(env, &vloc, &module_name, *member, ty, value)?;
            Ok((vloc, mod_id, u))
        },
        AttributeValue::Vector(id, _elems) => {
            let vloc = env.get_node_loc(*id);
            env.error(&vloc, "expected a `u64` literal or a `u64` module constant");
            Err(ErrorReported)
        },
        AttributeValue::Pack(id, ..) => {
            let vloc = env.get_node_loc(*id);
            env.error(&vloc, "expected a `u64` literal or a `u64` module constant");
            Err(ErrorReported)
        },
    }
}

/// Looks up the module and named constant a `module::CONST` path refers to, returning its
/// resolved type and value.
fn resolve_named_constant(
    env: &GlobalEnv,
    current_module: &ModuleName,
    opt_module_name: &Option<ModuleName>,
    member: Symbol,
    vloc: &Loc,
) -> Checked<(ModuleName, Type, Value)> {
    let module_env = match resolve_module_env(env, current_module, opt_module_name) {
        Some(module_env) => module_env,
        None => {
            let name = opt_module_name.as_ref().unwrap_or(current_module);
            env.error(
                vloc,
                &format!(
                    "cannot find module `{}` in this scope",
                    name.display_full(env)
                ),
            );
            return Err(ErrorReported);
        },
    };
    let module_name = module_env.get_name().clone();
    match module_env.find_named_constant(member) {
        Some(named_constant_env) => Ok((
            module_name,
            named_constant_env.get_type(),
            named_constant_env.get_value(),
        )),
        None => {
            env.error(
                vloc,
                &format!(
                    "cannot find constant `{}` in module `{}`",
                    member.display(env.symbol_pool()),
                    module_name.display_full(env)
                ),
            );
            Err(ErrorReported)
        },
    }
}

/// Validates that a resolved named constant's value fits in a `u64`. Accepts both
/// `u64`-typed constants and not-yet-narrowed `num`-typed literals.
fn constant_value_to_u64(
    env: &GlobalEnv,
    vloc: &Loc,
    module_name: &ModuleName,
    member: Symbol,
    ty: Type,
    value: Value,
) -> Checked<u64> {
    let path = format!(
        "{}::{}",
        module_name.display_full(env),
        member.display(env.symbol_pool())
    );
    let Value::Number(u) = value else {
        env.error_with_notes(
            vloc,
            &format!("constant `{}` is not a numeric value", path),
            vec!["only `u64` constants are permitted here".to_string()],
        );
        return Err(ErrorReported);
    };
    match ty {
        Type::Primitive(PrimitiveType::U64) if u <= BigInt::from(u64::MAX) => {
            Ok(u.to_u64().expect("u <= u64::MAX checked in guard"))
        },
        // The type checker already committed this constant to `u64`, so an out-of-range value
        // here means the checker's own invariant was violated, not a user error.
        Type::Primitive(PrimitiveType::U64) => {
            env.diag(
                Severity::Bug,
                vloc,
                &format!("constant `{}` value is out of range for `u64`", path),
            );
            Err(ErrorReported)
        },
        Type::Primitive(PrimitiveType::Num) if u <= BigInt::from(u64::MAX) => {
            Ok(u.to_u64().expect("u <= u64::MAX checked in guard"))
        },
        Type::Primitive(PrimitiveType::Num) => {
            env.error(
                vloc,
                &format!("constant `{}` value is out of range for `u64`", path),
            );
            Err(ErrorReported)
        },
        _ => {
            env.error_with_notes(
                vloc,
                &format!("constant `{}` does not have type `u64`", path),
                vec!["only `u64` constants are permitted here".to_string()],
            );
            Err(ErrorReported)
        },
    }
}

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

fn resolve_module_id(env: &GlobalEnv, _vloc: Loc, module: Option<ModuleName>) -> Option<ModuleId> {
    let module_name = module?;
    let addr = module_name.addr();
    let sym = module_name.name();
    let sym_core_id = Identifier::new(env.symbol_pool().string(sym).to_string())
        .expect("symbol pool string is a valid identifier");
    let account_address = match addr {
        Address::Numerical(addr) => Some(*addr),
        Address::Symbolic(sym) => env.resolve_address_alias(*sym),
    }?;
    Some(ModuleId::new(account_address, sym_core_id))
}

fn literal_value_to_u64(env: &GlobalEnv, loc: Loc, value: &Value) -> Checked<(Loc, u64)> {
    match value {
        Value::Number(u) if u <= &BigInt::from(u64::MAX) => {
            Ok((loc, u.to_u64().expect("u <= u64::MAX checked in guard")))
        },
        _ => {
            env.error(&loc, "expected a `u64` literal value");
            Err(ErrorReported)
        },
    }
}

/// Errors if `location` is absent, otherwise unwraps it.
pub(super) fn require_location_attr<T>(
    env: &GlobalEnv,
    loc: Loc,
    attr: &str,
    location: Option<T>,
) -> Checked<T> {
    match location {
        Some(value) => Ok(value),
        None => {
            env.error(
                &loc,
                &format!(
                    "expected `{}` following `{}`",
                    TestingAttribute::ERROR_LOCATION,
                    attr
                ),
            );
            Err(ErrorReported)
        },
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
    TypeMismatch { declared: Type },
    OutOfRange { min: BigInt, max: BigInt },
    UnsupportedParameterType,
    UnknownStruct,
    StructNotConstructible { struct_id: QualifiedId<StructId> },
    ConstructorMismatch { expected_positional: bool },
    MissingFields(Vec<Symbol>),
    UnknownField(Symbol),
    FieldCountMismatch { expected: usize, found: usize },
}

/// Converts a `#[test(...)]` attribute value into the `MoveValue` a parameter of type `target`
/// expects. `value` carries its own `NodeId` at every nesting level (`AttributeValue::Value` and
/// `AttributeValue::Vector` both do), so explicit-type and suffix checks work identically whether
/// `value` is the whole attribute assignment or an element nested inside a vector.
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
        (AttributeValue::Vector(node_id, elems), Type::Vector(inner)) => {
            // `translate_attribute_value` (module_builder.rs) only calls `update_node_type` on a
            // vector's own node when the literal carried an explicit `vector<T>[...]` annotation;
            // otherwise the node keeps the `Type::Tuple(vec![])` sentinel `env.new_node` initialized
            // it with. That sentinel means "no explicit type to check", exactly mirroring how an
            // unsuffixed scalar's node type stays an unresolved `Type::Var` until conversion time.
            let declared = env.get_node_type(*node_id);
            if declared != Type::Tuple(vec![]) && declared != Type::Vector(inner.clone()) {
                return Err(ConversionError::TypeMismatch { declared });
            }
            let converted = elems
                .iter()
                .map(|elem| to_move_value(elem, inner, current_module, env))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(MoveValue::Vector(converted))
        },
        (
            AttributeValue::Pack(_node_id, opt_module, name, opt_type_args, fields),
            Type::Struct(target_mid, target_sid, target_args),
        ) => to_move_struct(
            opt_module,
            *name,
            opt_type_args,
            fields,
            *target_mid,
            *target_sid,
            target_args,
            current_module,
            env,
        ),
        (AttributeValue::Value(node_id, _), _) => Err(ConversionError::TypeMismatch {
            declared: env.get_node_type(*node_id),
        }),
        (AttributeValue::Vector(node_id, _), _) => Err(ConversionError::TypeMismatch {
            declared: env.get_node_type(*node_id),
        }),
        (AttributeValue::Pack(node_id, ..), _) => Err(ConversionError::TypeMismatch {
            declared: env.get_node_type(*node_id),
        }),
        (AttributeValue::Name(..), _) => Err(ConversionError::UnsupportedParameterType),
    }
}

/// The `Pack` counterpart of `to_move_value`: resolves struct identity, checks visibility and
/// type-argument agreement, checks field completeness, and recurses per field.
fn to_move_struct(
    opt_module: &Option<ModuleName>,
    name: Symbol,
    opt_type_args: &Option<Vec<Type>>,
    fields: &PackFields,
    target_mid: ModelModuleId,
    target_sid: StructId,
    target_args: &[Type],
    current_module: &ModuleName,
    env: &GlobalEnv,
) -> Result<MoveValue, ConversionError> {
    let module_env = resolve_module_env(env, current_module, opt_module)
        .ok_or(ConversionError::UnknownStruct)?;
    let struct_env = module_env
        .find_struct(name)
        .ok_or(ConversionError::UnknownStruct)?;
    if (struct_env.module_env.get_id(), struct_env.get_id()) != (target_mid, target_sid) {
        return Err(ConversionError::TypeMismatch {
            declared: Type::Struct(struct_env.module_env.get_id(), struct_env.get_id(), vec![]),
        });
    }

    let calling_module_env = env
        .find_module(current_module)
        .expect("current module exists in the model that is compiling it");
    check_construction_visibility(env, &struct_env, &calling_module_env)?;

    let effective_args: Vec<Type> = match opt_type_args {
        Some(explicit) if explicit != target_args => {
            return Err(ConversionError::TypeMismatch {
                declared: Type::Struct(target_mid, target_sid, explicit.clone()),
            });
        },
        _ => target_args.to_vec(),
    };

    let is_positional_struct = struct_env
        .get_fields()
        .next()
        .map(|f| f.is_positional())
        .unwrap_or(false);
    let is_empty = struct_env.is_empty_struct();

    match fields {
        PackFields::Named(named) => {
            if is_empty && named.is_empty() {
                return Ok(MoveValue::Struct(MoveStruct::new(vec![MoveValue::Bool(
                    false,
                )])));
            }
            if is_positional_struct {
                return Err(ConversionError::ConstructorMismatch {
                    expected_positional: true,
                });
            }
            let by_name: BTreeMap<Symbol, &AttributeValue> =
                named.iter().map(|(s, v)| (*s, v)).collect();
            let declared: Vec<_> = struct_env.get_fields().collect();
            let mut missing = Vec::new();
            let mut values = Vec::new();
            for field in &declared {
                match by_name.get(&field.get_name()) {
                    Some(v) => values.push((field, *v)),
                    None => missing.push(field.get_name()),
                }
            }
            if !missing.is_empty() {
                return Err(ConversionError::MissingFields(missing));
            }
            let declared_names: std::collections::BTreeSet<Symbol> =
                declared.iter().map(|f| f.get_name()).collect();
            if let Some((unknown, _)) = named.iter().find(|(s, _)| !declared_names.contains(s)) {
                return Err(ConversionError::UnknownField(*unknown));
            }
            let converted = values
                .into_iter()
                .map(|(field, v)| {
                    to_move_value(
                        v,
                        &field.get_type().instantiate(&effective_args),
                        current_module,
                        env,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(MoveValue::Struct(MoveStruct::new(converted)))
        },
        PackFields::Positional(positional) => {
            if is_empty && positional.is_empty() {
                return Ok(MoveValue::Struct(MoveStruct::new(vec![MoveValue::Bool(
                    false,
                )])));
            }
            if !is_positional_struct {
                return Err(ConversionError::ConstructorMismatch {
                    expected_positional: false,
                });
            }
            let declared: Vec<_> = struct_env.get_fields().collect();
            if positional.len() != declared.len() {
                return Err(ConversionError::FieldCountMismatch {
                    expected: declared.len(),
                    found: positional.len(),
                });
            }
            let converted = declared
                .iter()
                .zip(positional.iter())
                .map(|(field, v)| {
                    to_move_value(
                        v,
                        &field.get_type().instantiate(&effective_args),
                        current_module,
                        env,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(MoveValue::Struct(MoveStruct::new(converted)))
        },
    }
}

/// Attribute-driven equivalent of `function_checker.rs::check_struct_op`'s Pack-visibility rule,
/// re-implemented independently since attribute conversion never builds a real `Exp`/
/// `Operation::Pack` node for that post-pass to see. Mirrors its exact policy, including the
/// language-version gate: before `language_version_for_public_struct`, every struct is treated as
/// `Private` regardless of its declared visibility, so a struct this program can't legally make
/// `public` yet can't be attribute-constructed across modules either.
fn check_construction_visibility(
    env: &GlobalEnv,
    struct_env: &StructEnv,
    current_module: &ModuleEnv,
) -> Result<(), ConversionError> {
    if struct_env.module_env.get_id() == current_module.get_id() {
        return Ok(());
    }
    let struct_visibility_supported = env.language_version().language_version_for_public_struct();
    let visibility = if struct_visibility_supported {
        struct_env.get_visibility()
    } else {
        Visibility::Private
    };
    let allowed = match visibility {
        Visibility::Public => true,
        Visibility::Friend => struct_env.module_env.has_friend(&current_module.get_id()),
        Visibility::Private => false,
    };
    if allowed {
        Ok(())
    } else {
        Err(ConversionError::StructNotConstructible {
            struct_id: struct_env
                .module_env
                .get_id()
                .qualified(struct_env.get_id()),
        })
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
