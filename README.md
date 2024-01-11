# aksim

`aksim` is a program for calculating pull probability distributions over series of banners in the game Arknights.

## The basics

Arknights has many playable characters. Each character in the game has a rarity ranging from 1★ (lowest) to 6★ (highest).

Arknights is a **gacha game**. One way to obtain characters is by **pulling**/**rolling** on **banners**. Each pull has the following probabilities of obtaining a character of a particular rarity:

| Rarity | Probability |
| :----: | :---------: |
|   1★   |     0%      |
|   2★   |     0%      |
|   3★   |     40%     |
|   4★   |     50%     |
|   5★   |     8%      |
|   6★   |     2%      |

Banners have two important mechanics: **pity** and **rate-up**.

The pity mechanic (for 6★s) works like this: if no 6★ was obtained in the last 50 pulls, the probability of obtaining a 6★ on the next pull increases by 2%. A banner's **pity count** is the number of pulls since the last obtained 6★.

The rate-up mechanic works like this: generally, each banner has one or two "rate-up" 6★s. When someone obtains a 6★ character, the character has a chance of being a rate-up. We'll call this chance the **subrate**. The subrate and number of rate-up 6★s depends on the type of banner:

| Banner type | Number of rate-ups | Subrate (of each rate-up) |
| :---------: | :----------------: | :-----------------------: |
|  standard   |         2          |            25%            |
|    event    |         1          |            50%            |
|   limited   |         2          |            35%            |

Players generally pull on banners with a **target** in mind, such as obtaining a certain number of copies of a specific rate-up 6★ character. Let's call this specific character the **target character**. We'll focus on calculating the probability of achieving the target.

## A single banner

What is the probability of achieving a target within a certain number of pulls? It turns out that calculating this isn't exactly simple or straightforward.

Let's look at an easy case: the target is obtaining one copy of the target character. Let $n$ be the number of pulls and $r_s$ be the subrate. We can express the probability as the output of a function $p$ with inputs $n$ and $r_s$. For the first 50 pulls, the pity mechanic doesn't apply:

$$p(n, r_s) = 1 - (1 - 0.02 \cdot r_s)^n$$

Explanation:

- The probability of pulling a 6★ is $0.02$.
- The probability of pulling a 6★ *and* it being the target character is $0.02 \cdot r_s$.
- The probability of *not* pulling the target character is $1 - 0.02 \cdot r_s$.
    - Since there are only two possible outcomes—the character either is pulled or is not—and they are mutually exclusive, their probabilities must add up to $1$.
- The probability of *not* obtaining the target character in $n$ pulls is $(1 - 0.02 \cdot r_s)^n$.
    - Not obtaining the character in $n$ pulls is the same as not obtaining on pull 1 *and* not obtaining on pull 2 *and* ... *and* not obtaining on pull $n$. The probability of not obtaining the character on each pull is the same: $1 - 0.02 \cdot r_s$.
- The probability of obtaining the target character in $n$ pulls is $1 - (1 - 0.02 \cdot r_s)^n$.
    - In other words, this is the probability of *not* *not* obtaining the character in $n$ pulls. Again, since there are only two possible outcomes—the character either is obtained in $n$ pulls or is not—and they are mutually exclusive, their probabilities must add up to $1$.

However, after the first 50 pulls, the calculation becomes much more complicated. We have to account for the possibility that a non-rate-up 6★ was pulled, resetting the pity mechanic but not counting towards the target. Also, here, the target is only one copy of the target character. What if the target was multiple copies instead?

Clearly, we need a better approach. We'll use computers to crunch numbers from here on out and cover various methods for calculating probabilities.

### The Monte Carlo method

One approach is to simulate a player pulling on a banner until they achieve the target. We can infer useful statistics by running this simulation many times and recording the number of pulls spent for each run. For example, if we run the simulation 100 times and observe that 26 of those runs took 30 pulls or less, then it's reasonable to conclude that the probability of achieving the target within 30 pulls is around $\frac{26}{100}$, or 26%.

We can simulate one pull by generating a random number. The pull result depends on the number generated. Let's assume that we're generating numbers in the range $[0, 1)$, and each number is equally likely to be generated. 

We could say a 6★ is pulled if the number is in the range $[0, 0.02)$. This works because the probability that the number is in the range is $0.02$, and the probability of pulling a 6★ is also $0.02$. Similarly, we can say that the target character is pulled if the number is in the range $[0, 0.02 \cdot r_s)$, where $r_s$ is the subrate. Obviously, the ranges don't specifically have to be $[0, 0.02)$ and $[0, 0.02 \cdot r_s)$, but they're convenient for our purposes.

However, these ranges don't account for the pity mechanism increasing the probability of pulling a 6★. So, our simulation also has to keep track of the pity count and adjust ranges accordingly.

Our simulation must also keep track of the target character count so we know when the target is achieved.

Here's what a computer program for this task might look like:

```rust
// we use the `rand` crate to generate random numbers
use rand::Rng;
use rand::rngs::StdRng;

const RUNS: u32 = 100000; // number of simulation runs
const TARGET: u8 = 1; // target character count needed to achieve the target
const SUBRATE: f32 = 0.5;

fn main() {
    let mut rng = StdRng::from_entropy(); // initialize the random number generator
    let mut pull_counts = Vec::new(); // initialize the list of pull counts for each run

    // repeat the simulation `RUNS` times
    for _ in 0..RUNS {
        let mut pulls = 0;
        let mut target_count = 0;
        let mut pity_count = 0;

        // keep pulling until the target is achieved
        while target_count < TARGET {
            // calculate the probability of pulling a 6★ based on the current pity count
            let six_star_rate = if pity_count < 50 {
                0.02
            } else {
                0.02 * (pity_count - 48) as f32
            };

            pulls += 1;

            let r: f32 = rng.gen(); // generate a random number

            if r >= 0 && r < six_star_rate * SUBRATE {
                // pulled the target character
                target_count += 1;
                pity_count = 0;
            } else if r >= six_star_rate * SUBRATE && r < six_star_rate {
                // pulled a 6★ that wasn't the target character
                pity_count = 0;
            } else if r >= six_star_rate && r < 1 {
                // didn't pull a 6★
                pity_count += 1;
            }
        }

        pull_counts.push(pulls); // record the number of pulls spent
    }

    // analyze `pull_counts` after the simulation is over
}
```