use frame_support::{assert_noop, assert_ok};
use sp_runtime::{DispatchError, DispatchError::BadOrigin, traits::Zero};

use pallet_parachain_utils::mock_functions::{another_valid_content_ipfs, invalid_content_ipfs, valid_content_ipfs};
use pallet_parachain_utils::new_who_and_when;

use crate::{DomainByInnerValue, Event, mock::*};
use crate::Error;
use crate::types::*;

// `register_domain` tests

#[test]
fn register_domain_should_work() {
    const LOCAL_DOMAIN_DEPOSIT: Balance = 10;

    ExtBuilder::default()
        .domain_deposit(LOCAL_DOMAIN_DEPOSIT)
        .build()
        .execute_with(|| {
            let owner = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());
            let expected_domain = default_domain();
            let expected_domain_lc = default_domain_lc();

            assert!(get_reserved_balance(&owner).is_zero());

            assert_ok!(_register_default_domain());

            assert_eq!(Domains::domains_by_owner(&owner), vec![expected_domain_lc.clone()]);

            let domain_meta = Domains::registered_domain(&expected_domain_lc).unwrap();
            assert_eq!(domain_meta, DomainMeta {
                created: new_who_and_when::<Test>(DOMAIN_OWNER),
                updated: None,
                expires_at: ExtBuilder::default().reservation_period_limit + 1,
                owner: DOMAIN_OWNER,
                screen_domain: expected_domain.clone(),
                content: valid_content_ipfs(),
                inner_value: None,
                outer_value: None,
                domain_deposit: LOCAL_DOMAIN_DEPOSIT,
                outer_value_deposit: Zero::zero()
            });

            assert_eq!(get_reserved_balance(&owner), LOCAL_DOMAIN_DEPOSIT);

            System::assert_last_event(Event::<Test>::DomainRegistered(
                owner,
                expected_domain,
            ).into());
        });
}

#[test]
fn register_domain_should_fail_with_bad_origin() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            _register_domain_with_origin(Origin::signed(DOMAIN_OWNER)),
            BadOrigin
        );
    });
}

#[test]
fn register_domain_should_fail_when_reservation_period_zero() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            _register_domain_with_expires_in(0),
            Error::<Test>::ZeroReservationPeriod,
        );
    });
}

#[test]
fn register_domain_should_fail_when_reservation_above_limit() {
    ExtBuilder::default()
        .reservation_period_limit(1000)
        .build()
        .execute_with(|| {
            assert_noop!(
                _register_domain_with_expires_in(1001),
                Error::<Test>::TooBigRegistrationPeriod,
            );
        });
}

#[test]
fn register_domain_should_fail_when_domain_reserved() {
    ExtBuilder::default().build().execute_with(|| {
        let _ = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());
        assert_ok!(_reserve_default_domain());
        assert_noop!(
            _register_default_domain(),
            Error::<Test>::DomainIsReserved,
        );
    });
}

#[test]
fn register_domain_should_fail_when_domain_already_owned() {
    ExtBuilder::default().build().execute_with(|| {
        let _ = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());
        assert_ok!(_register_default_domain());
        assert_noop!(
            _register_default_domain(),
            Error::<Test>::DomainAlreadyOwned,
        );
    });
}

#[test]
fn register_domain_should_fail_when_too_many_domains_registered() {
    ExtBuilder::default()
        .max_domains_per_account(1)
        .build()
        .execute_with(|| {
            let _ = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

            let domain_one = domain_from(b"domain-one".to_vec());
            let domain_two = domain_from(b"domain-two".to_vec());

            assert_ok!(_register_domain_with_name(domain_one));
            assert_noop!(
                _register_domain_with_name(domain_two),
                Error::<Test>::TooManyDomainsPerAccount,
            );
        });
}

#[test]
fn register_domain_should_fail_when_balance_is_insufficient() {
    ExtBuilder::default()
        .domain_deposit(10)
        .build()
        .execute_with(|| {
            let _ = account_with_balance(DOMAIN_OWNER, 9);

            assert_noop!(
                _register_default_domain(),
                pallet_balances::Error::<Test>::InsufficientBalance,
            );
        });
}

// `set_inner_value` tests

#[test]
fn set_inner_value_should_work() {
    ExtBuilder::default().build_with_domain().execute_with(|| {
        let owner = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

        let domain_lc = default_domain_lc();
        let old_value = get_inner_value(&domain_lc);

        assert_ok!(_set_default_inner_value());

        let result_value = get_inner_value(&domain_lc);
        assert!(old_value != result_value);

        let expected_value = Some(inner_value_account_domain_owner());
        assert_eq!(expected_value, result_value);

        assert_eq!(
            DomainByInnerValue::<Test>::get(&expected_value.unwrap()),
            Some(default_domain()),
        );

        System::assert_last_event(Event::<Test>::DomainUpdated(
            owner,
            domain_lc,
        ).into());
    });
}

#[test]
fn set_inner_value_should_fail_when_domain_has_expired() {
    ExtBuilder::default().build_with_domain().execute_with(|| {
        let _ = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

        System::set_block_number(ExtBuilder::default().reservation_period_limit + 1);

        assert_noop!(
            _set_default_inner_value(),
            Error::<Test>::DomainHasExpired,
        );
    });
}

#[test]
fn set_inner_value_should_fail_when_not_domain_owner() {
    ExtBuilder::default().build_with_domain().execute_with(|| {
        let not_an_owner = account_with_balance(DUMMY_ACCOUNT, BalanceOf::<Test>::max_value());

        assert_noop!(
            _set_inner_value_with_origin(Origin::signed(not_an_owner)),
            Error::<Test>::NotDomainOwner,
        );
    });
}

#[test]
fn set_inner_value_should_fail_when_inner_value_not_differ() {
    ExtBuilder::default().build_with_domain().execute_with(|| {
        let _ = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

        assert_ok!(_set_default_inner_value());

        assert_noop!(
            _set_default_inner_value(),
            Error::<Test>::InnerValueNotChanged,
        );
    });
}

// `set_outer_value` tests

#[test]
fn set_outer_value_should_work() {
    const LOCAL_DOMAIN_DEPOSIT: Balance = 10;
    const LOCAL_BYTE_DEPOSIT: Balance = 1;

    ExtBuilder::default()
        .domain_deposit(LOCAL_DOMAIN_DEPOSIT)
        .outer_value_byte_deposit(LOCAL_BYTE_DEPOSIT)
        .build_with_domain()
        .execute_with(|| {
            let owner = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

            let domain_lc = default_domain_lc();
            let old_value = get_outer_value(&domain_lc);

            assert_ok!(_set_default_outer_value());

            let expected_value = Some(default_outer_value(None));
            let result_value = get_outer_value(&domain_lc);

            assert!(old_value != result_value);
            assert_eq!(expected_value, result_value);

            let reserved_balance = get_reserved_balance(&owner);
            assert_eq!(
                reserved_balance,
                expected_value.unwrap().len() as u64 * LOCAL_BYTE_DEPOSIT + LOCAL_DOMAIN_DEPOSIT
            );

            System::assert_last_event(Event::<Test>::DomainUpdated(
                owner,
                domain_lc,
            ).into());
        });
}

#[test]
fn set_outer_value_should_work_when_deposit_changed() {
    const LOCAL_BYTE_DEPOSIT_INIT: Balance = 1;
    let domain_deposit = ExtBuilder::default().base_domain_deposit;

    ExtBuilder::default()
        .outer_value_byte_deposit(LOCAL_BYTE_DEPOSIT_INIT)
        .build_with_domain()
        .execute_with(|| {
            let owner = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());
            let calc_deposit = |value| value as Balance * LOCAL_BYTE_DEPOSIT_INIT + domain_deposit;

            let initial_value = Some(default_outer_value(Some(10)));
            assert_ok!(_set_outer_value_with_value(initial_value.clone()));
            assert_eq!(get_reserved_balance(&owner), calc_deposit(initial_value.unwrap().len()));

            let new_value = Some(default_outer_value(Some(20)));
            assert_ok!(_set_outer_value_with_value(new_value.clone()));
            assert_eq!(get_reserved_balance(&owner), calc_deposit(new_value.unwrap().len()));
        });
}

#[test]
fn set_outer_value_should_fail_when_domain_has_expired() {
    ExtBuilder::default().build_with_domain().execute_with(|| {
        let _ = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

        System::set_block_number(ExtBuilder::default().reservation_period_limit + 1);

        assert_noop!(
            _set_default_outer_value(),
            Error::<Test>::DomainHasExpired,
        );
    });
}

#[test]
fn set_outer_value_should_fail_when_not_domain_owner() {
    ExtBuilder::default().build_with_domain().execute_with(|| {
        let not_owner = account_with_balance(DUMMY_ACCOUNT, BalanceOf::<Test>::max_value());

        assert_noop!(
            _set_outer_value_with_origin(Origin::signed(not_owner)),
            Error::<Test>::NotDomainOwner,
        );
    });
}

#[test]
fn set_outer_value_should_fail_when_value_not_differ() {
    ExtBuilder::default().build_with_domain().execute_with(|| {
        let _ = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

        assert_ok!(_set_default_outer_value());

        assert_noop!(
            _set_default_outer_value(),
            Error::<Test>::OuterValueNotChanged,
        );
    });
}

#[test]
fn set_outer_value_should_fail_when_balance_is_insufficient() {
    const LOCAL_DOMAIN_DEPOSIT: Balance = 10;
    const LOCAL_BYTE_DEPOSIT: Balance = 1;

    ExtBuilder::default()
        .domain_deposit(LOCAL_DOMAIN_DEPOSIT)
        .outer_value_byte_deposit(LOCAL_BYTE_DEPOSIT)
        .build_with_domain()
        .execute_with(|| {
            let _ = account_with_balance(DOMAIN_OWNER, LOCAL_BYTE_DEPOSIT - 1);

            assert_noop!(
                _set_default_outer_value(),
                pallet_balances::Error::<Test>::InsufficientBalance,
            );
        });
}

// `set_domain_content` tests

#[test]
fn set_domain_content_should_work() {
    ExtBuilder::default().build_with_domain().execute_with(|| {
        let owner = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

        let domain_lc = default_domain_lc();
        let old_content = get_domain_content(&domain_lc);

        assert_ok!(_set_default_domain_content());

        let result_content = get_domain_content(&domain_lc);

        assert!(old_content != result_content);
        assert_eq!(another_valid_content_ipfs(), result_content);

        System::assert_last_event(Event::<Test>::DomainUpdated(
            owner,
            domain_lc,
        ).into());
    });
}

#[test]
fn set_domain_content_should_fail_when_domain_expired() {
    ExtBuilder::default().build_with_domain().execute_with(|| {
        let _ = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

        System::set_block_number(ExtBuilder::default().reservation_period_limit + 1);

        assert_noop!(
            _set_default_domain_content(),
            Error::<Test>::DomainHasExpired,
        );
    });
}

#[test]
fn set_domain_content_should_fail_when_not_domain_owner() {
    ExtBuilder::default().build_with_domain().execute_with(|| {
        let not_owner = account_with_balance(DUMMY_ACCOUNT, BalanceOf::<Test>::max_value());

        assert_noop!(
            _set_domain_content_with_origin(Origin::signed(not_owner)),
            Error::<Test>::NotDomainOwner,
        );
    });
}

#[test]
fn set_domain_content_should_fail_when_content_not_differ() {
    ExtBuilder::default().build_with_domain().execute_with(|| {
        let _ = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

        assert_ok!(_set_default_domain_content());

        assert_noop!(
            _set_default_domain_content(),
            Error::<Test>::DomainContentNotChanged,
        );
    });
}

#[test]
fn set_domain_content_should_fail_when_content_is_invalid() {
    ExtBuilder::default().build_with_domain().execute_with(|| {
        let _ = account_with_balance(DOMAIN_OWNER, BalanceOf::<Test>::max_value());

        assert_noop!(
            _set_domain_content_with_content(invalid_content_ipfs()),
            DispatchError::Other(pallet_parachain_utils::Error::InvalidIpfsCid.into()),
        );
    });
}

// `reserve_domains` tests

#[test]
fn reserve_domains_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(_reserve_default_domain());

        assert!(Domains::is_domain_reserved(default_domain_lc()));

        System::assert_last_event(Event::<Test>::DomainsReserved(1).into());
    });
}

#[test]
fn reserve_domains_should_fail_when_tried_to_insert_too_many_domains() {
    ExtBuilder::default()
        // When limit is 2
        .domains_insert_limit(2)
        .build()
        .execute_with(|| {
            // Trying to insert a list of 3 domains
            let domains_list = vec![
                domain_from(b"domain-one".to_vec()),
                domain_from(b"domain-two".to_vec()),
                domain_from(b"domain-three".to_vec()),
            ];

            assert_noop!(
                _reserve_domains_with_list(domains_list),
                Error::<Test>::DomainsInsertLimitReached,
            );
        });
}

// Test domain name validation function

#[test]
fn ensure_valid_domain_should_work() {
    ExtBuilder::default()
        .min_domain_length(3)
        .build()
        .execute_with(|| {
            assert_ok!(Domains::ensure_valid_domain(b"abcde.sub"));
            assert_ok!(Domains::ensure_valid_domain(b"a-b-c.sub"));
            assert_ok!(Domains::ensure_valid_domain(b"12345.sub"));

            assert_noop!(
                Domains::ensure_valid_domain(b"a.sub"),
                Error::<Test>::DomainNameIsTooShort,
            );
            assert_noop!(
                Domains::ensure_valid_domain(b"-ab.sub"),
                Error::<Test>::DomainContainsInvalidChar,
            );
            assert_noop!(
                Domains::ensure_valid_domain(b"ab-.sub"),
                Error::<Test>::DomainContainsInvalidChar,
            );
            assert_noop!(
                Domains::ensure_valid_domain(b"a--b.sub"),
                Error::<Test>::DomainContainsInvalidChar,
            );
        });
}
