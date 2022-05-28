GRPC
====

Server
------

Start the server, from the project root:
```
./target/debug/playrs -l=info grpc start 0.0.0.0:5001 funk
```

Client
------

1. Using [evan](https://github.com/ktr0731/evans#basic-usage) do from this directory,

```
evan --proto ./proto/hello.proto -p 5001
```

2. Using the client command, from the project root:

```
./target/debug/playrs grpc call https://0.0.0.0:5001 --name <message>
```