extern crate dotenv;
use tokio::*;
use dotenv::dotenv;
use log::LevelFilter;
use log::{error, info};
use log4rs;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::{
    roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger,
};
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::Config;
use log4rs::encode::pattern::PatternEncoder;
use std::collections::HashMap;
use std::{env};
use reqwest::{Client, Url};
use serde_json::{from_str, Value};
use urlencoding::encode;
use chrono::Utc;
use chrono::format::strftime::StrftimeItems;



#[tokio::main]
async fn main() {
    let logo = r#"
        88888888888 8888888b.  888888b.   888    d8P 888     888 
            888     888  "Y88b 888  "88b  888   d8P  888     888 
            888     888    888 888  .88P  888  d8P   888     888 
            888     888    888 8888888K.  888d88K    Y88b   d88P 
            888     888    888 888  "Y88b 8888888b    Y88b d88P  
            888     888    888 888    888 888  Y88b    Y88o88P   
            888     888  .d88P 888   d88P 888   Y88b    Y888P    
            888     8888888P"  8888888P"  888    Y88b    Y8P     
        "#;

        print!("{}", logo);
        println!("{}","");
        println!("{}","Developed by Triophore Technologies");
    
}
