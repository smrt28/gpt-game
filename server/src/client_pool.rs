use std::sync::{Arc, Mutex as StdMutex};

use tokio::sync::Notify;

use crate::{app_error::AppError, config};

pub trait PollableClientFactory<Client>: Send + Sync {
    fn build_client(&self) -> Client;
    fn get_config(&self) -> &config::Gpt;
}

pub type Factory<Client> = Arc<dyn PollableClientFactory<Client> + Send + Sync>;

struct ClientsStorage<Client> {
    clients: Vec<Arc<Client>>,
    clients_total: u32,
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
    factory: Factory<Client>,
}


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
        self.client.is_some()
    }
    pub async fn update(&mut self) -> Result<(), AppError> {
        if self.client.is_some() {
            return Ok(());
        }
        loop {
            self.notify.notified().await;
            self.client = self.pool.raw_client();
            if self.has_client() {
                return Ok(());
            }
        }
    }
}

impl<Client> Drop for ClientGuard<Client> {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            self.pool.return_client(client);
        }
    }
}

impl<Client> ClientsPool<Client> {
    pub fn new(factory: Factory<Client>) -> Self {
        Self {
            storage: StdMutex::new(ClientsStorage::<Client>::new()),
            factory,
        }
    }

    pub fn pop(self: &Arc<Self>) -> ClientGuard<Client> {
        let config = self.factory.get_config();
        let mut storage = self.storage.lock().unwrap();
        if storage.clients.is_empty() {
            if storage.clients_total >= config.max_clients_count {
                return ClientGuard {
                    client: None,
                    pool: Arc::clone(self),
                    notify: storage.notify.clone(),
                };
            }
            storage.clients_total += 1;
            println!("Creating client {}", storage.clients_total);
            ClientGuard {
                client: Some(self.create_client()),
                pool: Arc::clone(self),
                notify: storage.notify.clone(),
            }
        } else {
            println!("Using free client; there are {}", storage.clients.len());
            let client = storage.clients.pop().unwrap();
            ClientGuard { 
                client: Some(client), 
                pool: Arc::clone(self), 
                notify: storage.notify.clone(),
            }
        }
    }

    pub fn return_client(&self, client: Arc<Client>) {
        if let Ok(mut storage) = self.storage.lock() {
            storage.clients.push(client);
            storage.notify.notify_one();
        }
    }
    fn raw_client(&self) -> Option<Arc<Client>> {
        self.storage.lock().ok()?.clients.pop()
    }

    fn create_client(&self) -> Arc<Client> {
        Arc::new(self.factory.build_client())
    }
}
