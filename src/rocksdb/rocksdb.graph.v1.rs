#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Node {
    #[prost(uint64, tag = "1")]
    pub id: u64,
    #[prost(uint64, tag = "2")]
    pub type_code: u64,
    #[prost(string, tag = "3")]
    pub type_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Edge {
    #[prost(uint64, tag = "1")]
    pub id: u64,
    #[prost(uint64, tag = "2")]
    pub type_code: u64,
    #[prost(string, tag = "3")]
    pub type_name: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    pub head: u64,
    #[prost(uint64, tag = "5")]
    pub tail: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Document {
    #[prost(uint64, tag = "1")]
    pub id: u64,
    #[prost(map = "string, string", tag = "2")]
    pub metadata: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        ::prost::alloc::string::String,
    >,
    #[prost(string, tag = "3")]
    pub content_type: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "4")]
    pub content: ::prost::alloc::vec::Vec<u8>,
}
