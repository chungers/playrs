Rocksdb
=======

This module uses Rocksdb as storage backend for a graph database.
Currently the db backend is single threaded since it's a CLI based tool
for storing nodes and edges.

## TODO

+ Use column family to store sequence
+ Use column family to index nodes by name and store nodes
+ Use column family to index edges by (head,tail) and name