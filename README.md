# Dockery

`dockery` is a command-line tool designed to streamline interactions with Docker registry v2. It provides a set of commands to simplify common tasks related to Docker image management, making it easier for developers and administrators to work with self-maintained Docker registry.

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
  - [Command Overview](#command-overview)
  - [Examples](#examples)
- [Configuration](#configuration)
- [Contributing](#contributing)
- [License](#license)

## Installation

### Compile

You can install `Rust` first, then compile it yourself.

## Usage

### Command Overview

Dockery supports the following commands:

- `dockery images`: List images in the registry.
- `dockery rmi`: Delete an image from a registry.

For detailed information on each command, use the `--help` option:

```bash
dockery <command> --help
```

## Examples

### List images:

```bash
dockery images
```

output:
```bash
Registry: 0.0.0.0:5000
+------------+-------------------------------------------------+--------------+-------------+--------+
| REPOSITORY | TAG                                             | IMAGE ID     | CREATED     | SIZE   |
+------------+-------------------------------------------------+--------------+-------------+--------+
| mlx        | torch-2.1.2-nvidia                              | 1c388b1713fc | 7 days ago  | 4.47GB |
+------------+-------------------------------------------------+--------------+-------------+--------+
| mlx        | torch-1.13.1-nvidia                             | b6c565623a3c | 17 days ago | 4.42GB |
+------------+-------------------------------------------------+--------------+-------------+--------+
| mlx        | torch-2.0.1                                     | 0682bf1b7a49 | 1 month ago | 2.84GB |
+------------+-------------------------------------------------+--------------+-------------+--------+
| mlx-jog    | http-yolov8-4c647f4482c11f154ace75140b13a130-v1 | b50e3cd26e54 | 7 days ago  | 4.48GB |
+------------+-------------------------------------------------+--------------+-------------+--------+
| mlx-jogger | http-yolov8-d758a86ac090b7ac280e9cfedcfa75e9-v1 | bc8c9aa2b911 | 17 days ago | 4.43GB |
+------------+-------------------------------------------------+--------------+-------------+--------+
| mlx-jogger | http-yolov8-205e82fe20aa8dd89359e23c7295b2c9-v1 | 692992389117 | 17 days ago | 4.43GB |
+------------+-------------------------------------------------+--------------+-------------+--------+
```

### Delete image:

```bash
dockery rmi abc:torch-2.0.1
```

output:
```bash
abc:torch-2.0.1
```

## Configuration

### Registry URI

By default, `dockery` use `0.0.0.0:5000` to connect to registry, if you have customized the port number, or the registry is remote, make sure to set environment variable `DOCKERY`:

```bash
DOCKERY=<your-host>:<your-port> dockery images
```

ouput:
```bash
Registry: <your-host>:<your-port>
+------------+-------------------------------------------------+--------------+-------------+--------+
| REPOSITORY | TAG                                             | IMAGE ID     | CREATED     | SIZE   |
+------------+-------------------------------------------------+--------------+-------------+--------+
| mlx        | torch-2.1.2-nvidia                              | 1c388b1713fc | 7 days ago  | 4.47GB |
+------------+-------------------------------------------------+--------------+-------------+--------+
| mlx        | torch-1.13.1-nvidia                             | b6c565623a3c | 17 days ago | 4.42GB |
+------------+-------------------------------------------------+--------------+-------------+--------+
| mlx        | torch-2.0.1                                     | 0682bf1b7a49 | 1 month ago | 2.84GB |
+------------+-------------------------------------------------+--------------+-------------+--------+
```

Or more convenient, just set `DOCKERY` with `Shell` command:
```bash
export DOCKERY=<your-host>:<your-port>
```

### Enable delete

`dockery rmi` would fail unless you set environment variable `REGISTRY_STORAGE_DELETE_ENABLED=true` in your registry.

## Contributing

If you would like to contribute to Dockery, please follow the guidelines in [CONTRIBUTING.md](CONTRIBUTING.md).

## License

This project is licensed under the [Apache License 2.0](LICENSE).
