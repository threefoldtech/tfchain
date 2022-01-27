# Validator Request Flow

See: [flow diagram](./validator_request_flow.md)

## Flow explained

Any user can create a request to become a validator, the object looks as following:

```rust
pub struct ValidatorRequest<AccountId> {
    pub id: u32,
    pub council_account: AccountId,
    pub validator_account: AccountId,
    pub stash_account: AccountId,
    pub description: Vec<u8>,
    pub tf_connect_id: u64,
    pub info: Vec<u8>,
    pub state: ValidatorRequestState(created, approved, executed),
}
```

When the council decides to approve the request, the council should propose a motion with following extrinsic: `(CouncilMembership -> AddMember(who)` (who being the `validatorRequest.council_account`)

If the motion is closed, two things happen:

- the account id provided is added as a council member
- the validator request (if any) goes to an approved state

Now that the validator request is in an `Approved` state, the newly added council member can call `(ValidatorSet -> ActivateValidator(requestID))`.

Following things happen when this is executed:

- Chain reads validator request
- Extract validator_account field, insert that in the list of active validators
- Move the state of the validator request to `Executed`
- Rotate Session
