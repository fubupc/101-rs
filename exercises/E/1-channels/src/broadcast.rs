use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    task::{Poll, Waker},
};

use futures::Stream;

struct Inner<T> {
    buffer: VecDeque<T>,
    deleted_msg_count: usize,
    txs_left: usize,
    next_rx_id: usize,
    rxs: HashMap<usize, Option<Waker>>,
}

pub struct Sender<T> {
    inner: Arc<Mutex<Inner<T>>>,
}

pub struct Receiver<T> {
    rx_id: usize,
    next_msg_idx: usize, // index into `Inner.buffer`
    inner: Arc<Mutex<Inner<T>>>,
}

#[derive(Debug)]
pub enum SendError<T> {
    ReceiverDropped(T),
}

impl<T: Clone> Inner<T> {
    fn get_msg(&self, msg_idx: usize) -> Option<T> {
        if msg_idx - self.deleted_msg_count >= self.buffer.len() {
            return None;
        }
        Some(self.buffer[msg_idx - self.deleted_msg_count].clone())
    }
}

impl<T> Inner<T> {
    fn set_waker(&mut self, rx_id: usize, waker: Waker) {
        self.rxs.insert(rx_id, Some(waker));
    }

    // create a new receiver, returns (new receiver id, next message index)
    fn new_rx(&mut self) -> (usize, usize) {
        let rx_id = self.next_rx_id;
        self.next_rx_id += 1;
        self.rxs.insert(rx_id, None);
        (rx_id, self.deleted_msg_count)
    }

    fn delete_rx(&mut self, rx_id: usize) {
        self.rxs.remove(&rx_id);
    }

    fn wake_rxs(&self) {
        for (_, waker) in self.rxs.iter() {
            if let Some(waker) = waker {
                waker.wake_by_ref();
            }
        }
    }
}

impl<T> Sender<T> {
    fn send(&self, value: T) -> Result<(), SendError<T>> {
        let mut inner = self.inner.lock().unwrap();
        if inner.rxs.len() == 0 {
            return Err(SendError::ReceiverDropped(value));
        }

        inner.buffer.push_back(value);
        for (_, waker) in inner.rxs.iter() {
            if let Some(waker) = waker {
                waker.wake_by_ref();
            }
        }
        Ok(())
    }
}

impl<T: Clone> Stream for Receiver<T> {
    type Item = T;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut inner = self.inner.lock().unwrap();

        let next_msg = inner.get_msg(self.next_msg_idx);
        match next_msg {
            Some(v) => {
                drop(inner); // lifetime hack
                self.next_msg_idx += 1;
                Poll::Ready(Some(v))
            }
            None => {
                if inner.txs_left == 0 {
                    Poll::Ready(None)
                } else {
                    inner.set_waker(self.rx_id, cx.waker().clone());
                    Poll::Pending
                }
            }
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.inner.lock().unwrap().txs_left += 1;
        Sender {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        let (rx_id, next_msg_idx) = self.inner.lock().unwrap().new_rx();
        Receiver {
            rx_id,
            next_msg_idx,
            inner: self.inner.clone(),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.txs_left -= 1;
        if inner.txs_left > 0 {
            return;
        }
        inner.wake_rxs()
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.inner.lock().unwrap().delete_rx(self.rx_id);
    }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(Mutex::new(Inner {
        buffer: VecDeque::new(),
        deleted_msg_count: 0,
        next_rx_id: 1,
        txs_left: 1,
        rxs: HashMap::from([(0, None)]),
    }));

    let tx = Sender {
        inner: inner.clone(),
    };
    let rx = Receiver {
        rx_id: 0,
        next_msg_idx: 0,
        inner: inner.clone(),
    };
    (tx, rx)
}

#[cfg(test)]
mod tests {
    use futures::{future, StreamExt};
    use tokio::task::{self, JoinHandle};

    use crate::broadcast::{channel, SendError};

    #[tokio::test]
    async fn test_send_recv() {
        let (tx, mut rx) = channel();
        for i in 0..3 {
            println!("send #{i}");
            tx.send(i).unwrap();
        }
        for i in 0..3 {
            println!("receive #{i}");
            assert_eq!(rx.next().await.unwrap(), i);
        }
    }

    #[tokio::test]
    async fn test_drop() {
        let (tx, mut rx) = channel::<()>();
        drop(tx);
        assert!(rx.next().await.is_none());

        let (tx, rx) = channel::<()>();
        drop(rx);
        assert!(matches!(tx.send(()), Err(SendError::ReceiverDropped(()))));
    }

    #[tokio::test]
    async fn test_multiple_tx_rx() {
        let (tx, rx) = channel();

        let mut handles: Vec<JoinHandle<()>> = Vec::new();
        for i in 1..=3 {
            let tx = tx.clone();
            handles.push(task::spawn(async move {
                for j in 0..=1 {
                    let x = 10 * i + j;
                    println!("tx[#{i}]: {x}");
                    tx.send(x).unwrap();
                }
            }));
        }
        drop(tx);

        for i in 0..2 {
            let mut rx = rx.clone();
            handles.push(task::spawn(async move {
                while let Some(msg) = rx.next().await {
                    println!("rx[#{i}]: {}", msg);
                }
            }));
        }
        drop(rx);

        future::join_all(handles).await;
    }
}
