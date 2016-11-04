extern crate fruently;
use std::env;
use fruently::fluent::Fluent;
use fruently::retry_conf::RetryConf;
use std::collections::HashMap;
use fruently::forwardable::MsgpackForwardable;

fn main() {
    let home = env::home_dir().unwrap();
    let file = home.join("buffer");
    let conf = RetryConf::new().store_file(file.clone()).max(5).multiplier(2_f64);
    let mut obj: HashMap<String, String> = HashMap::new();
    obj.insert("name".to_string(), "fruently".to_string());
    let fruently = Fluent::new_with_conf("noexistent.local:24224", "test", conf);
    match fruently.post(&obj) {
        Err(e) => println!("{:?}", e),
        Ok(_) => {
            assert!(file.exists());
            return
        },
    }
}
