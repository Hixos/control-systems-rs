use config::{Config, FileFormat};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::HashMap,
    default,
    fs::File,
    io::{self, BufReader, Read},
    path::{Path, PathBuf},
};
use thiserror::Error;
use toml::Table;

pub struct ParameterStore {
    file: PathBuf,
    config: Config,
    control_system_name: String,

    write_back: Table,
}

impl ParameterStore {
    pub fn new(file: &Path, control_sys_name: &str) -> Result<Self, ParameterStoreError> {
        let config = if file.exists() {
            Config::builder()
                .add_source(config::File::new(
                    file.as_os_str().to_str().unwrap(),
                    FileFormat::Toml,
                ))
                .build()?
        } else {
            Config::default()
        };

        Ok(ParameterStore {
            file: file.to_owned(),
            config,
            control_system_name: control_sys_name.to_string(),
            write_back: Table::new(),
        })
    }

    pub fn get_parameters<T: DeserializeOwned + Serialize + Clone>(
        &mut self,
        block_name: &str,
        default: T,
    ) -> Result<T, ParameterStoreError> {
        let default = Config::try_from(&default).unwrap();
        let key = format!("{}.{}", self.control_system_name, block_name);
        let param: T = Config::builder()
            .set_default(key.as_str(), default.cache)?
            .add_source(self.config.clone())
            .build()?
            .get(key.as_str())?;

        let v_table = Table::try_from(&param)?;
        self.write_back
            .insert(block_name.to_string(), toml::Value::Table(v_table));

        Ok(param)
    }

    pub fn save(&self) -> Result<(), ParameterStoreError> {
        let mut root = Table::new();
        root.insert(
            self.control_system_name.clone(),
            toml::Value::Table(self.write_back.clone()),
        );

        let ser_toml: String = toml::to_string_pretty(&root)?;

        std::fs::write(&self.file, ser_toml)?;

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ParameterStoreError {
    #[error("File operation error")]
    Io(#[from] io::Error),

    #[error(transparent)]
    Deserialization(#[from] DeserializationError),

    #[error(transparent)]
    Serialization(#[from] SerializationError),
}

impl From<config::ConfigError> for ParameterStoreError {
    fn from(value: config::ConfigError) -> Self {
        ParameterStoreError::Deserialization(value.into())
    }
}

impl From<toml::ser::Error> for ParameterStoreError {
    fn from(value: toml::ser::Error) -> Self {
        ParameterStoreError::Serialization(value.into())
    }
}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct DeserializationError {
    #[from]
    source: config::ConfigError,
}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct SerializationError {
    #[from]
    source: toml::ser::Error,
}
