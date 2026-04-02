use wincode::{SchemaRead, SchemaWrite};

#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
pub enum ServerMsg {
    Hello { username: String },
    Unauthenticated,
}
