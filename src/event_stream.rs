use std::pin::Pin;
use std::task::{Poll, Context};

use web_sys::{Event, EventTarget};
use gloo::events::{EventListener, EventListenerOptions};
use futures::channel::mpsc::{self, Receiver};
use futures::stream::Stream;
use futures::sink::SinkExt;
use wasm_bindgen_futures::spawn_local;

pub(crate) struct EventStream<T> {
    queue: Receiver<(T, Event)>,
    _listeners: Vec<EventListener>,
}

impl<T> EventStream<T> where T: Clone + 'static {
    pub(crate) fn new(desc: &[(&EventTarget, T, &'static str)]) -> Self {
        let (tx, rx) = mpsc::channel(32);
        let mut listeners = vec![];
        for (target, token, event) in desc {
            let token = token.clone();
            let tx = tx.clone();
            let opt = EventListenerOptions::enable_prevent_default();
            let listener = EventListener::new_with_options(target, *event, opt, move |event| {
                let item = (token.clone(), event.clone());
                let mut tx = tx.clone();
                spawn_local(async move {
                    tx.send(item).await.unwrap();
                });
            });
            listeners.push(listener);
        }
        Self {
            queue: rx,
            _listeners: listeners,
        }
    }
}

impl<T> Stream for EventStream<T> {
    type Item = (T, Event);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Self { ref mut queue, .. } = self.get_mut();
        let polled = Pin::new(queue).poll_next(cx);
        match polled {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some((token, event))) => Poll::Ready(Some((token, event))),
            Poll::Ready(None) => Poll::Ready(None),
        }
    }
}
