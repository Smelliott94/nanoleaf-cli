extern crate reqwest;
extern crate tokio;
use std::time::Duration;
use serde_json::{json, Value};
use std::{env, process};
use clap::{arg, command, Command, ArgAction};

// todo get IP based on MAC address and store
// nanoleaf discover 
// todo get and store auth token
// nanoleaf pair < should display a message
// " hold the nanoleaf controller on-off button for 5-7 seconds until the LED starts flashing in a pattern"
// keep trying to  get the auth token from http://192.188.x.x:16021/api/v1/new

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let api_url = "http://192.168.0.29:16021/api/v1/4dGvb5rIPOYeN0LINLI1THXkDeG079Pn";
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
        .get_matches();
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
