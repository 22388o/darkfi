# version 0

## misc

- [ ] git grep TODO

## documentation

- [ ] document v0 architecture 
- [ ] finish cashier explainer
- [ ] mint/ burn explainer
- [ ] v1 roadmap

# v1-research

## blockchain

- [ ] evaluate themelio
- [ ] evaluate casper-cbc
- [ ] basic sequencer architecture design
- [ ] basic DHT design
- [ ] consensus algorithm
- [ ] solve double verify problem (potentially need need a payment inside the contract to handle exceptions)
- [ ] research polygon design
- [ ] code up a simple demo

## bridges

- [ ] evaluate arbitrum vs optics
- [ ] evaluate layerzero

# general-research

Open research questions.

## light-clients

- [ ] Fast efficient batch DH technique. Currently all new transactions need to be scanned. There should be a means of efficiently batching this test for light clients initially syncing against a server.
- [ ] Anonymous fetch using an Oblivious-Transfer protocol. Light clients potentially leak info to servers based on the data they request, but with an OT protocol they do not reveal exactly what they are requesting.

## cryptography

- [ ] halo2 lookup
- [ ] read groth permutation paper
- [ ] fflonk

## token

- [ ] simple amm script
- [ ] bonded curve script
- [ ] quadratic funding script
- [ ] write up DRK tokenomics
- [ ] simulate in CADCAD

## product

- [ ] move DRK in and out of contracts from the root chain
- [ ] first MPC services
- [ ] DAO
- [ ] auctions
- [ ] swaps
- [ ] token issuance
- [ ] NFTs

## dev

- [ ] make bitreich halo2 impl
- [ ] doc on circuit design

# org

- [ ] clean up shared repo and migrate to wiki
- [ ] md blog for website

# darkpulse

- [ ] ...
