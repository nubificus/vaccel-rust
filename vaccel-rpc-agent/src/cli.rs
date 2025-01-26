// SPDX-License-Identifier: Apache-2.0

use crate::Error;
use clap::Parser;
use std::str::FromStr;

#[derive(Debug, Default, Parser)]
#[command(name = "vAccel RPC Agent")]
#[command(about = "A vAccel RPC agent that can respond to acceleration requests")]
pub struct Cli {
    #[arg(short = 'a')]
    #[arg(long = "server-address")]
    #[arg(help = "The server address in the format '<socket-type>://<host>:<port>'")]
    #[arg(default_value = "tcp://127.0.0.1:65500")]
    pub server_address: String,

    #[arg(long = "vaccel-config")]
    #[arg(help = "Configuration options passed to vAccel in the format 'opt1=value1,opt2=value2'")]
    pub vaccel_config: Option<VaccelConfig>,
}

#[derive(Debug, Clone)]
pub struct VaccelConfig {
    pub plugins: Option<String>,
    pub log_level: Option<u8>,
    pub log_file: Option<String>,
    pub profiling_enabled: Option<bool>,
    pub version_ignore: Option<bool>,
}

impl FromStr for VaccelConfig {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut plugins = None;
        let mut log_level = None;
        let mut log_file = None;
        let mut profiling_enabled = None;
        let mut version_ignore = None;

        for part in s.split(',') {
            let mut kv = part.splitn(2, '=');
            let key = kv
                .next()
                .ok_or(Error::CliError("Missing key".to_string()))?;
            let value = kv
                .next()
                .ok_or(Error::CliError("Missing value".to_string()))?;
            match key {
                "plugins" => plugins = Some(value.to_string()),
                "log_level" => {
                    log_level = Some(value.parse::<u8>().map_err(|_| {
                        Error::CliError(format!("Invalid integer for log_level: {}", value))
                    })?)
                }
                "log_file" => log_file = Some(value.to_string()),
                "profiling_enabled" => {
                    profiling_enabled = Some(value.parse::<bool>().map_err(|_| {
                        Error::CliError(format!("Invalid boolean for profiling_enabled: {}", value))
                    })?)
                }
                "version_ignore" => {
                    version_ignore = Some(value.parse::<bool>().map_err(|_| {
                        Error::CliError(format!("Invalid boolean for version_ignore: {}", value))
                    })?)
                }
                _ => return Err(Error::CliError(format!("Unknown key: {}", key))),
            }
        }

        Ok(VaccelConfig {
            plugins,
            log_level,
            log_file,
            profiling_enabled,
            version_ignore,
        })
    }
}

impl TryFrom<VaccelConfig> for vaccel::Config {
    type Error = Error;

    fn try_from(config: VaccelConfig) -> Result<Self, Self::Error> {
        let plugins = config.plugins.as_deref();
        let log_level = config.log_level.unwrap_or(0);
        let log_file = config.log_file.as_deref();
        let profiling_enabled = config.profiling_enabled.unwrap_or(false);
        let version_ignore = config.version_ignore.unwrap_or(false);

        Ok(vaccel::Config::new(
            plugins,
            log_level,
            log_file,
            profiling_enabled,
            version_ignore,
        )?)
    }
}
