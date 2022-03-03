use quick_xml::{Reader, events::Event};

#[derive(Debug, PartialEq)]
pub struct Metar {
    pub raw_text: String,
    pub station_id: String,
    pub observation_time: String,
    pub latitude: f32,
    pub longitude: f32,
    pub temperature: f32,
    pub dewpoint: f32,
    pub wind_dir_degrees: i32,
    pub wind_speed_knots: i32,
    pub visibility: f32,
    pub altim_in_hg: f32,
    pub quality_control_flags: (String, String),
    pub wx: String,
    pub sky_condition: Vec<(String, i32)>, // Ex: SCT 4000
    pub flight_category: String,
    pub metar_type: String,
    pub elevation_meters: f32
}

impl Metar {
    pub fn new() -> Metar{
        Metar { 
            raw_text: String::new(),
            station_id: String::new(),
            observation_time: String::new(),
            latitude: 0., 
            longitude: 0., 
            temperature: 0., 
            dewpoint: 0., 
            wind_dir_degrees: 0, 
            wind_speed_knots: 0, 
            visibility: 0., 
            altim_in_hg: 0.,
            quality_control_flags: (String::new(), String::new()),
            wx: String::new(),
            sky_condition: Vec::new(), 
            flight_category: String::new(), 
            metar_type: String::new(), 
            elevation_meters: 0. 
        }
    }

    pub fn not_found(&self) -> bool {
        self == &Metar::new()
    }

    pub fn print_metar(&self) -> () {
        println!("{}", self.raw_text)
    }

    pub fn print_metar_full(&self) -> () {
        println!("Id: {}", self.station_id);
        println!("Observation time: {}", self.observation_time);
        println!("Latitude: {}, longitude: {}", self.latitude, self.longitude);
        println!("Temperature: {} celcius, Dewpoint: {} celcius", self.temperature, self.dewpoint);
        println!("Wind is {} degrees, {} knots", self.wind_dir_degrees, self.wind_speed_knots);
        println!("Visibility: {} miles", self.visibility);
        println!("Altimeter: {} Hg", self.altim_in_hg);
        println!("Quality control flags: {} {}", self.quality_control_flags.0, self.quality_control_flags.1);
        println!("Present weather: {}", self.wx);
        println!("Sky condition: {}", "broken");
        println!("Metar type: {}", self.metar_type);
        println!("Station elevation: {} meter", self.elevation_meters);
    }
}

/// Searches the xml for the metar of a given station and returns
/// that stations metar info as a Metar struct if found.
///
/// # Arguments
///
/// * `reader` - The quick-xml Reader to look through.
/// * `id`- The station id to look for.
pub fn search_xml_for_metar(reader: &mut Reader<&[u8]>, id: &String) -> Metar {
    let mut metar = Metar::new();
    let mut buf = Vec::new();
    let mut nest_buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == b"raw_text" => {
                let raw_metar = reader.read_text(b"raw_text", &mut Vec::new()).expect("firck");
                if raw_metar.starts_with(id.to_uppercase().as_str()) {
                    loop {
                        nest_buf.clear();
                        let event =  reader.read_event(&mut nest_buf).unwrap();
                        let continue_reading = set_metar_data_from_xml(reader, event, &mut metar);
                        if !continue_reading { break }
                    }
                    metar.raw_text = raw_metar;
                    break
                } else {
                    reader.read_to_end(b"METAR", &mut buf).expect("Couldn't read to end");
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => ()
        }        
        buf.clear();
    }
    metar
}

fn set_metar_data_from_xml(reader: &mut Reader<&[u8]>, event: Event, metar: &mut Metar) -> bool {
    match event {
        Event::Start(ref e) => {
            // Quality control flags needs special treatment because of xml setup.
            if e.name() == b"quality_control_flags" {
                match reader.read_event(&mut Vec::new()) {
                    Ok(Event::Start(ref e)) => {
                        let text = reader.read_text(e.name(), &mut Vec::new());
                        metar.quality_control_flags = (String::from_utf8(e.name().to_vec()).unwrap(), text.unwrap());
                        return true
                    },
                    _ => ()
                }
                return true
            }

            // If no quality control. Parse rest
            let text = reader.read_text(e.name(), &mut Vec::new()).expect("help me im ded");
            match e.name() {
                b"station_id" => {metar.station_id = text; true},
                b"observation_time" => {metar.observation_time = text; true},
                b"latitude" => {metar.latitude = text.parse().unwrap(); true},
                b"longitude" => {metar.longitude = text.parse().unwrap(); true},
                b"temp_c" => {metar.temperature = text.parse().unwrap(); true},
                b"dewpoint_c" => {metar.dewpoint = text.parse().unwrap(); true},
                b"wind_dir_degrees" => {metar.wind_dir_degrees = text.parse().unwrap(); true},
                b"wind_speed_kt" => {metar.wind_speed_knots = text.parse().unwrap(); true},
                b"visibility_statute_mi" => {metar.visibility = text.parse().unwrap(); true},
                b"altim_in_hg" => {metar.altim_in_hg = text.parse().unwrap(); true},
                b"wx_string" => {metar.wx = text.parse().unwrap(); true},
                // TODO: Fixa sky condition parsern.
                b"sky_condition" => {metar.sky_condition.push(("HAIHAI".to_string(), 1337)); true},
                b"flight_category" => {metar.flight_category = text; true},
                b"metar_type" => {metar.metar_type = text; true},
                b"elevation_m" => {metar.elevation_meters = text.parse().unwrap(); true},
                _ => true
            }
        },
        Event::End(ref e) if e.name() == b"METAR" => {
            false
        }
        _ => true
    }
}

/// Prints all avalible metar stations.
///
/// # Arguments
///
/// * `reader` - The quick-xml Reader to look through
pub fn list_avalible_stations(reader: &mut Reader<&[u8]>) -> () {
    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == b"station_id" => {
                let id = reader.read_text(b"station_id", &mut Vec::new()).expect("Couldn't read station id");
                print!("{}, ", id);
                reader.read_to_end(b"METAR", &mut buf).expect("Couldn't read to end");
            },
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => ()
        }        
        buf.clear();
    }
}