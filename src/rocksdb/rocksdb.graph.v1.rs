#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Node {
    #[prost(uint64, tag = "1")]
    pub id: u64,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "3")]
    pub doc: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Edge {
    #[prost(uint64, tag = "1")]
    pub id: u64,
<<<<<<<< HEAD:src/rocksdb/graph.proto.v1.rs
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
========
    #[prost(uint64, tag = "2")]
>>>>>>>> 67429fd (Refactoring open_db; start building a graph db):src/rocksdb/rocksdb.graph.v1.rs
    pub head: u64,
    #[prost(uint64, tag = "3")]
    pub tail: u64,
    #[prost(string, tag = "4")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "5")]
    pub doc: ::core::option::Option<::prost::alloc::string::String>,
}
