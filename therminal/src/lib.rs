use chrono::Utc;
use clap::{Arg, Command};

use cursive::align::HAlign;
use cursive::event::{Event, Key::Esc};
use cursive::view::{Nameable, Resizable};
use cursive::views::Dialog;
use cursive::{Cursive, CursiveExt};
use cursive_table_view::{TableView, TableViewItem};

use regex::Regex;
use walkdir::DirEntry;

use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, Read};
use std::thread::sleep;
use std::time;
use std::time::SystemTime;

pub type TherminalResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
pub struct Config {
    refresh_rate: u64, //seconds
    threshold: Option<f32>,
    sensor_id: Option<String>,
    with_tui: bool,
}
#[derive(Debug, Clone)]
pub struct ThermalInfo {
    temp: f32,
    sensor: String,
    kind: String,
    name: String,
}

impl Display for ThermalInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}C\t{}\t{}\t{}",
            self.temp, self.sensor, self.kind, self.name
        )
    }
}

pub fn run(config: Config) -> TherminalResult<()> {
    //println!("{:#?}", config);
    let show_with_threshold = |t: &ThermalInfo| {
        if let Some(threshold) = config.threshold {
            match (
                threshold * 1.5 < t.temp,
                threshold * 1.2 < t.temp,
                threshold < t.temp,
            ) {
                (true, _, _) => "!!!",
                (_, true, _) => "!!",
                (_, _, true) => "!",
                _ => "",
            }
        } else {
            ""
        }
    };
    if config.with_tui {
        tui(&config)?;
        Ok(())
    } else {
        loop {
            println!("\t# {} #", Utc::now().to_rfc2822());
            for t in read_temp_data()? {
                match config.clone().sensor_id {
                    Some(sensor) => {
                        if t.name.ends_with(&*sensor) {
                            println!("{}\t{:>4}\t{}", t.name, t.temp, show_with_threshold(&t))
                        }
                    }
                    _ => {
                        println!("{}\t{:>4}\t{}", t.name, t.temp, show_with_threshold(&t))
                    }
                }
            }
            sleep(time::Duration::from_secs(config.refresh_rate));
        }
    }
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
        .arg(
            Arg::new("threshold")
                .short('t')
                .long("threshold")
                .value_name("C")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("sensor_id")
                .short('s')
                .long("sensor-id")
                .value_name("SENSOR")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("tui")
                .long("tui")
                .takes_value(false)
                .required(false)
                .help("Run with UI"),
        )
        .get_matches();
    let refresh_rate = matches
        .value_of("refresh_rate")
        .unwrap_or_default()
        .parse::<u64>()?;
    let threshold = match matches.value_of("threshold") {
        Some(t) => Some(t.parse::<f32>()?),
        None => None,
    };
    let with_tui = matches.is_present("tui");
    let sensor_id = matches.value_of("sensor_id").map(String::from);
    Ok(Config {
        refresh_rate,
        threshold,
        sensor_id,
        with_tui,
    })
}

fn read_temp_data_after(
    last_updated: &SystemTime,
    after_seconds: core::time::Duration,
) -> TherminalResult<Option<Vec<ThermalInfo>>> {
    let mut results = None;
    if SystemTime::now().duration_since(*last_updated)?.as_secs() > after_seconds.as_secs() {
        //todo
        let mut readings: Vec<ThermalInfo> = Vec::new();
        let sensor_files = get_available_temp_sensors()?;
        for sf in sensor_files {
            if let Ok(res) = read_file_to_string(&sf) {
                let label = get_label_for_sensor(&sf)?;
                let file_contents: String = res.split_whitespace().collect();
                let raw_value = file_contents.parse::<usize>()?;
                let value_in_celsius = raw_value as f32 / 1000.0;
                readings.push(ThermalInfo {
                    temp: value_in_celsius,
                    sensor: sf.clone(),
                    kind: "TODO".to_string(),
                    name: label,
                })
            }
        }
        results = Some(readings);
    }

    Ok(results)
}
fn read_temp_data() -> TherminalResult<Vec<ThermalInfo>> {
    let mut results: Vec<ThermalInfo> = Vec::new();
    let sensor_files = get_available_temp_sensors()?;
    for sf in sensor_files {
        if let Ok(res) = read_file_to_string(&sf) {
            let label = get_label_for_sensor(&sf)?;
            //println!("label {label}");
            let file_contents: String = res.split_whitespace().collect();
            let raw_value = file_contents.parse::<usize>()?;
            let value_in_celsius = raw_value as f32 / 1000.0;
            results.push(ThermalInfo {
                temp: value_in_celsius,
                sensor: sf.clone(),
                kind: "TODO".to_string(),
                name: label,//sf.clone(),
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
pub fn get_label_for_sensor(path: &str)-> TherminalResult<String> {
    let mut result = String::new();
    if path.ends_with("_input") { //hwmon
        if let Ok(mut bufread ) = open(&*path.replace("input", "label")) {
            bufread.read_to_string(&mut result)?;
        }
    }
    else if path.ends_with("temp"){
           if let Ok(mut bufread ) = open(&*path.replace("temp", "type")) {
            bufread.read_to_string(&mut result)?;
        }

    }
    if !result.is_empty(){
        result = result.split_whitespace().collect::<String>();
    }
   Ok(result)
}
pub fn get_available_temp_sensors() -> TherminalResult<Vec<String>> {
    let interesting = [Regex::new(r"^temp$")?, Regex::new(r"^temp.*_input$")?];
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
            .filter_map(|e| e.ok())
            .filter(name_filter)
            .map(|direntry| direntry.path().display().to_string())
            .collect::<Vec<_>>();
        for e in entries {
            results.push(e)
        }
    }

    Ok(results)
}

// --- TUI --- \\
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ThermalInfoColumn {
    Sensor,
    Temp,
    Name,
}

impl TableViewItem<ThermalInfoColumn> for ThermalInfo {
    fn to_column(&self, column: ThermalInfoColumn) -> String {
        match column {
            ThermalInfoColumn::Sensor => self.sensor.to_string(),
            ThermalInfoColumn::Temp => format!("{}", self.temp),
            ThermalInfoColumn::Name => self.name.to_string(),
        }
    }

    fn cmp(&self, other: &Self, column: ThermalInfoColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            ThermalInfoColumn::Sensor => self.sensor.cmp(&other.sensor),
            ThermalInfoColumn::Temp => self.temp.partial_cmp(&other.temp).unwrap(),
            ThermalInfoColumn::Name => self.name.cmp(&other.name),
        }
    }
}
fn tui(config: &Config) -> TherminalResult<()> {
    let refresh_interval = config.refresh_rate;
    let items = read_temp_data()?;
    let mut last_updated = SystemTime::now();

    let mut siv = Cursive::new();
    siv.add_global_callback('q', |s| s.quit());
    siv.add_global_callback(Esc, |s| s.quit());
    siv.add_global_callback('u', |s| {
        s.call_on_name(
            "table",
            move |table: &mut TableView<ThermalInfo, ThermalInfoColumn>| {
                {
                    table.set_items(read_temp_data().unwrap())
                }
            },
        )
        .unwrap()
    });

    siv.add_global_callback(Event::Refresh, move |s| {
        sleep(time::Duration::from_millis(200));
        s.call_on_name(
            "table",
            |table: &mut TableView<ThermalInfo, ThermalInfoColumn>| {
                if let Some(readings) = read_temp_data_after(
                    &last_updated,
                    core::time::Duration::new(refresh_interval, 0),
                )
                .unwrap()
                {
                    table.set_items(readings);
                    last_updated = SystemTime::now();
                }
            },
        )
        .unwrap()
    });
    siv.set_autorefresh(true);

    let mut table = TableView::<ThermalInfo, ThermalInfoColumn>::new()
        .column(ThermalInfoColumn::Sensor, "Sensor", |c| c.width_percent(55))
        .column(ThermalInfoColumn::Temp, "Temp (°C)", |c| {
            c.align(HAlign::Center).width_percent(15)
        })
        .column(ThermalInfoColumn::Name, "Name", |c| {
            c.ordering(Ordering::Greater).align(HAlign::Right)
        });
    table.set_items(items);

    siv.add_layer(
        Dialog::around(
            table
                .with_name("table")
                .max_size((800, 600))
                .min_size((80, 60)),
        )
        .title("Thermal Info"),
    );
    siv.run();
    Ok(())
}

//---------------
#[cfg(test)]
mod tests {}
