# Moe Logger

(＞ω＜) Another logger based on [pretty-env-logger](https://github.com/seanmonstar/pretty-env-logger) and [env_logger](https://github.com/env-logger-rs/env_logger/). Allow writing log to file with features like formatting, file rotation.

## Usage

Append following lines to `Cargo.toml`:

```rust
log = "0.4"
moe_logger = "0.1"
```

There's an example:

```rust
use log::info;
use moe_logger::LogConfig;

fn main() {
    let mut log_config = LogConfig::new();
    log_config.env("RUST_LOG"); // Which environment variable for log level
    log_config.output("run.log"); // Log output file, default is stdout
    log_config.format("{t} {L} {T} > {M}\n"); // Log format for file
    log_config.rotation(10000); // Rotate file after how many lines
    moe_logger::init(log_config);

    info!("Di di ba ba wu~");
    debug!("Debug...");
    warn!("WARNING!");
    error!("Oops >_<");
}
```

## Features

(^ω^) Here is some notice about features provided.

### Output

If you specify a path to store log, Moe Logger would write formatted log to that path and unformatted log to stdout in the meanwhile.

(;>△<) If log file exists, Moe Logger will only use stdout! So move old logs to another place before running.

### Format

We are using [TinyTemplate](https://github.com/bheisler/TinyTemplate) to format content wrote to file. If you are interested in more fancy logs, you may should check its document. Moe Logger provided variables listed below:

- t - [RFC3339](https://www.ietf.org/rfc/rfc3339.txt) Date & Time
- L - Log Level
- T - Log Target
- M - Log Message
- F - File Name

Default format: `{L} {T} > {M}\n`

(;>△<) DO NOT FORGET `\n`

### Rotation

You can specify after how many line written, Moe Logger would rename it like `output.log.x`. Default 0 for disabled.

## Performance

（｡・`ω´･）ノ Writing log to disk would worse the efficiency of your code. But we are always trying to optimize this problem. If you have any ideas, pull requests and issues are welcomed.

The table below shows the performance difference when you enable different features. (By running an actix-web back-end on my PC)

| Feature                      | Requests per Second |
| ---------------------------- | ------------------- |
| No Log                       | ~16000              |
| Only to Stdout               | ~15000              |
| Stdout & File                | ~5600               |
| Stdout & File(with Rotation) | ~5200               |

## License

Moe Logger is distributed under the terms of both Apache-2.0 and MIT license.

