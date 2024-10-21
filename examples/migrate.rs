use serde::Deserialize;
use toml_migrate::{build_migration_chain, ConfigMigrator};

#[derive(Debug, Deserialize)]
struct ConfigV1 {
    name: String,
    timeout: u32,
}

#[derive(Debug, Deserialize)]
struct ConfigV2 {
    name: String,
    timeout: u32,
    retries: u8,
}

impl From<ConfigV1> for ConfigV2 {
    fn from(prev: ConfigV1) -> Self {
        Self {
            name: prev.name,
            timeout: prev.timeout,
            retries: 4,
        }
    }
}

build_migration_chain!(ConfigV1 = 1, ConfigV2 = 2);

fn main() {
    let config_str = r#"
        version = 1
        name = "MyApp"
        timeout = 60
    "#;

    let migrator = ConfigMigrator::new("version");

    let (config, migrated) = migrator
        .migrate_config::<ConfigV2>(config_str)
        .expect("failed to read and/or migrate config");

    if migrated {
        println!("Migration occured! New config: {:?}", config);
    }
}
