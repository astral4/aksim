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

Banners have two mechanics that are important for understanding the rest of this document: **pity** and **rate-up**.

The pity mechanic (for 6★ characters) works like this: if no 6★ was obtained in the last 50 pulls, the probability of obtaining a 6★ on the next pull increases by 2%.

The rate-up mechanic works like this: generally, each banner has one or two "rate-up" 6★ characters. When someone obtains a 6★ character, the character has a chance of being a rate-up. We'll call this chance the **subrate**. The subrate and number of rate-up 6★ characters depends on the type of banner:

| Banner type | Number of rate-ups | Subrate (of each rate-up) |
| :---------: | :----------------: | :-----------------------: |
|  standard   |         2          |            25%            |
|    event    |         1          |            50%            |
|   limited   |         2          |            35%            |

Players generally pull on banners with a **target** in mind, such as obtaining a certain number of copies of a specific rate-up 6★ character. We'll focus on calculating probabilities for just this scenario.
