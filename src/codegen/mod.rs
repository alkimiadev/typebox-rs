mod rust;
mod typescript;

pub use rust::RustGenerator;
pub use typescript::TypeScriptGenerator;

use crate::schema::Schema;
use std::collections::HashMap;

pub struct SchemaRegistry {
    schemas: HashMap<String, Schema>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, name: impl Into<String>, schema: Schema) {
        self.schemas.insert(name.into(), schema);
    }
    
    pub fn get(&self, name: &str) -> Option<&Schema> {
        self.schemas.get(name)
    }
    
    pub fn schemas(&self) -> impl Iterator<Item = (&String, &Schema)> {
        self.schemas.iter()
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}
