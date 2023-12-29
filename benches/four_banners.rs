use aksim::{calculate, Banner, Float};
use divan::{bench, black_box, AllocProfiler};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

#[bench]
fn target_1() -> Float {
    let banners = [
        Banner {
            target: 1,
            subrate: 0.35,
            bonus_pulls: 24,
        },
        Banner {
            target: 1,
            subrate: 0.5,
            bonus_pulls: 0,
        },
        Banner {
            target: 1,
            subrate: 0.2,
            bonus_pulls: 48,
        },
        Banner {
            target: 1,
            subrate: 0.4,
            bonus_pulls: 0,
        },
    ];

    let pulls = 400;

    calculate(black_box(&banners), black_box(pulls))
}

#[bench]
fn target_10() -> Float {
    let banners = [
        Banner {
            target: 10,
            subrate: 0.35,
            bonus_pulls: 240,
        },
        Banner {
            target: 10,
            subrate: 0.5,
            bonus_pulls: 0,
        },
        Banner {
            target: 10,
            subrate: 0.2,
            bonus_pulls: 480,
        },
        Banner {
            target: 10,
            subrate: 0.4,
            bonus_pulls: 0,
        },
    ];

    let pulls = 4000;

    calculate(black_box(&banners), black_box(pulls))
}
