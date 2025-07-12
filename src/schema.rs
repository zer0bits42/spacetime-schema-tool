use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// Import SATS types
use self::sats_types::{AlgebraicType, ProductType, SatsSchema, SumType, TypeDef};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    Pretty,
    Json,
    Raw,
}

pub struct SchemaArgs {
    pub db: String,
    pub server: String,
    pub version: Option<String>,
    pub cloud: bool,
    pub format: OutputFormat,
    pub table: Option<String>,
    pub type_filter: Option<String>,
    pub enum_filter: Option<String>,
    pub search: Option<String>,
}

// SATS type definitions (from the parser tool)
// These must match the JSON format exactly
#[allow(non_snake_case)]
mod sats_types {
    use super::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct SatsSchema {
        pub typespace: TypeSpace,
        pub tables: Vec<TableInfo>,
        pub types: Vec<NamedType>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct TypeSpace {
        pub types: Vec<TypeDef>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum TypeDef {
        Product { Product: ProductType },
        Sum { Sum: SumType },
        Builtin { Builtin: BuiltinType },
        Ref { Ref: u32 },
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum BuiltinType {
        Bool { Bool: Vec<()> },
        I8 { I8: Vec<()> },
        U8 { U8: Vec<()> },
        I16 { I16: Vec<()> },
        U16 { U16: Vec<()> },
        I32 { I32: Vec<()> },
        U32 { U32: Vec<()> },
        I64 { I64: Vec<()> },
        U64 { U64: Vec<()> },
        I128 { I128: Vec<()> },
        U128 { U128: Vec<()> },
        F32 { F32: Vec<()> },
        F64 { F64: Vec<()> },
        String { String: Vec<()> },
        Array { Array: Box<AlgebraicType> },
        Map { Map: MapType },
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct MapType {
        pub key_ty: Box<AlgebraicType>,
        pub ty: Box<AlgebraicType>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct ProductType {
        pub elements: Vec<Element>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct SumType {
        pub variants: Vec<Variant>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Element {
        pub name: OptionalName,
        pub algebraic_type: AlgebraicType,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Variant {
        pub name: OptionalName,
        pub algebraic_type: AlgebraicType,
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum OptionalName {
        Some { some: String },
        None { none: Vec<()> },
    }

    impl OptionalName {
        pub fn as_option(&self) -> Option<&str> {
            match self {
                OptionalName::Some { some } => Some(some.as_str()),
                OptionalName::None { .. } => None,
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum AlgebraicType {
        Bool { Bool: Vec<()> },
        I8 { I8: Vec<()> },
        U8 { U8: Vec<()> },
        I16 { I16: Vec<()> },
        U16 { U16: Vec<()> },
        I32 { I32: Vec<()> },
        U32 { U32: Vec<()> },
        I64 { I64: Vec<()> },
        U64 { U64: Vec<()> },
        I128 { I128: Vec<()> },
        U128 { U128: Vec<()> },
        I256 { I256: Vec<()> },
        U256 { U256: Vec<()> },
        F32 { F32: Vec<()> },
        F64 { F64: Vec<()> },
        String { String: Vec<()> },
        Array { Array: Box<AlgebraicType> },
        Product { Product: ProductType },
        Sum { Sum: SumType },
        Ref { Ref: u32 },
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct TableInfo {
        pub name: String,
        pub product_type_ref: usize,
        pub primary_key: Vec<usize>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct NamedType {
        pub name: TypeName,
        pub ty: usize,
        pub custom_ordering: bool,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct TypeName {
        pub scope: Vec<String>,
        pub name: String,
    }
}
// Schema operations
pub async fn fetch_schema(args: SchemaArgs) -> Result<()> {
    let server = if args.cloud {
        "cloud"
    } else {
        &args.server
    };

    let client = crate::spacetime_client::SpacetimeClient::new(server)?;
    println!(
        "{} {}",
        "🌐 Fetching schema from:".cyan(),
        client.base_url()
    );

    let schema_json = client.fetch_schema(&args.db, args.version).await?;
    let schema_text = serde_json::to_string_pretty(&schema_json)?;
    println!("{} {} bytes", "✅ Fetched".green(), schema_text.len());

    match args.format {
        OutputFormat::Raw | OutputFormat::Json => {
            println!("{schema_text}");
        }
        OutputFormat::Pretty => {
            let schema: SatsSchema = serde_json::from_value(schema_json)?;
            display_schema_pretty(
                &schema,
                args.table,
                args.type_filter,
                args.enum_filter,
                args.search,
            );
        }
    }

    Ok(())
}

fn display_schema_pretty(
    schema: &SatsSchema,
    table_filter: Option<String>,
    type_filter: Option<String>,
    enum_filter: Option<String>,
    search_pattern: Option<String>,
) {
    // Extract real names
    let mut type_names = HashMap::new();
    for named_type in &schema.types {
        type_names.insert(named_type.ty, named_type.name.name.clone());
    }

    // Apply filters
    if let Some(table_name) = table_filter {
        display_single_table(schema, &type_names, &table_name);
        return;
    }

    if let Some(type_name) = type_filter {
        display_single_type(schema, &type_names, &type_name);
        return;
    }

    if let Some(enum_name) = enum_filter {
        display_single_enum(schema, &type_names, &enum_name);
        return;
    }

    if let Some(pattern) = search_pattern {
        display_search_results(schema, &type_names, &pattern);
        return;
    }

    // Default: show everything
    println!("\n{}", "📋 SPACETIMEDB SCHEMA".bold().cyan());
    println!("{}", "=".repeat(60));

    // Show tables
    println!(
        "\n{} {}",
        "📊 TABLES".yellow(),
        format!("({})", schema.tables.len()).dimmed()
    );
    for table in &schema.tables {
        let type_name = type_names
            .get(&table.product_type_ref)
            .cloned()
            .unwrap_or_else(|| format!("Type_{}", table.product_type_ref));

        println!(
            "  {} {} → {}",
            "▸".green(),
            table.name.bold(),
            type_name.dimmed()
        );

        // Show fields
        if let Some(TypeDef::Product { Product }) =
            schema.typespace.types.get(table.product_type_ref)
        {
            for element in &Product.elements {
                if let Some(field_name) = element.name.as_option() {
                    let field_type = format_type(&element.algebraic_type, &type_names);
                    println!("    {} {}: {}", "├".dimmed(), field_name, field_type.cyan());
                }
            }
        }
        println!();
    }

    // Show other types (enums, structs)
    println!(
        "{} {}",
        "🔧 OTHER TYPES".yellow(),
        "(enums, structs)".dimmed()
    );
    println!("{}", "-".repeat(40));

    // Find types that aren't used as tables
    let table_type_refs: HashSet<usize> =
        schema.tables.iter().map(|t| t.product_type_ref).collect();

    let mut standalone_types: Vec<_> = type_names
        .iter()
        .filter(|(type_idx, _)| !table_type_refs.contains(type_idx))
        .collect();
    standalone_types.sort_by_key(|(_, name)| name.to_lowercase());

    for (type_idx, real_name) in standalone_types {
        if let Some(type_def) = schema.typespace.types.get(*type_idx) {
            match type_def {
                TypeDef::Sum { Sum } => {
                    // Check for special types
                    if let Some(special_type) = detect_spacetimedb_sum_type(Sum) {
                        println!(
                            "  {} {}: {} {}",
                            "⚡".yellow(),
                            real_name.bold(),
                            special_type,
                            "(SpacetimeDB type)".dimmed()
                        );
                    } else {
                        println!(
                            "  {} {} {}",
                            "🔀".cyan(),
                            real_name.bold(),
                            format!("(enum with {} variants)", Sum.variants.len()).dimmed()
                        );

                        // Show enum variants
                        for (i, variant) in Sum.variants.iter().enumerate() {
                            let is_last = i == Sum.variants.len() - 1;
                            let prefix = if is_last { "└" } else { "├" };

                            if let Some(variant_name) = variant.name.as_option() {
                                // Check if variant has associated data
                                match &variant.algebraic_type {
                                    AlgebraicType::Product { Product }
                                        if Product.elements.is_empty() =>
                                    {
                                        // Unit variant
                                        println!("    {} {}", prefix.dimmed(), variant_name);
                                    }
                                    _ => {
                                        // Variant with data
                                        let variant_type =
                                            format_type(&variant.algebraic_type, &type_names);
                                        println!(
                                            "    {} {}({})",
                                            prefix.dimmed(),
                                            variant_name,
                                            variant_type.cyan()
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                TypeDef::Product { Product } => {
                    // Check for special types
                    if let Some(special_type) = detect_spacetimedb_type(Product) {
                        println!(
                            "  {} {}: {} {}",
                            "⚡".yellow(),
                            real_name.bold(),
                            special_type,
                            "(SpacetimeDB type)".dimmed()
                        );
                    } else {
                        println!(
                            "  {} {} {}",
                            "📦".blue(),
                            real_name.bold(),
                            format!("(struct with {} fields)", Product.elements.len()).dimmed()
                        );

                        // Show struct fields
                        for (i, element) in Product.elements.iter().enumerate() {
                            let is_last = i == Product.elements.len() - 1;
                            let prefix = if is_last { "└" } else { "├" };

                            if let Some(field_name) = element.name.as_option() {
                                let field_type = format_type(&element.algebraic_type, &type_names);
                                println!(
                                    "    {} {}: {}",
                                    prefix.dimmed(),
                                    field_name,
                                    field_type.cyan()
                                );
                            } else {
                                // Unnamed field (tuple struct)
                                let field_type = format_type(&element.algebraic_type, &type_names);
                                println!("    {} {}: {}", prefix.dimmed(), i, field_type.cyan());
                            }
                        }
                    }
                }
                TypeDef::Builtin { .. } => {
                    // Skip builtins in this view
                }
                TypeDef::Ref { .. } => {
                    // Skip refs in this view
                }
            }
        }
    }

    println!();

    // Summary
    let enum_count = schema
        .typespace
        .types
        .iter()
        .filter(|t| matches!(t, TypeDef::Sum { .. }))
        .count();

    println!("{}", "📈 SUMMARY".yellow());
    println!("  {} tables", schema.tables.len());
    println!("  {} types total", schema.typespace.types.len());
    println!("  {} enums", enum_count);
}

fn format_type(alg_type: &AlgebraicType, type_names: &HashMap<usize, String>) -> String {
    match alg_type {
        AlgebraicType::Bool { .. } => "bool".to_string(),
        AlgebraicType::I8 { .. } => "i8".to_string(),
        AlgebraicType::U8 { .. } => "u8".to_string(),
        AlgebraicType::I16 { .. } => "i16".to_string(),
        AlgebraicType::U16 { .. } => "u16".to_string(),
        AlgebraicType::I32 { .. } => "i32".to_string(),
        AlgebraicType::U32 { .. } => "u32".to_string(),
        AlgebraicType::I64 { .. } => "i64".to_string(),
        AlgebraicType::U64 { .. } => "u64".to_string(),
        AlgebraicType::I128 { .. } => "i128".to_string(),
        AlgebraicType::U128 { .. } => "u128".to_string(),
        AlgebraicType::I256 { .. } => "i256".to_string(),
        AlgebraicType::U256 { .. } => "u256".to_string(),
        AlgebraicType::F32 { .. } => "f32".to_string(),
        AlgebraicType::F64 { .. } => "f64".to_string(),
        AlgebraicType::String { .. } => "String".to_string(),
        AlgebraicType::Array { Array } => {
            format!("Vec<{}>", format_type(Array, type_names))
        }
        AlgebraicType::Ref { Ref } => type_names
            .get(&(*Ref as usize))
            .cloned()
            .unwrap_or_else(|| format!("Type_{}", Ref)),
        AlgebraicType::Sum { Sum } => {
            // Check if this is a SpacetimeDB ScheduledAt pattern
            if let Some(stdb_type) = detect_spacetimedb_sum_type(Sum) {
                return stdb_type;
            }

            // Check if this is an Option<T> pattern
            if is_option_type(Sum) {
                if let Some(inner_type) = get_option_inner_type(Sum) {
                    return format!("Option<{}>", format_type(inner_type, type_names));
                }
                return "Option<?>".to_string();
            }

            format!("Sum({} variants)", Sum.variants.len())
        }
        AlgebraicType::Product { Product } => {
            // Check for SpacetimeDB well-known types
            if let Some(stdb_type) = detect_spacetimedb_type(Product) {
                return stdb_type;
            }

            // Handle tuples and named structs
            if Product.elements.is_empty() {
                "()".to_string()
            } else if Product
                .elements
                .iter()
                .all(|e| e.name.as_option().is_none())
            {
                // This is a tuple
                let types: Vec<_> = Product
                    .elements
                    .iter()
                    .map(|e| format_type(&e.algebraic_type, type_names))
                    .collect();
                format!("({})", types.join(", "))
            } else {
                // This is a named struct
                format!("Product({} fields)", Product.elements.len())
            }
        }
    }
}

// Helper functions for type detection
fn detect_spacetimedb_type(product: &ProductType) -> Option<String> {
    // Check for single-field products with special names (SpacetimeDB well-known types)
    if product.elements.len() == 1 {
        let element = &product.elements[0];
        if let Some(field_name) = element.name.as_option() {
            match field_name {
                "__identity__" => {
                    // Identity type: Product with single U256 field named __identity__
                    if matches!(element.algebraic_type, AlgebraicType::U256 { .. }) {
                        return Some("Identity".to_string());
                    }
                }
                "__timestamp_micros_since_unix_epoch__" => {
                    // Timestamp type: Product with single I64 field
                    if matches!(element.algebraic_type, AlgebraicType::I64 { .. }) {
                        return Some("Timestamp".to_string());
                    }
                }
                "__time_duration_micros__" => {
                    // Duration type: Product with single I64 field
                    if matches!(element.algebraic_type, AlgebraicType::I64 { .. }) {
                        return Some("Duration".to_string());
                    }
                }
                _ => {}
            }
        }
    }

    None
}

fn detect_spacetimedb_sum_type(sum: &SumType) -> Option<String> {
    // Check for SpacetimeDB ScheduledAt pattern
    if sum.variants.len() == 2 {
        let variant_names: Vec<_> = sum
            .variants
            .iter()
            .filter_map(|v| v.name.as_option())
            .collect();

        // Check for Interval vs Time variants
        if variant_names.contains(&"Interval") && variant_names.contains(&"Time") {
            return Some("ScheduledAt".to_string());
        }
    }

    None
}

fn is_option_type(sum: &SumType) -> bool {
    if sum.variants.len() != 2 {
        return false;
    }

    let variant_names: Vec<_> = sum
        .variants
        .iter()
        .filter_map(|v| v.name.as_option())
        .collect();

    // Pattern 1: Classic Some/None
    if variant_names.len() == 2 {
        let has_some = variant_names.contains(&"Some");
        let has_none = variant_names.contains(&"None");

        if has_some && has_none {
            return true;
        }
    }

    // Pattern 2: One empty variant (None) and one with data (Some)
    let mut has_unit_variant = false;
    let mut has_data_variant = false;

    for variant in &sum.variants {
        match &variant.algebraic_type {
            AlgebraicType::Product { Product } if Product.elements.is_empty() => {
                has_unit_variant = true;
            }
            _ => {
                has_data_variant = true;
            }
        }
    }

    has_unit_variant && has_data_variant
}

fn get_option_inner_type(sum: &SumType) -> Option<&AlgebraicType> {
    for variant in &sum.variants {
        if let Some(name) = variant.name.as_option() {
            if name == "Some" {
                return Some(&variant.algebraic_type);
            }
        }
    }

    // For unnamed variants, pick the one with data
    for variant in &sum.variants {
        match &variant.algebraic_type {
            AlgebraicType::Product { Product } if !Product.elements.is_empty() => {
                // If it's a product with fields, get the first field's type
                if Product.elements.len() == 1 {
                    return Some(&Product.elements[0].algebraic_type);
                }
                return Some(&variant.algebraic_type);
            }
            // If it's not a Product or not empty, it's likely the Some variant
            AlgebraicType::Product { Product } if Product.elements.is_empty() => {
                // This is the None variant, skip it
            }
            _ => {
                // This is likely the Some variant with data
                return Some(&variant.algebraic_type);
            }
        }
    }
    None
}

// Display functions for filtered views
fn display_single_table(
    schema: &SatsSchema,
    type_names: &HashMap<usize, String>,
    table_name: &str,
) {
    let table = schema
        .tables
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case(table_name));

    if let Some(table) = table {
        println!("\n{} {}", "📊 TABLE:".yellow(), table.name.bold());
        println!("{}", "-".repeat(40));

        let type_name = type_names
            .get(&table.product_type_ref)
            .cloned()
            .unwrap_or_else(|| format!("Type_{}", table.product_type_ref));
        println!("Type: {}", type_name.dimmed());

        if let Some(TypeDef::Product { Product }) =
            schema.typespace.types.get(table.product_type_ref)
        {
            println!("\nFields ({}):", Product.elements.len());
            for element in &Product.elements {
                if let Some(field_name) = element.name.as_option() {
                    let field_type = format_type(&element.algebraic_type, type_names);
                    println!("  {} {}: {}", "▸".green(), field_name, field_type.cyan());
                }
            }
        }

        if !table.primary_key.is_empty() {
            println!("\nPrimary Key: {:?}", table.primary_key);
        }
    } else {
        println!("{} Table '{}' not found", "❌".red(), table_name);
        println!("\nAvailable tables:");
        for t in &schema.tables {
            println!("  - {}", t.name);
        }
    }
}

fn display_single_type(schema: &SatsSchema, type_names: &HashMap<usize, String>, type_name: &str) {
    let type_entry = type_names
        .iter()
        .find(|(_, name)| name.eq_ignore_ascii_case(type_name));

    if let Some((type_idx, real_name)) = type_entry {
        if let Some(type_def) = schema.typespace.types.get(*type_idx) {
            match type_def {
                TypeDef::Product { Product } => {
                    println!("\n{} {}", "📦 STRUCT:".blue(), real_name.bold());
                    println!("{}", "-".repeat(40));

                    if let Some(special) = detect_spacetimedb_type(Product) {
                        println!("SpacetimeDB Type: {}", special.yellow());
                    }

                    println!("\nFields ({}):", Product.elements.len());
                    for element in &Product.elements {
                        if let Some(field_name) = element.name.as_option() {
                            let field_type = format_type(&element.algebraic_type, type_names);
                            println!("  {} {}: {}", "▸".green(), field_name, field_type.cyan());
                        }
                    }
                }
                TypeDef::Sum { Sum } => {
                    display_single_enum_by_ref(schema, type_names, real_name, Sum);
                }
                _ => {
                    println!("{} '{}' is not a struct or enum", "❌".red(), type_name);
                }
            }
        }
    } else {
        println!("{} Type '{}' not found", "❌".red(), type_name);
        suggest_similar_types(type_names, type_name);
    }
}

fn display_single_enum(schema: &SatsSchema, type_names: &HashMap<usize, String>, enum_name: &str) {
    let type_entry = type_names
        .iter()
        .find(|(_, name)| name.eq_ignore_ascii_case(enum_name));

    if let Some((type_idx, real_name)) = type_entry {
        if let Some(TypeDef::Sum { Sum }) = schema.typespace.types.get(*type_idx) {
            display_single_enum_by_ref(schema, type_names, real_name, Sum);
        } else {
            println!("{} '{}' is not an enum", "❌".red(), enum_name);
            suggest_enum_types(schema, type_names);
        }
    } else {
        println!("{} Enum '{}' not found", "❌".red(), enum_name);
        suggest_enum_types(schema, type_names);
    }
}

fn display_single_enum_by_ref(
    _schema: &SatsSchema,
    type_names: &HashMap<usize, String>,
    real_name: &str,
    sum: &SumType,
) {
    println!("\n{} {}", "🔀 ENUM:".cyan(), real_name.bold());
    println!("{}", "-".repeat(40));

    if let Some(special) = detect_spacetimedb_sum_type(sum) {
        println!("SpacetimeDB Type: {}", special.yellow());
    }

    println!("\nVariants ({}):", sum.variants.len());
    for variant in &sum.variants {
        if let Some(variant_name) = variant.name.as_option() {
            match &variant.algebraic_type {
                AlgebraicType::Product { Product } if Product.elements.is_empty() => {
                    println!("  {} {}", "▸".green(), variant_name);
                }
                _ => {
                    let variant_type = format_type(&variant.algebraic_type, type_names);
                    println!(
                        "  {} {}({})",
                        "▸".green(),
                        variant_name,
                        variant_type.cyan()
                    );
                }
            }
        }
    }
}

fn display_search_results(schema: &SatsSchema, type_names: &HashMap<usize, String>, pattern: &str) {
    let pattern_lower = pattern.to_lowercase();

    println!("\n{} '{}'", "🔍 SEARCH RESULTS FOR:".yellow(), pattern);
    println!("{}", "=".repeat(60));

    // Search tables
    let matching_tables: Vec<_> = schema
        .tables
        .iter()
        .filter(|t| t.name.to_lowercase().contains(&pattern_lower))
        .collect();

    if !matching_tables.is_empty() {
        println!("\n{}", "📊 TABLES:".bold());
        for table in &matching_tables {
            let type_name = type_names
                .get(&table.product_type_ref)
                .cloned()
                .unwrap_or_else(|| format!("Type_{}", table.product_type_ref));
            println!(
                "  {} {} → {}",
                "▸".green(),
                table.name.bold(),
                type_name.dimmed()
            );
        }
    }

    // Search types
    let matching_types: Vec<_> = type_names
        .iter()
        .filter(|(type_idx, name)| {
            name.to_lowercase().contains(&pattern_lower)
                && !schema
                    .tables
                    .iter()
                    .any(|t| t.product_type_ref == **type_idx)
        })
        .collect();

    if !matching_types.is_empty() {
        println!("\n{}", "🔧 OTHER TYPES:".bold());
        for (type_idx, name) in &matching_types {
            if let Some(type_def) = schema.typespace.types.get(**type_idx) {
                match type_def {
                    TypeDef::Sum { Sum } => {
                        println!(
                            "  {} {} {}",
                            "🔀".cyan(),
                            name.bold(),
                            format!("(enum with {} variants)", Sum.variants.len()).dimmed()
                        );
                    }
                    TypeDef::Product { Product } => {
                        println!(
                            "  {} {} {}",
                            "📦".blue(),
                            name.bold(),
                            format!("(struct with {} fields)", Product.elements.len()).dimmed()
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    if matching_tables.is_empty() && matching_types.is_empty() {
        println!("{} No matches found for '{}'", "❌".red(), pattern);
    }
}

fn suggest_similar_types(type_names: &HashMap<usize, String>, search: &str) {
    println!("\nDid you mean one of these?");
    let search_lower = search.to_lowercase();

    let mut suggestions: Vec<_> = type_names
        .values()
        .filter(|name| {
            let name_lower = name.to_lowercase();
            name_lower.contains(&search_lower)
                || search_lower.contains(&name_lower)
                || name_lower.starts_with(&search_lower.chars().take(3).collect::<String>())
        })
        .take(5)
        .collect();

    suggestions.sort();
    for name in suggestions {
        println!("  - {}", name);
    }
}

fn suggest_enum_types(schema: &SatsSchema, type_names: &HashMap<usize, String>) {
    println!("\nAvailable enums:");
    let mut enums: Vec<_> = type_names
        .iter()
        .filter(|(idx, _)| matches!(schema.typespace.types.get(**idx), Some(TypeDef::Sum { .. })))
        .map(|(_, name)| name)
        .collect();

    enums.sort();
    for name in enums.iter().take(10) {
        println!("  - {}", name);
    }
}