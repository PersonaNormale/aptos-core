// Parts of the file are Copyright (c) The Diem Core Contributors
// Parts of the file are Copyright (c) The Move Contributors
// Parts of the file are Copyright (c) Aptos Foundation
// All Aptos Foundation code and content is licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

//! Resolves a `#[test(...)]`/`#[expected_failure(...)]` attribute value's module-name
//! qualifier to the `ModuleEnv` it refers to.

use move_model::{
    ast::ModuleName,
    model::{GlobalEnv, ModuleEnv},
};

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
