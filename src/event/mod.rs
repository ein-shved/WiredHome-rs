pub trait Event {
    type Data;
    fn initial(&mut self) -> Option<Self::Data> {
        None
    }
    async fn next(&mut self) -> Self::Data;
}

pub trait Handler {
    type Data;
    async fn handle(&mut self, data: Self::Data);
}

pub struct Connection<Ev: Event, Hn: Handler> {
    event: Ev,
    handler: Hn,
}

impl<Ev, Hn, EvData, HnData> Connection<Ev, Hn>
where
    Ev: Event<Data = EvData>,
    Hn: Handler<Data = HnData>,
    HnData: From<EvData>,
{
    pub fn new(event: Ev, handler: Hn) -> Self {
        Self { event, handler }
    }
    pub async fn run(&mut self) {
        if let Some(data) = self.event.initial() {
            self.handler.handle(data.into()).await
        }
        loop {
            let data = self.event.next().await;
            self.handler.handle(data.into()).await
        }
    }
}
