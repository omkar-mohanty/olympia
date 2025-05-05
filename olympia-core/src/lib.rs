use goblin::{
    Object,
    elf::{self, Elf},
    error,
};
use petgraph::{graph::NodeIndex, prelude::StableGraph};
use std::{borrow::Cow, collections::HashMap, fs, io, path::Path};
use thiserror::Error;
use uuid::Uuid;
#[derive(Clone, Copy, Debug)]
pub struct Function {}

#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub enum CallTarget {
    /// Reference to external function
    Symbolic(String, Uuid),
    /// Resolved and disassembled function
    Concrete(Function),
    /// Resolved but not yet disassembled
    Todo(Rvalue, Option<String>, Uuid),
}

/// The type of Call used by the function
#[derive(Clone, Debug)]
pub enum CallKind {
    /// Not yet disassembled
    Unresolved,
    /// Conditional call via if statements, while etc
    Conditional,
    /// Goto statements
    Unconditional,
    /// Function calls
    Call,
}

pub struct Program {
    uuid: Uuid,
    call_graph: StableGraph<CallTarget, CallKind>,
    symbol_table: HashMap<Uuid, NodeIndex>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            uuid: Uuid::new_v4(),
            call_graph: StableGraph::new(),
            symbol_table: HashMap::new(),
        }
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn call_graph(&self) -> &StableGraph<CallTarget, CallKind> {
        &self.call_graph
    }

    pub fn call_graph_mut(&mut self) -> &mut StableGraph<CallTarget, CallKind> {
        &mut self.call_graph
    }

    pub fn symbol_table_mut(&mut self) -> &mut HashMap<Uuid, NodeIndex> {
        &mut self.symbol_table
    }

    pub fn symbol_table(&self) -> &HashMap<Uuid, NodeIndex> {
        &self.symbol_table
    }
}

type Result<T> = std::result::Result<T, LoadError>;

pub fn load_elf(elf: Elf, name: String) -> Result<Program> {
    log::info!("Non Strippable Symbols : {:?}", &elf.dynsyms.len());
    log::info!("Total Strippable Symbols : {:?}", &elf.syms.len());

    let mut program = Program::new();

    let elf_entry = elf.entry;

    let name = if let Some(ref soname) = elf.soname {
        soname.to_string()
    } else {
        name
    };

    let entry_uuid = Uuid::new_v4();

    let entry_node_index = program.call_graph_mut().add_node(CallTarget::Todo(
        Rvalue::new_u64(elf_entry),
        Some(name),
        entry_uuid,
    ));

    program
        .symbol_table_mut()
        .insert(entry_uuid, entry_node_index);

    let add_sym = |program: &mut Program, sym: &elf::Sym, name: &str| {
        let name = name.to_string();
        let addr = sym.st_value;
        let sym_uuid = Uuid::new_v4();
        if sym.is_function() {
            if sym.is_import() {
                let sym_node_index = program
                    .call_graph_mut()
                    .add_node(CallTarget::Symbolic(name, sym_uuid));
                program.symbol_table_mut().insert(sym_uuid, sym_node_index);
            } else {
                let sym_node_index = program.call_graph_mut().add_node(CallTarget::Todo(
                    Rvalue::new_u64(addr),
                    Some(name),
                    sym_uuid,
                ));
                program.symbol_table_mut().insert(sym_uuid, sym_node_index);
            }
        }
    };

    //TODO: Resolve import addresses in the binary
    //

    for sym in &elf.dynsyms {
        let name = &elf.dynstrtab[sym.st_name];
        add_sym(&mut program, &sym, name);
    }

    for sym in &elf.syms {
        let name = &elf.strtab[sym.st_name];

        add_sym(&mut program, &sym, &name);
    }

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
