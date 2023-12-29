use aksim::{calculate, Banner, Float};

fn main() {
    divan::main();
}

#[divan::bench]
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

    calculate(&banners, pulls)
}

#[divan::bench]
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

    calculate(&banners, pulls)
}
