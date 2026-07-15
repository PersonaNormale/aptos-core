// Parts of the file are Copyright (c) The Diem Core Contributors
// Parts of the file are Copyright (c) The Move Contributors
// Parts of the file are Copyright (c) Aptos Foundation
// All Aptos Foundation code and content is licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

//! Resolves a single `#[expected_failure(...)]` attribute into an `ExpectedFailure`.

use super::{
    convert::{
        ensure_no_attribute_params, expect_assigned_value, require_location_attr,
        resolve_expected_failure_kind, resolve_location, resolve_u64_constant_or_literal,
        ExpectedFailureKind,
    },
    error::{Checked, ErrorReported},
};
use codespan_reporting::diagnostic::Severity;
use legacy_move_compiler::{
    shared::known_attributes::{AttributeKind, TestingAttribute},
    unit_test::{ExpectedFailure, ExpectedMoveError},
};
use move_binary_format::errors::Location;
use move_core_types::vm_status::StatusCode;
use move_model::{
    ast::{Attribute, ModuleName},
    model::{GlobalEnv, Loc},
};
use std::collections::BTreeMap;

pub(super) fn parse_failure_attribute(
    env: &GlobalEnv,
    current_module: &ModuleName,
    expected_attr: &Attribute,
) -> Checked<ExpectedFailure> {
    match expected_attr {
        Attribute::Assign { node_id, .. } => {
            let assign_loc = env.get_node_loc(*node_id);
            env.error(
                &assign_loc,
                "expected an `#[expected_failure(...)]` attribute for error specification",
            );
            Err(ErrorReported)
        },
        Attribute::Apply {
            node_id,
            name,
            attrs,
            ..
        } => {
            assert!(
                TestingAttribute::EXPECTED_FAILURE == env.symbol_pool().string(*name).to_string(),
                "ICE: We should only be parsing a raw expected failure attribute"
            );
            if attrs.is_empty() {
                return Ok(ExpectedFailure::Expected);
            }
            let outer_loc = env.get_node_loc(*node_id);
            let mut attrs: BTreeMap<String, &Attribute> = attrs
                .iter()
                .map(|attr| (env.symbol_pool().string(attr.name()).to_string(), attr))
                .collect();
            let (kind, attr) = resolve_expected_failure_kind(env, outer_loc, &mut attrs)?;
            let location_opt = attrs.remove(TestingAttribute::ERROR_LOCATION);
            let attr_loc = env.get_node_loc(attr.node_id());
            resolve_failure_kind(env, current_module, kind, attr, attr_loc, location_opt, attrs)
        },
    }
}

/// Dispatches on the resolved failure kind and builds the final `ExpectedFailure`.
///
/// The `AbortCode` kind has a deprecated no-location fallback that returns early, skipping the
/// unused-attribute warnings below; every other kind falls through to them.
fn resolve_failure_kind(
    env: &GlobalEnv,
    current_module: &ModuleName,
    kind: ExpectedFailureKind,
    attr: &Attribute,
    attr_loc: Loc,
    location_opt: Option<&Attribute>,
    mut attrs: BTreeMap<String, &Attribute>,
) -> Checked<ExpectedFailure> {
    let (status_code, sub_status_code, location) = match kind {
        ExpectedFailureKind::AbortCode => {
            let (_value_name_loc, attr_value) =
                expect_assigned_value(env, TestingAttribute::ABORT_CODE_NAME, attr)?;
            let (value_loc, opt_const_module_id, u) =
                resolve_u64_constant_or_literal(env, current_module, &attr_value)?;
            let location = if let Some(location_attr) = location_opt {
                resolve_location(env, location_attr)?
            } else if let Some(location) = opt_const_module_id {
                location
            } else {
                env.diag_with_labels(
                    Severity::Warning,
                    &attr_loc,
                    "test will pass on an abort from *any* module",
                    vec![(
                        value_loc,
                        format!("missing `{}=...`", TestingAttribute::ERROR_LOCATION),
                    )],
                );
                return Ok(ExpectedFailure::ExpectedWithCodeDEPRECATED(u));
            };
            (StatusCode::ABORTED, Some(u), location)
        },
        ExpectedFailureKind::ArithmeticError => {
            ensure_no_attribute_params(env, TestingAttribute::ARITHMETIC_ERROR_NAME, attr)?;
            let location_attr = require_location_attr(
                env,
                attr_loc,
                TestingAttribute::ARITHMETIC_ERROR_NAME,
                location_opt,
            )?;
            let location = resolve_location(env, location_attr)?;
            (StatusCode::ARITHMETIC_ERROR, None, location)
        },
        ExpectedFailureKind::OutOfGas => {
            ensure_no_attribute_params(env, TestingAttribute::OUT_OF_GAS_NAME, attr)?;
            let location_attr = require_location_attr(
                env,
                attr_loc,
                TestingAttribute::OUT_OF_GAS_NAME,
                location_opt,
            )?;
            let location = resolve_location(env, location_attr)?;
            (StatusCode::OUT_OF_GAS, None, location)
        },
        ExpectedFailureKind::VectorError => {
            ensure_no_attribute_params(env, TestingAttribute::VECTOR_ERROR_NAME, attr)?;
            let minor_status = resolve_optional_minor_status(env, current_module, &mut attrs)?;
            let location_attr = require_location_attr(
                env,
                attr_loc,
                TestingAttribute::VECTOR_ERROR_NAME,
                location_opt,
            )?;
            let location = resolve_location(env, location_attr)?;
            (StatusCode::VECTOR_OPERATION_ERROR, minor_status, location)
        },
        ExpectedFailureKind::MajorStatus => {
            let (value_name_loc, attr_value) =
                expect_assigned_value(env, TestingAttribute::MAJOR_STATUS_NAME, attr)?;
            let (major_value_loc, _, major_status_u64) =
                resolve_u64_constant_or_literal(env, current_module, &attr_value)?;
            let major_status = StatusCode::try_from(major_status_u64).map_err(|_| {
                env.error_with_labels(
                    &value_name_loc,
                    &format!("invalid value for `{}`", TestingAttribute::MAJOR_STATUS_NAME),
                    vec![(
                        major_value_loc,
                        format!(
                            "no status code associated with value `{}`",
                            major_status_u64
                        ),
                    )],
                );
                ErrorReported
            })?;
            let minor_status = resolve_optional_minor_status(env, current_module, &mut attrs)?;
            let location_attr = require_location_attr(
                env,
                attr_loc,
                TestingAttribute::MAJOR_STATUS_NAME,
                location_opt,
            )?;
            let location = resolve_location(env, location_attr)?;
            (major_status, minor_status, location)
        },
    };
    for (_, attr) in attrs {
        let loc = env.get_node_loc(attr.node_id());
        env.diag(
            Severity::Warning,
            &loc,
            &format!(
                "unused attribute for `{}`",
                TestingAttribute::ExpectedFailure.name()
            ),
        );
    }
    Ok(ExpectedFailure::ExpectedWithError(ExpectedMoveError(
        status_code,
        sub_status_code,
        Location::Module(location),
        None,
    )))
}

/// Resolves an optional `minor_status = ...` sibling, shared by `vector_error` and
/// `major_status`.
fn resolve_optional_minor_status(
    env: &GlobalEnv,
    current_module: &ModuleName,
    attrs: &mut BTreeMap<String, &Attribute>,
) -> Checked<Option<u64>> {
    let Some(minor_attr) = attrs.remove(TestingAttribute::MINOR_STATUS_NAME) else {
        return Ok(None);
    };
    let (_minor_value_loc, minor_value) =
        expect_assigned_value(env, TestingAttribute::MINOR_STATUS_NAME, minor_attr)?;
    let (_, _, minor_status) = resolve_u64_constant_or_literal(env, current_module, &minor_value)?;
    Ok(Some(minor_status))
}
