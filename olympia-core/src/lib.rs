use goblin::{Object, elf::Elf, error};
use petgraph::{Directed, graph::Graph};
use std::{borrow::Cow, fs, io, path::Path};
use thiserror::Error;
use uuid::Uuid;

pub struct Function {}

pub enum Rvalue {
    /// Undefined value of unknown length
    Undefined,
    /// Variable reference
    Variable {
        /// Variable name. Names starting with "__" are reserved.
        name: Cow<'static, str>,
        /// SSA subscript. This can be set to None in most cases.
        subscript: Option<usize>,
        /// First bit of the variable we want to read. Can be set to 0 in most cases.
        offset: usize,
        /// Number of bits we want to read.
        size: usize,
    },
    /// Constant
    Constant {
        /// Value
        value: u64,
        /// Size in bits
        size: usize,
    },
}

impl Rvalue {
    pub fn new_u64(v: u64) -> Self {
        Rvalue::Constant { value: v, size: 64 }
    }
}

/// Node of the program call
pub enum CallTarget {
    /// Reference to external function
    Symbolic(String, Uuid),
    /// Resolved and disassembled function
    Concrete(Function),
    /// Resolved but not yet disassembled
    Todo(Rvalue, Option<String>, Uuid),
}

pub struct Program {
    uuid: Uuid,
    call_graph: Graph<CallTarget, Directed>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            uuid: Uuid::new_v4(),
            call_graph: Graph::new(),
        }
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn call_graph(&mut self) -> &mut Graph<CallTarget, Directed> {
        &mut self.call_graph
    }
}

type Result<T> = std::result::Result<T, LoadError>;

pub fn load_elf(elf: Elf<'_>, name: String) -> Result<Program> {
    log::info!("Non Strippable Symbols : {:?}", &elf.dynsyms.len());
    log::info!("Total Strippable Symbols : {:?}", &elf.syms.len());

    let mut program = Program::new();

    let elf_entry = elf.entry;

    let name = if let Some(ref soname) = elf.soname {
        soname.to_string()
    } else {
        name
    };

    program.call_graph().add_node(CallTarget::Todo(
        Rvalue::new_u64(elf_entry),
        Some(name),
        Uuid::new_v4(),
    ));

    Ok(program)
}

/// Loads the binary from the input path.
pub fn load(path: &Path) -> Result<Program> {
    let buffer = fs::read(path)?;

    let name = path
        .file_name()
        .map(|x| x.to_string_lossy())
        .unwrap_or("encoding_error".to_string().into())
        .to_string();

    match Object::parse(&buffer)? {
        Object::Elf(elf) => load_elf(elf, name),
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
