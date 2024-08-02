//
#[cfg(feature = "tracing")]
use tracing::{info, warn, error, debug};

#[cfg(not(feature = "tracing"))]
use log::{info, warn, error, debug};

use serde_json::Value;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};


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


#[derive(Debug)]
pub struct State {
    pub data: HashMap<String, Value>,
    pub meta: Option<HashMap<String, Value>>,    // TODO meta, retry, path, current, id
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
    pub retry: Option<RetryConfig>,
}

#[derive(Deserialize, Debug)]
pub struct RetryConfig {
    // interval: u32,
    // max_retries: u32,
    // exp_backoff: bool,
}

#[derive(Deserialize, Debug)]
pub struct TaskConfig {
    pub config: HashMap<String, StepConfig>,
    pub init_step: String,
}

pub type FlowConfig = HashMap<String, TaskConfig>;


/* Executor */

#[derive(Clone)]
pub struct Freactor {
    pub funcs: Arc<HashMap<String, BoxAsyncFn>>, // mapping from func name to func, never change once created
    pub flow_config: Arc<FlowConfig>,   // step config that define the flow transfer like a graph, never change
}

impl Freactor {
    pub fn new(func_map: Vec<(String, BoxAsyncFn)>, flow_config_str: String) -> Self {
        let funcs: HashMap<String, BoxAsyncFn> = func_map.iter().cloned().collect();
        let flow_config: FlowConfig = serde_json::from_str(&flow_config_str).unwrap();
        Self {
            funcs: Arc::new(funcs),
            flow_config: Arc::new(flow_config)
        }
    }

    pub async fn run(&self, task_name: &str, states: Arc<Mutex<State>>) -> Result<(), Box<dyn Error + Send>> {
        let tc = match self.flow_config.get(task_name) {
            Some(v) => {
                info!("start task: {}", task_name);
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
        loop {
            debug!("current step: {}", current_step_name);
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
        info!("finish task: {}", task_name);
        Ok(())
    }
}


/*  */
pub fn version() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let mut features = vec![];
    if cfg!(feature = "tracing") {
        features.push("tracing".to_string());
    }
    let v = format!("version: {}, features: {:?}", version, features);
    debug!("features: {}", v);
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
