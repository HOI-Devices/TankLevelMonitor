use tungstenite::{connect, Message,WebSocket};
use tungstenite::client::AutoStream;
use url::Url;
use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
use std::env;
use serde_json::Value;
use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;
use console_logger::ConsoleLogger;

pub struct Client{
    host:String,
    port:String,
    password:String,
    name:String,
    device_type:String,
    server_name:String,
    logger:ConsoleLogger,
    distance_high:i16,
    distance_low:i16
}

impl  Client{
    pub fn new(host_data:String,
        port_data:String,
        pass:String, 
        device_name:String,
        type_of_bot:String,
        outside_server_name:String)->Self{

        Self{
            host:host_data.to_string(),
            port:port_data,
            password:pass,
            name:device_name,
            device_type:type_of_bot,
            server_name:outside_server_name,
            logger: ConsoleLogger::new(),
            distance_high : 1,
            distance_low:1
        }
    }

    pub fn authenticate(&mut self, socket:&mut WebSocket<AutoStream>)->bool{
        let send_password_result = socket.write_message(Message::Text(self.password.to_owned()));

        //send password
        if send_password_result.is_ok(){
            let send_name_and_type_result =
                socket.write_message(Message::Text(self.name_and_type()));
            //send name and type    
            if send_name_and_type_result.is_ok(){
                let send_server_name_result = 
                    socket.write_message(Message::Text(self.server_name.to_owned()));
                    // send server name
                if send_server_name_result.is_ok(){
                    return self.check_auth_response(socket);
                }
            }
        }
        return false;
    }

    fn check_auth_response(&mut self,  socket:&mut WebSocket<AutoStream>)-> bool{
        let msg_result = socket.read_message();

        if msg_result.is_ok(){
            let msg = msg_result.unwrap().into_text().unwrap();
            if msg == "success"{
                self.logger.log_basic_row("Successfully Authenticated!!\n","green");
                return true;
            }
            else{
                self.logger.log_failed_auth();
                return false;
            }
        }
        else{
            self.logger.log_failed_auth();
            return false;
        }
    }

     //keep listening for server requests and route the requests
    fn enter_main_loop(&mut self, 
        encountered_error:&mut bool, 
        socket:&mut WebSocket<AutoStream>,
        current_distance:&Arc<Mutex<i16>>){
        loop {
            let msg_result = socket.read_message();
            if msg_result.is_ok(){
                let msg = msg_result.unwrap().into_text().unwrap();
                if msg == "disconnect"{
                    let response_msg_result = socket.write_message(Message::Text("success".into()));
                    if response_msg_result.is_ok(){
                        *encountered_error = false;
                        break;
                    }
                }
                else{
                    self.route_message(msg,socket,current_distance);
                }
            }
            else{
                self.logger.log_error_encounter();
                *encountered_error = true;
                break;
            }
        }
    }

    pub fn begin_monitoring(&mut self,
        encountered_error: &mut bool,
        current_distance:&Arc<Mutex<i16>>
         ){
        self.logger.log_welcome();
        let url = format!("ws://{}:{}",self.host,self.port);
        let attempt = connect(Url::parse(&url).unwrap());

        if attempt.is_ok(){
            let (mut socket, response) = attempt.unwrap();
            //if we successfully authenticated
            if self.authenticate(&mut socket) == true{
                self.enter_main_loop(encountered_error,&mut socket,current_distance);
            }
            else{
                self.logger.log_failed_auth();
                self.logger.log_error_encounter();
                *encountered_error = true;
            }
        }
        else{
            self.logger.log_error_encounter();
            *encountered_error = true;
        }
    }

    fn route_message(&mut self,message:String,socket:&mut WebSocket<AutoStream>,
        current_distance:&Arc<Mutex<i16>>){
        if message =="deactivate"{
            let send_deactivate_status_result = socket.write_message(Message::Text("success".into()));
               // successfully notified the server of the success
            if send_deactivate_status_result.is_ok(){
                self.logger.log_basic_row("Deactivating Device!","red");
                self.enter_deactivation_loop(socket);
                self.logger.log_basic_row("Activated Device!","green");
            }
        }
        else if message == "passive_data"{
            let distance = *current_distance.lock().unwrap();
            socket.write_message(
                Message::Text(
                    self.formatted_passive_data(&distance)));
        }
    }

    fn enter_deactivation_loop(&mut self, socket:&mut WebSocket<AutoStream>){
        loop{
            let msg_result = socket.read_message();

            // if the msg result is valid
            if msg_result.is_ok(){
                let msg = msg_result.unwrap();
                let msg_text_result = msg.into_text();
                if msg_text_result.is_ok(){
    
                    let msg_text = msg_text_result.unwrap();
                    if msg_text == "activate"{
                        let send_activate_status_result = 
                            socket.write_message(Message::Text("success".into()));
                        // successfully notified the server of the success
                        if send_activate_status_result.is_ok(){
                            break;
                        };
                    };
                };
            }
            //connection error
            else{
                break;
            }
        }
    }

    fn name_and_type(&mut self)-> String{
        let name_and_type = json!({
            "name":&self.name,
            "type":&self.device_type
        });
        return name_and_type.to_string();
    }

    fn formatted_passive_data(&mut self,current_distance:&i6)->String{
        if current_distance >= self.distance_high{
            let formatted = json!({"current_distance":current_distance.to_string(),
            "status":"alert_present","message":format("{}(TankLevelMonitor) is at high capacity!",self.name)}).to_string();
            return formatted;
        }
        else if current_distance <= distance_low{
            let formatted = json!({"current_distance":current_distance.to_string(),
            "status":"alert_present","message":format("{}(TankLevelMonitor) is at low capacity!",self.name)}).to_string();
            return formatted;
        }
        else{
            let formatted = json!({"current_distance":current_distance.to_string(),
            "status":"alert_not_present"}).to_string();
            return formatted;
        }
    }
}