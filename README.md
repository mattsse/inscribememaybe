# InscribeMeMaybe CLI

InscribeMeMaybe CLI simplifies the process of inscribing important messages, sign them using your private key, and effortlessly send them to a blockchain that includes them (maybe).

> [!CAUTION]
> This is intended for testing purposes only. 
> Use at your own risk.

## Installation

```bash
$ cargo install inscribememaybe
```

## Usage

```bash
$ inscribememaybe --message "{"p":"fair-20","op":"mint","tick":"brr","amt":"1000"}" --private-key "your_private_key" --rpc-url <rpc-url>
```


## FAQ

### Q: My mint transaction succeeded, does that mean the mint was successful?

A: Maybe

### Q: My deploy transaction succeeded, does that mean the deployment was successful?

A: Maybe


## License

This project is licensed under the [MIT License](LICENSE).
