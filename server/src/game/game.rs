use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time;

use fetris_protocol::{ClientRequest, ServerRequest};

use crate::game::players::Player;
use crate::game::pools::{self, Pool, PoolId, PoolState};
use crate::network::{NetworkAction, NetworkPacket, StreamList};

pub fn game_main_thread(stream_list: StreamList, receiver: Receiver<NetworkPacket>) {
    let mut players: HashMap<SocketAddr, Player> = HashMap::new();
    let mut pools: HashMap<PoolId, Pool> = HashMap::new();
    let mut pending_pool: HashMap<SocketAddr, ()> = HashMap::new();

    loop {
        thread::sleep(time::Duration::from_millis(10));
        for (_, pool) in pools.iter_mut() {
            pool.update();
        }

        for packet in receiver.try_iter() {
            if let Err(_) = match packet.action {
                NetworkAction::OpenStream => {
                    players.insert(packet.addr, Player::new());
                    println!("{} opened stream", packet.addr);
                    Ok(())
                }
                NetworkAction::CloseStream => {
                    let player = players.get(&packet.addr).unwrap();

                    if let PoolState::Pool(pool_id) = player.pool() {
                        let pool = pools.get_mut(&pool_id).unwrap();

                        pool.remove_user(&packet.addr);
                        if pool.len() == 0 {
                            pools.remove(&pool_id);
                        }
                    } else if player.pool() == PoolState::PendingPool {
                        pending_pool.remove(&packet.addr);
                    }
                    players.remove(&packet.addr);
                    println!("{} closed stream", packet.addr);
                    Ok(())
                }
                NetworkAction::Request(ClientRequest::SetName(name)) => {
                    let player = players.get_mut(&packet.addr).unwrap();

                    if player.pool() != PoolState::None {
                        Err(())
                    } else {
                        player.set_name(name);
                        Ok(())
                    }
                }
                NetworkAction::Request(ClientRequest::AskForAGame) => {
                    let player = players.get_mut(&packet.addr).unwrap();

                    if player.pool() != PoolState::None {
                        Err(())
                    } else {
                        player.change_pool(PoolState::PendingPool);
                        pending_pool.insert(packet.addr, ());
                        if pending_pool.len() == pools::POOL_SIZE {
                            let (id, pool) = Pool::create(&mut players, &stream_list, pending_pool);
                            pools.insert(id, pool);
                            pending_pool = HashMap::new();
                        }
                        Ok(())
                    }
                }
                NetworkAction::Request(ClientRequest::Input(input)) => {
                    let player = players.get_mut(&packet.addr).unwrap();

                    if let PoolState::Pool(id) = player.pool() {
                        let pool = pools.get_mut(&id).unwrap();

                        pool.handle_player_input(&packet.addr, input);
                    }
                    Ok(())
                }
            } {
                let _ = stream_list.send_to(&packet.addr, ServerRequest::BadRequest);
            }
        }
    }
}
