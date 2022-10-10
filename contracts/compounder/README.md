# Compounder

## Questions
- which assets can be staked? (answer: ukuji)

- assuming multiple assets can be staked, how do users stake multiple different assets with the same validator?

- assuming rewards are given in multiple denoms, how are rewards withdrawn

- do validator addresses need to be validated? (would it be safe to let this address be passed in OR do we need to go down the same route as pairs)
    - if you pass an incorrect address you get the message: failed to execute message; message index: 0: dispatch: submessages: validator does not exist

- what is the flow of staking events
    - lets capture this and right it in notion
        - send delegation message with amount
        - receive new_shares based on your delegation

- what is the flow of auto compounding
    - contract sends coins to validator
    - on some interval it withdraws rewards and then stakes the rewards with that validator

- look at the ways in which an undelegate can fail
    - if maximum amount of undelegations is exceeded on the validator (7 per delegator - validator pair)

- how to claim asssets after unbonding
    - assets sent back to child contract
    - user should be able to query balance on child contract and withdraw all assets

## Technical challenges
- our contract can at most have 7 pending undelegate messages on any validator
    - we could look at creating a new contract for every user, but then each user has its own rewards pool and

## Interaction Requirements
- deposit funds to be staked
- withdraw funds
- get all balances
- a user can stake with multiple validators

## Integration Use Case
- user DCA's into some asset X
- everytime a swap is made send the resulting assets to the compounder

