extern crate wpactrl;
extern crate clap;

use std::{thread, time, process::Command};
use clap::{Arg, App, SubCommand};
use wpactrl::WpaCtrlAttached;

//<3>CTRL-EVENT-SSID-TEMP-DISABLED id=0 ssid="iflytek" auth_failures=2 duration=23 reason=WRONG_KEY
//<3>CTRL-EVENT-NETWORK-NOT-FOUND
//<3>CTRL-EVENT-CONNECTED - Connection to dc:da:80:a9:e6:90 completed [id=0 id_str=]
fn log_process(log: String) -> (&'static str, bool) {
    match log.find("reason=WRONG_KEY") {
        Some(_) => return ("-1", true),
        None => "",
    };
    match log.find("CTRL-EVENT-NETWORK-NOT-FOUND") {
        Some(_) => return ("-2", true),
        None => "",
    };
    match log.find("CTRL-EVENT-CONNECTED") {
        Some(_) => return ("0", true),
        None => return ("链接中....", false),
    };
}

#[cfg(debug_assertions)]
fn request_cmd(wpa: &mut WpaCtrlAttached, cmd: &str) -> String {
    println!("cmd is: {}", cmd);
    let req = wpa.request(cmd).unwrap();
    println!("response is: {}", req);
    req
}

#[cfg(not(debug_assertions))]
fn request_cmd(wpa: &mut WpaCtrlAttached, cmd: &str) -> String {
    wpa.request(cmd).unwrap()
}

fn remove_networks(wpa: &mut WpaCtrlAttached) {
    let networks_meta: String = request_cmd(wpa, "LIST_NETWORKS");
    let networks: Vec<&str> = networks_meta.split("\n").collect();
    for item in &networks[1..] {
        if let Some(id) = item.chars().next() {
            request_cmd(wpa, format!("DISABLE_NETWORK {}", id).as_str());
            request_cmd(wpa, format!("REMOVE_NETWORK {}", id).as_str());
        }

    }
}

fn connect_wifi(ssid: &str, password: Option<&str>) {
    let mut wpa_attached = wpactrl::WpaCtrl::new().open().unwrap().attach().unwrap();

    remove_networks(&mut wpa_attached);

    let network_id = request_cmd(&mut wpa_attached, "ADD_NETWORK");
    request_cmd(&mut wpa_attached, format!("SET_NETWORK {} ssid \"{}\"", network_id, ssid).as_str());

    let password_cmd = match password {
        Some(x) => format!("SET_NETWORK {} psk \"{}\"", network_id, x),
        None => format!("SET_NETWORK {} key_mgmt NONE", network_id)
    };

    request_cmd(&mut wpa_attached, password_cmd.as_str());
    request_cmd(&mut wpa_attached, format!("ENABLE_NETWORK {}", network_id).as_str());

    loop {
        let data = wpa_attached.recv().unwrap();
        wpa_attached.request("STATUS").unwrap();
        match data {
            Some(x) => {
                let (message, exit) = log_process(x);
                if exit {
                    request_cmd(&mut wpa_attached, format!("SELECT_NETWORK {}", network_id).as_str());
                    request_cmd(&mut wpa_attached, "SAVE_CONFIG");
                    Command::new("sh").arg("-c").arg("killall dhcpcd").output().expect("failed to execute process");
                    Command::new("sh").arg("-c").arg("dhcpcd -A wlan0").output().expect("failed to execute process");
                    println!("{}", message);
                    return;
                }
            }
            None => thread::sleep(time::Duration::from_millis(1)),
        }
    }
}

fn main() {

    let wifi_command = SubCommand::with_name("wifi")
        .about("wifi连接\nreturn:\n0: 连接成功\n-1: 密码错误\n-2: ssid 错误\n-3: 未知错误")
        .arg(Arg::with_name("ssid").short("s").help("wifi ssid").required(true).value_name("SSID"))
        .arg(Arg::with_name("password").short("p").help("wifi password").value_name("PASSWORD"));

    let matches = App::new("iflyos_link")
        .version("1.0")
        .about("集成wifi连接工具，BLE 工具")
        .author("MJ")
        .subcommand(wifi_command)
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("wifi") {
        let ssid = matches.value_of("ssid").unwrap();
        let password = matches.value_of("password");
        connect_wifi(ssid, password);
    }

}
