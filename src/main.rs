extern crate dotenv;
use tokio::*;
use dotenv::{dotenv, var};
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
// use std::error::Error;
use std::sync::Arc;
use std::{env};
use reqwest::{Client, Url};
use serde_json::{from_str, Value};
use urlencoding::encode;
use chrono::Utc;
use chrono::format::strftime::StrftimeItems;


use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use bb8::{Pool, RunError};

use async_trait::async_trait;

use anyhow::Error;

use rmp_serde::{Deserializer, Serializer};
use tokio::io::{ AsyncBufReadExt};

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone)]
struct TcpStreamConnectionManager;

#[async_trait::async_trait]
impl bb8::ManageConnection for TcpStreamConnectionManager {
    type Connection = TcpStream;
    type Error = Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        // Not used in this context
        unreachable!() 
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        // Attempt to peek at the connection to check if it's still active
        match conn.peek(&mut [0]).await {
            Ok(0) => Err(Error::msg("Connection closed")),  // No bytes peeked means the connection is closed
            Ok(_) => Ok(()),                            // Some bytes peeked, connection is likely valid
            Err(e) => Err(e.into()),                     // An error occurred during the peek
        }
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

#[derive(Deserialize, Serialize, Debug)]
// Define a struct or enum to represent your expected JSON data structure.
struct ClientData {
    // ... your fields
}


// Helper function to read lines from a `PooledConnection`
async fn read_lines_from_conn(
    conn: &mut PooledConnection<'_, TcpStreamConnectionManager>,
) -> Result<(), Error> {
    let mut reader = tokio::io::BufReader::new(conn);
    let mut lines = reader.lines();

    while let Some(line_result) = lines.next_line().await {
        match line_result {
            Ok(line) if !line.is_empty() => {
                match serde_json::from_str::<ClientData>(&line) {
                    Ok(data) => {
                        // Serialize to MessagePack
                        let mut buf = Vec::new();
                        data.serialize(&mut Serializer::new(&mut buf)).unwrap();

                        // Process or store the MessagePack data (buf)
                        println!("Received and serialized data: {:?}", buf);
                    }
                    Err(e) => {
                        eprintln!("Error parsing JSON: {:?}", e);
                    }
                }
            }
            Ok(_) => (), // Ignore empty lines or successful reads
            Err(e) => {
                eprintln!("Error reading line: {:?}", e);
                return Err(e.into());
            }
        }
    }

    Ok(()) // Return Ok when done reading (connection closed or all lines processed)
}


#[tokio::main]
async fn main() -> Result<(), Error> {
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

        let log_line_pattern = "{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5} | {m}{n}";

        let trigger_size = byte_unit::n_mb_bytes!(1) as u64;
        let trigger = Box::new(SizeTrigger::new(trigger_size));
    
        let roller_pattern = "logs/arch/log_{}.gz";
        let roller_count = 5;
        let roller_base = 1;
        let roller = Box::new(
            FixedWindowRoller::builder()
                .base(roller_base)
                .build(roller_pattern, roller_count)
                .unwrap(),
        );
    
        let _compound_policy = Box::new(CompoundPolicy::new(trigger, roller));
    
        let step_ap = RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(log_line_pattern)))
            .build("logs/log/log.log", _compound_policy)
            .unwrap();
    
        //let stdout = ConsoleAppender::builder().build();
    
        let config = Config::builder()
            //.appender(Appender::builder().build("stdout", Box::new(stdout)))
            .appender(Appender::builder().build("step_ap", Box::new(step_ap)))
            .logger(
                Logger::builder()
                    .appender("step_ap")
                    .build("step", LevelFilter::Trace),
            )
    
            .build(
                Root::builder()
                    .appender("step_ap")
                    .build(LevelFilter::Trace),
            )
            .unwrap();
    
        let _handle = log4rs::init_config(config).unwrap();
    
    
        dotenv().ok();

        let server_host = env::var("TDBKV_HOST").unwrap_or("127.0.0.1".to_string());
        let server_port = env::var("TDBKV_PORT").unwrap_or("6500".to_string());

        let connection_pool = env::var("TDBKV_POOL_SIZE").unwrap_or("10".to_string());

        let mut connection_pool_int : u32 = 10;

        let connection_pool_int_res = connection_pool.parse::<u32>();

        match connection_pool_int_res {
            Ok(conn)=>{
                if conn > 0 {
                    connection_pool_int = conn;
                }else{
                    println!("{}","Error parsing connection pool count ,less than 1");
                    println!("{}","Using default pool count of 10"); 
                }
            },
            Err(_)=>{
                println!("{}","Error parsing connection pool count");
                println!("{}","Using default pool count of 10");
            }
        }


        let manager = TcpStreamConnectionManager;
        let pool = Arc::new(Pool::builder().max_size(connection_pool_int.clone()).build(manager).await?);

      


        let listener = TcpListener::bind(format!("{}:{}",server_host,server_port)).await?;
        println!("TDBKV Server listnening on {}:{}",server_host,server_port);
        println!("TDBKV {} {}","Server running with pool",connection_pool_int);

        loop {
            let (mut socket, _) = listener.accept().await?;
            let pool_clone = pool.clone();
    
            tokio::spawn(async move {
                let mut conn = match pool_clone.get().await {
                    Ok(conn) => {
                        let mut reader = tokio::io::BufReader::new(&mut conn);
                        let mut line = String::new();
                        while let Ok(_) = reader.read_line(&mut line).await {
                            if !line.is_empty() { // Ensure line is not empty
                                match serde_json::from_str::<ClientData>(&line) {
                                    Ok(data) => {
                                        // Serialize to MessagePack
                                        let mut buf = Vec::new();
                                        data.serialize(&mut Serializer::new(&mut buf)).unwrap();
            
                                        // Process or store the MessagePack data (buf)
                                        println!("Received and serialized data: {:?}", buf); 
                                    },
                                    Err(e) => {
                                        eprintln!("Error parsing JSON: {:?}", e);
                                        // Handle invalid JSON (e.g., send an error message to the client)
                                    }
                                }
                                line.clear(); // Clear the line for the next read
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to get connection from pool: {:?}", e);
                        return; // Exit the task if getting a connection fails
                    }
                };
    
               
            });
        }



        Ok(())
        
    
}
