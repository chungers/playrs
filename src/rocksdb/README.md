Rocksdb
=======

This module uses Rocksdb as storage backend for a graph database.
Currently the db backend is single threaded since it's a CLI based tool
for storing nodes and edges.

## TODO

- [x] Use column family to store sequence
- [x] Use column family to store (id, node)
- [x] Use column family to store (id, edge)
- [ ] Transactions for inserting into home cf (for nodes, edges) and indexes
- [ ] Index lookup by name for nodes and edges - iterator (names not unique key)
- [ ] Index lookup by (head,tail) id for edges - iterator
