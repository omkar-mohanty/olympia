use goblin::{Object, elf::Elf, error};
use std::{fs, io, path::Path};
use thiserror::Error;

pub struct Program {}

type Result<T> = std::result::Result<T, LoadError>;

pub fn load_elf(elf: Elf<'_>) -> Result<Program> {
    log::info!("Non Strippable Symbols : {:?}", &elf.dynsyms.len());
    log::info!("Total Strippable Symbols : {:?}", &elf.syms.len());

    Ok(Program {})
}

/// Loads the binary from the input path.
pub fn load(path: &Path) -> Result<Program> {
    let buffer = fs::read(path)?;
    match Object::parse(&buffer)? {
        Object::Elf(elf) => {
            load_elf(elf)
        }
        Object::PE(_pe) => {
            log::error!("pe files not supported");
            Err(LoadError::Unsupported)
        }
        Object::COFF(_coff) => {
            log::error!("coff files not supported");
            Err(LoadError::Unsupported)
        }
        Object::Mach(_mach) => {
            log::error!("mach files not supported");

            Err(LoadError::Unsupported)
        }
        _ => Err(LoadError::Unsupported),
    }
}

/// Custom error type for loading
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Unsupported file type")]
    Unsupported,
    #[error("Io Error")]
    Io(#[from] io::Error),
    #[error("Goblin Error")]
    Goblin(#[from] error::Error),
}
