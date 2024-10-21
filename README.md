# toml_migrate
A crate that lets you read versioned config files and easily migrate them to the latest version.
Inspired by the [magic_migrate](https://crates.io/crates/magic_migrate) library.

## Why?
Many applications need their configuration files to evolve over time as features get added and changed. This aims to simplify the process of taking a old config file and transforming it to the latest version.

## How does it work?
For every version of your library, define a configuration struct that can be deserialized. This struct should not include the version field:
```rust
#[derive(Deserialize)]
struct ConfigV1 {
    name: String,
    timeout: u32,
}

#[derive(Deserialize)]
struct ConfigV2 {
    name: String,
    timeout: u32,
    // new!
    retries: u8,
}
```
Then, implement `From<PreviousConfig>` for all of your config types:
```rust
impl From<ConfigV1> for ConfigV2 {
    fn from(prev: ConfigV1) -> Self {
        Self {
            name: prev.name,
            timeout: prev.timeout,
            retries: 4,
        }
    }
}
```
Finally, use the [`build_migration_chain!`] macro to automatically implement the `Migrate` trait for all of your structs:
```rust
build_migration_chain!(ConfigV1 = 1, ConfigV2 = 2);
```
From there, you can use the [`ConfigMigrator`] to easily migrate your config file from a string:
```rust
fn read_config() -> ConfigV2 {
    let config_str = r#"
        version = 1
        name = "MyApp"
        timeout = 60
    "#;

    let migrator = ConfigMigrator::new("version");
    let (config, _) = migrator.migrate_config(config_str).unwrap();

    config
}
```