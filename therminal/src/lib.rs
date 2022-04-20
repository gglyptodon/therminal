use clap::{Arg, Command};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, Read};

pub type TherminalResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    refresh_rate: usize, //seconds
}

pub fn run(config: Config) -> TherminalResult<()> {
    println!("{:#?}", config);
    println!("{}", read_temp_data()?);
    Ok(())
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
    let refresh_rate: usize = matches
        .value_of("refresh_rate")
        .unwrap_or_default()
        .parse::<usize>()?;
    Ok(Config { refresh_rate })
}


fn read_temp_data() -> TherminalResult<f32>{
    let file_contents:String = read_file_to_string("/sys/class/thermal/thermal_zone0/temp")?.split_whitespace().collect();
    let raw_value = file_contents.parse::<usize>()?;
    let result = raw_value as f32/1000.0;
    Ok(result)
}

fn read_file_to_string(path: &str)-> TherminalResult<String>{
    let mut result_buff = String::new();
    let mut reader = open(path)?;
    reader.read_to_string(&mut result_buff)?;
    Ok(result_buff)
}

pub fn open(filename: &str) -> TherminalResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(std::io::BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(std::io::BufReader::new(File::open(filename)?))),
    }
}
