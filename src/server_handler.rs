use std::collections::HashMap;
use std::net::SocketAddr;
use std::thread;
use serde_json::{json,Value};
use std::time::{SystemTime,Duration,Instant};
use std::sync::{Arc,RwLock};
use laminar::{Socket,Packet,SocketEvent};

use crate::tiles::Tiles;
use crate::traits::EntityTrait;

pub struct ServerHandler{
    socket: Arc<RwLock<Socket>>,
    sender: crossbeam_channel::Sender<Packet>,          
    receiver: crossbeam_channel::Receiver<SocketEvent>,
    pub clients: Arc<RwLock<HashMap<std::net::SocketAddr, SystemTime>>>,
    pub entities: Arc<RwLock<HashMap<String, Arc<RwLock<dyn EntityTrait>>>>>,
    pub tiles: Arc<RwLock<Tiles>>,
    on_client_connect: Option<Arc<dyn Fn(&mut ServerHandler, &SocketAddr) + Send + Sync + 'static>>,
}

impl ServerHandler{
    pub fn new(socket: Socket) -> ServerHandler{
        let arc_socket = Arc::new(RwLock::new(socket));
        let sender: crossbeam_channel::Sender<Packet> = {
            let s = arc_socket.read().unwrap();
            s.get_packet_sender()
        };
        let receiver: crossbeam_channel::Receiver<SocketEvent> = {
            let s = arc_socket.read().unwrap();
            s.get_event_receiver()
        };

        ServerHandler{
            socket: arc_socket,
            sender: sender,
            receiver: receiver,
            clients: Arc::new(RwLock::new(HashMap::new())),
            entities: Arc::new(RwLock::new(HashMap::new())),
            tiles: Arc::new(RwLock::new(Tiles::new(500, 500, 16))),
            on_client_connect: None,
        }
    }

    pub fn check_clients(&mut self){
        let mut entities: std::sync::RwLockWriteGuard<'_, HashMap<String, _>> = self.entities.write().unwrap();
        let mut clients: std::sync::RwLockWriteGuard<'_, _> = self.clients.write().unwrap();

        let timed_out: Vec<SocketAddr> = clients.iter()
            .filter(|(_, time)| time.elapsed().map_or(true, |e| e.as_secs() >= 4))
            .map(|(addr, _)| *addr)
            .collect();

        for addr in timed_out {
            let key: String = format!("player{}", addr);
            entities.remove(&key);
            clients.remove(&addr);
        }
    }

    fn get_player<'a>(&mut self, src: &SocketAddr) -> Option<Arc<RwLock<dyn EntityTrait>>> {
        let entities: std::sync::RwLockReadGuard<'_, HashMap<String, _>> = self.entities.read().unwrap();
        let key = format!("player{}", src);
        if let Some(entity) = entities.get(&key) {
            Some(Arc::clone(entity))
        } else {
            None
        }
    }

    pub fn set_on_client_connect<F>(&mut self, callback: F) 
    where 
        F: Fn(&mut ServerHandler, &SocketAddr) + Send + Sync + 'static
    {
        self.on_client_connect = Some(Arc::new(callback));
    }

    fn add_client(&mut self, src: &SocketAddr){
        if let Some(callback) = self.on_client_connect.clone() {
            callback(self, src);
        }
        else{
            eprintln!("NO CALLBACK SET!");
        }
    }

    fn disconnect_client(&mut self, src: &SocketAddr){
        let mut entities: std::sync::RwLockWriteGuard<'_, HashMap<String, _>> = self.entities.write().unwrap();
        let key = format!("player{}", src);
        
        if let Some(entity) = entities.get(&key) {
            let entity_lock = entity.read().unwrap();
            let (x, y) = entity_lock.get_position();
            let (tile_x, tile_y) = self.tiles.read().unwrap().world_to_tile_index(x, y);
            self.tiles.write().unwrap().remove_entity(&key, tile_x, tile_y);
        }
        
        entities.remove(&key);
        self.clients.write().unwrap().retain(|client_addr, _| client_addr != src);
    }

    fn update_client(&mut self, src: &SocketAddr){
        let mut clients: std::sync::RwLockWriteGuard<'_, _> = self.clients.write().unwrap();
        if let Some(time) = clients.get_mut(src){
            *time = SystemTime::now();
        }
    }

    pub fn receive_events(&mut self) -> HashMap<SocketAddr, String>{
        let mut unhandled_data: HashMap<SocketAddr, String> = HashMap::new();
        {
            let mut socket = self.socket.write().unwrap();
            socket.manual_poll(Instant::now());
        }

        while let Ok(event) = self.receiver.try_recv() {
            match event {
                SocketEvent::Packet(packet) => {
                    let src = packet.addr();
                    let data = String::from_utf8_lossy(packet.payload()).to_string();
                    let args: Vec<&str> = data.split('|').collect();

                    {
                        let clients_lock = self.clients.read().unwrap();
                        if !clients_lock.contains_key(&src) {
                            drop(clients_lock);
                            self.add_client(&src);
                        }
                    }

                    if args[0] == "PACKETS" {
                        self.update_client(&src);
                    } else if args[0] == "CONNECT" {
                    } else if args[0] == "DISCONNECT" {
                        self.disconnect_client(&src);
                    } else {
                        unhandled_data.insert(src, data);
                    }
                }
                _ => {}
            }
        }

        unhandled_data
    }

    pub fn broadcast(&mut self){
        {
            let entities_lock: std::sync::RwLockReadGuard<'_, HashMap<String, Arc<RwLock<dyn EntityTrait>>>> = self.entities.read().unwrap();
            let clients_lock: std::sync::RwLockReadGuard<'_, HashMap<SocketAddr, SystemTime>> = self.clients.read().unwrap();

            let mut combined: Value = json!({});
            for (name, entity) in entities_lock.iter(){
                let entity_lock = entity.read().unwrap();
                let entity_json: Value = entity_lock.to_json(name);
                if let Value::Object(map) = entity_json {
                    for (k, v) in map {
                        combined[k] = v;
                    }
                }
            }

            let data: String = combined.to_string();
            let msg: String = format!("LIBRARYEVENT|{}", data);
            for client in clients_lock.keys() {
                let packet: Packet = Packet::unreliable(*client, msg.as_bytes().to_vec());
                let _ = self.sender.send(packet);
            }
        }
        {
            let mut socket = self.socket.write().unwrap();
            socket.manual_poll(Instant::now());
        }
    }

    pub fn get_sender(&self) -> crossbeam_channel::Sender<Packet>{
        self.sender.clone()
    }

    pub fn get_entities(&self) -> Arc<RwLock<HashMap<String, Arc<RwLock<dyn EntityTrait>>>>>{
        Arc::clone(&self.entities)
    }

    pub fn get_clients(&self) -> Arc<RwLock<HashMap<std::net::SocketAddr, SystemTime>>>{
        Arc::clone(&self.clients)
    }

    pub fn get_tiles(&self) -> Arc<RwLock<Tiles>>{
        Arc::clone(&self.tiles)
    }
}