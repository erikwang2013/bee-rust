use tracing::Level;
use tracing_subscriber::{fmt, prelude::*, util::SubscriberInitExt, EnvFilter};

pub enum Output {
    Stdout,
    File(String),
    MultiFile(String),
}

pub struct Logger {
    level: Level,
    output: Output,
    async_mode: bool,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            level: Level::INFO,
            output: Output::Stdout,
            async_mode: false,
        }
    }

    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    pub fn output(mut self, output: Output) -> Self {
        self.output = output;
        self
    }

    pub fn async_(mut self) -> Self {
        self.async_mode = true;
        self
    }

    pub fn init(self) -> Result<(), Box<dyn std::error::Error>> {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(self.level_str()));

        match &self.output {
            Output::Stdout => {
                tracing_subscriber::registry()
                    .with(filter)
                    .with(fmt::layer().with_target(true))
                    .try_init()?;
            }
            Output::File(path) => {
                let file =
                    std::fs::OpenOptions::new().create(true).append(true).open(path)?;
                let (writer, _guard) = tracing_appender::non_blocking(file);
                tracing_subscriber::registry()
                    .with(filter)
                    .with(fmt::layer().with_writer(writer))
                    .try_init()?;
            }
            Output::MultiFile(dir) => {
                let appender = tracing_appender::rolling::daily(dir, "app.log");
                let (writer, _guard) = tracing_appender::non_blocking(appender);
                tracing_subscriber::registry()
                    .with(filter)
                    .with(fmt::layer().with_writer(writer).json())
                    .try_init()?;
            }
        }
        Ok(())
    }

    fn level_str(&self) -> String {
        match self.level {
            Level::TRACE => "trace",
            Level::DEBUG => "debug",
            Level::INFO => "info",
            Level::WARN => "warn",
            Level::ERROR => "error",
        }
        .into()
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_stdout() {
        // Ignore result — subscriber may already be set from another test
        let _ = Logger::new()
            .level(Level::DEBUG)
            .output(Output::Stdout)
            .init();
    }

    #[test]
    fn test_logger_default() {
        let _ = Logger::default().init();
    }
}
