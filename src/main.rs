use std::env;
use metar_fetcher::list_avalible_stations;
use reqwest::*;
use quick_xml::Reader;

use crate::metar_fetcher::search_xml_for_metar;

mod metar_fetcher;

#[derive(PartialEq)]
enum Settings {
    GetMetar(String, bool),
    GetTaf(String, bool),
    Help,
    List
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let settings = parse_arguments(args);

    // TODO: Byt till https://tgftp.nws.noaa.gov/data/observations/metar/ istället för aviationweather.
    // där kan man hämta TAF å gamla metar. Inge XML eller, de är uppdelade i separate .txt filer efter ID

    // TODO: Börja med att fixa TAF med den här: https://tgftp.nws.noaa.gov/data/forecasts/taf/stations/

    match settings {
        Settings::GetMetar(_, _) | Settings::List => {
            let request_url = "https://www.aviationweather.gov/adds/dataserver_current/current/metars.cache.xml";
            let response_xml = reqwest::get(request_url).await?.text().await?;
            let mut reader = Reader::from_str(&response_xml);
            reader.trim_text(true);

            do_metar_actions(settings, reader);
        },
        Settings::GetTaf(_, _) => {
            get_taf().await?;
        },
        Settings::Help => { print_help_screen(); return Ok(()) }
    }
    Ok(())
}

async fn get_taf() -> Result<()> {
    let taf_address = "https://tgftp.nws.noaa.gov/data/forecasts/taf/stations/".to_owned();
    let id = "ESSD".to_owned();
    let response = reqwest::get(taf_address + &id + ".TXT").await?;
    let mut response_text: String = "None".to_string();
    match response.error_for_status_ref() {
        Ok(_req) => {
            response_text = response.text().await?;
        },
        Err(_err) => {
            println!("No TAF was found for [insert station id here]");
        }
    }
    println!("{}", response_text);
    Ok(())
}

fn do_metar_actions(settings: Settings, mut reader: Reader<&[u8]>) {
    match settings {
        Settings::GetMetar(station_id, full) => {
            let metar_data = search_xml_for_metar(&mut reader, &station_id);
            if metar_data.not_found() {
                println!("Could not find station with id: {}", station_id);
            } else {
                if full {
                    metar_data.print_metar_full()
                } else {
                    metar_data.print_metar()
                }
            }
        },
        Settings::List => {
            list_avalible_stations(&mut reader)
        },
        _ => ()
    }
}

fn parse_arguments(args: Vec<String>) -> Settings {
    if args.len() <= 1 {
        return Settings::Help
    }

    if args[1].as_str().starts_with('-') {
        match args[1].as_str() {
            "-t" | "-taf" => return Settings::GetTaf(String::new(), false),
            "-h" | "-help" => return Settings::Help,
            "-l" | "-list" => {
                println!("Listing all avalible stations...");
                return Settings::List
            },
            _ => return Settings::Help
        }
    } else {
        println!("Fetching metar...");
    }

    let mut settings: Settings = Settings::Help;
    if args.len() < 2 { return Settings::Help };
    if args.len() == 2 {
        settings = Settings::GetMetar(args[1].to_string(), false)
    } else if args.len() == 3 {
        match args[2].as_str() {
            "-a" => settings = Settings::GetMetar(args[1].to_string(), true),
            _ => settings = Settings::Help
        }
    }
    return settings
}

fn print_help_screen() -> () {
    println!("USAGE:");
    println!("\tmetar_fetcher [OPTION or STATION_ID] [SUBCOMMAND]");

    println!("OPTIONS:");
    println!("\t<STATION_ID> <FLAG>         The station id to get the metar data from. Use flag -a for decoded version.");
    println!("\t-t, -taf <STATION_ID>       Get the taf from a given station.");
    println!("\t-l, -list                   List all avalible stations for metar.");
    println!("\t-h, -help                   Show this help screen.");

    println!("EXAMPLES:");
    println!("\tmetar_fetcher ESSD -a       Prints the decoded version of metar for ESSD.");
    println!("\tmetar_fetcher -t ESNU       Prints the taf for ESNU.");
}