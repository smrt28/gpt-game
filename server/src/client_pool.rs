#![allow(dead_code)]

use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::sync::futures::Notified;
use crate::gpt::GptClient;
use tokio::sync::Notify;

#[derive(Clone, Default)]
pub struct ClientFactoryConfig {
    pub max_clients: i32,
}

pub trait PollableClientFactory<Client> : Send + Sync {
    fn build_client(&self) -> Client;
    fn get_config(&self) -> &ClientFactoryConfig;
}

pub type Factory<Client> =
    Arc<dyn PollableClientFactory<Client> + Send + Sync>;



struct ClientsStorage<Client> {
    clients: Vec<Arc<Client>>,
    clients_total: i32,
    notify: Arc<Notify>,
}

impl<Client> ClientsStorage<Client> {
    fn new() -> Self {
        Self {
            clients: Vec::new(),
            clients_total: 0,
            notify: Arc::new(Notify::new()),
        }
    }
}


pub struct ClientsPool<Client> {
    storage: StdMutex<ClientsStorage<Client>>,
    factory: Arc<dyn PollableClientFactory<Client> + Send + Sync>,
}

#[derive(Clone)]
pub struct ClientGuard<Client> {
    client: Option<Arc<Client>>,
    pool: Arc<ClientsPool<Client>>,
    notify: Arc<Notify>,
}


impl<Client> ClientGuard<Client> {
    pub fn client(&self) -> &Client {
        self.client.as_ref().unwrap().as_ref()
    }
    pub fn has_client(&self) -> bool {
        if self.client.is_none() {
            return false;
        }
        true
    }
    pub async fn update(&mut self) -> bool {
        if !self.client.is_none() {
            return true;
        }
        self.notify.notified().await;
        self.client = self.pool.raw_client();
        self.has_client()
    }
}

impl<Client> Drop for ClientGuard<Client>
{
     fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            self.pool.return_client(client);
        }
    }
}

impl<Client> ClientsPool<Client> {
    pub fn new(factory: Arc<dyn PollableClientFactory<Client> + Send + Sync>) -> Self {
        Self {
            storage: StdMutex::new(ClientsStorage::<Client>::new()),
            factory: factory,
        }
    }

    pub fn pop(self: &Arc<Self>) -> ClientGuard<Client> {
        let config = self.factory.get_config();
        let mut storage = self.storage.lock().unwrap();
        if storage.clients.len() == 0 {
            if storage.clients_total >= config.max_clients {
                return ClientGuard {
                    client: None,
                    pool: Arc::clone(self),
                    notify: storage.notify.clone(),
                }
            }
            storage.clients_total += 1;
            println!("creating client {}", storage.clients_total);
            return ClientGuard {
                client: Some(Arc::new(self.factory.build_client())),
                pool: Arc::clone(self),
                notify: storage.notify.clone(),
            };
        } else {
            println!("using free client; there are {}", storage.clients.len());
        }
        let client = storage.clients.pop().unwrap();
        ClientGuard { client: Some(client), pool: Arc::clone(self), notify: storage.notify.clone(), }
    }

    pub fn return_client(&self, client: Arc<Client>) {
        if let Ok(mut storage) = self.storage.lock() {
            storage.clients.push(client);
            storage.notify.notify_one();
        }
    }
    fn raw_client(&self) -> Option<Arc<Client>> {
        None
    }
}
