//! Parse WIT files and extract model interface definitions.

use crate::interface::{FieldDef, ModelInterface, RecordDef, WitType};
use anyhow::{Context, Result};
use heck::ToSnakeCase;
use std::path::Path;
use wit_parser::{Resolve, Type, TypeDefKind, UnresolvedPackageGroup};

/// Parse a WIT file and extract the model interface for a given world.
pub fn parse_wit_file(path: &Path, world_name: &str, model_name: &str) -> Result<ModelInterface> {
    let mut resolve = Resolve::default();

    // Parse the WIT file
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read WIT file: {:?}", path))?;

    let pkg_group = UnresolvedPackageGroup::parse(path, &contents)
        .with_context(|| format!("Failed to parse WIT file: {:?}", path))?;

    let package_id = resolve
        .push_group(pkg_group)
        .with_context(|| "Failed to resolve WIT package")?;

    let package = &resolve.packages[package_id];
    let package_name = format!("{}:{}", package.name.namespace, package.name.name);

    // Find the world
    let world_id = package
        .worlds
        .get(world_name)
        .ok_or_else(|| anyhow::anyhow!("World '{}' not found in WIT file", world_name))?;

    let world = &resolve.worlds[*world_id];

    // Find the exported interface that matches our model
    // Convention: the interface name matches the model name (e.g., "drag" interface for "drag" model)
    let mut model_interface = None;

    for (_key, export) in &world.exports {
        if let wit_parser::WorldItem::Interface { id, .. } = export {
            let iface = &resolve.interfaces[*id];
            if let Some(iface_name) = &iface.name {
                if iface_name == model_name {
                    model_interface = Some(iface);
                    break;
                }
            }
        }
    }

    let interface = model_interface
        .ok_or_else(|| anyhow::anyhow!("Interface '{}' not found in world exports", model_name))?;

    // Find the model resource
    let mut params_record = None;
    let mut inputs_record = None;
    let mut outputs_record = None;

    // First, collect all type definitions in this interface
    for (type_name, type_id) in &interface.types {
        let type_def = &resolve.types[*type_id];

        match type_name.as_str() {
            "params" => {
                params_record = Some(extract_record(&resolve, type_def, "Params")?);
            }
            "inputs" => {
                inputs_record = Some(extract_record(&resolve, type_def, "Inputs")?);
            }
            "outputs" => {
                outputs_record = Some(extract_record(&resolve, type_def, "Outputs")?);
            }
            _ => {}
        }
    }

    // If we didn't find records by those names, look for the resource's methods
    // to infer the types from constructor and step signatures
    if params_record.is_none() || inputs_record.is_none() || outputs_record.is_none() {
        // Try to find records from the resource definition
        for (_type_name, type_id) in &interface.types {
            let type_def = &resolve.types[*type_id];
            if let TypeDefKind::Resource = &type_def.kind {
                // This is the model resource, check its functions
                for (func_name, func) in &interface.functions {
                    // Look at constructor and step method
                    if func_name.contains("constructor") && params_record.is_none() {
                        // Constructor parameter should be the params type
                        if let Some((_name, param_type)) = func.params.first() {
                            if let Some(record) =
                                try_extract_record_from_type(&resolve, param_type, "Params")
                            {
                                params_record = Some(record);
                            }
                        }
                    }
                    if func_name.ends_with(".step") || func_name == "step" {
                        // step(inputs) -> outputs
                        if inputs_record.is_none() {
                            if let Some((_name, input_type)) =
                                func.params.iter().find(|(n, _)| n != "self")
                            {
                                if let Some(record) =
                                    try_extract_record_from_type(&resolve, input_type, "Inputs")
                                {
                                    inputs_record = Some(record);
                                }
                            }
                        }
                        if outputs_record.is_none() {
                            match &func.results {
                                wit_parser::Results::Named(named) => {
                                    if let Some((_, output_type)) = named.first() {
                                        if let Some(record) = try_extract_record_from_type(
                                            &resolve,
                                            output_type,
                                            "Outputs",
                                        ) {
                                            outputs_record = Some(record);
                                        }
                                    }
                                }
                                wit_parser::Results::Anon(output_type) => {
                                    if let Some(record) = try_extract_record_from_type(
                                        &resolve,
                                        output_type,
                                        "Outputs",
                                    ) {
                                        outputs_record = Some(record);
                                    }
                                }
                            }
                        }
                    }
                }
                break;
            }
        }
    }

    let params = params_record
        .ok_or_else(|| anyhow::anyhow!("Could not find 'params' record in interface"))?;
    let inputs = inputs_record
        .ok_or_else(|| anyhow::anyhow!("Could not find 'inputs' record in interface"))?;
    let outputs = outputs_record
        .ok_or_else(|| anyhow::anyhow!("Could not find 'outputs' record in interface"))?;

    Ok(ModelInterface {
        name: model_name.to_string(),
        package: package_name,
        params,
        inputs,
        outputs,
    })
}

fn extract_record(
    resolve: &Resolve,
    type_def: &wit_parser::TypeDef,
    name: &str,
) -> Result<RecordDef> {
    match &type_def.kind {
        TypeDefKind::Record(record) => {
            let fields = record
                .fields
                .iter()
                .map(|field| {
                    let wit_type = convert_type(resolve, &field.ty)?;
                    Ok(FieldDef {
                        name: field.name.to_snake_case(),
                        wit_name: field.name.clone(),
                        ty: wit_type,
                        docs: field.docs.contents.clone(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(RecordDef {
                name: name.to_string(),
                fields,
            })
        }
        _ => anyhow::bail!("Expected a record type for '{}'", name),
    }
}

fn try_extract_record_from_type(resolve: &Resolve, ty: &Type, name: &str) -> Option<RecordDef> {
    match ty {
        Type::Id(id) => {
            let type_def = &resolve.types[*id];
            extract_record(resolve, type_def, name).ok()
        }
        _ => None,
    }
}

fn convert_type(resolve: &Resolve, ty: &Type) -> Result<WitType> {
    match ty {
        Type::Bool => Ok(WitType::Bool),
        Type::U8 => Ok(WitType::U8),
        Type::U16 => Ok(WitType::U16),
        Type::U32 => Ok(WitType::U32),
        Type::U64 => Ok(WitType::U64),
        Type::S8 => Ok(WitType::S8),
        Type::S16 => Ok(WitType::S16),
        Type::S32 => Ok(WitType::S32),
        Type::S64 => Ok(WitType::S64),
        Type::F32 => Ok(WitType::Float32),
        Type::F64 => Ok(WitType::Float64),
        Type::Char => Ok(WitType::Char),
        Type::String => Ok(WitType::String),
        Type::Id(id) => {
            let type_def = &resolve.types[*id];
            match &type_def.kind {
                TypeDefKind::List(inner) => {
                    let inner_type = convert_type(resolve, inner)?;
                    Ok(WitType::List(Box::new(inner_type)))
                }
                TypeDefKind::Option(inner) => {
                    let inner_type = convert_type(resolve, inner)?;
                    Ok(WitType::Option(Box::new(inner_type)))
                }
                TypeDefKind::Tuple(tuple) => {
                    let types = tuple
                        .types
                        .iter()
                        .map(|t| convert_type(resolve, t))
                        .collect::<Result<Vec<_>>>()?;
                    Ok(WitType::Tuple(types))
                }
                TypeDefKind::Type(inner) => convert_type(resolve, inner),
                TypeDefKind::Record(_) => {
                    // This is a nested record reference
                    anyhow::bail!("Nested record types not yet supported")
                }
                TypeDefKind::Resource => {
                    anyhow::bail!("Resource types cannot be used as field types")
                }
                other => {
                    anyhow::bail!("Unsupported type kind: {:?}", other)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_simple_wit() {
        let wit_content = r#"
package test:models;

interface example {
    record params {
        coefficient: f64,
    }

    record inputs {
        value: f64,
    }

    record outputs {
        output-value: f64,
    }

    resource model {
        constructor(p: params);
        step: func(i: inputs) -> outputs;
    }
}

world example-model {
    export example;
}
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(wit_content.as_bytes()).unwrap();

        let model = parse_wit_file(file.path(), "example-model", "example").unwrap();

        assert_eq!(model.name, "example");
        assert_eq!(model.package, "test:models");
        assert_eq!(model.params.fields.len(), 1);
        assert_eq!(model.params.fields[0].name, "coefficient");
        assert_eq!(model.inputs.fields.len(), 1);
        assert_eq!(model.outputs.fields.len(), 1);
    }

    #[test]
    fn test_parse_drag_wit() {
        let wit_content = r#"
package myorg:physics;

interface drag {
    record params {
        area: f64,
        cd: f64,
    }

    record inputs {
        rho: f64,
        velocity: f64,
    }

    record outputs {
        drag-force: f64,
        dynamic-pressure: f64,
    }

    resource model {
        constructor(p: params);
        step: func(i: inputs) -> outputs;
    }
}

world drag-model {
    export drag;
}
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(wit_content.as_bytes()).unwrap();

        let model = parse_wit_file(file.path(), "drag-model", "drag").unwrap();

        assert_eq!(model.name, "drag");
        assert_eq!(model.package, "myorg:physics");

        // Params
        assert_eq!(model.params.fields.len(), 2);
        assert_eq!(model.params.fields[0].name, "area");
        assert_eq!(model.params.fields[0].wit_name, "area");
        assert_eq!(model.params.fields[1].name, "cd");

        // Inputs
        assert_eq!(model.inputs.fields.len(), 2);
        assert_eq!(model.inputs.fields[0].name, "rho");
        assert_eq!(model.inputs.fields[1].name, "velocity");

        // Outputs - note kebab-case conversion
        assert_eq!(model.outputs.fields.len(), 2);
        assert_eq!(model.outputs.fields[0].name, "drag_force");
        assert_eq!(model.outputs.fields[0].wit_name, "drag-force");
        assert_eq!(model.outputs.fields[1].name, "dynamic_pressure");
    }
}
