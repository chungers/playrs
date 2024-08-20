Rocksdb
=======

This module uses Rocksdb as storage backend for a graph database.
Currently the db backend is single threaded since it's a CLI based tool
for storing nodes and edges.

## TODO

- [x] Counter column families to track number of types, objects.
- [ ] ?? Generic value container as the value of the Value Index (column family).
This is a container protobuf that has Any and type information that
tracks previous versions of the object as protobuf.  The container
is useful for rolling back and for updating of indexes since it will have
a previous version of the object.  May have to limit the number of versions to avoid huge protobufs.
- [x] Value index - an entity type needs to identify a value index where
(id, entity_as_bytes) is stored.
- [x] Generic implementation of Operations by asking entities to provide a value index.
- [x] Re-index for mutations: index updates must also take into account of (E and E'), where E' is the mutated E.
Mutated E has index key (E.a', E.b') != (E.a, E.b) so re-indexing
requires a remove and insert added to the batch write txn.
- [ ] Deletion - deletion is by updating the Value Index with a wrapper
protobuf type that backs up the previous versions.
- [x] Index lookup by name for nodes and edges - iterator (names not unique key)
- [x] Index lookup by (head,tail) id for edges - iterator
- [x] Use column family to store sequence
- [x] Use column family to store (id, node)
- [x] Use column family to store (id, edge)
- [x] Transactions for inserting into home cf (for nodes, edges) and indexes
