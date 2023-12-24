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

## Example

1. Launch [anvil](https://github.com/foundry-rs/foundry) in a separate terminal

```bash
anvil
```

2. mint

```bash
cargo r -- mint '{"p":"fair-20","op":"mint","tick":"brr","amt":"1000"}' --pk "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" --rpc-url "ws://127.0.0.1:8545" --transactions 10
2023-12-24T12:02:31.101724Z  INFO connect_to{url="sqlite://inscribememaybe.sqlite"}: inscribememaybe: connected to database
2023-12-24T12:02:38.148369Z  INFO inscribememaybe: minted hash=0x9daa7e3ddedea4863ef413eb1bbc24e60c0d381935e9062cf4fc94bc3702735d block=1
2023-12-24T12:02:38.150491Z  INFO inscribememaybe: minted hash=0xcbae71c3d907625c9912c96bf129dd3377eda13520a37be9a3ffa9c2e95bc478 block=1
2023-12-24T12:02:38.151506Z  INFO inscribememaybe: minted hash=0x013129931240dbb7d31f9269b954d1d1e582109bafe80eed8529a7b1f02557b2 block=1
2023-12-24T12:02:38.153320Z  INFO inscribememaybe: minted hash=0xc0880f2a308bc866d71af8ed3ff62d5401ed456c6bb20d2540b2c392f520b997 block=2
2023-12-24T12:02:38.154102Z  INFO inscribememaybe: minted hash=0x5535e5290ffc6766d842adbf34367690dd03364781fd48ac6a7db8a5e09683a8 block=1
2023-12-24T12:02:38.154674Z  INFO inscribememaybe: minted hash=0x0720b242e09dfdd323c13f4c6437da55f4ee8ccde9c4e9dee6c24584424d7961 block=1
2023-12-24T12:02:38.155154Z  INFO inscribememaybe: minted hash=0xf878ffa5b1c07ceab9856e46d4dd04f4587f8b4f63ed8345f34c2dae203d6884 block=1
2023-12-24T12:02:38.155860Z  INFO inscribememaybe: minted hash=0xe517d59f0a7949adcdda21dc141fd058c99851095e3fe8c735de88d516fe9711 block=1
2023-12-24T12:02:38.156407Z  INFO inscribememaybe: minted hash=0x3a4200311341920dac33705ec1f594639f6275f594d449c096ca50b15f31563b block=2
2023-12-24T12:02:38.156805Z  INFO inscribememaybe: minted hash=0x050389319e03d242cf9ddc3c4fb67a28ed3fef9a4c201963677cdae11f40e335 block=2
2023-12-24T12:02:38.157271Z  INFO inscribememaybe: finished minting mints=10
```

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

