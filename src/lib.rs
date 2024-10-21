#![doc = include_str!("../README.md")]

use std::any::{Any, TypeId};

use serde::de::DeserializeOwned;
use thiserror::Error;
use toml_edit::DocumentMut;

/// Trait used to determine config versions and migration order
///
/// You should probably not implement this yourself, but instead use the [`build_migration_chain!`] macro.
pub trait Migrate: From<Self::From> + DeserializeOwned + Any {
    type From: Migrate;
    const VERSION: i64;

    fn migrate_from_doc(version: i64, doc: DocumentMut) -> Result<Self, Error> {
        if version == Self::VERSION {
            Ok(toml_edit::de::from_document(doc)?)
        } else if TypeId::of::<Self>() == TypeId::of::<Self::From>() {
            Err(Error::NoValidVersion)
        } else {
            Self::From::migrate_from_doc(version, doc).map(Into::into)
        }
    }
}

/// Struct that contains some configuration on how to migrate a config
///
/// ```no_run
/// let migrator = ConfigMigrator::new("version").with_default_version(0);
///
/// let (config, migration_occured) = migrator.migrate::<ConfigV2>(config_str).unwrap();
/// ```
pub struct ConfigMigrator<'a> {
    version_key: &'a str,
    default_version: Option<i64>,
}

impl<'a> ConfigMigrator<'a> {
    /// Creates a new [`ConfigMigrator`] using the provided key to find the version of the config
    #[must_use]
    pub const fn new(version_key: &'a str) -> Self {
        Self {
            version_key,
            default_version: None,
        }
    }

    /// Adds a default version to use if it cannot be read from the config file
    #[must_use]
    pub const fn with_default_version(mut self, default_version: i64) -> Self {
        self.default_version = Some(default_version);
        self
    }

    /// Handles the migration between versions of a configuration
    ///
    /// On success, returns a tuple with the config and whether any migrations were performed.
    /// Errors if it could not read the version (and no default was provided), if the version failed to match
    /// any of the config structs, or if the config file failed to parse.
    pub fn migrate_config<T: Migrate>(&self, config_str: &str) -> Result<(T, bool), Error> {
        let mut doc = config_str.parse::<DocumentMut>()?;
        let version = doc
            .remove(self.version_key)
            .and_then(|x| x.as_integer())
            .or(self.default_version)
            .ok_or(Error::NoValidVersion)?;

        let config = T::migrate_from_doc(version, doc)?;
        let migration_occured = version != T::VERSION;

        Ok((config, migration_occured))
    }
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

#[derive(Debug, Error)]
pub enum Error {
    /// Syntax error when parsing the TOML
    #[error("parsing error")]
    Parse(#[from] toml_edit::TomlError),
    /// Error when deserializing the TOML
    #[error("deserialization error")]
    Deser(#[from] toml_edit::de::Error),
    /// Either version field could not be read or provided version field doesn't match a valid version
    #[error("no valid config version")]
    NoValidVersion,
}
