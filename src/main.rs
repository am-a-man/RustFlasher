use crate::serial_utils::SerialItem;
use serialport::{SerialPort};
use std::process::Command;
use slint::{Timer, TimerMode};
use config::{Config, File, FileFormat};
use strfmt::{strfmt,strfmt_builder};
use std::time::{Duration, Instant};

pub mod serial_utils;

fn read_config() -> Result<Config, config::ConfigError> {

    // setting default values
    let config_builder = Config::builder()
                .set_default("flash.command", "")?
                .add_source(File::new("config", FileFormat::Toml));

    config_builder.build()
}


fn main() {    

    let app = App::new().unwrap();
    // basic setup using the config, can add more app initializing logic
    ({
        // weak borrow app 
        let weak_app = app.as_weak();
        move || {
            let app = weak_app.unwrap();
            match read_config() {
                Ok(config) => {
                    if let Ok(flash_command) = config.get::<String>("flash.command") {
                        app.set_default_flash_command(flash_command.into());
                    }
                },
                Err(err) => {
                    eprintln!("Error: {}",err)
                }
            }
        }
    })();
    
    // Logic handling loading the port information on screen
    app.global::<Callbacks>().on_insert_data({
        let weak_app = app.as_weak();
        move || {
            let items = SerialItem::serial_list();
            let mut data = Vec::new();
            for item in &items {
                let info = PortData {
                    name: String::from(item.get_name()).into(),
                    vendor: String::from(item.get_vendor()).into(),
                    product: String::from(item.get_product()).into(),
                    usb: String::from(item.get_usb()).into(),
                };
                data.push(info);
            }
            let app = weak_app.unwrap();
            app.set_port_info_list(slint::ModelRc::from(std::rc::Rc::<slint::VecModel<PortData>>::new(data.clone().into()).clone()));
        }
    });
    
    // Periodic update for refreshing the port information on screen
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, std::time::Duration::from_millis(5000), {
        let weak_app = app.as_weak();
        move || {
            let app = weak_app.unwrap();
            app.global::<Callbacks>().invoke_insert_data();
        }
    });

    // Handling logic to flash the device
    app.global::<Callbacks>().on_execute_command({
        let weak_app = app.as_weak();
        move |port_name, _port_usb| {
            let app = weak_app.unwrap();
            let current_os = std::env::consts::OS;
            let port: Box<dyn SerialPort> = serialport::new(String::from(port_name).as_str(), 9600).open().expect("Failed to open port");

            app.set_status("running command".into());
            if current_os == "linux" {
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(format!("{}", app.get_flash_command()))
                    .output()
                    .expect("failed to execute the program");
                app.set_status(format!("{:?}", output).into());
            } else if current_os == "windows" {
                let output = Command::new("cmd")
                    .arg("/C")
                    .arg(format!("{}", app.get_flash_command()))
                    .output()
                    .expect("failed to execute the program");
                app.set_status(format!("{:?}", output).into());
            }
            std::mem::drop(port);
            app.set_current_port(PortData {
                name: String::from("").into(),
                vendor: String::from("").into(),
                product: String::from("").into(),
                usb: String::from("").into()
            });

            match read_config() {
                Ok(config) => {
                    if let Ok(flash_command) = config.get::<String>("flash.command") {
                        app.set_default_flash_command(flash_command.into());
                    }
                },
                Err(err) => {
                    eprintln!("Error: {}",err)
                }
            }
            app.set_port_open(false);       
        }
    });


    // For sending strings, required to send the password and upgrade key
    app.global::<Callbacks>().on_write_data({
        let weak_app = app.as_weak();
        move |port_name, port_vendor, port_product, port_usb| {
            let app = weak_app.unwrap();
            
            // binding current state of application to a single port
            app.set_port_open(true);
            app.set_current_port(PortData {
                name: port_name.clone(),
                vendor: port_vendor.clone(),
                product: port_product.clone(),
                usb: port_usb.clone(), 
            });
            let mut flash_command: String = app.get_default_flash_command().into();
            flash_command = strfmt!(flash_command.as_str(), port_name => port_name.to_string()).unwrap();
            app.set_default_flash_command(flash_command.into());

            let mut port: Box<dyn SerialPort> = serialport::new(String::from(port_name).as_str(), 9600)
                .timeout(Duration::from_secs(5))
                .open().expect("Failed to open port");
            
            let mut output = app.get_password().to_string();
            port.write(output.as_bytes()).expect("Write failed!");

            output = app.get_initial_command().to_string();
            port.write(output.as_bytes()).expect("Write failed!");

            std::mem::drop(port);
        }
    });


    app.global::<Callbacks>().on_reset_cache({
        let weak_app = app.as_weak();
        move || {
            let app = weak_app.unwrap();
            app.set_current_port(PortData {
                name: String::from("").into(),
                vendor: String::from("").into(),
                product: String::from("").into(),
                usb: String::from("").into()
            });
            match read_config() {
                Ok(config) => {
                    if let Ok(flash_command) = config.get::<String>("flash.command") {
                        app.set_default_flash_command(flash_command.into());
                    }
                },
                Err(err) => {
                    eprintln!("Error: {}",err)
                }
            }
            app.set_port_open(false);    
        }
    });
 
    app.run().unwrap(); 
}


slint::slint! { 
    import { Button , HorizontalBox, VerticalBox, GroupBox, ScrollView, TextEdit} from "std-widgets.slint";

    struct PortData {
        name: string,
        vendor: string,
        product: string,
        usb: string,
    }

    export global Callbacks {
        callback insert_data();
        callback write_data(string, string, string, string);
        callback execute_command(string, string);
        callback reset_cache();
    }

    export component App inherits Window {
        title: "RustFlasherPro";
        in property <PortData> current_port;
        in property <[PortData]> port_info_list: [];
        in property <percent> cell_width: 15%;
        in property <string> password: password-input.text;
        in property <string> initial_command: initial-command-input.text;
        in property <string> flash_command: flash-command.text; 
        in property <string> default_flash_command: "";

        in property <string> status;
        in property <bool> port_open: false;

        ScrollView {
            width: 720px;
            min-height: 720px;
            
            VerticalBox {
                HorizontalBox {
                    Text {
                        text: "Enter password: ";
                        vertical-alignment: center;
                        horizontal-alignment: center;
                        width: 240px;
                        wrap: word-wrap;
                    }
                    Rectangle {
                        border-color: black;
                        border-radius: 10px;
                        border-width: 1px;
                        height: 40px;   
                        password_input := TextInput {
                            vertical-alignment: center;
                            height: 90%;
                            width: 90%;
                            text: "";
                            wrap: word-wrap;   
                        }
                    }
                }
                HorizontalBox {
                    Text {
                        text: "Enter Command: ";
                        vertical-alignment: center;
                        horizontal-alignment: center;
                        width: 240px;
                        wrap: word-wrap;
                    }
                    Rectangle {
                        border-color: black;
                        border-radius: 10px;
                        border-width: 1px;
                        height: 40px;
                        initial_command_input := TextInput {
                            vertical-alignment: center;
                            height: 90%;
                            width: 90%;
                            text: "enter_dfu";
                            wrap: word-wrap;
                        }
                    }
                }
                HorizontalBox { 
                    Text {
                        text: "Flash Command: ";
                        vertical-alignment: center;
                        horizontal-alignment: center;
                        width: 240px;
                        wrap: word-wrap;
                    }
                    Rectangle {
                        border-color: black;
                        border-radius: 10px;
                        border-width: 1px;
                        height: 100px;
                        flash_command := TextInput {
                            height: 90%;
                            width: 90%;
                            wrap: word-wrap;
                            text: {default-flash-command};
                        }
                    }
                }
                Button {
                    text: "Execute";
                    height: 40px;
                    enabled: {port-open}
                    clicked => {Callbacks.execute-command(current-port.name, current-port.usb)}
                }
                ScrollView { 
                    height: 50px;
                    HorizontalBox {
                        Text {
                            text: "Status: ";
                            width: {cell-width}
                            vertical-alignment: center;
                            horizontal-alignment: center;
                        }
                        Rectangle { 
                            Text {
                                text: {status};
                            }
                        }
                    }
                }
                HorizontalBox {
                    Text {
                        text: "Name";
                        vertical-alignment: center;
                        horizontal-alignment: center;
                        width: {cell-width};
                        wrap: word-wrap;
                    }
                    Text {
                        text: "Vendor";
                        vertical-alignment: center;
                        horizontal-alignment: center;
                        width: {cell-width};
                        wrap: word-wrap;
                    }
                    Text {
                        text: "Product";
                        vertical-alignment: center;
                        horizontal-alignment: center;
                        wrap: word-wrap;
                        width: {cell-width};
                    }
                    Text {
                        text: "USB";
                        vertical-alignment: center;
                        horizontal-alignment: center;
                        wrap: word-wrap;
                        width: {cell-width};
                    }
                    Text {
                        text: "";
                        wrap: word-wrap;
                        width: {cell-width};
                    }
                    height: 40px;
                }
                for info[i] in port_info_list : HorizontalBox {
                    Rectangle { 
                        background: port-open ? current-port.name == info.name ? lightgreen : root.background : root.background;
                        HorizontalBox {
                            Text {
                                text: info.name;
                                vertical-alignment: center;
                                wrap: word-wrap;
                                horizontal-alignment: center;
                                width: {cell-width};
                            }
                            Text {
                                text: info.vendor;
                                vertical-alignment: center;
                                horizontal-alignment: center;
                                wrap: word-wrap;
                                width: {cell-width};
                            }
                            Text {
                                text: info.product;
                                vertical-alignment: center;
                                horizontal-alignment: center;
                                wrap: word-wrap;
                                width: {cell-width};
                            }
                            Text {
                                text: info.usb;
                                vertical-alignment: center;
                                horizontal-alignment: center;
                                wrap: word-wrap;
                                width: {cell-width};
                            }
                            Button {
                                text: "Send Key";
                                height: 40px;
                                width: {cell-width};
                                clicked => {Callbacks.write-data(info.name, info.vendor, info.product, info.usb)}
                                enabled: !port-open;
                            }
                            Button {
                                text: "Exit";
                                height: 40px;
                                width: {cell-width};
                                clicked => {Callbacks.reset-cache()}
                                enabled: port-open ? info.name == current-port.name ? true : false : false;
                            }
                        } 
                    }
                }
            }
        }
    
    }
}
