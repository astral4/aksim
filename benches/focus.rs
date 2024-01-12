use aksim::focus::{calculate, Banner};
use aksim::Float;
use divan::{bench, black_box, AllocProfiler};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

#[bench]
fn small() -> Float {
    let banners = [
        Banner {
            target: 1,
            subrate: 0.35,
            bonus_pulls: 24,
            has_focus: false,
        },
        Banner {
            target: 1,
            subrate: 0.5,
            bonus_pulls: 0,
            has_focus: true,
        },
    ];

    let pulls = 170;

    calculate(black_box(&banners), black_box(pulls))
}

#[bench]
fn large() -> Float {
    let banners = [
        Banner {
            target: 10,
            subrate: 0.35,
            bonus_pulls: 240,
            has_focus: false,
        },
        Banner {
            target: 10,
            subrate: 0.5,
            bonus_pulls: 0,
            has_focus: true,
        },
        Banner {
            target: 10,
            subrate: 0.2,
            bonus_pulls: 480,
            has_focus: true,
        },
        Banner {
            target: 10,
            subrate: 0.4,
            bonus_pulls: 0,
            has_focus: false,
        },
    ];

    let pulls = 4000;

    calculate(black_box(&banners), black_box(pulls))
}
