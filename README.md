# SpacetimeDB Schema Tool

A minimal CLI tool for inspecting SpacetimeDB schemas. This tool allows you to explore and understand the structure of any SpacetimeDB database.

## Features

- Fetch and display schema from live SpacetimeDB instances
- Pretty-print schemas with colored output
- Filter by table, type, or enum
- Search for specific patterns
- Support for both local and cloud databases

## Installation

```bash
cargo build --release
```

## Usage

```bash
# Fetch from local instance
spacetime-schema-tool --db my_database

# Fetch from cloud
spacetime-schema-tool --db my_database --cloud

# Fetch from custom server
spacetime-schema-tool --db my_database --server http://myserver:3000

# Output as JSON
spacetime-schema-tool --db my_database --format json

# Filter to show only a specific table
spacetime-schema-tool --db my_database --table users

# Search for types/tables containing a pattern
spacetime-schema-tool --db my_database --search "user"
```

## Examples

```bash
# Show all tables and types
spacetime-schema-tool --db my_database

# Show only the 'users' table
spacetime-schema-tool --db my_database --table users

# Show only enums
spacetime-schema-tool --db my_database --enum Status

# Search for anything containing "user"
spacetime-schema-tool --db my_database -s user
```

## Output Format

The tool provides a colored, hierarchical view of:
- Tables with their fields and types
- Enums with their variants
- Structs with their fields
- Special SpacetimeDB types (Identity, Timestamp, Duration, ScheduledAt)
- Option<T> types are displayed clearly

## License

This project is released into the public domain using The Unlicense. See the LICENSE file for details.