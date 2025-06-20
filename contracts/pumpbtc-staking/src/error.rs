use soroban_sdk::{self, contracterror};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PumpBTCStakingError {
    NegativeAmountNotAllowed = 0,
    CallerIsNotOperator = 1,
    NoOperatorSet = 2,
    OnlyAllowStakeAtFirst = 3,
    AlreadyInitialized = 4,
    StakingCapTooSmall = 5,
    FeeShouldBeBetween0And10000 = 6,
    NoFeeToCollect = 7,
    NoPendingStakeAmount = 8,
    ExceedStakingCap = 9,
    ClaimPreviousUnstakeFirst = 10,
    NotReachedClaimableTime = 11,
    InsufficientPendingStakeAmount = 12,
    MathOverflow = 13,
    AssetDecimalTooSmall = 14,
    NoPendingUnstake = 15,
    InvalidPumpTokenDecimal = 16,
    NoPendingAdminTransfer = 17,
    ContractIsPaused = 18,
    ContractIsNotPaused = 19,
}
