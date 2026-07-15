// Parts of the file are Copyright (c) The Diem Core Contributors
// Parts of the file are Copyright (c) The Move Contributors
// Parts of the file are Copyright (c) Aptos Foundation
// All Aptos Foundation code and content is licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

//! Build a vector of module test plans for a Move program compiled with V2.
//!
//! This reimplements the legacy move compiler functionality.
//!
//! Each module containing any labeled `#[test]` functions gets an item in the output list, which
//! includes info about each '#[test]' function: name, arguments to provide, and expected failure or
//! success.

mod arguments;
mod build;
mod collect;
mod convert;
mod error;
mod failure;

use build::build_test_info;
use crate::options::Options;
use legacy_move_compiler::unit_test::ModuleTestPlan;
use move_command_line_common::{address::NumericalAddress, parser::NumberFormat};
use move_core_types::{account_address::AccountAddress, identifier::Identifier};
use move_model::{
    ast::Address,
    model::{GlobalEnv, ModuleEnv},
    symbol::Symbol,
};
use std::collections::BTreeMap;

/// Constructs a test plan for each module in `env.target`. This also validates the structure of
/// the attributes as the test plan is constructed.
pub fn construct_test_plan(
    env: &GlobalEnv,
    package_filter: Option<Symbol>,
) -> Option<Vec<ModuleTestPlan>> {
    let options = env.get_extension::<Options>().expect("options");
    if !options.compile_test_code {
        return None;
    }

    Some(
        env.get_modules()
            .filter_map(|module| {
                if module.is_primary_target() {
                    construct_module_test_plan(env, package_filter, module)
                } else {
                    None
                }
            })
            .collect(),
    )
}

fn construct_module_test_plan(
    env: &GlobalEnv,
    _package_filter: Option<Symbol>,
    module: ModuleEnv,
) -> Option<ModuleTestPlan> {
    // TODO (#12885): what is a package?  Do we need this code?
    // if package_filter.is_some() && module.package_name != package_filter {
    // return None;
    // }

    let current_module = module.get_name();
    let tests: BTreeMap<_, _> = module
        .get_functions()
        .flat_map(|func| build_test_info(env, current_module, func).into_iter())
        .collect();

    if tests.is_empty() {
        return None;
    }

    let module_id = module.get_identifier();
    let addr = current_module.addr();
    let name_sym = current_module.name();
    let name_str = env.symbol_pool().string(name_sym).to_string();
    if let Some(module_identifier) = module_id {
        let name_id = Identifier::new(name_str.clone()).expect("name is valid for identifier");
        assert!(name_id == module_identifier);
    }
    let optional_num_addr: Option<AccountAddress> = match addr {
        Address::Numerical(num_addr) => Some(*num_addr),
        Address::Symbolic(sym) => env.resolve_address_alias(*sym),
    };
    optional_num_addr.map(|addr_bytes| {
        ModuleTestPlan::new(
            &NumericalAddress::new(*addr_bytes, NumberFormat::Hex),
            &name_str,
            tests,
        )
    })
}
