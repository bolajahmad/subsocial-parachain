#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::{DispatchError, DispatchResult};

use crate::SpaceId;

pub trait SpacePermissionsProvider<AccountId, SpacePermissionsInfo> {
    fn space_permissions_info(id: SpaceId) -> Result<SpacePermissionsInfo, DispatchError>;

    fn ensure_space_owner(id: SpaceId, account: &AccountId) -> DispatchResult;
}

pub trait SpaceFollowsProvider {
    type AccountId;

    fn is_space_follower(account: Self::AccountId, space_id: SpaceId) -> bool;
}

pub trait ProfileManager<AccountId> {
    fn profile_space_id(account: &AccountId) -> Option<SpaceId>;

    fn try_set_profile(account: &AccountId, space_id: SpaceId) -> DispatchResult;

    fn unlink_space_from_profile(account: &AccountId, space_id: SpaceId) -> DispatchResult;
}
