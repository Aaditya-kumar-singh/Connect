use axum::extract::ws::Message;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ConnectionMetadata {
    pub connection_id: Uuid,
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub device_id: Uuid,
}

#[derive(Clone)]
pub struct ConnectionHandle {
    pub metadata: ConnectionMetadata,
    pub sender: mpsc::Sender<Message>,
}

#[derive(Clone, Default)]
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<Uuid, ConnectionHandle>>>,
}

impl ConnectionManager {
    pub async fn insert(&self, handle: ConnectionHandle) -> bool {
        self.connections
            .write()
            .await
            .insert(handle.metadata.connection_id, handle)
            .is_none()
    }

    pub async fn remove(&self, connection_id: Uuid) -> Option<ConnectionHandle> {
        self.connections.write().await.remove(&connection_id)
    }

    pub async fn count(&self) -> usize {
        self.connections.read().await.len()
    }

    pub async fn user_connection_count(&self, user_id: Uuid) -> usize {
        self.connections
            .read()
            .await
            .values()
            .filter(|connection| connection.metadata.user_id == user_id)
            .count()
    }

    pub async fn contains(&self, connection_id: Uuid) -> bool {
        self.connections.read().await.contains_key(&connection_id)
    }

    pub async fn send_to_user(
        &self,
        user_id: Uuid,
        message: Message,
        exclude_connection: Option<Uuid>,
    ) -> usize {
        let handles: Vec<ConnectionHandle> = self
            .connections
            .read()
            .await
            .values()
            .filter(|connection| {
                connection.metadata.user_id == user_id
                    && Some(connection.metadata.connection_id) != exclude_connection
            })
            .cloned()
            .collect();

        let mut sent = 0;
        for handle in handles {
            if handle.sender.send(message.clone()).await.is_ok() {
                sent += 1;
            }
        }
        sent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tracks_multiple_connections_for_same_user() {
        let manager = ConnectionManager::default();
        let user_id = Uuid::new_v4();
        for _ in 0..2 {
            let (sender, _receiver) = mpsc::channel(4);
            manager
                .insert(ConnectionHandle {
                    metadata: ConnectionMetadata {
                        connection_id: Uuid::new_v4(),
                        user_id,
                        session_id: Uuid::new_v4(),
                        device_id: Uuid::new_v4(),
                    },
                    sender,
                })
                .await;
        }
        assert_eq!(manager.count().await, 2);
        assert_eq!(manager.user_connection_count(user_id).await, 2);
    }

    #[tokio::test]
    async fn sends_to_all_user_connections_except_selected_connection() {
        let manager = ConnectionManager::default();
        let user_id = Uuid::new_v4();
        let first_id = Uuid::new_v4();
        let (first_tx, mut first_rx) = mpsc::channel(4);
        let (second_tx, mut second_rx) = mpsc::channel(4);
        manager
            .insert(ConnectionHandle {
                metadata: ConnectionMetadata {
                    connection_id: first_id,
                    user_id,
                    session_id: Uuid::new_v4(),
                    device_id: Uuid::new_v4(),
                },
                sender: first_tx,
            })
            .await;
        manager
            .insert(ConnectionHandle {
                metadata: ConnectionMetadata {
                    connection_id: Uuid::new_v4(),
                    user_id,
                    session_id: Uuid::new_v4(),
                    device_id: Uuid::new_v4(),
                },
                sender: second_tx,
            })
            .await;

        let sent = manager
            .send_to_user(user_id, Message::Text("hello".into()), Some(first_id))
            .await;
        assert_eq!(sent, 1);
        assert!(first_rx.try_recv().is_err());
        assert!(second_rx.try_recv().is_ok());
    }
}
