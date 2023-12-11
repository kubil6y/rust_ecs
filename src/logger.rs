use chrono::Utc;
use colored::Colorize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
}

impl LogEntry {
    fn new(level: LogLevel, message: &str) -> Self {
        Self {
            level,
            message: message.to_string(),
        }
    }
}

pub struct Logger {
    level: LogLevel,
    log_entries: Vec<LogEntry>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            level: LogLevel::Info,
            log_entries: vec![],
        }
    }

    pub fn get_log_entires(&self) -> &Vec<LogEntry> {
        &self.log_entries
    }

    pub fn set_log_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    pub fn log(&mut self, message: &str) {
        self.log_entry(LogLevel::Info, message);
    }

    pub fn warning(&mut self, message: &str) {
        self.log_entry(LogLevel::Warning, message);
    }

    pub fn error(&mut self, message: &str) {
        self.log_entry(LogLevel::Error, message);
    }

    fn log_entry(&mut self, level: LogLevel, message: &str) {
        if self.level > level {
            return;
        }

        let prefix = match level {
            LogLevel::Info => "INFO:".blue(),
            LogLevel::Warning => "WARNING:".yellow(),
            LogLevel::Error => "ERROR:".red(),
        };

        let timestamp = Utc::now().to_string();
        let message = format!("{} [{}]: {}", prefix, timestamp, message);
        let entry = LogEntry::new(level, &message);
        self.log_entries.push(entry);
        println!("{}", message);
    }
}
