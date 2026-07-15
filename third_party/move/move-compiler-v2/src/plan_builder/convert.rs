// Parts of the file are Copyright (c) The Diem Core Contributors
// Parts of the file are Copyright (c) The Move Contributors
// Parts of the file are Copyright (c) Aptos Foundation
// All Aptos Foundation code and content is licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

//! Value conversion helpers for `#[expected_failure(...)]` and `#[test(...)]` attribute
//! payloads.

use super::error::{Checked, ErrorReported};
use codespan_reporting::diagnostic::Severity;
use legacy_move_compiler::shared::known_attributes::TestingAttribute;
use move_core_types::{identifier::Identifier, language_storage::ModuleId, value::MoveValue};
use move_model::{
    ast::{Address, Attribute, AttributeValue, ModuleName, Value},
    model::{GlobalEnv, Loc},
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
    let module_env = match opt_module_name {
        Some(module_name) => match env.find_module(module_name) {
            Some(module_env) => module_env,
            None => {
                env.error(
                    vloc,
                    &format!(
                        "cannot find module `{}` in this scope",
                        module_name.display_full(env)
                    ),
                );
                return Err(ErrorReported);
            },
        },
        None => env
            .find_module(current_module)
            .expect("current module exists"),
    };
    let module_name = opt_module_name.as_ref().unwrap_or(current_module).clone();
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

/// Converts an attribute payload into the `MoveValue` it denotes.
///
/// Not `std::convert::TryFrom`: resolving a symbolic address alias needs `env`, and
/// `TryFrom`'s signature has no room for it. Returns `Err` without emitting a diagnostic; the
/// caller owns the surrounding assignment's location and reports the failure itself.
pub(super) trait ToMoveValue {
    fn to_move_value(&self, env: &GlobalEnv) -> Checked<MoveValue>;
}

impl ToMoveValue for AttributeValue {
    fn to_move_value(&self, env: &GlobalEnv) -> Checked<MoveValue> {
        match self {
            AttributeValue::Value(_id, Value::Address(addr)) => match addr {
                Address::Numerical(num) => Some(*num),
                Address::Symbolic(sym) => env.resolve_address_alias(*sym),
            }
            .map(MoveValue::Address)
            .ok_or(ErrorReported),
            _ => Err(ErrorReported),
        }
    }
}
