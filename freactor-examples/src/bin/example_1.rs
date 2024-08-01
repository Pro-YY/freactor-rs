use std::error::Error;
use tokio::time;
use tracing::info;
use tracing_subscriber;
use std::sync::{Arc, Mutex};
use tokio::task::JoinSet;
use serde_json::{Value, json};
use std::collections::HashMap;

use reqwest;

use freactor::{
    Code,
    State,
    box_async_fn,
    StepConfig,
    TaskConfig,
    FlowConfig,
    Freactor,
    version,
};


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

    let v;
    {
        let mut s = state.lock().unwrap();
        s.id += 1;
        v = s.id;
    }
    info!("r1 done! {}", v);
    Ok(Code::Success)
    // Ok(FrtCode::Retry(Some("What?".to_string())))
    // Ok(FrtCode::Failure(1, Some("Whoo".to_string())))
}

async fn r2(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    info!("r2 processing...");
	time::sleep(time::Duration::from_secs(1)).await;
    let mut s = state.lock().unwrap();
    s.id += 2;
    info!("r2 done! {}", s.id);
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

    let mut s = state.lock().unwrap();
    s.id += 3;
    info!("r3 done! {}", s.id);
    Ok(Code::Success)
}

async fn r4(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    info!("r4 processing...");
	time::sleep(time::Duration::from_secs(1)).await;
    let mut s = state.lock().unwrap();
    s.id += 4;
    info!("r4 done! {}", s.id);
    Ok(Code::Success)
}

async fn r5(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    info!("r5 processing...");
	time::sleep(time::Duration::from_secs(1)).await;
    let mut s = state.lock().unwrap();
    s.id += 5;
    info!("r5 done! {}", s.id);
    // Ok(FrtCode::Success)
    Ok(Code::Failure(1, Some("Whoo".to_string())))
}

async fn r6(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    info!("r6 processing...");
	time::sleep(time::Duration::from_secs(1)).await;
    let mut s = state.lock().unwrap();
    s.id += 6;
    info!("r6 done! {}", s.id);
    Ok(Code::Success)
}



async fn sleeper(id: i32) {
    info!("{}: Sleeping", id);
	time::sleep(time::Duration::from_secs(1)).await;
    info!("{}:Awake!", id);
}


async fn run() {
    sleeper(0).await;

    // TODO loadfrom config
    let mut flow_config: FlowConfig = HashMap::new();
    flow_config.insert("ExampleTask1".to_string(), TaskConfig { init_step: "r1".to_string(), config: HashMap::new()});
    for (k, v) in &flow_config {
        info!("{}:starts with #{}",k, v.init_step);
    }

    let tc = flow_config.get_mut("ExampleTask1").unwrap();
    // let scm = &mut tc.config;
    tc.config.insert("r1".to_string(),
        StepConfig {edges: vec!["r2".to_string(), "r3".to_string(), "r4".to_string()], retry: None});
    tc.config.insert("r2".to_string(),
        StepConfig {edges: vec!["r5".to_string(), "r3".to_string()], retry: None});
    tc.config.insert("r3".to_string(),
        StepConfig {edges: vec!["r6".to_string(), "r4".to_string()], retry: None});
    tc.config.insert("r4".to_string(),
        StepConfig {edges: vec![], retry: None});
    tc.config.insert("r5".to_string(),
        StepConfig {edges: vec!["r6".to_string(), "r3".to_string()], retry: None});
    tc.config.insert("r6".to_string(),
        StepConfig {edges: vec![], retry: None});


    let scm = &tc.config;
    for (k, v) in scm.into_iter() {
        info!("{}, {:?}", k, v.edges);
    }

    let funcs: HashMap<String, _> = [
        ("r1".to_string(), box_async_fn(r1)),
        ("r2".to_string(), box_async_fn(r2)),
        ("r3".to_string(), box_async_fn(r3)),
        ("r4".to_string(), box_async_fn(r4)),
        ("r5".to_string(), box_async_fn(r5)),
        ("r6".to_string(), box_async_fn(r6)),
    ].iter().cloned().collect();
    let f = Freactor::new(funcs, flow_config); // move by design

    let shared_vecs: Vec<Arc<Mutex<State>>> = vec![
        Arc::new(Mutex::new(State{id: 0, data: vec![
            ("k1".to_string(), json!(10)),
            // ("k2".to_string(), json!("abc")),
            // ("k3".to_string(), json!({"array": [1,2,3], "object": {"a": 1, "b": "hello"}})),
        ].into_iter().collect(), ctx: None})),
        // multi instance concurrently
        Arc::new(Mutex::new(State{id: 1, data: HashMap::new(), ctx: None})),
        Arc::new(Mutex::new(State{id: 2, data: HashMap::new(), ctx: None})),
        Arc::new(Mutex::new(State{id: 3, data: HashMap::new(), ctx: None})),
        // Arc::new(Mutex::new(State{id: 4, data: HashMap::new(), ctx: None})),
        // Arc::new(Mutex::new(State{id: 5, data: HashMap::new(), ctx: None})),
        // Arc::new(Mutex::new(State{id: 6, data: HashMap::new(), ctx: None})),
        // Arc::new(Mutex::new(State{id: 7, data: HashMap::new(), ctx: None})),
        // Arc::new(Mutex::new(State{id: 8, data: HashMap::new(), ctx: None})),
        // Arc::new(Mutex::new(State{id: 9, data: HashMap::new(), ctx: None})),
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
    while let Some(res) = jset.join_next().await {
        info!("{:?}", res);
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
