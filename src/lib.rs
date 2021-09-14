use env_logger::{
    fmt::{Color, Style, StyledValue},
    Builder,
};
use log::Level;
use serde::Serialize;
use std::fmt;
use std::fs::rename;
use std::sync::atomic::{AtomicUsize, Ordering};
use tinytemplate::{format_unescaped, TinyTemplate};
use tokio_uring::fs::OpenOptions;

static WRITE_SEEK: AtomicUsize = AtomicUsize::new(0);
static WRITE_LINE: AtomicUsize = AtomicUsize::new(0);
static FILE_COUNT: AtomicUsize = AtomicUsize::new(0);
static DEFAULT_TEMPLATE: &str = "{L} {T} > {M}\n";

pub struct LogConfig {
    pub env: &'static str,
    pub output: &'static str,
    pub file: bool,
    pub format: &'static str,
    pub rotation: usize,
}

impl LogConfig {
    pub fn new() -> LogConfig {
        LogConfig {
            env: "RUST_LOG",
            output: "stdout",
            file: false,
            format: DEFAULT_TEMPLATE,
            rotation: 0,
        }
    }

    pub fn env(&mut self, env: &'static str) {
        self.env = env;
    }

    pub fn output(&mut self, output: &'static str) {
        tokio_uring::start(async {
            match OpenOptions::new()
                .append(true)
                .create_new(true)
                .open(output)
                .await
            {
                Ok(f) => {
                    f.close().await.unwrap();
                    self.file = true;
                    self.output = output;
                }
                Err(e) => {
                    eprintln!("Failed to open log file: {}", e);
                    eprintln!("Moe Logger would only use stdout.");
                    self.file = false;
                    self.output = "stdout";
                }
            }
        });
    }

    pub fn format(&mut self, format: &'static str) {
        let mut tt = TinyTemplate::new();
        tt.add_template("default", DEFAULT_TEMPLATE).unwrap();
        match tt.add_template("custom", format) {
            Ok(_) => {
                self.format = format;
            }
            Err(e) => {
                eprintln!("Failed to parse log format: {}", e);
                eprintln!("Moe Logger would use default format.");
                self.format = DEFAULT_TEMPLATE;
            }
        };
    }

    pub fn rotation(&mut self, rotation: usize) {
        self.rotation = rotation;
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct Context<'a> {
    L: String,
    T: String,
    M: String,
    t: String,
    F: &'a str,
}

pub fn init(log_config: LogConfig) {
    init_builder(log_config);
}

pub fn init_builder(config: LogConfig) {
    let mut builder = Builder::new();
    let env_var = std::env::var(config.env).unwrap_or_else(|_| "info".to_string());

    builder
        .format(move |buf, record| {
            use std::io::Write;
            let target = record.target();
            let max_width = max_target_width(target);

            let mut style = buf.style();
            let level = colored_level(&mut style, record.level());

            let mut style = buf.style();
            let target = style.set_bold(true).value(Padded {
                value: target,
                width: max_width,
            });

            let ret = writeln!(buf, "{} {} > {}", level, target, record.args());

            if config.file {
                tokio_uring::start(async {
                    let context = Context {
                        L: record.level().to_string(),
                        T: record.target().to_string(),
                        M: record.args().to_string(),
                        t: buf.timestamp_millis().to_string(),
                        F: record.file().unwrap_or(""),
                    };
                    let mut tt = TinyTemplate::new();
                    tt.set_default_formatter(&format_unescaped);
                    tt.add_template("0", config.format).unwrap();

                    let lines = WRITE_LINE.load(Ordering::Relaxed) + 1;
                    WRITE_LINE.store(lines, Ordering::Relaxed);

                    let rendered = tt.render("0", &context).unwrap();
                    let buf = rendered.as_bytes().to_vec();
                    let file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(config.output)
                        .await
                        .unwrap();
                    let (res, _) = file
                        .write_at(buf, WRITE_SEEK.load(Ordering::Relaxed) as u64)
                        .await;
                    if let Ok(res) = res {
                        WRITE_SEEK.fetch_add(res, Ordering::SeqCst);
                    }

                    if lines == config.rotation {
                        let file_num = FILE_COUNT.load(Ordering::Relaxed);
                        let file_name = format!("{}.{}", config.output, file_num);
                        match rename(config.output, file_name) {
                            Ok(_) => {
                                FILE_COUNT.fetch_add(1, Ordering::SeqCst);
                                WRITE_LINE.store(0, Ordering::Relaxed);
                            },
                            Err(e) => {
                                eprintln!("Failed to rotate log: {}", e);
                            }
                        }
                    }
                });
            }

            ret
        })
        .parse_filters(&env_var);

    builder.try_init().unwrap()
}

struct Padded<T> {
    value: T,
    width: usize,
}

impl<T: fmt::Display> fmt::Display for Padded<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <width$}", self.value, width = self.width)
    }
}

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);

fn max_target_width(target: &str) -> usize {
    let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
    if max_width < target.len() {
        MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
        target.len()
    } else {
        max_width
    }
}

fn colored_level<'a>(style: &'a mut Style, level: Level) -> StyledValue<'a, &'static str> {
    match level {
        Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
        Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
        Level::Info => style.set_color(Color::Green).value("INFO "),
        Level::Warn => style.set_color(Color::Yellow).value("WARN "),
        Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}
