/*
 * counter - A proof of work based DoS protection system
 * Copyright © 2021 Aravinth Manivannan <realravinth@batsense.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * al
 */
use std::time::Duration;

use actix::clock::sleep;
use actix::dev::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Count {
    pub search_threshold: u32,
    pub duration: u64,
}

impl Count {
    /// increments the search count by one
    #[inline]
    pub fn add_search(&mut self) {
        self.search_threshold += 1;
    }

    /// decrements the search count by specified count
    #[inline]
    pub fn decrement_search_by(&mut self, count: u32) {
        if self.search_threshold > 0 {
            if self.search_threshold >= count {
                self.search_threshold -= count;
            } else {
                self.search_threshold = 0;
            }
        }
    }

    /// get [Counter]'s current search_threshold
    #[inline]
    pub fn get_searches(&self) -> u32 {
        self.search_threshold
    }
}

/// This struct represents the counter state and is used
/// to configure leaky-bucket lifetime and manage defense
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Counter(Count);

impl From<Count> for Counter {
    fn from(c: Count) -> Counter {
        Counter(c)
    }
}
impl Actor for Counter {
    type Context = Context<Self>;
}

/// Message to decrement the search count
#[derive(Message)]
#[rtype(result = "()")]
struct DeleteSearch;

impl Handler<DeleteSearch> for Counter {
    type Result = ();
    fn handle(&mut self, _msg: DeleteSearch, _ctx: &mut Self::Context) -> Self::Result {
        self.0.decrement_search_by(1);
    }
}

/// Message to increment the search count
/// returns difficulty factor and lifetime
#[derive(Message)]
#[rtype(result = "u32")]
pub struct AddSearch;

impl Handler<AddSearch> for Counter {
    type Result = MessageResult<AddSearch>;

    fn handle(&mut self, _: AddSearch, ctx: &mut Self::Context) -> Self::Result {
        self.0.add_search();
        let addr = ctx.address();

        let duration: Duration = Duration::new(self.0.duration, 0);
        let wait_for = async move {
            sleep(duration).await;
            //delay_for(duration).await;
            addr.send(DeleteSearch).await.unwrap();
        }
        .into_actor(self);
        ctx.spawn(wait_for);

        MessageResult(self.0.get_searches())
    }
}

/// Message to get the search count
#[derive(Message)]
#[rtype(result = "u32")]
pub struct GetCurrentSearchCount;

impl Handler<GetCurrentSearchCount> for Counter {
    type Result = MessageResult<GetCurrentSearchCount>;

    fn handle(&mut self, _: GetCurrentSearchCount, _ctx: &mut Self::Context) -> Self::Result {
        MessageResult(self.0.get_searches())
    }
}

/// Message to stop [Counter]
#[derive(Message)]
#[rtype(result = "()")]
pub struct Stop;

impl Handler<Stop> for Counter {
    type Result = ();

    fn handle(&mut self, _: Stop, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    // constants for testing
    // (search count, level)
    pub const LEVEL_1: (u32, u32) = (50, 50);
    pub const LEVEL_2: (u32, u32) = (500, 500);
    pub const DURATION: u64 = 5;

    type MyActor = Addr<Counter>;

    async fn race(addr: Addr<Counter>, count: (u32, u32)) {
        for _ in 0..count.0 as usize - 1 {
            let _ = addr.send(AddSearch).await.unwrap();
        }
    }

    pub fn get_counter() -> Counter {
        Counter(Count {
            duration: DURATION,
            search_threshold: 0,
        })
    }

    #[test]
    fn counter_decrement_by_works() {
        let mut m = get_counter();
        for _ in 0..100 {
            m.0.add_search();
        }
        assert_eq!(m.0.get_searches(), 100);
        m.0.decrement_search_by(50);
        assert_eq!(m.0.get_searches(), 50);
        m.0.decrement_search_by(500);
        assert_eq!(m.0.get_searches(), 0);
    }

    #[actix_rt::test]
    async fn get_current_search_count_works() {
        let addr: MyActor = get_counter().start();

        addr.send(AddSearch).await.unwrap();
        addr.send(AddSearch).await.unwrap();
        addr.send(AddSearch).await.unwrap();
        addr.send(AddSearch).await.unwrap();
        let count = addr.send(GetCurrentSearchCount).await.unwrap();

        assert_eq!(count, 4);
    }

    #[actix_rt::test]
    async fn counter_defense_loosenup_works() {
        let addr: MyActor = get_counter().start();

        race(addr.clone(), LEVEL_2).await;
        addr.send(AddSearch).await.unwrap();
        assert_eq!(addr.send(GetCurrentSearchCount).await.unwrap(), LEVEL_2.1);

        let duration = Duration::new(DURATION + 1, 0);
        sleep(duration).await;
        //delay_for(duration).await;

        addr.send(AddSearch).await.unwrap();
        let count = addr.send(GetCurrentSearchCount).await.unwrap();
        assert_eq!(count, 1);
    }

    #[actix_rt::test]
    #[should_panic]
    async fn stop_works() {
        let addr: MyActor = get_counter().start();
        addr.send(Stop).await.unwrap();
        addr.send(AddSearch).await.unwrap();
    }
}
