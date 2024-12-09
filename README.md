## About This Template

This template is based on the `polkadot-sdk-minimal-template`. This is the most bare-bone template
that ships with `polkadot-sdk`, and most notably has no consensus mechanism. That is, any node in
the network can author blocks. This makes it possible to easily run a single-node network with any
block-time that we wish.

### ☯️ `omni-node`-only

Moreover, this template has been stripped to only contain the `runtime` part of the template. This
is because we provide you with an omni-node that can run this runtime. An `omni-node` is broadly a
substrate-based node that has no dependency to any given runtime, and can run a wide spectrum of
runtimes. The `omni-node` provided below is based on the aforementioned template and therefore has
no consensus engine baked into it.

## How to Run

### Individual Pallets

To test while developing, without a full build:

```sh
cargo test -p <pallet-name>
```

### Entire Runtime

#### Using `omni-node`

First, make sure to install the special omni-node of the PBA assignment, if you have not done so
already from the previous activity.

```sh
cargo install --force --git https://github.com/kianenigma/pba-omni-node.git
```

Then, you have two options:

1. Run with the default genesis using the `--runtime` flag:

```sh
cargo build --release
pba-omni-node --runtime ./target/release/wbuild/daniel-defi-hub-runtime/daniel_defi_hub_runtime.wasm --tmp
```

2. Run with a more flexible genesis using the `--chain` flag:

```sh
cargo install chain-spec-builder
chain-spec-builder create -n daniel-dex-hub -i 6969696969 -t Development -r target/release/wbuild/daniel-defi-hub-runtime/daniel_defi_hub_runtime.wasm default
```

Feel free to populate your chain-spec file then with more accounts, like:

```json
// under `balances.balance`
["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", 1000000000000000000],
["5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", 1000000000000000000],
["5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y", 1000000000000000000],
["5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy", 1000000000000000000],
["5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw", 1000000000000000000],
["5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL", 1000000000000000000],
["5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY", 1000000000000000000],
["5HpG9w8EBLe5XCrbczpwq5TSXvedjrBGCwqxK1iQ7qUsSWFc", 1000000000000000000],
["5Ck5SLSHYac6WFt5UZRSsdJjwmpSZq85fd5TRNAdZQVzEAPT", 1000000000000000000],
["5HKPmK9GYtE1PSLsS1qiYU9xQ9Si1NcEhdeCq9sw5bqu4ns8", 1000000000000000000],
["5FCfAonRZgTFrTd9HREEyeJjDpT397KMzizE6T3DvebLFE7n", 1000000000000000000],
["5CRmqmsiNFExV6VbdmPJViVxrWmkaXXvBrSX8oqBT8R9vmWk", 1000000000000000000],
["5Fxune7f71ZbpP2FoY3mhYcmM596Erhv1gRue4nsPwkxMR4n", 1000000000000000000],
["5CUjxa4wVKMj3FqKdqAUf7zcEMr4MYAjXeWmUf44B41neLmJ", 1000000000000000000]
```

Add genesis assets:

```json
// under "assets"
"accounts": [
    [1, "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", 1000000000000000000],
    [1, "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", 1000000000000000000],
    [1, "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y", 1000000000000000000],
    [1, "5CUjxa4wVKMj3FqKdqAUf7zcEMr4MYAjXeWmUf44B41neLmJ", 1000000000000000000],
    [2, "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", 1000000000000000000],
    [2, "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", 1000000000000000000],
    [2, "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y", 1000000000000000000],
    [2, "5CUjxa4wVKMj3FqKdqAUf7zcEMr4MYAjXeWmUf44B41neLmJ", 1000000000000000000]
],
"assets": [
    [1, "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", true, 1000000000000000000],
    [2, "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", true, 1000000000000000000]
],
"metadata": [
    [1, [ 84, 111, 107, 101, 110, 49 ], [ 84, 49 ], 1],
    [2, [ 84, 111, 107, 101, 110, 50 ], [ 84, 50 ], 1]
]
```

You can create your own pair or add genesis pair builder:
```json
// under "antiMevAmm.pairs"
["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", 1, 1000, 1000000000000, 1000000000000]
```

If you wish to set "Alice" as the sudo:

```json
// under `sudo.key`
"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
```

And more details like:

```json
"chainType": "Development"
"properties": {
  "tokenDecimals": 1,
  "tokenSymbol": "D"
}
```

Both of these patterns have already been explored in the FRAMELess activity, so you should be
familiar with them.

Run with chain spec

```sh
cargo build --release
pba-omni-node --runtime ./target/release/wbuild/daniel-defi-hub-runtime/daniel_defi_hub_runtime.wasm  --chain chain_spec.json --tmp
```


![Result](/assets/ScreenShot2024-12-09.png)