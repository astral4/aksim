use aksim::{calculate, Banner, Float};
use divan::{bench, black_box};

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
    ];

    let pulls = 170;

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
    ];

    let pulls = 1700;

    calculate(black_box(&banners), black_box(pulls))
}
