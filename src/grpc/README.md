GRPC
====

To test using [evan](https://github.com/ktr0731/evans#basic-usage) do

1. Start the server, from the project root:
```
./target/debug/playrs -l=info grpc start 0.0.0.0:5001 funk
```

2. From this directory,
```
evan --proto ./proto/hello.proto -p 5001
```
