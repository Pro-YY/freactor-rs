use std::error::Error;
use tokio::time;
use tracing::{info, debug};
use tracing_subscriber;
use std::sync::{Arc, Mutex};
use tokio::task::JoinSet;
use serde_json::{Value, json};

use reqwest;

use freactor::{
    Code,
    State,
    box_async_fn,
    Freactor,
    version,
};


fn add_x(state: Arc<Mutex<State>>, x: i64) {
    let mut s = state.lock().unwrap();
    if let Some(value) = s.data.get_mut("value") {
        let v = value.as_i64().unwrap() + x as i64;
        *value = Value::Number(serde_json::Number::from(v));
    }
}

fn print_state(state: Arc<Mutex<State>>, fn_name: &str) {
    let s = state.lock().unwrap();
    debug!("{} done with: {:?}", fn_name, s.data.get("value").unwrap());
}

async fn fetch_my_ip() -> Result<(), Box<dyn Error + Send>>{
    let response = reqwest::get("https://ipinfo.io")
    .await
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
    let ip = json["ip"].as_str().unwrap_or("Unknown IP");
    info!("IP Address: {}", ip);
    Ok(())
}

async fn r1(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    debug!("r1 processing...");
	time::sleep(time::Duration::from_secs(1)).await;

    // fetch_my_ip().await?;
    add_x(state.clone(), 1);
    print_state(state, "r1");

    Ok(Code::Success)
    // Ok(FrtCode::Retry(Some("What?".to_string())))
    // Ok(FrtCode::Failure(1, Some("Whoo".to_string())))
}


async fn r2(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    debug!("r2 processing...");
	time::sleep(time::Duration::from_secs(1)).await;

    add_x(state.clone(), 2);
    print_state(state, "r2");

    Ok(Code::Success)
    // Ok(FrtCode::Failure(1, Some("Whoo".to_string())))
}

async fn r3(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    debug!("r3 processing...");
	time::sleep(time::Duration::from_secs(1)).await;

    // fetch_my_ip().await?;
    add_x(state.clone(), 3);
    print_state(state, "r3");

    Ok(Code::Success)
}

async fn r4(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    debug!("r4 processing...");
	time::sleep(time::Duration::from_secs(1)).await;

    add_x(state.clone(), 4);
    print_state(state, "r4");

    Ok(Code::Success)
}

async fn r5(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    debug!("r5 processing...");
	time::sleep(time::Duration::from_secs(1)).await;

    add_x(state.clone(), 5);
    print_state(state, "r5");

    // Ok(FrtCode::Success)
    Ok(Code::Failure(1, Some("Whoo".to_string())))
}

async fn r6(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    debug!("r6 processing...");
	time::sleep(time::Duration::from_secs(1)).await;

    add_x(state.clone(), 6);
    print_state(state, "r6");

    Ok(Code::Success)
}


async fn _sleeper(id: i32) {
    info!("{}: Sleeping", id);
	time::sleep(time::Duration::from_secs(1)).await;
    info!("{}:Awake!", id);
}


async fn run() {
    let flow_config = r#"
    {
        "ExampleTask1": {
            "init_step": "r1",
            "config": {
                "r1": { "edges": ["r2", "r3", "r4"]},
                "r2": { "edges": ["r5", "r3"]},
                "r3": { "edges": ["r6", "r4"]},
                "r4": { "edges": []},
                "r5": { "edges": ["r6", "r3"]},
                "r6": { "edges": [], "retry": null}
            }
        }
    }
    "#.to_string();
    // let flow_config = r#"{ "ExampleTask1": { "init_step": "r1", "config": { "r1": { "edges": []} } }}"#.to_string();

    let func_map = vec![
        ("r1".to_string(), box_async_fn(r1)),
        ("r2".to_string(), box_async_fn(r2)),
        ("r3".to_string(), box_async_fn(r3)),
        ("r4".to_string(), box_async_fn(r4)),
        ("r5".to_string(), box_async_fn(r5)),
        ("r6".to_string(), box_async_fn(r6)),
    ];
    let f = Freactor::new(func_map, flow_config); // ownership moved by design

    // multi instance concurrently
    let mut shared_vecs: Vec<Arc<Mutex<State>>> = Vec::with_capacity(10);
    for i in 0..shared_vecs.capacity() {
        let state = State {
            data: vec![("value".to_string(), json!(i))].into_iter().collect(),
            meta: None,
        };
        shared_vecs.push(Arc::new(Mutex::new(state)));
    }

    let mut jset = JoinSet::new();
    // concurrent
    for v in shared_vecs.clone() {
        // clone flow config for every spawn/req
        let fc = f.clone();
        jset.spawn(async move {
            let _r =
                fc.run("ExampleTask1", v).await;
        });
    }
    while let Some(_res) = jset.join_next().await {}

    // print outcome
    for v in shared_vecs {
        let vec = v.lock().unwrap();
        info!("Mutated Vec: {:?}", *vec);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // env_logger::init();
    tracing_subscriber::fmt::init();

    info!("{}", version());

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build().unwrap();

	runtime.block_on(async move {
		run().await;
	});

    Ok(())
}
