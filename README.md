# InscribeMeMaybe 

InscribeMeMaybe CLI simplifies the process of inscribing messages and sending them to the chain.

> [!CAUTION]
> This is intended for testing purposes only and still WIP.
> Use at your own risk.

## Usage

```bash
$ inscribememaybe mint "{"p":"fair-20","op":"mint","tick":"brr","amt":"1000"}" --private-key "your_private_key" --rpc-url <rpc-url> --transactions 10
```

The first time this is run it will create an `inscribememaybe.sqlite` sqlite database in the current directory. This database will be used to keep track of the transactions

## FAQ

### Q: My mint transaction succeeded, does that mean the mint was successful?

A: Maybe

### Q: My deploy transaction succeeded, does that mean the deployment was successful?

A: Maybe


## License

Licensed under either of these:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  https://opensource.org/licenses/MIT)

