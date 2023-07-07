

Encrypting service account key

```
openssl enc -aes-256-cbc -salt -in playrs-2023-07-05-489b4cc9b61e.json -out playrs-2023-07-05-489b4cc9b61e.json.enc
```

Decrypting service account key

```
openssl enc -aes-256-cbc -d -in playrs-2023-07-05-489b4cc9b61e.json.enc -out playrs-2023-07-05-489b4cc9b61e.json
```
