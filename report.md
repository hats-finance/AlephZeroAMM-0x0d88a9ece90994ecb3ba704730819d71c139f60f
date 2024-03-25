# **AlephZeroAMM Audit Competition on Hats.finance** 


## Introduction to Hats.finance


Hats.finance builds autonomous security infrastructure for integration with major DeFi protocols to secure users' assets. 
It aims to be the decentralized choice for Web3 security, offering proactive security mechanisms like decentralized audit competitions and bug bounties. 
The protocol facilitates audit competitions to quickly secure smart contracts by having auditors compete, thereby reducing auditing costs and accelerating submissions. 
This aligns with their mission of fostering a robust, secure, and scalable Web3 ecosystem through decentralized security solutions​.

## About Hats Audit Competition


Hats Audit Competitions offer a unique and decentralized approach to enhancing the security of web3 projects. Leveraging the large collective expertise of hundreds of skilled auditors, these competitions foster a proactive bug hunting environment to fortify projects before their launch. Unlike traditional security assessments, Hats Audit Competitions operate on a time-based and results-driven model, ensuring that only successful auditors are rewarded for their contributions. This pay-for-results ethos not only allocates budgets more efficiently by paying exclusively for identified vulnerabilities but also retains funds if no issues are discovered. With a streamlined evaluation process, Hats prioritizes quality over quantity by rewarding the first submitter of a vulnerability, thus eliminating duplicate efforts and attracting top talent in web3 auditing. The process embodies Hats Finance's commitment to reducing fees, maintaining project control, and promoting high-quality security assessments, setting a new standard for decentralized security in the web3 space​​.

## AlephZeroAMM Overview

UniswapV2 protocol implementation in ink_ smart contract language.

## Competition Details


- Type: A public audit competition hosted by AlephZeroAMM
- Duration: 11 days
- Maximum Reward: $112,000
- Submissions: 48
- Total Payout: $100,004.8 distributed among 9 participants.

## Scope of Audit

## Project overview

The contracts implement a constant-product AMM based on the classical Uniswap V2 model together with a farm contract for boosting LP's rewards. Smart contracts are implemented in ink! smart contract language and adapted to work on Substrate platform. 
## Audit competition scope


```
|-- common-amm
     |-- amm
          |-- contracts
               |-- factory
                    |-- lib.rs
               |-- pair
                    |-- lib.rs
               |-- router
                    |-- lib.rs
          |-- traits
               |-- factory.rs
               |-- pair.rs
               |-- router.rs
               |-- swap_callee.rs
     |-- farm
           |-- contract
                |-- lib.rs
           |-- trait
                |-- lib.rs
```

## High severity issues


- **Issue with Claiming Granted Rewards in Future-set Inactive Farm Contract**

  The issue pertains to a flaw in the farm contract that allows the owner to inadvertently or intentionally stop users from claiming their rewards. This happens when the owner attempts to withdraw tokens while the farm is inactive. The current check allows this if the farm is set to become active in the future. However, in the absence of real reward tokens in the contract, the 'claim_rewards' function fails, preventing users from retrieving their previous rewards.

An attached PoC outlines a scenario in which:
- User A holds unclaimed rewards (100 tokens) in the contract.
- The owner sets up a new farm for the future.
- The owner withdraws tokens while the farm is inactive.
- Once the new farm gets activated, User A earns new rewards (50 tokens).
- However, when User A tries to claim rewards, only the initial 100 tokens are present in the contract, which causes the 'claim_rewards' function to fail.

A suggested solution is to modify the 'owner_withdraw_token' function to use 'owner_stop_farm' instead of 'self.is_active()'. This alternative design aims to prevent possible vulnerabilities, such as a 'rug pull' scenario where the farm owner withdraws all rewards unexpectedly, leaving users locked in the farm with nothing to claim. Other contributions also suggest some code improvements for rewarding and balance check procedures.


  **Link**: [Issue #10](https://github.com/hats-finance/AlephZeroAMM-0x0d88a9ece90994ecb3ba704730819d71c139f60f/issues/10)


- **Farming Protocol Allows Unauthorized Token Withdrawal by Owner**

  The issue highlighted revolves around a discovered vulnerability where the owner of a farming pool can siphon off all the tokens. In the outlined attack scenario, a project owner creates a pool, where users can deposit 'ICE' tokens and receive 'WOOD' tokens as rewards. The owner manipulates the system by setting the reward tokens as [WOOD, WOOD] upon contract creation.

The owner starts a new farm with specified start and end times, setting the rewards as [0, amount], with 'amount' being a value of their choosing. Users, happy with their returns, deposit their tokens to farm the liquidity. The issue arises when the farm is stopped by the owner. Despite accumulated rewards for users, the owner can withdraw them due to a protocol loophole: the reward for 0th token is 0. Consequently, users cannot access their rewards.

As evidence, a proof of concept (PoC) session illustrating the scenario is provided, including the code in Rust. The mitigation recommended suggests preventing duplicate `AccountId`s in the `reward_tokens` vector to stop potential manipulations. Upon review, the submission was deemed valid due to the high level of risk presented.


  **Link**: [Issue #20](https://github.com/hats-finance/AlephZeroAMM-0x0d88a9ece90994ecb3ba704730819d71c139f60f/issues/20)


- **Bug in Factory Contract Causes Incorrect Liquidity Distribution in AMM Pairing**

  This summary pertains to a notable bug identified in the GitHub code base concerning a factory contract within a finance protocol. The contract's objective is to accumulate 5 basis points fee from trades where a determined portion is meant for the protocol fee beneficiary. However, the mistake in the contract leads to a significant share of the liquidity being added or removed, instead of just a portion - which is the intended functionality.

To illustrate this flaw, an example is provided of a user named Bob. When Bob adds liquidity, a parameter `k_last` gets updated. However, due to an error, it's set to `Some(0)` as it multiplies members of a local tuple initialized to `(0, 0)`. Consequently, when Bob applies the same amount of liquidity, the system mints 16% excess liquidity. 

A proof of concept for this issue is shown via a Rust function `add_liquidity_collects_too_much_fee`, which simulates the situation. Ultimately, Bob can withdraw less than he provided, illustrating the defect's capacity for exploitation. 

Recommended mitigation is to set the `k_last` during `mint` and `burn` operations to a freshly updated value. This modification is suggested at two instances in the contract's code base for effective resolution. After manual inspection, this has been confirmed as a valid high-level severity bug.


  **Link**: [Issue #37](https://github.com/hats-finance/AlephZeroAMM-0x0d88a9ece90994ecb3ba704730819d71c139f60f/issues/37)

## Low severity issues


- **Calculation Error Causes Users to Receive Fewer Rewards Than Expected in Farming Feature**

  The issue has identified a problem in the reward calculation in a farming system. Due to a lack of floating numbers and hidden division before multiplication, users receive fewer rewards than intended. For instance, an owner intending to give 1000 USDC sees an actual reward of 777 USDC. A suggested fix is to use a scale factor in the reward rate calculation to prevent such discrepancies.


  **Link**: [Issue #44](https://github.com/hats-finance/AlephZeroAMM-0x0d88a9ece90994ecb3ba704730819d71c139f60f/issues/44)

## Minor severity issues


- **Missing Event Emission for Farm Start and Stop Functions**

  The 'farm::owner_stop_farm' and 'farm::owner_start_new_farm' functions in the farm contract lack events, impacting transparency and user awareness of important changes. Proposed solution involves adding event definitions 'FarmStopped' and 'FarmStarted', modifying the said functions to emit these events. Such changes will provide users with essential information about the state of the farm contract.


  **Link**: [Issue #6](https://github.com/hats-finance/AlephZeroAMM-0x0d88a9ece90994ecb3ba704730819d71c139f60f/issues/6)


- **Inaccurate Error Message in `farm::claim_rewards` Function When No Rewards Available**

  The issue pertains to the `farm::claim_rewards` function, which erroneously generates the error `FarmError::CallerNotFarmer` even if users have no claimable rewards. The suggested solution is to replace it with `FarmError::CallerHasNoClaimableRewards` to prevent misinterpretation and incorrect assumptions about the status of the caller.


  **Link**: [Issue #25](https://github.com/hats-finance/AlephZeroAMM-0x0d88a9ece90994ecb3ba704730819d71c139f60f/issues/25)


- **Issue with `stop_farm` Function Allowing Repeated Updates in Farm's End Timestamp**

  The 'stop_farm' function enables the owner to halt farming at any point, setting the 'end' of the farm to the current time stamp. However, due to insufficient input validation, the function can be repeatedly called, continuously updating the farm 'end,' effectively keeping the farm active. Thus, the parameters set by the 'owner_start_new_farm' can be altered at will, potentially causing future issues if checks for active farming are implemented. The proposed solution is to ensure that 'stop_farm' truly halts the farm and cannot be restarted with a new timestamp.


  **Link**: [Issue #32](https://github.com/hats-finance/AlephZeroAMM-0x0d88a9ece90994ecb3ba704730819d71c139f60f/issues/32)


- **Pair Contract Metadata Implementation Lacking in Rust Uniswap Version**

  The issue pertains to a lack in metadata implementation in the pair contract. This can impact clarity and interoperability with external systems. Unlike the original Uniswap which has a fixed symbol and name for metadata, the Rust implementation uses default values. Providing appropriate values for name and symbol metadata in the Pair contract would resolve this and improve compatibility.


  **Link**: [Issue #41](https://github.com/hats-finance/AlephZeroAMM-0x0d88a9ece90994ecb3ba704730819d71c139f60f/issues/41)


- **Owner Withdraw Token Function Restricting Extra Pool_ID Tokens Withdrawal Issue**

  The issue revolves around the `owner_withdraw_token` function which allows the owner to withdraw extra tokens from the contract. However, the current code prevents the owner from withdrawing extra tokens linked to the `pool_id`. The proposed code revision removes this restriction, enabling the owner to withdraw any token, including those associated with `pool_id`.


  **Link**: [Issue #47](https://github.com/hats-finance/AlephZeroAMM-0x0d88a9ece90994ecb3ba704730819d71c139f60f/issues/47)



## Conclusion

The Hats.finance audit competition was hosted by AlephZeroAMM, with a maximum reward of $112,000 spanning 11 days. The competition derived 48 submissions, and 9 participants shared a total payout of $100,004.8. The audit revealed the efficacy of Hats.finance's decentralized security infrastructure and its capacity to quickly secure smart contracts by attracting top auditors. However, many issues were found, including high severity issues such as flaws allowing farm owners to withdraw tokens unintentionally, thereby preventing users from claiming their rewards. Problems concerning unauthorized token withdrawal by the owner, and bugs in the factory contract resulting in incorrect liquidity distribution. Furthermore, minor severity issues were highlighted around calculation errors in the reward system, and missing events in functions critical to user transparency. Overall, the event underscored the importance of continuous audit and improvement in the pursuit of robust, decentralized security solutions.

## Disclaimer


This report does not assert that the audited contracts are completely secure. Continuous review and comprehensive testing are advised before deploying critical smart contracts./n/n
The AlephZeroAMM audit competition illustrates the collaborative effort in identifying and rectifying potential vulnerabilities, enhancing the overall security and functionality of the platform.


Hats.finance does not provide any guarantee or warranty regarding the security of this project. All smart contract software should be used at the sole risk and responsibility of users.

