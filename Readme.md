# sfn-rust

This is yet another implementation of sfn, written in Rust. It uses the standard sfn L3 protocol, making it compatible with other implementations.

Stability: **alpha**

## Build

To build this project, you will need to have Rust installed on your machine. If you don't have it installed, you can download it from the [official Rust website](https://www.rust-lang.org/tools/install).

Once Rust is installed, you can build the project by running the following command in the terminal:

```bash
cargo build --release
```

This will compile the source code and generate an executable file.

## Run

Just run the binary (after build it will be placed to `target/release/sfn`).

### CLI Options

```
USAGE:
    sfn [OPTIONS] [FILE]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --connect <IP:PORT>    Connect to other host
    -l, --listen <IP:PORT>     Wait for connections

ARGS:
    <FILE>...    Files to send
```

Note: You must use one of `--connect` or `--listen`, but not both. In any case, sfn will also receive any queued files from the other side in parallel to sending.

### Examples

This will start the program in listen mode on all IPs and port 3214, and it will send the files file1.txt and file2.txt when a connection is established. It will also receive any queued files from the other side.

```
sfn --listen 0.0.0.0:3214  file1.txt file2.txt
```

This will start the program in connect mode to IP 192.168.1.2 and port 3214, and it will send the files file1.txt and file2.txt when a connection is established. It will also receive any queued files from the other side.

```
sfn --connect 192.168.1.2:3214  file1.txt file2.txt
```

## Testing

To run automated checks, type:

```
script/test
```

## Protocol specs
You can find the protocol specifications here: https://github.com/sfn-software/protocol


## License
This project is licensed under the MIT License.
