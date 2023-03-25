/*
 * ForgeFlux StarChart - A federated software forge spider
 * Copyright © 2022 Aravinth Manivannan <realaravinth@batsense.net>
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
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

pub(crate) fn get_random(len: usize) -> String {
    use rand::{distributions::Alphanumeric, rngs::ThreadRng, thread_rng, Rng};
    use std::iter;

    let mut rng: ThreadRng = thread_rng();

    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(len)
        .collect::<String>()
}

pub(crate) fn get_random_number(upper_limit: i64) -> usize {
    use rand::{rngs::ThreadRng, thread_rng, Rng};

    let mut rng: ThreadRng = thread_rng();

    rng.gen_range(0..upper_limit as u32) as usize
}

pub struct TextProcessor {
    txt: String,
    count: i64,
}

impl TextProcessor {
    pub fn new(txt: String) -> Self {
        let lines = txt.lines();
        let count = lines.clone().count() as i64;
        Self { txt, count }
    }

    pub(crate) fn get_random_shakespeare(&self) -> &str {
        let lines = self.txt.lines();
        lines.skip(get_random_number(self.count)).next().unwrap()
    }
}
