use core::iter::zip;
use realfft::num_complex::Complex;
use realfft::RealFftPlanner;

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

fn banner_pdist(target: usize, pulls: usize, subrate: Float) -> Vec<Float> {
    let mut pdist = Vec::with_capacity(pulls);

    let mut probs = vec![[0.; 100]; target + 1];
    probs[0][0] = 1.;

    let mut temp_probs = vec![[0.; 100]; target + 1];

    for _ in 0..pulls {
        for (pity_count, rate) in SIX_STAR_RATES.iter().enumerate() {
            for target_count in 0..target {
                let prob = probs[target_count][pity_count];

                temp_probs[target_count][pity_count + 1] += prob * (1. - rate);
                temp_probs[target_count][0] += prob * rate * (1. - subrate);
                temp_probs[target_count + 1][0] += prob * rate * subrate;
            }
        }

        probs = temp_probs;
        pdist.push(probs[target][0]);
        temp_probs = vec![[0.; 100]; target + 1];
    }

    pdist
}

#[allow(clippy::cast_precision_loss, clippy::similar_names)]
fn calculate(banners: &[Banner], pulls: usize) -> Float {
    let mut conv_size = 0;
    let mut pdists = Vec::with_capacity(banners.len());
    let mut total_bonus_pulls = 0;

    for banner in banners {
        // at least 1 pull needs to be spent on each banner, so for any single banner,
        // we subtract the number of other banners to calculate `max_pulls`
        let max_pulls = pulls + banner.bonus_pulls - (banners.len() - 1);

        let pdist = banner_pdist(banner.target, max_pulls, banner.subrate);

        conv_size += max_pulls;
        pdists.push(pdist);
        total_bonus_pulls += banner.bonus_pulls;
    }

    // initialize FFT calculator
    let mut planner = RealFftPlanner::<Float>::new();
    let fft = planner.plan_fft_forward(conv_size);

    // the complex multiplication identity is 1 + 0i
    let mut combined_dft = vec![Complex::new(1., 0.); conv_size / 2 + 1];

    for mut pdist in pdists {
        // vectors are padded with 0s to calculate a "full" convolution
        pdist.resize_with(conv_size, Default::default);
        // apply FFT to probability distribution
        let mut dft = fft.make_output_vec();
        fft.process(&mut pdist, &mut dft).unwrap();

        // multiply the DFTs together
        for (sample, combined_sample) in zip(dft, &mut combined_dft) {
            *combined_sample *= sample;
        }
    }

    // apply IFFT to get the convolved distribution
    // the result needs to be divided by `conv_size` to get the actual convolution values
    let ifft = planner.plan_fft_inverse(conv_size);
    let mut combined_seq = ifft.make_output_vec();
    ifft.process(&mut combined_dft, &mut combined_seq).unwrap();

    // calculate final probability
    combined_seq
        .into_iter()
        .take(pulls + total_bonus_pulls - (banners.len() - 1))
        .sum::<Float>()
        / (conv_size as Float)
}

struct Banner {
    target: usize,
    subrate: Float,
    bonus_pulls: usize,
}

fn main() {
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

    let prob = calculate(&banners, pulls);

    println!("Probability: {prob}");
}
