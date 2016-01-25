use std::collections::{HashMap};
use std::sync::Arc;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub struct Symbol {
    source: Arc<String>,
    #[cfg(debug)]
    table: *const Table
}

impl Symbol {
    #[cfg(debug)]
    pub fn from_str_table(source: &str, table: *const Table) -> Self {
        Symbol {
            source: Arc::new(source.to_owned()),
            table: table
        }
    }

    #[cfg(not(debug))]
    pub fn from_str_table(source: &str, _table: *const Table) -> Self {
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

impl Hash for Symbol {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        state.write_usize(&*self.source as *const String as usize)
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.check_table(other);
        (&*self.source as *const String) == (&*other.source as *const String)
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
        let new_symbol = Symbol::from_str_table(source, self);
        self.symbols.insert(source.to_owned().into_boxed_str(), new_symbol.clone());
        new_symbol
    }

    pub fn is_interned(&self, source: &str) -> bool {
        self.symbols.contains_key(source)
    }
}

#[test]
fn it_interns() {
    let mut tab = Table::new();
    let mut tab2 = Table::new();
    assert_eq!(tab.intern("test"), tab.intern(&"test".to_owned()));
    assert!(tab.intern("test") != tab2.intern("test"));
}
