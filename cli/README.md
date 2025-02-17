# Iroha CLI

The binary `iroha` crate contains the Iroha peer binary. The binary is used to instantiate a peer and bootstrap an Iroha-based network. The capabilities of the network are determined by the feature flags used to compile the binary.

The `iroha` crate contains the Iroha peer binary, which is used to instantiate a peer and bootstrap an Iroha-based network. The capabilities of the network are determined by the feature flags used to compile said binary.

## Build

**Requirements:** a working [Rust toolchain](https://www.rust-lang.org/learn/get-started) (version 1.62.1), installed and configured.

Optionally, [Docker](https://www.docker.com/) can be used to build images containing any of the provided binaries. Using [Docker buildx](https://docs.docker.com/buildx/working-with-buildx/) is recommended, but not required.

### Build the default Iroha binary

Build the Iroha peer binary as well as every other supporting binary:

```bash
cargo build --release
```

The results of the compilation can be found in `<IROHA REPO ROOT>/target/release/`, where `<IROHA REPO ROOT>` is the path to where you cloned this repository (without the angle brackets).

### Add features

To add optional features, use ``--features``. For example, to add the support for _dev_telemetry_, run:

```bash
cargo build --release --features dev-telemetry
```

A full list of features can be found in the [cargo manifest file](Cargo.toml) for this crate.

### Disable default features

By default, the Iroha binary is compiled with the `telemetry`, and `schema-endpoint` features. If you wish to remove those features, add `--no-default-features` to the command.

```bash
cargo build --release --no-default-features
```

This flag can be combined with the `--features` flag in order to precisely specify the feature set that you wish.

## Configuration

To run the Iroha peer binary, you must [generate the keys](#generating-keys) and provide a [configuration file](#configuration-file).

### Generating Keys

We highly recommend you to generate a new key pair for any non-testing deployment. We also recommend using the `Ed25519` algorithm. For convenience, you can use the provided [`kagami`](../tools/kagami/README.md) tool to generate key pairs. For example,

<!-- TODO, update the links for the release version.  -->

```bash
cargo run --bin kagami -- crypto
```

<details> <summary>Expand to see the output</summary>

```bash
Public key (multihash): "ed0120BDF918243253B1E731FA096194C8928DA37C4D3226F97EEBD18CF5523D758D6C"
Private key (ed25519): "0311152FAD9308482F51CA2832FDFAB18E1C74F36C6ADB198E3EF0213FE42FD8BDF918243253B1E731FA096194C8928DA37C4D3226F97EEBD18CF5523D758D6C"
```

</details>

To see the command-line options for `kagami`, you must first terminate the arguments passed to `cargo`. For example, run the `kagami` binary with JSON formatting:

```bash
cargo run --bin kagami -- crypto --json
```

**NOTE**: The `kagami` binary can be run without `cargo` using the `<IROHA REPO ROOT>/target/release/kagami` binary.
Refer to [generating key pairs with `kagami`](../tools/kagami#crypto) for more details.

### Configuration file

**Note:** this section is under development. You can track it in the [issue](https://github.com/hyperledger/iroha-2-docs/issues/392).

## Deployment

You may deploy Iroha as a [native binary](#native-binary) or by using [Docker](#docker).

### Native binary

1. Prepare a deployment environment.

    If you plan on running the `iroha` peer binary from the directory `deploy`, copy `config.json` and `genesis.json`:

    ```bash
    cp ./target/release/iroha
    cp ./configs/peer/config.json deploy
    cp ./configs/peer/genesis.json deploy
    ```

2. Make necessary edits to `config.json` and `genesis.json`, such as:

    - Generate new key pairs and add their values to `genesis.json`)
    - Adjust the port values for your initial set of trusted peers
    - Change the number of trusted peers to fit your initial network topology

    **NOTE**: the number of peers needed for tolerating _f_ byzantine faults is _3f+1_.

3. Start an Iroha peer.

    You can do this either with `--genesis` parameter to specify `genesis.json` location or without. Pay attention that for multi-peer setup only one peer should be started with `--genesis` parameter.

    ```bash
    cd deploy
    ./iroha --submit-genesis
    ```

### Docker

We provide a sample configuration for Docker in [`docker-compose.yml`](../docker-compose.yml). We highly recommend that you adjust the `config.json` to include a set of new key pairs.

[Generate the keys](#generating-keys) and put them into `services.*.environment` in `docker-compose.yml`. Don't forget to update the public keys of `TRUSTED_PEERS`.

- Build images:

    ```bash
    docker-compose build
    ```

- Run containers:

    ```bash
    docker-compose up
    ```

  To keep containers up and running after closing the terminal, use the `-d` (*detached*) flag:

    ```bash
    docker-compose up -d
    ```

- Stop containers:

    ```bash
    docker-compose stop
    ```

- Remove containers:

    ```bash
    docker-compose down
    ```

