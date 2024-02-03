use std::{
    future::Future,
    pin::{self, Pin},
    task::{Context, Poll},
};

struct Join<'a, T1, T2, F1: Future<Output = T1>, F2: Future<Output = T2>> {
    future1: Pin<&'a mut F1>,
    future2: Pin<&'a mut F2>,
    result1: Option<T1>,
    result2: Option<T2>,
}

impl<'a, T1, T2, F1: Future<Output = T1>, F2: Future<Output = T2>> Join<'a, T1, T2, F1, F2> {
    fn new(future1: Pin<&'a mut F1>, future2: Pin<&'a mut F2>) -> Self {
        Self {
            future1,
            future2,
            result1: None,
            result2: None,
        }
    }
}

impl<T1, T2, F1: Future<Output = T1>, F2: Future<Output = T2>> Unpin for Join<'_, T1, T2, F1, F2> {}

impl<T1, T2, F1: Future<Output = T1>, F2: Future<Output = T2>> Future for Join<'_, T1, T2, F1, F2> {
    type Output = (T1, T2);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.result1.is_none() {
            if let Poll::Ready(result1) = self.future1.as_mut().poll(cx) {
                self.result1 = Some(result1);
            }
        };
        if self.result2.is_none() {
            if let Poll::Ready(result2) = self.future2.as_mut().poll(cx) {
                self.result2 = Some(result2);
            }
        }
        if self.result1.is_some() && self.result2.is_some() {
            Poll::Ready((self.result1.take().unwrap(), self.result2.take().unwrap()))
        } else {
            Poll::Pending
        }
    }
}

pub async fn join2<T1, T2>(
    mut future1: impl Future<Output = T1>,
    mut future2: impl Future<Output = T2>,
) -> (T1, T2) {
    Join::new(pin::pin!(future1), pin::pin!(future2)).await
}

pub async fn join3<T1, T2, T3>(
    mut future1: impl Future<Output = T1>,
    mut future2: impl Future<Output = T2>,
    mut future3: impl Future<Output = T3>,
) -> (T1, T2, T3) {
    let ((result1, result2), result3) = Join::new(
        pin::pin!(Join::new(pin::pin!(future1), pin::pin!(future2))),
        pin::pin!(future3),
    )
    .await;
    (result1, result2, result3)
}

pub async fn join4<T1, T2, T3, T4>(
    mut future1: impl Future<Output = T1>,
    mut future2: impl Future<Output = T2>,
    mut future3: impl Future<Output = T3>,
    mut future4: impl Future<Output = T4>,
) -> (T1, T2, T3, T4) {
    let ((result1, result2), (result3, result4)) = Join::new(
        pin::pin!(Join::new(pin::pin!(future1), pin::pin!(future2),)),
        pin::pin!(Join::new(pin::pin!(future3), pin::pin!(future4))),
    )
    .await;
    (result1, result2, result3, result4)
}

pub async fn join5<T1, T2, T3, T4, T5>(
    mut future1: impl Future<Output = T1>,
    mut future2: impl Future<Output = T2>,
    mut future3: impl Future<Output = T3>,
    mut future4: impl Future<Output = T4>,
    mut future5: impl Future<Output = T5>,
) -> (T1, T2, T3, T4, T5) {
    let (((result1, result2), (result3, result4)), result5) = Join::new(
        pin::pin!(Join::new(
            pin::pin!(Join::new(pin::pin!(future1), pin::pin!(future2))),
            pin::pin!(Join::new(pin::pin!(future3), pin::pin!(future4))),
        )),
        pin::pin!(future5),
    )
    .await;
    (result1, result2, result3, result4, result5)
}

pub async fn join6<T1, T2, T3, T4, T5, T6>(
    mut future1: impl Future<Output = T1>,
    mut future2: impl Future<Output = T2>,
    mut future3: impl Future<Output = T3>,
    mut future4: impl Future<Output = T4>,
    mut future5: impl Future<Output = T5>,
    mut future6: impl Future<Output = T6>,
) -> (T1, T2, T3, T4, T5, T6) {
    let (((result1, result2), (result3, result4)), (result5, result6)) = Join::new(
        pin::pin!(Join::new(
            pin::pin!(Join::new(pin::pin!(future1), pin::pin!(future2))),
            pin::pin!(Join::new(pin::pin!(future3), pin::pin!(future4))),
        )),
        pin::pin!(Join::new(pin::pin!(future5), pin::pin!(future6))),
    )
    .await;
    (result1, result2, result3, result4, result5, result6)
}

pub async fn join7<T1, T2, T3, T4, T5, T6, T7>(
    mut future1: impl Future<Output = T1>,
    mut future2: impl Future<Output = T2>,
    mut future3: impl Future<Output = T3>,
    mut future4: impl Future<Output = T4>,
    mut future5: impl Future<Output = T5>,
    mut future6: impl Future<Output = T6>,
    mut future7: impl Future<Output = T7>,
) -> (T1, T2, T3, T4, T5, T6, T7) {
    let (((result1, result2), (result3, result4)), ((result5, result6), result7)) =
        pin::pin!(Join::new(
            pin::pin!(Join::new(
                pin::pin!(Join::new(pin::pin!(future1), pin::pin!(future2))),
                pin::pin!(Join::new(pin::pin!(future3), pin::pin!(future4))),
            )),
            pin::pin!(Join::new(
                pin::pin!(Join::new(pin::pin!(future5), pin::pin!(future6))),
                pin::pin!(future7),
            )),
        ))
        .await;
    (
        result1, result2, result3, result4, result5, result6, result7,
    )
}
