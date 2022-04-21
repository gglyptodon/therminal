use clap::{Arg, Command};
use regex::Regex;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, Read};
use std::thread::sleep;
use std::time;
use std::time::SystemTime;
use chrono::Utc;
use walkdir::DirEntry;
use walkdir::WalkDir;

pub type TherminalResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    refresh_rate: u64, //seconds
}
#[derive(Debug)]
pub struct ThermalInfo {
    temp: f32,
    info: String,
    kind: String,
}

impl Display for ThermalInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}C\t{}\t{}", self.temp, self.info, self.kind)
    }
}

pub fn run(config: Config) -> TherminalResult<()> {
    //println!("{:#?}", config);

    loop {
        println!("\t# {} #", Utc::now().to_rfc2822());

        //println!("{:#?}", read_temp_data()?);
        for t in read_temp_data()? {
            println!("{}", t);
        }
        sleep(time::Duration::from_secs(config.refresh_rate));
    }

    // Ok(())
}

pub fn parse_args() -> TherminalResult<Config> {
    let matches = Command::new("therminal")
        .arg(
            Arg::new("refresh_rate")
                .short('r')
                .long("refresh")
                .value_name("SEC")
                .default_value("30")
                .help("read sensor values again after SEC seconds"),
        )
        .get_matches();
    let refresh_rate = matches
        .value_of("refresh_rate")
        .unwrap_or_default()
        .parse::<u64>()?;
    Ok(Config { refresh_rate })
}

fn read_temp_data() -> TherminalResult<Vec<ThermalInfo>> {
    let mut results: Vec<ThermalInfo> = Vec::new();
    let sensor_files = get_available_temp_sensors()?;
    for sf in sensor_files {
        if let Ok(res) = read_file_to_string(&sf) {
            let file_contents: String = res.split_whitespace().collect();
            let raw_value = file_contents.parse::<usize>()?;
            let value_in_celsius = raw_value as f32 / 1000.0;
            results.push(ThermalInfo {
                temp: value_in_celsius,
                info: sf,
                kind: "TODO".to_string(),
            })
        }
    }

    Ok(results)
}

fn read_file_to_string(path: &str) -> TherminalResult<String> {
    let mut result_buff = String::new();
    let mut reader = open(path).map_err(|e| {
        eprintln!("path: {}, {}", path, e);
        e
    })?;
    reader.read_to_string(&mut result_buff)?;

    Ok(result_buff)
}

pub fn open(filename: &str) -> TherminalResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(std::io::BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(std::io::BufReader::new(File::open(filename)?))),
    }
}

pub fn get_available_temp_sensors() -> TherminalResult<Vec<String>> {
    let interesting = [Regex::new(r"^temp$")?, Regex::new(r"temp.*_input")?];
    //let only_files = |entry: &DirEntry| entry.file_type().is_file();

    let name_filter = |entry: &DirEntry| {
        interesting.is_empty()
            || interesting
                .iter()
                .any(|re| re.is_match(&entry.file_name().to_string_lossy()))
    };

    let to_try = ["/sys/class/thermal/", "/sys/class/hwmon/"];
    let mut results: Vec<String> = Vec::new();

    for path in to_try {
        let entries = walkdir::WalkDir::new(path)
            .max_depth(2)
            .follow_links(true)
            .into_iter()
            .filter_map(|e|// match e {
            e.ok())
            .filter(name_filter)
            .map(|direntry| direntry.path().display().to_string())
            .collect::<Vec<_>>();
        for e in entries {
            results.push(e)
        }
    }

    Ok(results)
}
