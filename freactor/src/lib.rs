use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::Value;
use serde::Deserialize;

#[cfg(feature = "tracing")]
use tracing::{info, warn, error, debug};

#[cfg(not(feature = "tracing"))]
use log::{info, warn, error, debug};

#[cfg(feature = "runtime-tokio")]
use tokio::time::sleep;

#[cfg(feature = "runtime-async-std")]
use async_std::task::sleep;


async fn async_sleep(seconds: u64) {
    #[cfg(feature = "runtime-tokio")]
    {
        debug!("using tokio sleep!");
        sleep(std::time::Duration::from_secs(seconds)).await;
    }
    #[cfg(feature = "runtime-async-std")]
    {
        debug!("using async-std sleep!");
        sleep(std::time::Duration::from_secs(seconds)).await;
    }
    #[cfg(not(any(feature = "runtime-tokio", feature = "runtime-async-std")))]
    {
        eprintln!("Async runtime feature, i.e. \"runtime-tokio\" should be enabled! Thread sleep is for test purpose only!");
        std::thread::sleep(std::time::Duration::from_secs(seconds));
    }
}


// Define a type alias for a boxed future with a specific lifetime
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;


/* Async Function */

#[derive(Debug)]
pub enum Code {
    Success,
    Failure(usize, Option<String>),
    Retry(Option<String>),
    Cancel(Option<String>),
}


// #[derive(Debug)]
pub struct State {
    pub data: HashMap<String, Value>,
    meta: Meta,
}


#[derive(Debug)]
pub struct Meta {
    pub id: u64,
    pub task: String,
    pub path: Vec<(String, u32)>,
}


impl State {
    pub fn new(data: HashMap<String, Value>) -> Self {
        State {
            data: data,
            meta: Meta { id: 0, task: "unspecified".to_string(), path: vec![] },
        }
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path_string: String = self.meta.path
            .iter()
            .map(|(s, n)| if *n > 0 { format!("{}@{}", s, n) } else { s.clone() })
            .collect::<Vec<String>>()
            .join(" -> ");
        let meta_string = format!("{}#{}: {}", self.meta.task, self.meta.id, path_string);
        f.debug_struct("Task State")
            .field("data", &self.data)
            .field("meta", &meta_string)
            .finish()
    }
}


// A pointer of funture-boxed async function with specific input state and output code for state transfer.
type BoxAsyncFn = Arc<dyn Fn(Arc<Mutex<State>>) -> BoxFuture<'static, Result<Code, Box<dyn Error + Send>>> + Send + Sync>;


pub fn box_async_fn<F, Fut>(f: F) -> BoxAsyncFn
where
    F: Fn(Arc<Mutex<State>>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Code, Box<dyn Error + Send>>> + Send + 'static,
{
    Arc::new(move |state| Box::pin(f(state)))
}


/* Step Config */

#[derive(Deserialize, Debug)]
pub struct StepConfig {
    pub edges: Vec<String>,
    retry: Option<RetryPolicy>,
}

#[derive(Deserialize, Debug)]
struct RetryPolicy {
    attempts: u32,
    interval: u64,
    strategy: Option<String>,   // like `exp_backoff`
}

#[derive(Deserialize, Debug)]
pub struct TaskConfig {
    pub config: HashMap<String, StepConfig>,
    pub init_step: String,
}

pub type FlowConfig = HashMap<String, TaskConfig>;


/* Executor */

const DEFAULT_RETRY_ATTEMPTS: u32 = 3;  // max retries
const DEFAULT_RETRY_INTERVAL: u64 = 2;  // seconds for retry cooldown


#[derive(Clone)]
pub struct Freactor {
    pub funcs: Arc<HashMap<String, BoxAsyncFn>>, // mapping from func name to func, never change once created
    pub flow_config: Arc<FlowConfig>,   // step config that define the flow transfer like a graph, never change
    id_counter: Arc<AtomicU64>,
}

impl Freactor {
    pub fn new(func_map: Vec<(String, BoxAsyncFn)>, flow_config_str: String) -> Self {
        let funcs: HashMap<String, BoxAsyncFn> = func_map.iter().cloned().collect();
        let flow_config: FlowConfig = serde_json::from_str(&flow_config_str).unwrap();
        Self {
            funcs: Arc::new(funcs),
            flow_config: Arc::new(flow_config),
            id_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn run(&self, task_name: &str, states: Arc<Mutex<State>>) -> Result<(), Box<dyn Error + Send>> {
        let tc = match self.flow_config.get(task_name) {
            Some(v) => {
                debug!("start task: {}", task_name);
                v
            }
            None => {
                error!("no such task: {}", task_name);
                return Ok(())
            }
        };
        let v = states;
        let mut current_step_name = &tc.init_step;
        let mut current_step = self.funcs.get(current_step_name).unwrap().clone();
        let mut retrying = false;
        {
            let mut v = v.lock().unwrap();
            v.meta.id = self.next_id();
            v.meta.task = task_name.to_string();
        }
        loop {
            debug!("current step: {}", current_step_name);
            debug!("state: {:?}", v);
            // update meta
            {
                let mut v = v.lock().unwrap();
                if retrying == false {
                    v.meta.path.push((current_step_name.to_string(), 0));
                }
                retrying = false;
                debug!("current path: {:?}", v.meta.path);
            }
            // run current step and dispatch with code
            match current_step(v.clone()).await {
                Ok(Code::Success) => {
                    debug!("#Success!");
                    let next_step_name = tc.config.get(current_step_name).unwrap().edges.get(0);
                    match next_step_name {
                        Some(next_step_name) => {
                            current_step_name = next_step_name;
                            current_step = self.funcs.get(current_step_name).unwrap().clone();
                        },
                        None => break
                    }
                }
                Ok(Code::Failure(code , msg)) => {
                    debug!("#Failue! {}, {}", code, msg.unwrap());
                    let next_step_name = tc.config.get(current_step_name).unwrap().edges.get(code);
                    match next_step_name {
                        Some(next_step_name) => {
                            current_step_name = next_step_name;
                            current_step = self.funcs.get(current_step_name).unwrap().clone();
                        },
                        None => break
                    }
                }
                Ok(Code::Retry(msg)) => {
                    debug!("#Retry! {}", msg.unwrap());
                    // get retry policy of current step from config
                    let current_retry_policy = &tc.config.get(current_step_name).unwrap().retry;
                    let attempts = current_retry_policy.as_ref().map_or(DEFAULT_RETRY_ATTEMPTS, |r| r.attempts);
                    let interval = current_retry_policy.as_ref().map_or(DEFAULT_RETRY_INTERVAL, |r| r.interval);
                    let strategy: Option<String> = current_retry_policy.as_ref().map_or(None, |r| r.strategy.clone());
                    debug!("retry policy with attempts {} time(s) and interval {} second(s) with strategy {:?}", attempts, interval, &strategy);
                    let retried :u32;   // nubmer of (already) retried times
                    // read meta, abort if reach max retry times, increment retried counter
                    {
                        let mut v = v.lock().unwrap();
                        let len = v.meta.path.len();
                        retried = v.meta.path[len-1].1;
                        debug!("step `{}` has already retried {} time(s).", current_step_name, retried);
                        if retried >= attempts {
                            debug!("step `{}` has already retried {} time(s). abort!", current_step_name, retried);
                            break;
                        }
                        v.meta.path[len-1].1 = retried + 1;
                    }
                    // calculate final retry interval for async sleeping
                    let retry_interval = match strategy {
                        Some(s) => { if s == "exp_backoff" { interval * (1 << retried) } else { interval } },
                        None => interval,
                    };
                    debug!("retrying after {} second(s).", retry_interval);
                    async_sleep(retry_interval).await;
                    retrying = true;
                }
                Ok(Code::Cancel(msg)) => {
                    debug!("#Cancel! {}", msg.unwrap());
                    break;
                }
                Err(e) => {
                    warn!("Unknown error occured, Abort! {}", e.to_string());
                    break;
                }
            }
        }
        debug!("finish task: {}", task_name);
        Ok(())
    }

    fn next_id(&self) -> u64 {
        self.id_counter.fetch_add(1, Ordering::SeqCst)
    }
}


impl fmt::Debug for Freactor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Freactor")
            .field("flow_config", &self.flow_config)
            .finish()
    }
}


/*  */
pub fn version() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let mut features = vec![];
    if cfg!(feature = "tracing") {
        features.push("tracing".to_string());
    }
    if cfg!(feature = "runtime-tokio") {
        features.push("runtime-tokio".to_string());
    }
    if cfg!(feature = "runtime-async-std") {
        features.push("runtime-async-std".to_string());
    }
    let v = format!("version: {}, features: {:?}", version, features);
    info!("features: {}", v);
    v
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn sub(left: usize, right: usize) -> usize {
    left - right
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_sub() {
        let result = sub(2, 2);
        assert_eq!(result, 0);
    }

}
