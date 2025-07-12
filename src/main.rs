use anyhow::Result;
use clap::Parser;

mod schema;
mod spacetime_client;

use schema::{SchemaArgs, OutputFormat};

#[derive(Parser)]
#[command(name = "spacetime-schema-tool")]
#[command(about = "SpacetimeDB schema inspection tool", long_about = None)]
#[command(version)]
struct Cli {
    /// Database name
    #[arg(long)]
    db: String,

    /// Server URL (default: <http://localhost:3000>)
    #[arg(long, default_value = "http://localhost:3000")]
    server: String,

    /// Schema version to fetch
    #[arg(long = "schema-version")]
    version: Option<String>,

    /// Use `SpacetimeDB` cloud
    #[arg(long, conflicts_with = "server")]
    cloud: bool,

    /// Output format
    #[arg(long, value_enum, default_value = "pretty")]
    format: OutputFormat,

    /// Filter to show only specific table
    #[arg(long, conflicts_with_all = ["type_filter", "enum_filter"])]
    table: Option<String>,

    /// Filter to show only specific type
    #[arg(long = "type", conflicts_with_all = ["table", "enum_filter"])]
    type_filter: Option<String>,

    /// Filter to show only specific enum
    #[arg(long = "enum", conflicts_with_all = ["table", "type_filter"])]
    enum_filter: Option<String>,

    /// Search pattern (matches table/type/enum names)
    #[arg(long, short = 's')]
    search: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let args = SchemaArgs {
        db: cli.db,
        server: cli.server,
        version: cli.version,
        cloud: cli.cloud,
        format: cli.format,
        table: cli.table,
        type_filter: cli.type_filter,
        enum_filter: cli.enum_filter,
        search: cli.search,
    };

    schema::fetch_schema(args).await?;

    Ok(())
}