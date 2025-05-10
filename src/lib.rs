#![doc = include_str!("../README.md")]

use std::any::Any;

use serde::de::DeserializeOwned;

/// Trait used to determine config versions and migration order
///
/// You should probably not implement this yourself, but instead use the [`build_migration_chain!`] macro.
pub trait Migrate: From<Self::From> + DeserializeOwned + Any {
    type From: Migrate;
    const VERSION: i64;

    fn migrate_from_str(version: i64, config_str: &str) -> Result<Self, basic_toml::Error> {
        if version == Self::VERSION {
            basic_toml::from_str(config_str)
        } else {
            Self::From::migrate_from_str(version, config_str).map(Into::into)
        }
    }
}

pub trait Version: DeserializeOwned {
    fn version(&self) -> i64;
}

pub fn migrate_config<T: Migrate, Ver: Version>(
    config_str: &str,
) -> Result<(T, bool), basic_toml::Error> {
    let version: Ver = basic_toml::from_str(config_str)?;
    let version = version.version();

    let config = T::migrate_from_str(version, config_str)?;
    let migration_occured = version != T::VERSION;

    Ok((config, migration_occured))
}

/// Generates a chain connecting different config versions with the [`Migrate`] trait
///
/// ```no_run
/// build_migration_chain!(ConfigV1 = 1, ConfigV2 = 2, ConfigV3 = 3);
/// ```
#[macro_export]
macro_rules! build_migration_chain {
    ($type:ident = $ver:literal) => {
        impl $crate::Migrate for $type {
            type From = Self;
            const VERSION: i64 = $ver;
        }
    };
    ($first_type:ident = $first_ver:literal, $($rest:tt)*) => {
        build_migration_chain!($first_type = $first_ver);

        build_migration_chain!(@internal $first_type, $($rest)*);
    };
    (@internal $prev_type:ident, $type:ident = $ver:literal $(, $($rest:tt)*)?) => {
        impl $crate::Migrate for $type {
            type From = $prev_type;
            const VERSION: i64 = $ver;
        }

        $(build_migration_chain!(@internal $type, $($rest)*);)?
    };
}
