// Parts of the file are Copyright (c) The Diem Core Contributors
// Parts of the file are Copyright (c) The Move Contributors
// Parts of the file are Copyright (c) Aptos Foundation
// All Aptos Foundation code and content is licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

//! Resolves a test attribute's named-constant reference (`AttributeValue::Name`) to its
//! declared value and type. No visibility check: a `const`'s value carries no invariant to
//! protect the way a struct's `Pack` does, and the common case is reading another module's
//! private error/abort-code constants.

use super::{convert::ConversionError, module_lookup::resolve_module_env};
use move_model::{
    ast::{ModuleName, Value},
    model::GlobalEnv,
    symbol::Symbol,
    ty::Type,
};

/// Unqualified names (`opt_module` is `None`) try `current_module`'s own constant first,
/// then the builtin range-constant module, so a local declaration shadows a builtin of the
/// same name. Qualified names resolve only in the named module.
pub(super) fn resolve_test_constant(
    env: &GlobalEnv,
    current_module: &ModuleName,
    opt_module: &Option<ModuleName>,
    name: Symbol,
) -> Result<(Value, Type), ConversionError> {
    let module_env = resolve_module_env(env, current_module, opt_module).ok_or_else(|| {
        ConversionError::UnknownModule {
            module: opt_module.clone().unwrap_or_else(|| current_module.clone()),
        }
    })?;

    if let Some(entry) = module_env.find_named_constant(name) {
        return Ok((entry.get_value(), entry.get_type()));
    }

    if opt_module.is_none() {
        if let Some((value, ty)) = env.find_builtin_constant(name) {
            return Ok((value, ty));
        }
    }

    Err(ConversionError::UnknownConstant {
        opt_module: opt_module.clone(),
        name,
    })
}
