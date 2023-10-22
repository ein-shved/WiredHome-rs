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

pub trait Connection {
    async fn run(&mut self);
}

pub struct ConnectionConsistent<Ev: Event, Hn: Handler> {
    event: Ev,
    handler: Hn,
}

impl<Ev, Hn, EvData, HnData> Connection for ConnectionConsistent<Ev, Hn>
where
    Ev: Event<Data = EvData>,
    Hn: Handler<Data = HnData>,
    HnData: From<EvData>,
{
    async fn run(&mut self) {
        if let Some(data) = self.event.initial() {
            self.handler.handle(data.into()).await
        }
        loop {
            let data = self.event.next().await;
            self.handler.handle(data.into()).await
        }
    }
}

impl<Ev, Hn> ConnectionConsistent<Ev, Hn>
where
    Ev: Event,
    Hn: Handler,
{
    pub fn new(event: Ev, handler: Hn) -> Self {
        Self { event, handler }
    }
}

pub struct ConnectionInterrupting<Ev: Event, Hn: Handler> {
    event: Ev,
    handler: Hn,
}

impl<Ev, Hn, EvData, HnData> Connection for ConnectionInterrupting<Ev, Hn>
where
    Ev: Event<Data = EvData>,
    Hn: Handler<Data = HnData>,
    HnData: From<EvData>,
{
    async fn run(&mut self) {
        use embassy_futures::select;
        let (event, handler) = (&mut self.event, &mut self.handler);
        let mut data: Option<EvData> = event.initial();
        loop {
            data = if let Some(data) = data {
                match select::select(event.next(), handler.handle(data.into())).await {
                    select::Either::First(data) => Some(data),
                    select::Either::Second(_) => None,
                }
            } else {
                Some(event.next().await)
            }
        }
    }
}
impl<Ev, Hn> ConnectionInterrupting<Ev, Hn>
where
    Ev: Event,
    Hn: Handler,
{
    pub fn new(event: Ev, handler: Hn) -> Self {
        Self { event, handler }
    }
}

pub struct JoinEvent<Ev1: Event, Ev2: Event> {
    ev1: Ev1,
    ev2: Ev2,
}

impl<Ev1Data, Ev2Data, Ev1, Ev2> Event for JoinEvent<Ev1, Ev2>
where
    Ev1: Event<Data = Ev1Data>,
    Ev2: Event<Data = Ev2Data>,
    Ev1Data: From<Ev2Data>,
{
    type Data = Ev1Data;
    fn initial(&mut self) -> Option<Self::Data> {
        let mut data = self.ev1.initial();
        if data.is_none() {
            if let Some(d2) = self.ev2.initial() {
                data = Some(d2.into())
            }
        }
        data
    }
    async fn next(&mut self) -> Self::Data {
        use embassy_futures::select;
        use select::{select, Either};
        match select(self.ev1.next(), self.ev2.next()).await {
            Either::First(data) => data,
            Either::Second(data) => data.into(),
        }
    }
}

impl<Ev1, Ev2> JoinEvent<Ev1, Ev2>
where
    Ev1: Event,
    Ev2: Event,
{
    pub fn new(ev1: Ev1, ev2: Ev2) -> Self {
        Self { ev1, ev2 }
    }
}

pub struct JoinHandler<Hn1: Handler, Hn2: Handler> {
    ev1: Hn1,
    ev2: Hn2,
}

impl<Hn1Data, Hn2Data, Hn1, Hn2> Handler for JoinHandler<Hn1, Hn2>
where
    Hn1: Handler<Data = Hn1Data>,
    Hn2: Handler<Data = Hn2Data>,
    Hn2Data: From<Hn1Data>,
    Hn1Data: Copy,
{
    type Data = Hn1Data;
    async fn handle(&mut self, data: Self::Data) {
        self.ev1.handle(data).await;
        self.ev2.handle(data.into()).await
    }
}

impl<Hn1, Hn2> JoinHandler<Hn1, Hn2>
where
    Hn1: Handler,
    Hn2: Handler,
{
    pub fn new(ev1: Hn1, ev2: Hn2) -> Self {
        Self { ev1, ev2 }
    }
}

//struct MultiHandler<Hn: Handler> {
//    hn: Hn,
//}
//
//enum MultiData<Data1, Data2> {
//    D1(Data1),
//    D2(Data2),
//}
//
//impl<Data1, Hn> Handler for MultiHandler<Hn>
//where
//    Hn: Handler<Data = Data1> + Handler<Data = bool>,
//{
//    type Data = MultiData<Data1, bool>;
//
//    async fn handle(&mut self, data: Self::Data) {}
//}
