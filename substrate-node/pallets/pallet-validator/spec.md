## Flow explained

Anyone can apply to become a validator by calling the `create_validator` extrinsic on the `validator` pallet.

This call must be signed by the account that wants to become a member of the DAO Council & wants to participate in consensus.

The object looks as following:

```rust
pub struct Validator<AccountId> {
    pub validator_node_account: AccountId,
    pub stash_account: AccountId,
    pub description: Vec<u8>,
    pub tf_connect_id: Vec<u8>,
    pub info: Vec<u8>,
    pub state: ValidatorRequestState(created, approved, executed),
}
```

This object represents a Validator. A Validator is a combination of two things:

- Council member (account ID is inferred from the creator of this object).
- Consensus Validator (aura/gran) represented by the `validator_node_account` field.

When the council decides to approve the request, the council should propose a motion with following extrinsic: `(CouncilMembership -> ApproveValidator(who)` (who being the validator's account).

If the motion is closed, two things happen:

- the account id provided is added as a council member
- the validator request state goes to an approved state

Now that the validator request is in an `Approved` state, the newly added council member can call `(ValidatorSet -> ActivateValidator())`.

Following things happen when this is executed:

- Chain reads validator request
- Extract validator_node_account field, insert that in the list of active validators
- Move the state of the validator request to `Validating`
- Rotate Session