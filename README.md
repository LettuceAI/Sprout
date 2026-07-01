# Sprout

Sprout is a small HTTP service that reports a machine's GPUs, VRAM, memory, and CPU. LettuceAI queries it to work out which models a remote or headless machine can run, and how to spread them across multiple GPUs.

GPU detection is per vendor: NVIDIA through NVML (reported as `CUDA`), AMD and Intel through Vulkan, and Apple through Metal on macOS. None of these need a vendor SDK or a llama.cpp build; they rely only on the driver libraries already present on the machine. Sprout is a single binary of a few megabytes.

## Installation

Download a prebuilt binary for your platform from the [releases page](https://github.com/LettuceAI/Sprout/releases), or build from source with a [Rust toolchain](https://rustup.rs):

```sh
cargo build --release
```

The binary is written to `target/release/sprout`.

## Usage

Start the server:

```sh
sprout
```

The first run creates a config file, generates a 32-character API key, and prints it to the terminal. Requests to `/specs` carry that key as a bearer token:

```sh
curl -H "Authorization: Bearer <key>" http://127.0.0.1:8477/specs
```

Sprout serves three routes. `GET /ping` returns its name and version for discovery and `GET /health` is a liveness check; both are public. `GET /specs` returns the hardware snapshot and requires the key:

```json
{
  "schemaVersion": 1,
  "hostname": "megalith",
  "os": "linux",
  "arch": "x86_64",
  "availableMemoryBytes": 6819835904,
  "totalMemoryBytes": 16560168960,
  "cpuName": "Intel(R) Core(TM) i7-10700K CPU @ 3.80GHz",
  "cpuCores": 16,
  "unifiedMemory": false,
  "gpus": [
    {
      "index": 0,
      "name": "Vulkan0",
      "description": "NVIDIA GeForce RTX 4060",
      "vendor": "nvidia",
      "backend": "Vulkan",
      "memoryTotal": 8585740288,
      "memoryFree": 6839926784,
      "deviceType": "Gpu"
    }
  ]
}
```

Every memory value is in bytes. Each vendor is probed independently, so a GPU whose driver is missing is simply omitted; a machine with no detectable GPU reports an empty `gpus` list and the rest of the report still works.

## Running as a service

On Linux, `scripts/install.sh` builds the release binary, installs it to `~/.local/bin`, and registers a systemd user service:

```sh
./scripts/install.sh
```

It prints the API key and config path when it finishes. Manage the service with the usual commands:

```sh
systemctl --user status sprout
journalctl --user -u sprout -f
systemctl --user restart sprout
```

`scripts/uninstall.sh` removes the binary and service. To keep the service running while you are logged out, enable lingering once:

```sh
sudo loginctl enable-linger "$USER"
```

On other platforms, run the binary under any process supervisor, or detach it directly:

```sh
nohup sprout >/tmp/sprout.log 2>&1 &   # Linux, macOS
start "" /b sprout.exe                 # Windows
```

## Configuration

Sprout reads `config.toml` from the platform config directory, which you can override with `--config <path>`:

- Linux: `~/.config/sprout/config.toml`
- macOS: `~/Library/Application Support/sprout/config.toml`
- Windows: `%APPDATA%\sprout\config.toml`

```toml
host = "127.0.0.1"
port = 8477
api_key = "..."
require_auth = true
```

Set `host` to `0.0.0.0` to accept connections from other machines. Clearing `api_key` regenerates it on the next start, and setting `require_auth` to `false` serves `/specs` without a token. The file is created with `600` permissions because it holds the key; restart Sprout after editing it.

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
