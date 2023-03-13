# 8. Lower discount level threshold

Date: 2023-03-10

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/637) for more details.

## Decision

We decided to lower the discount level thresholds by reduzing them by half.
In consequence, a grid user get faster access to a better discount level

Before:

 | discount level | pricing discount | nr months staking |
 | -------------- | -----------------| ----------------- |
 | none           | - 0%             | 0 month           |
 | default        | - 20%            | 3 months          |
 | bronze         | - 30%            | 6 months          |
 | silver         | - 40%            | 12 months         |
 | gold           | - 60%            | 36 months         |

Now:

 | discount level | pricing discount | nr months staking |
 | -------------- | -----------------| ----------------- |
 | none           | - 0%             | 0 month           |
 | default        | - 20%            | 1.5 months        |
 | bronze         | - 30%            | 3 months          |
 | silver         | - 40%            | 6 months          |
 | gold           | - 60%            | 18 months         |
