extern crate reqwest;
extern crate tokio;
extern crate dirs;
extern crate ini;
use std::{time::Duration, path::PathBuf};
use serde_json::{json, Value};
use std::{env, process};
use clap::{arg, command, Command, ArgAction};
use regex::Regex;
use ini::Ini;

// todo get IP based on MAC address and store
// nanoleaf discover 
// todo get and store auth token
// nanoleaf pair < should display a message
// " hold the nanoleaf controller on-off button for 5-7 seconds until the LED starts flashing in a pattern"
// keep trying to  get the auth token from http://192.188.x.x:16021/api/v1/new


fn load_config(path: &PathBuf) -> Result<Ini, ini::Error> {    // Define the path to the dotfile
    // Define the path to the dotfile
    let conf = match Ini::load_from_file(&path) {
        Ok(conf) => conf,
        Err(_) => {
            let conf = Ini::new();
            conf.write_to_file(&path).unwrap();
            conf
        }
    };
    Ok(conf)
}

async fn pair(ip: &str) -> Result<String, reqwest::Error> {
    let pair_url = format!("http://{}:16021/api/v1/new", ip);
    let client = reqwest::Client::new();
    let mut success = false;
    let mut retries = 0;
    let max_retries = 60;
    let mut auth_token: String = "".to_string();
    
    while retries < max_retries && !success {
        let result: reqwest::Result<reqwest::Response> = client
            .post(&pair_url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(json!({}).to_string())
            .send()
            .await;
        match result {
            Ok(response) => {
                if response.status().is_success() {
                    println!("Pairing request was successful!");
                    let response_json: Value = response.json().await.unwrap();
                    auth_token = response_json["auth_token"].to_string();
                    success = true;
                } else {
                    println!("Pairing request failed with status code: {:?}", response.status());
                }
            }
            Err(err) => {
                println!("Pairing request error: {:?}", err);
            }
        }
        if !success {
            retries += 1;
            let wait_time = Duration::from_secs(2);
            tokio::time::sleep(wait_time).await;
        }
    }
    if !success {
        println!("Max retries reached.")
    }
    Ok(auth_token)
}

async fn get_ip_from_mac(target_mac: &str) -> Option<String> {
    let arp_output = process::Command::new("arp")
        .arg("-a")
        .output()
        .expect("failed to execute process");
    let output_str = String::from_utf8_lossy(&arp_output.stdout);
    let re = Regex::new(r"\((.*?)\) at (.*?) ").unwrap();
    for line in output_str.lines() {
        if let Some(cap) = re.captures(line) {
            let ip = &cap[1];
            let mac = &cap[2];
            if mac == target_mac {
                return Some(ip.to_string());
            }
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let matches = command! ()
        .subcommand(
            Command::new("on")
                .about("Turn nanoleaf on")
        )
        .subcommand(
            Command::new("off")
                .about("Turn nanoleaf off")
        )
        .subcommand(
            Command::new("effect")
                .about("Change effects")
                .arg(arg!([effect] "Effect Name"))
                .arg(arg!(-l --list "lists known effects").action(ArgAction::SetTrue))
        )
        .subcommand(
            Command::new("set_ip")
                .about("Set nanoleaf IP address")
                .arg(arg!([IP] "IP address of nanoleaf controller"))
        )
        .subcommand(
            Command::new("discover")
                .about("Discover and store nanoleaf IP address")
                .arg(arg!([MAC] "MAC address of nanoleaf controller"))
        )
        .subcommand(
            Command::new("pair")
                .about("Pair with nanoleaf controller")
        )
        .get_matches();

    let home_dir = dirs::home_dir().unwrap();
    let config_path = home_dir.join(".nanoleaf");
    let mut conf = load_config(&config_path).unwrap();
    let conf_section = conf.section(None::<String>).unwrap();

    if let Some(matches) = matches.subcommand_matches("set_ip") {
        if let Some(ip) = matches.get_one::<String>("IP") {
            conf.with_section(None::<String>)
                .set("ip", ip);
            conf.write_to_file(&config_path)
                .unwrap();
            process::exit(0);
        } else {
            println!("IP address not provided.");
            process::exit(1);
        }
    }

    if let Some(matches) = matches.subcommand_matches("discover") {
        if let Some(mac) = matches.get_one::<String>("MAC") {
            if let Some(ip) = get_ip_from_mac(&mac).await {
                println!("Found IP address: {}", &ip);
                conf.with_section(None::<String>)
                    .set("ip", &ip);
                conf.write_to_file(&config_path)
                    .unwrap();
            } else {
                println!("No IP address found for MAC address: {}", mac);
                process::exit(1);
            }
        } else {
            println!("MAC address not provided.");
            process::exit(1);
        }
        return Ok(());
    }
    let ip = conf_section.get("ip").unwrap();
    if let Some(_matches) = matches.subcommand_matches("pair") {
        println!("hold the power button on your nanoleaf for 5-7s until the LED flashes in a pattern");
        let auth_token = match pair(ip).await {
            Ok(auth_token) => auth_token,
            Err(err) => {
                println!("Pairing failed: {:?}", err);
                process::exit(1);
            }
        };
        let mut conf = load_config(&config_path).unwrap();
        conf.with_section(None::<String>)
            .set("auth_token", &auth_token);
        conf.write_to_file(&config_path)
            .unwrap();
        process::exit(0);
    }
    let auth_token =  match conf_section.get("auth_token") {
        Some(auth_token) => auth_token,
        None => {
            println!("No auth token found. Please run `nanoleaf pair` to pair with your nanoleaf controller.");
            process::exit(1);
        }
    };
    let api_url = format!("http://{}:16021/api/v1/{}", ip, auth_token);
    let payload: serde_json::Value;
    let full_api_url: String;
    let rest_method: &str;
    
    if let Some(_matches) = matches.subcommand_matches("on") {
        rest_method = "put";
        payload = json!({
            "on": {
                "value": true
            }
        });
        full_api_url = format!("{}/state", api_url);
    } else if let Some(_matches) = matches.subcommand_matches("off") {
        rest_method = "put";
        payload = json!({
            "on": {
                "value": false
            }
        });
        full_api_url = format!("{}/state", api_url);
    } else if let Some(matches) = matches.subcommand_matches("effect") {
        if let Some(effect_name) = matches.get_one::<String>("effect") {
            rest_method = "put";
            payload = json!({
                "select": effect_name
            });
            full_api_url = format!("{}/effects", api_url);
        } else if matches.get_flag("list") {
            rest_method = "get";
            payload = json!({});
            full_api_url = format!("{}/effects/effectsList", api_url);
        } else {
            println!("Effect name not provided.");
            process::exit(1); // Exit the process with a non-zero status code
        }
    } else {
        println!("No subcommand provided.");
        process::exit(1); // Exit the process with a non-zero status code
    }
    
    let body = payload.to_string();
    let client = reqwest::Client::new();
    let mut success = false;
    let mut retries = 0;
    let max_retries = 60;

    while retries < max_retries && !success {

        let result: reqwest::Result<reqwest::Response>;
        if rest_method == "put" {
            result = client
                .put(&full_api_url)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body.clone())
                .send()
                .await;
        } else {
            result = client
                .get(&full_api_url)
                .send()
                .await;
        }        

        match result {
            Ok(response) => {
                if response.status().is_success() {
                    println!("{} Request was successful!", rest_method.to_uppercase());
                    success = true;
                    
                    match response.headers().get("Content-Type") {
                        Some(content_type) => {
                            if let Ok(content_type) = content_type.to_str() {
                                if content_type.contains("application/json") {
                                    let response_json: Result<Value, reqwest::Error> = response.json().await;
                                
                                    match response_json {
                                        Ok(json) => {
                                            if let Some(array) = json.as_array() {
                                                for effect in array {
                                                    println!("{}", effect);
                                                }
                                            }
                                            if let Some(object) = json.as_object() {
                                                println!("{:?}", object);
                                            }
                                        }
                                        Err(err) => {
                                            println!("Failed to parse JSON: {:?}", err);
                                        }
                                    }
                                }
                            }
                        },
                        None => {}
                    }
                    
                } else {
                    println!("{} Request failed with status code: {:?}", rest_method.to_uppercase(), response.status());
                }
            }
            Err(err) => {
                println!("PUT Request error: {:?}", err);
            }
        }
        
        if !success {
            retries += 1;
            let wait_time = Duration::from_secs(2);
            tokio::time::sleep(wait_time).await;
        }
    }

    if !success {
        println!("Max retries reached.")
    }

    Ok(())
}
