#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DataRecord {
    #[prost(uint64, tag = "1")]
    pub id: u64,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "3")]
    pub description: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Relation {
    #[prost(uint64, tag = "1")]
    pub id: u64,
    #[prost(string, tag = "2")]
    pub relation: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub head: u64,
    #[prost(uint64, tag = "4")]
    pub tail: u64,
    #[prost(string, optional, tag = "5")]
    pub description: ::core::option::Option<::prost::alloc::string::String>,
}
