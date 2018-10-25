# sfn-rust

Yet another implementation. Uses the standard sfn protocol, compatible with other implementations.

## Protocol modes

### SM

| Comment | TCP data
| --- | ---
| **[First file]** | `0x01` (FILE)
|| Filename (UTF-8), terminate with LF
|| File size (64 bits, little endian)
|| File contents (N bytes)
| **[More files]** | ...
| **End of transmission** | `0x02` (DONE)

### SM-MD5

| Comment | TCP data
| --- | ---
| **[First file]** | `0x03` (FILE_WITH_MD5)
|| Filename (UTF-8), terminate with LF
|| File size (64 bits, little endian)
|| MD5 sum (ASCII string), terminate with LF
|| File contents (N byte
| **[More files]** | ...
| **End of transmission** | `0x02` (DONE)
