#![feature(generic_const_exprs)]

use core::any::Any;

type Float = f32;

#[rustfmt::skip]
const SIX_STAR_RATES: [Float; 99] = [
    0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02,
    0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02,
    0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02,
    0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02,
    0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02,
    0.04, 0.06, 0.08, 0.10, 0.12, 0.14, 0.16, 0.18, 0.20, 0.22,
    0.24, 0.26, 0.28, 0.30, 0.32, 0.34, 0.36, 0.38, 0.40, 0.42,
    0.44, 0.46, 0.48, 0.50, 0.52, 0.54, 0.56, 0.58, 0.60, 0.62,
    0.64, 0.66, 0.68, 0.70, 0.72, 0.74, 0.76, 0.78, 0.80, 0.82,
    0.84, 0.86, 0.88, 0.90, 0.92, 0.94, 0.96, 0.98, 1.00,
];

fn banner<const TARGET: usize, const PULLS: usize>(subrate: Float) -> [Float; PULLS]
where
    [(); TARGET + 1]: Any, // ?
{
    let mut pdist = [0.; PULLS];

    let mut probs = [[0.; 100]; { TARGET + 1 }];
    probs[0][0] = 1.;

    let mut temp_probs = [[0.; 100]; { TARGET + 1 }];

    for prob in &mut pdist {
        for (pity, rate) in SIX_STAR_RATES.iter().enumerate() {
            for target_count in 0..TARGET {
                let prob = probs[target_count][pity];

                temp_probs[target_count][pity + 1] += prob * (1. - rate);
                temp_probs[target_count][0] += prob * rate * (1. - subrate);
                temp_probs[target_count + 1][0] += prob * rate * subrate;
            }
        }
        probs = temp_probs;
        *prob = probs[TARGET][0];
        temp_probs = [[0.; 100]; { TARGET + 1 }];
    }

    pdist
}

fn main() {
    let pdist = banner::<1, 100>(0.5);
    let cumsum: Vec<Float> = pdist
        .into_iter()
        .scan(0., |acc, x| {
            *acc += x;
            Some(*acc)
        })
        .collect();

    println!("{pdist:?}");
    println!(
        "prob of getting pot1 rateup 6* on event banner in 57 pulls: {}",
        cumsum[56]
    );
}
