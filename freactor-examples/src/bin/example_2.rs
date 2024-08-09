use std::error::Error;
use tokio::time;
use tracing::{info, debug};
use tracing_subscriber;
use std::sync::{Arc, Mutex};
use std::thread;
use serde_json::{Value, json};

use axum::{
    extract::Extension,
    routing::get,
    Router,
};

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
    debug!("{} done with: {:?}, [{:?}]", fn_name, s.data.get("value").unwrap(), thread::current().id());
}

async fn _fetch_my_ip() -> Result<(), Box<dyn Error + Send>>{
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
    debug!("r1 processing...{:?}", thread::current().id());
	time::sleep(time::Duration::from_secs(1)).await;

    // fetch_my_ip().await?;
    add_x(state.clone(), 1);
    print_state(state, "r1");

    Ok(Code::Success)
    // Ok(Code::Retry(Some("What?".to_string())))
    // Ok(Code::Failure(1, Some("Whoo".to_string())))
}


async fn r2(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    debug!("r2 processing...{:?}", thread::current().id());
	time::sleep(time::Duration::from_secs(1)).await;

    add_x(state.clone(), 2);
    print_state(state, "r2");

    Ok(Code::Success)
    // Ok(Code::Failure(1, Some("Whoo".to_string())))
}

async fn r3(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    debug!("r3 processing...{:?}", thread::current().id());
	// time::sleep(time::Duration::from_secs(1)).await;

    // fetch_my_ip().await?;
    add_x(state.clone(), 3);
    print_state(state, "r3");

    Ok(Code::Success)
}

async fn r4(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    debug!("r4 processing...{:?}", thread::current().id());
	// time::sleep(time::Duration::from_secs(1)).await;

    add_x(state.clone(), 4);
    print_state(state, "r4");

    Ok(Code::Success)
}

async fn r5(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    debug!("r5 processing...{:?}", thread::current().id());
	// time::sleep(time::Duration::from_secs(1)).await;

    add_x(state.clone(), 5);
    print_state(state, "r5");

    // Ok(Code::Success)
    Ok(Code::Failure(1, Some("Whoo".to_string())))
}

async fn r6(state: Arc<Mutex<State>>) -> Result<Code, Box<dyn Error + Send>> {
    debug!("r6 processing...{:?}", thread::current().id());
	// time::sleep(time::Duration::from_secs(1)).await;

    add_x(state.clone(), 6);
    print_state(state, "r6");

    Ok(Code::Success)
}


async fn _sleeper(id: i32) {
    info!("{}: Sleeping", id);
	time::sleep(time::Duration::from_secs(1)).await;
    info!("{}:Awake!", id);
}


async fn root(Extension(f): Extension<Arc<Freactor>>) -> &'static str {
    debug!("request comes!");
    let v = Arc::new(Mutex::new(State::new(
        vec![("value".to_string(), json!(0))].into_iter().collect()
    )));
    let _ = f.run("Task1", v).await;
    "Hello, World!"
}

async fn handle_task_1(Extension(f): Extension<Arc<Freactor>>) -> &'static str {
    debug!("request comes!");
    let v = Arc::new(Mutex::new(State::new(
        vec![("value".to_string(), json!(0))].into_iter().collect()
    )));
    let _ = f.run("Task1", v).await;
    "Hello, World!"
}

async fn handle_task_2(Extension(f): Extension<Arc<Freactor>>) -> &'static str {
    debug!("request comes!");
    let v = Arc::new(Mutex::new(State::new(
        vec![("value".to_string(), json!(0))].into_iter().collect()
    )));
    let _ = f.run("Task2", v).await;
    "Hello, World!"
}


async fn run() {
    let flow_config = r#"
    {
        "Task1": {
            "init_step": "r1",
            "config": {
                "r1": { "edges": ["r2", "r3", "r4"]},
                "r2": { "edges": ["r5", "r3"]},
                "r3": { "edges": ["r6", "r4"]},
                "r4": { "edges": []},
                "r5": { "edges": ["r6", "r3"]},
                "r6": { "edges": [], "retry": null}
            }
        },
        "Task2": {
            "init_step": "r1",
            "config": {
                "r1": { "edges": ["r2", "r3", "r4"]},
                "r2": { "edges": ["r5", "r3"]},
                "r3": { "edges": ["r4", "r6"]},
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

    let shared_server_state = Arc::new(f);
    let app = Router::new()
    // `GET /` goes to `root`
    .route("/", get(root))
    .route("/1", get(handle_task_1))
    .route("/2", get(handle_task_2))
    .layer(Extension(shared_server_state));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

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
