---
title: Treasury
---

The Treasury is a pot of funds collected through transaction fees, slashing,
[staking inefficiencies](staking-tdfy.md#inflation), etc. The funds held in the Treasury can be spent by making a spending proposal that, if approved by the [Council](governance.md#council), will enter a waiting period before distribution. This waiting period is known as the budget period, and its duration is subject to [governance](governance.md), with the current default set to 24 days. The Treasury attempts to spend as many proposals in the queue as it can without running out of funds.

If the Treasury ends a budget period without spending all of its funds, it suffers a burn of a percentage of its funds -- thereby causing deflationary pressure. This percentage is currently at 1%.

When a stakeholder wishes to propose a spend from the Treasury, they must reserve a deposit of at least 5% of the proposed spend (see below for variations). This deposit will be slashed if the proposal is rejected, and returned if it is accepted.

Proposals may consist of (but are not limited to):

- Infrastructure deployment and continued operation.
- Network security operations (monitoring services, continuous auditing).
- Ecosystem provisions (collaborations with friendly chains).
- Marketing activities (advertising, paid features, collaborations).
- Community events and outreach (meetups, pizza parties, hackerspaces).
- Software development (wallets and wallet integration, clients and client upgrades).

The Treasury is ultimately controlled by the [Council](governance.md#council), and how the funds will be spent is up to their judgment.

## Funding the Treasury

The Treasury is funded from different sources:

1. Slashing: When a validator is slashed for any reason, the slashed amount is sent to the Treasury with a reward going to the entity that reported the validator (another validator). The reward is taken from the slash amount and varies per offence and number of reporters.

2. Transaction fees: A portion of each block's transaction fees goes to the Treasury, with the remainder going to the block author.

3. Staking inefficiency: [Inflation](staking-tdfy.md#inflation) is designed to be 10% in the first year, and the ideal staking ratio is set at 50%, meaning half of all tokens should be locked in staking. Any deviation from this ratio will cause a proportional amount of the inflation to go to the Treasury. In other words, if 50% of all tokens are staked, then 100% of the inflation goes to the validators as reward. If the staking rate is less than 50%, then the validators will receive less, with the remainder going to the Treasury.

## Creating a Treasury Proposal

The proposer has to deposit 5% of the requested amount or 100 TDFY (whichever is higher) as an anti-spam measure. This amount is burned if the proposal is rejected, or refunded otherwise. These values are subject to [governance](governance.md) so they may change in the future.

Please note that there is no way for a user to revoke a treasury proposal after it has been submitted. The Council will either accept or reject the proposal, and if the proposal is rejected, the bonded funds are burned.

### Announcing the Proposal

To minimize storage on chain, proposals don't contain contextual information. When a user submits a proposal, they will probably need to find an off-chain way to explain the proposal.

### Creating the Proposal

One way to create the proposal is to use the [Tidechain Explorer](http://explorer.tidefi.io/). From the website, use either the
[extrinsics tab](https://explorer.tidefi.io/#/extrinsics) and select the Treasury pallet, then `proposeSpend` and enter the desired amount and recipient, or use the [Treasury tab](http://explorer.tidefi.io/#/treasury) and its dedicated Submit Proposal button

![A proposal being created](./assets/treasury-submit-proposal.png)

The system will automatically take the required deposit, picking the higher of the two values mentioned [above](#creating-a-treasury-proposal).

Once created, your proposal will become visible in the Treasury screen and the Council can start voting on it.

![A proposal in queue](./assets/treasury-pending-proposal.png)

Remember that the proposal has no metadata, so it's up to the proposer to create a description and purpose that the Council could study and base their votes on.

At this point, a Council member can create a motion to accept or to reject the treasury proposal. It is possible that one motion to accept and another motion to reject are both created. The proportions to accept and reject Council proposals vary between accept or reject.

The threshold for accepting a treasury proposal is at least 3/5 of the Council. On the other hand, the threshold for rejecting a proposal is at least 1/2 of the Council.

![A pending motion](./assets/treasury-pending-motion.png)

## Bounties Spending

There are practical limits to Council Members curation capabilities when it comes to treasury proposals: Council members likely do not have the expertise to make a proper assessment of the activities described in all proposals. Even if individual Councillors have that expertise, it is highly unlikely that a majority of members are capable in such diverse topics.

Bounties Spending proposals aim to delegate the curation activity of spending proposals to experts called Curators: They can be defined as addresses with agency over a portion of the Treasury with the goal of fixing a bug or vulnerability, developing a strategy, or monitoring a set of tasks related to a specific topic: all for the benefit of the Tidefi ecosystem.

A proposer can submit a bounty proposal for the Council to pass, with a curator to be defined later, whose background and expertise is such that they are capable of determining when the task is complete. Curators are selected by the Council after the bounty proposal passes, and need to add an upfront payment to take the position. This deposit can be used to punish them if they act maliciously. However, if they are successful in their task of getting someone to complete the bounty work, they will receive their deposit back and part of the bounty reward.

When submitting the value of the bounty, the proposer includes a reward for curators willing to invest their time and expertise in the task: this amount is included in the total value of the bounty. In this sense, the curator's fee can be defined as the result of subtracting the value paid to the bounty rewardee from the total value of the bounty.

In general terms, curators are expected to have a well-balanced track record related to the issues the bounty tries to resolve: they should be at least knowledgeable on the topics the bounty touches, and show project management skills or experience. These recommendations ensure an effective use of the mechanism. A Bounty Spending is a reward for a specified body of work - or specified set of objectives - that needs to be executed for a predefined treasury amount to be paid out. The responsibility of assigning a payout address once the specified set of objectives is completed is
delegated to the curator.

After the Council has activated a bounty, it delegates the work that requires expertise to the curator who gets to close the active bounty. Closing the active bounty enacts a delayed payout to the payout address and a payout of the curator fee. The delay phase allows the Council to act if any issues arise.

To minimize storage on chain in the same way as any proposal, bounties don't contain contextual information. When a user submits a bounty spending proposal, they will probably need to find an off-chain way to explain the proposal.

The bounty has a predetermined duration of 90 days with the possibility of being extended by the curator. Aiming to maintain flexibility on the tasks’ curation, the curator will be able to create sub-bounties for more granularity and allocation in the next iteration of the mechanism.

### Creating a Bounty Proposal

Anyone can create a Bounty proposal using Tidechain Explorer: Users are able to submit a proposal on the dedicated Bounty section under Governance. The development of a robust user interface to view and manage bounties in the Tidefi Apps is still under development and it will serve Council members, Curators and Beneficiaries of the bounties, as well as all users observing the on-chain treasury governance. For now, the help of a Councillor is needed to open a bounty proposal as a motion to be voted.

To submit a bounty, please visit [Tidechain Explorer](http://explorer.tidefi.io/) and click on the governance tab in the options bar on the top of the site. After, click on 'Bounties' and find the button '+ Add Bounty' on the upper-right side of the interface. Complete the bounty title, the requested allocation (including curator's fee) and confirm the call.

After this, a Council member will need to assist you to pass the bounty proposal for vote as a motion.

A bounty can be cancelled by deleting the earmark for a specific treasury amount or be closed if the tasks have been completed. On the opposite side, the 90 days life of a bounty can be extended by amending the expiry block number of the bounty to stay active.

### Closing a bounty

The curator can close the bounty once they approve the completion of its tasks. The curator should make sure to set up the payout address on the active bounty beforehand. Closing the Active bounty enacts a delayed payout to the payout address and a payout of the curator fee.

A bounty can be closed by using the extrinsics tab and selecting the Bounties pallet, then `awardBounty`, making sure the right bounty is to be closed and finally sign the transaction. It is important to note that those who received a reward after the bounty is completed, must claim the specific amount of the payout from the payout address, by calling `claimBounty` after the curator closed the allocation.

## FAQ

### What prevents the Treasury from being captured by a majority of the Council?

The majority of the Council can decide the outcome of a treasury spend proposal. In an adversarial mindset, we may consider the possibility that the Council may at some point go rogue and attempt to steal all of the treasury funds. It is a possibility that the treasury pot becomes so great, that a large financial incentive would present itself.

For one, the Treasury has deflationary pressure due to the burn that is suffered every spend period.

The burn aims to incentivize the complete spend of all treasury funds at every burn period, so ideally the treasury pot doesn't have time to accumulate mass amounts of wealth.

However, it is the case that the Council is composed of mainly well-known members of the community. Remember, the Council is voted in by the token holders, so they must do some campaigning or otherwise be recognized to earn votes. In the scenario of an attack, the Council members would lose their social credibility. Furthermore, members of the Council are usually externally motivated by the proper operation of the chain/

This external motivation is either because they run businesses
that depend on the chain, or they have direct financial gain (through their holdings) of the token value remaining steady.

Concretely, there are a couple on-chain methods that resist this kind of attack. One, the Council majority may not be the token majority of the chain. This means that the token majority could vote to replace the Council if they attempted this attack - or even reverse the treasury spend. They would do this through a normal referendum. Two, there are time delays to treasury spends. They are only enacted every spend period.

This means that there will be some time to observe this attack is
taking place. The time delay then allows chain participants time to respond. The response may take the form of governance measures or - in the most extreme cases a liquidation of their holdings and a migration to a minority fork. However, the possibility of this scenario is quite low.
