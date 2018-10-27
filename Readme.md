# sfn-rust

Yet another implementation. Uses the standard sfn protocol, compatible with other implementations.

## Protocol modes

### SM

Protocol stream consists of chunks, one after another, each started by chunk opcode. Each client **must** send `0x02` (DONE) when it doesn't intend to send chunks anymore.

| [Chunk 1] | ... | [Chunk N] | `0x02` (DONE) |
| --------- | --- | --------- | ------------- |

#### 0x01 (`FILE` chunk)

| `0x01` | Filename (UTF-8),<br />end with `0x0A` (LF) | Size (64 bits, little endian) | File contents |
| ------ | ------------------------------------------- | ----------------------------- | ------------- |

#### 0x03 (`FILE_WITH_MD5` chunk)

| `0x03` | Filename (UTF-8),<br />end with `0x0A` (LF) | Size (64 bits, little endian) | MD5 sum (ASCII hex string),<br />end with `0x0A` (LF) | File contents |
| ------ | ------------------------------------------- | ----------------------------- | ----------------------------------------------------- | ------------- |
