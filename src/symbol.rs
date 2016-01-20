use std::collections::{HashMap};
use std::sync::Arc;

#[derive(Clone, Debug, Hash)]
pub struct Symbol {
    source: Arc<String>,
    #[cfg(debug)]
    table: *const Table
}

impl Symbol {
    #[cfg(debug)]
    pub fn from_str(source: &str, table: *const Table) -> Self {
        Symbol {
            source: Arc::new(source.to_owned()),
            table: table
        }
    }

    #[cfg(not(debug))]
    pub fn from_str(source: &str, _table: *const Table) -> Self {
        Symbol {
            source: Arc::new(source.to_owned())
        }
    }

    #[cfg(debug)]
    fn check_table(&self, other: &Self) {
        assert_eq!(self.table, other.table);
    }

    #[cfg(not(debug))]
    fn check_table(&self, _other: &Self) {
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.check_table(other);
        (&*self.source as *const _) == (&*other.source as *const _)
    }
}

impl Eq for Symbol {}

pub struct Table {
    symbols: HashMap<Box<str>, Symbol>
}

impl Table {
    pub fn new() -> Self {
        Table {
            symbols: HashMap::new()
        }
    }

    pub fn intern(&mut self, source: &str) -> Symbol {
        if let Some(symbol) = self.symbols.get(source) {
            return symbol.clone()
        }
        let new_symbol = Symbol::from_str(source, self);
        self.symbols.insert(source.to_owned().into_boxed_str(), new_symbol.clone());
        new_symbol
    }
}

#[test]
fn it_interns() {
    let mut tab = Table::new();
    assert_eq!(tab.intern("test"), tab.intern(&"test".to_owned()));
}
