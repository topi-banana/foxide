use bincode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub enum ServerMsg {
    Hello { username: String },
    Unauthenticated,
}
