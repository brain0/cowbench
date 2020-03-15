#![feature(test)]

use rand::{
    distributions::{Bernoulli, Distribution},
    SeedableRng,
};
use rand_chacha::ChaChaRng;
use std::{rc::Rc, sync::Arc, time::Instant};

#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(not(feature = "jemalloc"))]
static ALLOC_STR: &str = "default";
#[cfg(feature = "jemalloc")]
static ALLOC_STR: &str = "jemalloc";

fn main() {
    let n = 1_000_000_000;
    let denom = std::env::args().skip(1).next().unwrap().parse().unwrap();

    println!(
        "Running {} rounds with 1/{} clone probability using the {} allocator",
        n, denom, ALLOC_STR
    );

    {
        print!("Clone: ");
        let data = BenchData::new(denom);
        measure(move || bench_clone(n, data));
    }

    {
        print!("Rc: ");
        let data = BenchData::new(denom);
        measure(move || bench_rc(n, data));
    }

    {
        print!("Arc: ");
        let data = BenchData::new(denom);
        measure(move || bench_arc(n, data));
    }
}

fn use_string(s: &String) {
    std::hint::black_box(s);
}

fn use_string_mut(s: &mut String) {
    std::hint::black_box(s);
}

struct RandomBool {
    dist: Bernoulli,
    rng: ChaChaRng,
}

impl RandomBool {
    fn new(denom: u32) -> Self {
        Self {
            dist: Bernoulli::from_ratio(1, denom).unwrap(),
            // Use a fixed seed to get consistent results.
            rng: ChaChaRng::from_seed([
                54, 97, 98, 23, 123, 238, 124, 76, 0, 17, 76, 254, 200, 190, 100, 101, 143, 9, 3,
                237, 100, 102, 200, 0, 17, 38, 90, 74, 174, 12, 74, 9,
            ]),
        }
    }

    fn next_bool(&mut self) -> bool {
        self.dist.sample(&mut self.rng)
    }
}

struct BenchData {
    s: String,
    rng: RandomBool,
}

impl BenchData {
    fn new(denom: u32) -> Self {
        Self {
            s: "dhoqhwoidnwqpdwqdiwqd92132648903ß92uejoiwgfvurew7".into(),
            rng: RandomBool::new(denom),
        }
    }
}

fn measure(f: impl FnOnce()) {
    let start = Instant::now();
    f();
    let end = Instant::now();
    let duration = end.duration_since(start);

    println!("{}ms", duration.as_millis());
}

fn bench_clone(n: usize, data: BenchData) {
    let BenchData { s, mut rng } = data;

    for _ in 0..n {
        bench_clone_inner(s.clone(), rng.next_bool());
    }
}

fn bench_clone_inner(mut s: String, mutate: bool) {
    use_string(&s);
    if mutate {
        use_string_mut(&mut s);
    }
}

fn bench_rc(n: usize, data: BenchData) {
    let BenchData { s, mut rng } = data;
    let s = Rc::new(s);

    for _ in 0..n {
        bench_rc_inner(s.clone(), rng.next_bool());
    }
}

fn bench_rc_inner(mut s: Rc<String>, mutate: bool) {
    use_string(&s);
    if mutate {
        use_string_mut(Rc::make_mut(&mut s));
    }
}

fn bench_arc(n: usize, data: BenchData) {
    let BenchData { s, mut rng } = data;
    let s = Arc::new(s);

    for _ in 0..n {
        bench_arc_inner(s.clone(), rng.next_bool());
    }
}

fn bench_arc_inner(mut s: Arc<String>, mutate: bool) {
    use_string(&s);
    if mutate {
        use_string_mut(Arc::make_mut(&mut s));
    }
}
