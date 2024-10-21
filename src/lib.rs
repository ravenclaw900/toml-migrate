use std::any::{Any, TypeId};

use serde::de::DeserializeOwned;
use thiserror::Error;
use toml_edit::DocumentMut;

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

pub struct ConfigMigrator<'a> {
    version_key: &'a str,
    default_version: Option<i64>,
}

impl<'a> ConfigMigrator<'a> {
    pub fn new(version_key: &'a str) -> Self {
        Self {
            version_key,
            default_version: None,
        }
    }

    pub fn with_default_version(mut self, default_version: i64) -> Self {
        self.default_version = Some(default_version);
        self
    }

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
    #[error("parsing error")]
    Parse(#[from] toml_edit::TomlError),
    #[error("deserialization error")]
    Deser(#[from] toml_edit::de::Error),
    #[error("no valid config version")]
    NoValidVersion,
}
