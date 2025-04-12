use fast_log::config::Config;
use fast_log::plugin::file_split::{RollingType, KeepType, DateType, Rolling};
use fast_log::plugin::packer::LogPacker;
use server_config::config::ApplicationConfig;
use tracing_subscriber::fmt::time::LocalTime;
use time::macros::format_description;
use std::fs::{self, File};

use crate::APPLICATION_CONTEXT;

pub fn init_log() {
    let cassie_config = APPLICATION_CONTEXT.get::<ApplicationConfig>();
    let log_dir = cassie_config.log();
    // 创建日志目录
    fs::create_dir_all(&log_dir.log_dir).unwrap();
    fast_log::init(
        Config::new()
            .console()
            .file_split(
                &cassie_config.log.log_dir,
                Rolling::new(RollingType::ByDate(DateType::Day)),
                KeepType::KeepNum(20),
                LogPacker {},
            )
            .level(log::LevelFilter::Info),
    ).unwrap();
}
