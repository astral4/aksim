#![feature(generic_const_exprs)]

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
    [(); TARGET + 1]:, // generic const exprs need a `where` bound
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

#[allow(clippy::needless_range_loop)]
fn main() {
    const PULLS: usize = 166;
    const FREE_PULLS: usize = 24;

    let pdist_1 = banner::<1, { PULLS + FREE_PULLS - 1 }>(0.35);

    let pdist_2 = banner::<1, { PULLS - 1 }>(0.5);

    let mut prob = pdist_1[..FREE_PULLS].iter().sum::<Float>() * pdist_2.iter().sum::<Float>();

    for i2 in 0..(PULLS - 1) {
        for i1 in FREE_PULLS..(PULLS + FREE_PULLS - 1 - i2) {
            prob += pdist_1[i1] * pdist_2[i2];
        }
    }

    println!("Probability: {prob}");
}
