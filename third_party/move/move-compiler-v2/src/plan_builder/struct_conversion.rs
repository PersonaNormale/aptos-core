// Parts of the file are Copyright (c) The Diem Core Contributors
// Parts of the file are Copyright (c) The Move Contributors
// Parts of the file are Copyright (c) Aptos Foundation
// All Aptos Foundation code and content is licensed pursuant to the Innovation-Enabling Source Code License, available at https://github.com/aptos-labs/aptos-core/blob/main/LICENSE

//! Converts a `#[test(...)]`/`#[expected_failure(...)]` struct- or enum-variant-literal payload
//! (`AttributeValue::Pack`) into a `MoveValue`, including the allowlisted `Option`/`String`/
//! `ascii` stdlib constructors recognized in place of a field literal, since none of those types
//! exposes a public field to construct directly.

use super::convert::{resolve_module_env, to_move_value, ConversionError};
use move_binary_format::file_format::{VariantIndex, Visibility};
use move_core_types::value::{MoveStruct, MoveValue};
use move_model::{
    ast::{AttributeValue, ModuleName, PackFields},
    model::{
        FieldEnv, FunctionEnv, GlobalEnv, ModuleEnv, ModuleId as ModelModuleId, NodeId,
        StructEnv, StructId,
    },
    symbol::Symbol,
    ty::Type,
};
use std::collections::BTreeMap;

/// The `Pack` counterpart of `to_move_value`: resolves struct-or-enum identity, checks
/// visibility and type-argument agreement, checks field completeness, and recurses per field.
/// `variant` is `Some` for an enum-variant literal (`Enum::Variant(..)`/`Enum::Variant{..}`) and
/// `None` for a plain struct literal.
pub(super) fn to_move_struct(
    opt_module: &Option<ModuleName>,
    name: Symbol,
    variant: &Option<Symbol>,
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
    let struct_env = match module_env.find_struct(name) {
        Some(struct_env) => struct_env,
        None => {
            let func_env = module_env
                .find_function(name)
                .ok_or(ConversionError::UnknownStruct)?;
            let kind = allowlisted_constructor(&func_env).ok_or(ConversionError::UnknownStruct)?;
            return build_constructor_call(
                &func_env,
                kind,
                opt_type_args,
                fields,
                target_mid,
                target_sid,
                target_args,
                current_module,
                env,
            );
        },
    };
    if (struct_env.module_env.get_id(), struct_env.get_id()) != (target_mid, target_sid) {
        return Err(ConversionError::TypeMismatch {
            declared: Type::Struct(struct_env.module_env.get_id(), struct_env.get_id(), vec![]),
        });
    }

    // A syntactically valid `Pack` chain can carry a variant independently of whether the
    // resolved target actually has one, so every combination is handled explicitly rather than
    // guarding on `has_variants()` alone.
    let (variant_tag, declared_fields, is_empty) = match (struct_env.has_variants(), variant) {
        (false, None) => (
            None,
            struct_env.get_fields().collect::<Vec<_>>(),
            struct_env.is_empty_struct(),
        ),
        (false, Some(v)) => {
            return Err(ConversionError::VariantOnNonEnum {
                struct_id: struct_env.get_qualified_id(),
                variant: *v,
            })
        },
        (true, None) => {
            return Err(ConversionError::VariantRequired {
                struct_id: struct_env.get_qualified_id(),
            })
        },
        (true, Some(v)) => {
            let idx = struct_env
                .get_variant_idx(*v)
                .ok_or(ConversionError::UnknownVariant {
                    struct_id: struct_env.get_qualified_id(),
                    variant: *v,
                })?;
            let fields: Vec<_> = struct_env.get_fields_of_variant(*v).collect();
            let is_empty = fields.is_empty();
            (Some(idx), fields, is_empty)
        },
    };

    let calling_module_env = env
        .find_module(current_module)
        .expect("current module exists in the model that is compiling it");
    check_construction_visibility(env, &struct_env, &calling_module_env)?;

    build_struct_or_variant_value(
        variant_tag,
        declared_fields,
        is_empty,
        fields,
        opt_type_args,
        target_mid,
        target_sid,
        target_args,
        current_module,
        env,
    )
}

/// The field-conversion tail shared by an ordinary struct/enum-variant literal
/// (`to_move_struct`) and an allowlisted constructor call (`build_constructor_call`). Takes
/// the variant/field identity the caller has already resolved and never checks construction
/// visibility itself: `to_move_struct` checks it before calling in; `build_constructor_call`
/// never needs to.
fn build_struct_or_variant_value(
    variant_tag: Option<VariantIndex>,
    declared_fields: Vec<FieldEnv>,
    is_empty: bool,
    fields: &PackFields,
    opt_type_args: &Option<Vec<Type>>,
    target_mid: ModelModuleId,
    target_sid: StructId,
    target_args: &[Type],
    current_module: &ModuleName,
    env: &GlobalEnv,
) -> Result<MoveValue, ConversionError> {
    let effective_args: Vec<Type> = match opt_type_args {
        Some(explicit) if explicit != target_args => {
            return Err(ConversionError::TypeMismatch {
                declared: Type::Struct(target_mid, target_sid, explicit.clone()),
            });
        },
        _ => target_args.to_vec(),
    };

    let is_positional = declared_fields
        .first()
        .map(|f| f.is_positional())
        .unwrap_or(false);

    let build = |values: Vec<MoveValue>| match variant_tag {
        Some(tag) => MoveValue::Struct(MoveStruct::new_variant(tag, values)),
        None => MoveValue::Struct(MoveStruct::new(values)),
    };

    match fields {
        PackFields::Named(named) => {
            if is_empty && named.is_empty() {
                return Ok(match variant_tag {
                    Some(tag) => MoveValue::Struct(MoveStruct::new_variant(tag, vec![])),
                    None => MoveValue::Struct(MoveStruct::new(vec![MoveValue::Bool(false)])),
                });
            }
            if is_positional {
                return Err(ConversionError::ConstructorMismatch {
                    expected_positional: true,
                });
            }
            let by_name: BTreeMap<Symbol, &AttributeValue> =
                named.iter().map(|(s, v)| (*s, v)).collect();
            let mut missing = Vec::new();
            let mut values = Vec::new();
            for field in &declared_fields {
                match by_name.get(&field.get_name()) {
                    Some(v) => values.push((field, *v)),
                    None => missing.push(field.get_name()),
                }
            }
            if !missing.is_empty() {
                return Err(ConversionError::MissingFields(missing));
            }
            let declared_names: std::collections::BTreeSet<Symbol> =
                declared_fields.iter().map(|f| f.get_name()).collect();
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
            Ok(build(converted))
        },
        PackFields::Positional(positional) => {
            if is_empty && positional.is_empty() {
                return Ok(match variant_tag {
                    Some(tag) => MoveValue::Struct(MoveStruct::new_variant(tag, vec![])),
                    None => MoveValue::Struct(MoveStruct::new(vec![MoveValue::Bool(false)])),
                });
            }
            if !is_positional {
                return Err(ConversionError::ConstructorMismatch {
                    expected_positional: false,
                });
            }
            if positional.len() != declared_fields.len() {
                return Err(ConversionError::FieldCountMismatch {
                    expected: declared_fields.len(),
                    found: positional.len(),
                });
            }
            let converted = declared_fields
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
            Ok(build(converted))
        },
    }
}

/// The `Option`/`String`/`ascii` functions recognized in attribute-call position, in place of a
/// field literal, since none of these types exposes a public field to construct directly.
#[derive(Clone, Copy)]
enum ConstructorKind {
    OptionNone,
    OptionSome,
    OptionFromVec,
    StringUtf8,
    StringTryUtf8,
    AsciiString,
    AsciiTryString,
    AsciiChar,
}

/// The three stdlib modules whose constructors `allowlisted_constructor` recognizes.
enum StdModule {
    Option,
    String,
    Ascii,
}

fn stdmodule(module_env: &ModuleEnv) -> Option<StdModule> {
    if module_env.is_option() {
        Some(StdModule::Option)
    } else if module_env.is_string() {
        Some(StdModule::String)
    } else if module_env.is_ascii() {
        Some(StdModule::Ascii)
    } else {
        None
    }
}

/// Whether `func_env` is one of the allowlisted constructors, checked by module and function
/// identity, not by resolved struct/enum identity: at the point this is called, `name` has
/// already failed to resolve as a struct, so there is no struct/enum to identity-check against
/// yet.
fn allowlisted_constructor(func_env: &FunctionEnv) -> Option<ConstructorKind> {
    match (
        stdmodule(&func_env.module_env)?,
        func_env.get_name_str().as_str(),
    ) {
        (StdModule::Option, "none") => Some(ConstructorKind::OptionNone),
        (StdModule::Option, "some") => Some(ConstructorKind::OptionSome),
        (StdModule::Option, "from_vec") => Some(ConstructorKind::OptionFromVec),
        (StdModule::String, "utf8") => Some(ConstructorKind::StringUtf8),
        (StdModule::String, "try_utf8") => Some(ConstructorKind::StringTryUtf8),
        (StdModule::Ascii, "string") => Some(ConstructorKind::AsciiString),
        (StdModule::Ascii, "try_string") => Some(ConstructorKind::AsciiTryString),
        (StdModule::Ascii, "char") => Some(ConstructorKind::AsciiChar),
        _ => None,
    }
}

/// Every `AttributeValue` variant carries its own `NodeId` as its first field; this reads it
/// back generically, for use as a `Loc` source when minting a synthetic node.
fn attribute_value_node_id(value: &AttributeValue) -> NodeId {
    match value {
        AttributeValue::Value(id, _)
        | AttributeValue::Name(id, ..)
        | AttributeValue::Vector(id, ..)
        | AttributeValue::Pack(id, ..) => *id,
    }
}

/// Reads a single-vector-field struct's (`String`, `ascii::String`) backing bytes back out of
/// its `MoveValue::Struct`, so the constructor that built it can run the same validity check its
/// native body performs at runtime, at compile time instead.
fn bytes_from_single_vector_field(value: &MoveValue) -> Vec<u8> {
    let MoveValue::Struct(s) = value else {
        unreachable!("string::utf8 always builds a MoveValue::Struct")
    };
    let (_, fields) = s.optional_variant_and_fields();
    let MoveValue::Vector(byte_values) = &fields[0] else {
        unreachable!("String's one field is always a vector<u8>")
    };
    byte_values
        .iter()
        .map(|v| match v {
            MoveValue::U8(b) => *b,
            _ => unreachable!("vector<u8> elements are always MoveValue::U8"),
        })
        .collect()
}

/// `ascii::is_valid_char`'s predicate (`ascii.move:139-141`), reused at compile time so
/// `ascii::string`/`ascii::try_string`/`ascii::char` can validate their bytes the same way their
/// native bodies do at runtime.
fn is_valid_ascii_byte(b: u8) -> bool {
    b <= 0x7F
}

/// Reads `ascii::Char`'s single scalar `byte` field back out of its `MoveValue::Struct`, the
/// scalar counterpart of `bytes_from_single_vector_field`.
fn byte_from_single_scalar_field(value: &MoveValue) -> u8 {
    let MoveValue::Struct(s) = value else {
        unreachable!("a single-scalar-field struct constructor always builds a MoveValue::Struct")
    };
    let (_, fields) = s.optional_variant_and_fields();
    match &fields[0] {
        MoveValue::U8(b) => *b,
        _ => unreachable!("ascii::Char's one field is always a u8"),
    }
}

/// Builds a single-field, never-enum-declared struct (`String`, `ascii::String`, `ascii::Char`)
/// from one already-parsed constructor argument, reusing the ordinary field-conversion path an
/// equivalent hand-written struct literal would go through.
fn build_single_field_struct(
    struct_env: &StructEnv,
    arg: AttributeValue,
    current_module: &ModuleName,
    env: &GlobalEnv,
) -> Result<MoveValue, ConversionError> {
    let declared_fields: Vec<_> = struct_env.get_fields().collect();
    let field_name = declared_fields[0].get_name();
    build_struct_or_variant_value(
        None,
        declared_fields,
        false,
        &PackFields::Named(vec![(field_name, arg)]),
        &None,
        struct_env.module_env.get_id(),
        struct_env.get_id(),
        &[],
        current_module,
        env,
    )
}

/// Builds an `Option` value directly from an already-converted payload. Used by every
/// allowlisted constructor that produces an `Option`: `option::none`/`option::some` know their
/// variant from the constructor name alone; `option::from_vec`/`string::try_utf8`/
/// `ascii::try_string` (Layer 8) decide `Some` vs `None` from data the call itself carries.
/// Handles both the framework's enum-declared `Option` and the legacy struct-declared copy.
fn wrap_in_option(struct_env: &StructEnv, payload: Option<MoveValue>) -> MoveValue {
    if struct_env.has_variants() {
        let (name, values) = match payload {
            Some(v) => ("Some", vec![v]),
            None => ("None", vec![]),
        };
        let idx = struct_env
            .get_variant_idx(struct_env.symbol_pool().make(name))
            .expect("the enum-declared std::option::Option always has None and Some variants");
        MoveValue::Struct(MoveStruct::new_variant(idx, values))
    } else {
        let elems = payload.into_iter().collect::<Vec<_>>();
        MoveValue::Struct(MoveStruct::new(vec![MoveValue::Vector(elems)]))
    }
}

/// Lowers a call to an allowlisted `Option`/`String` constructor into the `MoveValue` a
/// hand-written literal for that constructor would need, reading the real field layout off the
/// resolved function's own return type so the result stays correct whether `Option` is declared
/// as the legacy `vec`-field struct or the current enum.
fn build_constructor_call(
    func_env: &FunctionEnv,
    kind: ConstructorKind,
    opt_type_args: &Option<Vec<Type>>,
    fields: &PackFields,
    target_mid: ModelModuleId,
    target_sid: StructId,
    target_args: &[Type],
    current_module: &ModuleName,
    env: &GlobalEnv,
) -> Result<MoveValue, ConversionError> {
    let Type::Struct(mid, sid, _) = func_env.get_result_type() else {
        unreachable!("every allowlisted constructor returns a struct/enum type")
    };
    if (mid, sid) != (target_mid, target_sid) {
        return Err(ConversionError::TypeMismatch {
            declared: Type::Struct(mid, sid, vec![]),
        });
    }
    if let Some(explicit) = opt_type_args {
        if explicit != target_args {
            return Err(ConversionError::TypeMismatch {
                declared: Type::Struct(target_mid, target_sid, explicit.clone()),
            });
        }
    }

    let PackFields::Positional(args) = fields else {
        return Err(ConversionError::ConstructorMismatch {
            expected_positional: true,
        });
    };
    let expected_arity = match kind {
        ConstructorKind::OptionNone => 0,
        ConstructorKind::OptionSome
        | ConstructorKind::OptionFromVec
        | ConstructorKind::StringUtf8
        | ConstructorKind::StringTryUtf8
        | ConstructorKind::AsciiString
        | ConstructorKind::AsciiTryString
        | ConstructorKind::AsciiChar => 1,
    };
    if args.len() != expected_arity {
        return Err(ConversionError::FieldCountMismatch {
            expected: expected_arity,
            found: args.len(),
        });
    }

    let struct_env = env.get_struct(mid.qualified(sid));

    match kind {
        ConstructorKind::OptionNone => Ok(wrap_in_option(&struct_env, None)),
        ConstructorKind::OptionSome => {
            let e = to_move_value(&args[0], &target_args[0], current_module, env)?;
            Ok(wrap_in_option(&struct_env, Some(e)))
        },
        ConstructorKind::StringUtf8 => {
            let value =
                build_single_field_struct(&struct_env, args[0].clone(), current_module, env)?;
            let bytes = bytes_from_single_vector_field(&value);
            if std::str::from_utf8(&bytes).is_err() {
                return Err(ConversionError::InvalidUtf8 {
                    node_id: attribute_value_node_id(&args[0]),
                });
            }
            Ok(value)
        },
        ConstructorKind::OptionFromVec => {
            let AttributeValue::Vector(vec_node_id, _, elems) = &args[0] else {
                return Err(ConversionError::TypeMismatch {
                    declared: env.get_node_type(attribute_value_node_id(&args[0])),
                });
            };
            match elems.len() {
                0 => Ok(wrap_in_option(&struct_env, None)),
                1 => {
                    let e = to_move_value(&elems[0], &target_args[0], current_module, env)?;
                    Ok(wrap_in_option(&struct_env, Some(e)))
                },
                _ => Err(ConversionError::OptionVecTooLong {
                    node_id: *vec_node_id,
                }),
            }
        },
        ConstructorKind::StringTryUtf8 => {
            let string_name = env.symbol_pool().make("String");
            let string_struct_env = func_env
                .module_env
                .find_struct(string_name)
                .expect("std::string declares String alongside try_utf8");
            let value = build_single_field_struct(
                &string_struct_env,
                args[0].clone(),
                current_module,
                env,
            )?;
            let bytes = bytes_from_single_vector_field(&value);
            let payload = if std::str::from_utf8(&bytes).is_ok() {
                Some(value)
            } else {
                None
            };
            Ok(wrap_in_option(&struct_env, payload))
        },
        ConstructorKind::AsciiString => {
            let value =
                build_single_field_struct(&struct_env, args[0].clone(), current_module, env)?;
            let bytes = bytes_from_single_vector_field(&value);
            if bytes.iter().any(|b| !is_valid_ascii_byte(*b)) {
                return Err(ConversionError::InvalidAscii {
                    node_id: attribute_value_node_id(&args[0]),
                });
            }
            Ok(value)
        },
        ConstructorKind::AsciiTryString => {
            let string_name = env.symbol_pool().make("String");
            let ascii_string_struct_env = func_env
                .module_env
                .find_struct(string_name)
                .expect("std::ascii declares String alongside try_string");
            let value = build_single_field_struct(
                &ascii_string_struct_env,
                args[0].clone(),
                current_module,
                env,
            )?;
            let bytes = bytes_from_single_vector_field(&value);
            let payload = if bytes.iter().all(|b| is_valid_ascii_byte(*b)) {
                Some(value)
            } else {
                None
            };
            Ok(wrap_in_option(&struct_env, payload))
        },
        ConstructorKind::AsciiChar => {
            let value =
                build_single_field_struct(&struct_env, args[0].clone(), current_module, env)?;
            let byte = byte_from_single_scalar_field(&value);
            if !is_valid_ascii_byte(byte) {
                return Err(ConversionError::InvalidAscii {
                    node_id: attribute_value_node_id(&args[0]),
                });
            }
            Ok(value)
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
