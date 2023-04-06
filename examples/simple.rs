#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]
use anyhow::Result;
use async_trait::async_trait;
use event_bus::worker::IdentityOfWorker;
use event_bus::{Bus, CopyOfBus};
use log::debug;
use std::any::{Any, TypeId};
use std::time::Duration;
use tokio::spawn;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    custom_utils::logger::logger_stdout_debug();
    let copy_of_bus = Bus::init();
    WorkerA::init(&copy_of_bus).await;
    WorkerB::init(&copy_of_bus).await;
    sleep(Duration::from_secs(1)).await;
    sleep(Duration::from_secs(5)).await
}

#[derive(Debug)]
struct AEvent;
#[derive(Debug)]
struct BEvent;

struct WorkerA {
    identity: IdentityOfWorker,
}

#[async_trait]
trait Worker {
    async fn login(bus: &CopyOfBus) -> Result<IdentityOfWorker> {
        bus.login().await
    }

    fn identity(&self) -> &IdentityOfWorker;

    fn subscribe(&self, type_id: TypeId) -> Result<()> {
        self.identity().subscribe(type_id)
    }

    fn dispatch_event<T: Any + Send + Sync + 'static>(&mut self, event: T) -> Result<()> {
        let identity = self.identity();
        identity.dispatch_event(event)
    }
}

impl Worker for WorkerA {
    fn identity(&self) -> &IdentityOfWorker {
        &self.identity
    }
}

impl WorkerA {
    pub async fn init(bus: &CopyOfBus) {
        let identity = <WorkerA as Worker>::login(bus).await.unwrap();
        Self { identity }.run();
    }
    fn run(mut self) {
        spawn(async move {
            self.subscribe(AEvent.type_id()).unwrap();
            while let Some(event) = self.identity.recv_event().await {
                debug!("WorkerA recv {:?}", event.as_ref().type_id());
                if let Some(a) = event.as_ref().downcast_ref::<AEvent>() {
                    debug!("WorkerA recv {:?}", a);
                }
            }
        });
    }
}

struct WorkerB {
    identity: IdentityOfWorker,
}

impl Worker for WorkerB {
    fn identity(&self) -> &IdentityOfWorker {
        &self.identity
    }
}

impl WorkerB {
    pub async fn init(bus: &CopyOfBus) {
        let identity = bus.login().await.unwrap();
        Self { identity }.run();
    }

    fn run(mut self) {
        spawn(async move {
            self.subscribe(BEvent.type_id()).unwrap();
            while let Some(event) = self.identity.recv_event().await {
                debug!("WorkerB recv {:?}", event.as_ref().type_id());
                if let Some(a) = event.as_ref().downcast_ref::<BEvent>() {
                    debug!("WorkerB recv {:?}", a);
                }
            }
        });
    }
}
