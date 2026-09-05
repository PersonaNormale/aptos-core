// Parts of the file are Copyright (c) The Diem Core Contributors
// Parts of the file are Copyright (c) The Move Contributors
// Parts of the file are Copyright (c) Aptos Foundation
// All Aptos Foundation code and content is licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

//! Resolves module names used in test attributes.

use move_model::{
    ast::ModuleName,
    model::{GlobalEnv, ModuleEnv},
};

/// Resolves a qualifier or the current module. The caller reports lookup failures.
pub(super) fn resolve_module_env<'env>(
    env: &'env GlobalEnv,
    current_module: &ModuleName,
    opt_module_name: &Option<ModuleName>,
) -> Option<ModuleEnv<'env>> {
    env.find_module(opt_module_name.as_ref().unwrap_or(current_module))
}
