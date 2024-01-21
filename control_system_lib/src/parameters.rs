use config::{Config, FileFormat};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    io::{self},
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

        let mut table_cs = Table::new();
        table_cs.insert("blocks".to_string(), toml::Value::Table(Table::new()));

        let mut write_back = Table::new();
        write_back.insert(control_sys_name.to_string(), toml::Value::Table(table_cs));

        Ok(ParameterStore {
            file: file.to_owned(),
            config,
            control_system_name: control_sys_name.to_string(),
            write_back,
        })
    }

    pub fn get_cs_params<T: DeserializeOwned + Serialize>(
        &mut self,
        default: T,
    ) -> Result<T, ParameterStoreError> {
        let default = Config::try_from(&default).unwrap();
        let key = format!("{}.params", self.control_system_name);
        let param: T = Config::builder()
            .set_default(key.as_str(), default.cache)?
            .add_source(self.config.clone())
            .build()?
            .get(key.as_str())?;

        let block_table = self.write_back.get_mut(&self.control_system_name).expect(
            "Internal toml table has a bad structure: does not contain control system root element",
        ).as_table_mut().expect("Internal toml table has a bad structure: control system root element is not a table");

        let v_table = Table::try_from(&param)?;
        block_table.insert("params".to_string(), toml::Value::Table(v_table));

        Ok(param)
    }

    pub fn get_block_params<T: DeserializeOwned + Serialize>(
        &mut self,
        block_name: &str,
        default: T,
    ) -> Result<T, ParameterStoreError> {
        let default = Config::try_from(&default).unwrap();
        let key = format!("{}.blocks.{}", self.control_system_name, block_name);
        let param: T = Config::builder()
            .set_default(key.as_str(), default.cache)?
            .add_source(self.config.clone())
            .build()?
            .get(key.as_str())?;

        let block_table = self
            .write_back
            .get_mut(&self.control_system_name)
            .expect("Internal toml table has a bad structure: does not contain control system root element")
            .get_mut("blocks")
            .expect("Internal toml table has a bad structure: Does not contain 'blocks' element")
            .as_table_mut()
            .expect("Internal toml table has a bad structure: 'blocks' element is not a table");

        let v_table = Table::try_from(&param)?;
        block_table.insert(block_name.to_string(), toml::Value::Table(v_table));

        Ok(param)
    }

    pub fn save(&self) -> Result<(), ParameterStoreError> {
        let ser_toml: String = toml::to_string_pretty(&self.write_back)?;

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
