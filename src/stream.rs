use crate::{import::*, RingBuffer};

impl<T: Copy> Stream for RingBuffer<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(read) = self.consumer.pop() {
            Poll::Ready(Some(read))
        } else if self.closed {
            Poll::Ready(None)
        } else {
            self.read_waker.replace(cx.waker().clone());
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        import::{assert_eq, *},
        RingBuffer,
    };
    use futures::StreamExt;

    #[test]
    fn stream() {
        block_on(async {
            let mut ring = RingBuffer::<u8>::new(2);

            // create a full buffer
            //
            ring.producer.push(b'a').expect("write");
            ring.producer.push(b'b').expect("write");

            // read 1
            //
            assert_eq!(StreamExt::next(&mut ring).await, Some(b'a'));

            assert!(!ring.is_empty());
            assert!(!ring.is_full());

            assert_eq!(ring.len(), 1);
            assert_eq!(ring.remaining(), 1);

            assert!(ring.read_waker.is_none());
            assert!(ring.write_waker.is_none());

            // read 2
            //
            assert_eq!(StreamExt::next(&mut ring).await, Some(b'b'));

            assert!(ring.is_empty());
            assert!(!ring.is_full());

            assert_eq!(ring.len(), 0);
            assert_eq!(ring.remaining(), 2);

            assert!(ring.read_waker.is_none());
            assert!(ring.write_waker.is_none());

            // read 3
            //
            let (waker, count) = new_count_waker();
            let mut cx = Context::from_waker(&waker);

            {
                let pinned_ring = Pin::new(&mut ring);
                assert!(pinned_ring.poll_next(&mut cx).is_pending());
            }
            assert!(ring.is_empty());
            assert!(!ring.is_full());

            assert_eq!(ring.len(), 0);
            assert_eq!(ring.remaining(), 2);

            assert!(ring.read_waker.is_some());
            assert!(ring.write_waker.is_none());

            // Write one back, verify read_waker get's woken up and we can read again
            //
            let arr = [b'c'];

            AsyncWriteExt::write(&mut ring, &arr).await.expect("write");

            assert!(!ring.is_empty());
            assert!(!ring.is_full());

            assert_eq!(ring.len(), 1);
            assert_eq!(ring.remaining(), 1);

            assert!(ring.read_waker.is_none());
            assert_eq!(count, 1);

            {
                let pinned_ring = Pin::new(&mut ring);
                assert_eq!(pinned_ring.poll_next(&mut cx), Poll::Ready(Some(b'c')));
            }
            assert!(ring.is_empty());
            assert!(!ring.is_full());

            assert_eq!(ring.len(), 0);
            assert_eq!(ring.remaining(), 2);
        })
    }

    #[test]
    fn closed_next() {
        block_on(async {
            let mut ring = RingBuffer::<u8>::new(2);
            let arr = [b'a'];

            AsyncWriteExt::write(&mut ring, &arr).await.expect("write");
            ring.close().await.unwrap();

            assert_eq!(StreamExt::next(&mut ring).await, Some(b'a'));
            assert_eq!(StreamExt::next(&mut ring).await, None);

            // try read again, just in case
            //
            assert_eq!(StreamExt::next(&mut ring).await, None);
        })
    }
}
