#!/usr/bin/env python3
"""
Setup or tear down a bare metal
environment running a debug build of Iroha.
"""

import argparse
import datetime
import ipaddress
import json
import logging
import os
import pathlib
import shutil
import subprocess
import sys
import time
import urllib.error
import urllib.request
import uuid


class Network:
    """
    A network of bootstrapped peers to run on bare metal.
    """
    def __init__(self, args: argparse.Namespace):
        logging.info("Setting up test environment...")

        self.out_dir = args.out_dir
        peers_dir = args.out_dir.joinpath("peers")
        os.makedirs(peers_dir, exist_ok=True)
        self.shared_env = dict(os.environ)

        self.peers = [_Peer(args, i) for i in range(args.n_peers)]

        try:
            shutil.copy2(f"{args.root_dir}/configs/peer/config.json", peers_dir)
            # genesis should be supplied only for the first peer
            peer_0_dir = self.peers[0].peer_dir
            shutil.copy2(f"{args.root_dir}/configs/peer/genesis.json", peer_0_dir)
            # assuming that `genesis.json` contains path to the executor as `./executor.wasm`
            shutil.copy2(f"{args.root_dir}/configs/peer/executor.wasm", peer_0_dir)
        except FileNotFoundError:
            logging.error(f"Some of the config files are missing. \
                          Please provide them in the `{args.root_dir}/configs/peer` directory")
            sys.exit(1)
        copy_or_prompt_build_bin("iroha", args.root_dir, peers_dir)

        self.shared_env["IROHA_CHAIN_ID"] = "00000000-0000-0000-0000-000000000000"
        self.shared_env["IROHA_CONFIG"] = str(peers_dir.joinpath("config.json"))
        self.shared_env["IROHA_GENESIS_PUBLIC_KEY"] = self.peers[0].public_key

        logging.info("Generating trusted peers...")
        self.trusted_peers = []
        for peer in self.peers:
            peer_entry = {"address": f"{peer.host_ip}:{peer.p2p_port}", "public_key": peer.public_key}
            self.trusted_peers.append(json.dumps(peer_entry))
        self.shared_env["SUMERAGI_TRUSTED_PEERS"] = f"[{','.join(self.trusted_peers)}]"

    def wait_for_genesis(self, n_tries: int):
        for i in range(n_tries):
            logging.info(f"Waiting for genesis block to be created... Attempt {i+1}/{n_tries}")
            try:
                with urllib.request.urlopen(f"http://{self.peers[0].host_ip}:{self.peers[0].api_port}/status/blocks") as response:
                    block_count = int(response.read())
                    if block_count >= 1:
                        logging.info(f"Genesis block created. Block count: {block_count}")
                        return
                    else:
                        logging.info("Sleeping 1 second...")
                        time.sleep(1)
            except urllib.error.URLError as e:
                logging.info(f"Error connecting to genesis peer: {e}. Sleeping 1 second...")
                time.sleep(1)
        logging.critical(f"Genesis block wasn't created within {n_tries} seconds. Aborting...")
        cleanup(self.out_dir)
        sys.exit(2)

    def run(self):
        for i, peer in enumerate(self.peers):
            peer.run(shared_env=self.shared_env, submit_genesis=(i == 0))
        self.wait_for_genesis(20)

class _Peer:
    """
    A single bootstrapped peer. Could be a genesis node or a regular peer.
    Should not be run directly, but rather as a part of a Network.
    """
    def __init__(self, args: argparse.Namespace, nth: int):
        self.nth = nth
        self.name = f"iroha{nth}"
        self.p2p_port = 1337 + nth
        self.api_port = 8080 + nth
        self.tokio_console_port = 5555 + nth
        self.out_dir = args.out_dir
        self.root_dir = args.root_dir
        self.peer_dir = self.out_dir.joinpath(f"peers/{self.name}")
        self.host_ip = args.host_ip

        logging.info(f"Peer {self.name} generating key pair...")

        command = [f"{self.out_dir}/kagami", "crypto", "-j"]
        if args.peer_name_as_seed:
            command.extend(["-s", self.name])
        kagami = subprocess.run(command, capture_output=True)
        if kagami.returncode:
            logging.error("Kagami failed to generate a key pair.")
            sys.exit(3)
        str_keypair = kagami.stdout
        json_keypair = json.loads(str_keypair)
        # public key is a string, private key is a json object
        self.public_key = json_keypair['public_key']
        self.private_key = json.dumps(json_keypair['private_key'])

        os.makedirs(self.peer_dir, exist_ok=True)
        os.makedirs(self.peer_dir.joinpath("storage"), exist_ok=True)

        logging.info(f"Peer {self.name} initialized")

    def run(self, shared_env: dict(), submit_genesis: bool = False):
        logging.info(f"Running peer {self.name}...")

        peer_env = dict(shared_env)
        peer_env["KURA_BLOCK_STORE_PATH"] = str(self.peer_dir.joinpath("storage"))
        peer_env["SNAPSHOT_DIR_PATH"] = str(self.peer_dir.joinpath("storage"))
        peer_env["LOG_LEVEL"] = "INFO"
        peer_env["LOG_FORMAT"] = '"pretty"'
        peer_env["LOG_TOKIO_CONSOLE_ADDR"] = f"{self.host_ip}:{self.tokio_console_port}"
        peer_env["IROHA_PUBLIC_KEY"] = self.public_key
        peer_env["IROHA_PRIVATE_KEY"] = self.private_key
        peer_env["SUMERAGI_DEBUG_FORCE_SOFT_FORK"] = "false"
        peer_env["TORII_P2P_ADDR"] = f"{self.host_ip}:{self.p2p_port}"
        peer_env["TORII_API_URL"] = f"{self.host_ip}:{self.api_port}"

        if submit_genesis:
            peer_env["IROHA_GENESIS_PRIVATE_KEY"] = self.private_key
            # Assuming it was copied to the peer's directory
            peer_env["IROHA_GENESIS_FILE"] = str(self.peer_dir.joinpath("genesis.json"))

        # FD never gets closed
        stdout_file = open(self.peer_dir.joinpath(".stdout"), "w")
        stderr_file = open(self.peer_dir.joinpath(".stderr"), "w")
        # These processes are created detached from the parent process already
        subprocess.Popen([self.name] + (["--submit-genesis"] if submit_genesis else []),
                    executable=f"{self.out_dir}/peers/iroha", env=peer_env, stdout=stdout_file, stderr=stderr_file)

def pos_int(arg):
    if int(arg) > 0:
        return int(arg)
    else:
        raise argparse.ArgumentTypeError(f"Argument {arg} must be a positive integer")

def copy_or_prompt_build_bin(bin_name: str, root_dir: pathlib.Path, target_dir: pathlib.Path):
    try:
        shutil.copy2(f"{root_dir}/target/debug/{bin_name}", target_dir)
    except FileNotFoundError:
        logging.error(f"The binary `{bin_name}` wasn't found in `{root_dir}` directory")
        while True:
            prompt = input(f"Build it by running `cargo build --bin {bin_name}`? (Y/n)\n")
            if prompt.lower() in ["y", "yes", ""]:
                subprocess.run(
                    ["cargo", "build", "--bin", bin_name],
                    cwd=root_dir
                )
                shutil.copy2(f"{root_dir}/target/debug/{bin_name}", target_dir)
                break
            elif prompt.lower() in ["n", "no"]:
                logging.critical("Can't launch the network without the binary. Aborting...")
                sys.exit(4)
            else:
                logging.error("Please answer with either `y[es]` or `n[o]`")

def main(args: argparse.Namespace):
    # Bold ASCII escape sequence
    logging.basicConfig(level=logging.INFO if args.verbose else logging.WARNING,
        style="{",
        format="{asctime} {levelname} \033[1m{funcName}:{lineno}\033[0m: {message}",)
    # ISO 8601 timestamps without timezone
    logging.Formatter.formatTime = (lambda self, record, datefmt=None:
        datetime.datetime.fromtimestamp(record.created)
        .isoformat(sep="T",timespec="microseconds"))
    # Colored log levels
    logging.addLevelName(logging.INFO, f"\033[32m{logging.getLevelName(logging.INFO)}\033[0m")
    logging.addLevelName(logging.ERROR, f"\033[35m{logging.getLevelName(logging.ERROR)}\033[0m")
    logging.addLevelName(logging.CRITICAL, f"\033[31m{logging.getLevelName(logging.CRITICAL)}\033[0m")
    if args.command == "setup":
        setup(args)
    elif args.command == "cleanup":
        cleanup(args.out_dir)

def setup(args: argparse.Namespace):
    logging.info(f"Starting iroha network with {args.n_peers} peers...")
    os.makedirs(args.out_dir, exist_ok=True)
    copy_or_prompt_build_bin("iroha_client_cli", args.root_dir, args.out_dir)
    with open(os.path.join(args.out_dir, "metadata.json"), "w") as f:
        f.write('{"comment":{"String": "Hello Meta!"}}')
    shutil.copy2(f"{args.root_dir}/configs/client/config.json", args.out_dir)
    copy_or_prompt_build_bin("kagami", args.root_dir, args.out_dir)

    Network(args).run()

def cleanup(out_dir: pathlib.Path):
    logging.info("Killing peer processes...")
    subprocess.run(["pkill", "-9", "iroha"])
    logging.info(f"Cleaning up test directory `{out_dir}`...")
    shutil.rmtree(out_dir)



if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)

    parser.add_argument("command", choices=["setup", "cleanup"],
                        help="Available actions. The `setup` command will create and run \
                        a new test environment with specified number of peers in a \
                        directory of choice. This is meant to be run from `iroha` root dir. \
                        The `cleanup` command will kill all peer processes \
                        that were started by the `setup` command and remove the test directory")
    parser.add_argument("n_peers", nargs="?", default=4, type=pos_int,
                        help="Number of peers to bootstrap. \
                        Defaults to 4. If setup was run with a custom number of peers, \
                        the same number doesn't need to be provided to cleanup as \
                        it kills all processes named `iroha`, so proper caution is advised")

    parser.add_argument("--out-dir", "-o", default="./test", type=pathlib.Path,
                        help="Directory to store config and log files. \
                        Defaults to `./test`. If setup was run with a custom directory, \
                        the same directory must be provided to cleanup, otherwise only \
                        peer processes will be destroyed")
    parser.add_argument("--host-ip", "-i", default="127.0.0.1", type=ipaddress.IPv4Address,
                        help="IP address of the host machine. Used in trusted peer \
                        generation. Defaults to localhost. Note that the port/s shouldn't \
                        be provided as for each peer's endpoints they're assigned automatically")
    parser.add_argument("--root-dir", "-r", default=".", type=pathlib.Path,
                        help="Directory containing Iroha project root. \
                        Defaults to `.`, i.e. the directory script is being run from. \
                        This is used to locate the `iroha` binary and config files")
    parser.add_argument("--peer-name-as-seed", action="store_true",
                        help="Use peer name as seed for key generation. \
                        This option could be useful to preserve the same peer keys between script invocations")

    parser.add_argument("--verbose", "-v", action="store_true",
                        help="Enable verbose output")

    args = parser.parse_args()
    main(args)
