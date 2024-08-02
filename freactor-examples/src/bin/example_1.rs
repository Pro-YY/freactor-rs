use std::error::Error;
use tokio::time;
use tracing::info;
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

async fn r1(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    info!("r1 processing...");
	time::sleep(time::Duration::from_secs(1)).await;

    let response = reqwest::get("https://ipinfo.io")
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
    let ip = json["ip"].as_str().unwrap_or("Unknown IP");
    println!("IP Address: {}", ip);

    add_x(state.clone(), 1);
    let s = state.lock().unwrap();
    info!("r1 done with: {:?}", s.data.get("value").unwrap());
    Ok(Code::Success)
    // Ok(FrtCode::Retry(Some("What?".to_string())))
    // Ok(FrtCode::Failure(1, Some("Whoo".to_string())))
}


async fn r2(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    info!("r2 processing...");
	time::sleep(time::Duration::from_secs(1)).await;
    add_x(state.clone(), 2);
    let s = state.lock().unwrap();
    info!("r2 done with: {:?}", s.data.get("value").unwrap());
    Ok(Code::Success)
    // Ok(FrtCode::Failure(1, Some("Whoo".to_string())))
}

async fn r3(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    info!("r3 processing...");
	time::sleep(time::Duration::from_secs(1)).await;

    let response = reqwest::get("https://ipinfo.io")
    .await
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
    let ip = json["ip"].as_str().unwrap_or("Unknown IP");
    println!("IP Address: {}", ip);

    add_x(state.clone(), 3);
    let s = state.lock().unwrap();
    info!("r3 done with: {:?}", s.data.get("value").unwrap());
    Ok(Code::Success)
}

async fn r4(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    info!("r4 processing...");
	time::sleep(time::Duration::from_secs(1)).await;
    add_x(state.clone(), 4);
    let s = state.lock().unwrap();
    info!("r4 done with: {:?}", s.data.get("value").unwrap());
    Ok(Code::Success)
}

async fn r5(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    info!("r5 processing...");
	time::sleep(time::Duration::from_secs(1)).await;
    add_x(state.clone(), 5);
    let s = state.lock().unwrap();
    info!("r5 done with: {:?}", s.data.get("value").unwrap());
    // Ok(FrtCode::Success)
    Ok(Code::Failure(1, Some("Whoo".to_string())))
}

async fn r6(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    info!("r6 processing...");
	time::sleep(time::Duration::from_secs(1)).await;
    add_x(state.clone(), 6);
    let s = state.lock().unwrap();
    info!("r6 done with: {:?}", s.data.get("value").unwrap());
    Ok(Code::Success)
}



async fn sleeper(id: i32) {
    info!("{}: Sleeping", id);
	time::sleep(time::Duration::from_secs(1)).await;
    info!("{}:Awake!", id);
}


async fn run() {
    sleeper(0).await;

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

    // let flow_config = r#"
    // {
    //     "ExampleTask1": {
    //         "init_step": "r1",
    //         "config": {
    //             "r1": { "edges": []}
    //         }
    //     }
    // }
    // "#.to_string();

    let func_map = vec![
        ("r1".to_string(), box_async_fn(r1)),
        ("r2".to_string(), box_async_fn(r2)),
        ("r3".to_string(), box_async_fn(r3)),
        ("r4".to_string(), box_async_fn(r4)),
        ("r5".to_string(), box_async_fn(r5)),
        ("r6".to_string(), box_async_fn(r6)),
    ];
    let f = Freactor::new(func_map, flow_config); // ownership moved by design

    let shared_vecs: Vec<Arc<Mutex<State>>> = vec![
        Arc::new(Mutex::new(State{data: vec![("value".to_string(), json!(0))].into_iter().collect(), meta: None})),
        // multi instance concurrently
        Arc::new(Mutex::new(State{data: vec![("value".to_string(), json!(1))].into_iter().collect(), meta: None})),
        Arc::new(Mutex::new(State{data: vec![("value".to_string(), json!(2))].into_iter().collect(), meta: None})),
        Arc::new(Mutex::new(State{data: vec![("value".to_string(), json!(3))].into_iter().collect(), meta: None})),
        Arc::new(Mutex::new(State{data: vec![("value".to_string(), json!(4))].into_iter().collect(), meta: None})),
        Arc::new(Mutex::new(State{data: vec![("value".to_string(), json!(5))].into_iter().collect(), meta: None})),
        Arc::new(Mutex::new(State{data: vec![("value".to_string(), json!(6))].into_iter().collect(), meta: None})),
        Arc::new(Mutex::new(State{data: vec![("value".to_string(), json!(7))].into_iter().collect(), meta: None})),
        Arc::new(Mutex::new(State{data: vec![("value".to_string(), json!(8))].into_iter().collect(), meta: None})),
        Arc::new(Mutex::new(State{data: vec![("value".to_string(), json!(9))].into_iter().collect(), meta: None})),
    ];

    let mut jset = JoinSet::new();
    // concurrent
    for v in shared_vecs.clone() {
        // clone flowconfig for every spawn/req
        let fc = f.clone();
        jset.spawn(async move {
            let _r =
                fc.run("ExampleTask1", v).await;
        });
    }
    while let Some(_res) = jset.join_next().await {
        // info!("{:?}", res);
    }

    // print outcome
    for v in shared_vecs {
        let vec = v.lock().unwrap();
        println!("Mutated Vec: {:?}", *vec);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // env_logger::init();
    tracing_subscriber::fmt::init();

    info!("Hello, world!");
    info!("{}", version());

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build().unwrap();

	runtime.block_on(async move {
		run().await;
	});

    Ok(())
}
