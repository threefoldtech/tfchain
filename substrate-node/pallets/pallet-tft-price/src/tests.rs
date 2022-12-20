use super::Event as TftPriceEvent;
use crate::{mock::Event as MockEvent, mock::*, Error};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use frame_system::{EventRecord, Phase, RawOrigin};
use sp_core::H256;

#[test]
fn test_calc_avg_rounding_works() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        let mut queue = TFTPriceModule::queue_transient();
        queue.push(499); // avg = 499.0
        assert_eq!(TFTPriceModule::calc_avg(), 499);
        queue.push(500); // avg = 499.5
        assert_eq!(TFTPriceModule::calc_avg(), 500);
        queue.push(500); // avg = 499.66
        assert_eq!(TFTPriceModule::calc_avg(), 500);
        queue.push(500); // avg = 499.75
        assert_eq!(TFTPriceModule::calc_avg(), 500);
        queue.push(499); // avg = 499.66
        assert_eq!(TFTPriceModule::calc_avg(), 500);
        queue.push(499); // avg = 499.5
        assert_eq!(TFTPriceModule::calc_avg(), 500);
        queue.push(499); // avg = 499.25
        assert_eq!(TFTPriceModule::calc_avg(), 499);
    })
}

#[test]
fn test_set_prices_works() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        for i in 1..1441 {
            let target_block = i * 100; // we set the price every 100 blocks
            run_to_block(target_block);
            match TFTPriceModule::set_prices(Origin::signed(alice()), 500, target_block) {
                Ok(_) => (),
                Err(_) => panic!("Couldn't set tft_price"),
            }
        }
        let queue = TFTPriceModule::queue_transient();
        let items = queue.get_all_values();
        assert_eq!(items.len(), 1440);

        assert_eq!(TFTPriceModule::tft_price(), 500);
        assert_eq!(TFTPriceModule::average_tft_price(), 500);
    })
}

#[test]
fn test_set_price_works() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        assert_ok!(TFTPriceModule::set_prices(Origin::signed(alice()), 500, 1));

        assert_eq!(TFTPriceModule::tft_price(), 500);
        assert_eq!(TFTPriceModule::average_tft_price(), 500);
    })
}

#[test]
fn test_set_price_below_min_price_works() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        assert_ok!(TFTPriceModule::set_prices(Origin::signed(alice()), 5, 1));

        let our_events = System::events();
        assert_eq!(
            our_events[our_events.len() - 1],
            record(MockEvent::TFTPriceModule(
                TftPriceEvent::<TestRuntime>::AveragePriceIsBelowMinPrice(
                    5,
                    TFTPriceModule::min_tft_price()
                )
            ))
        );

        assert_eq!(TFTPriceModule::tft_price(), 5);
        assert_eq!(TFTPriceModule::average_tft_price(), 5);
    })
}

#[test]
fn test_set_price_above_max_price_works() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        assert_ok!(TFTPriceModule::set_prices(Origin::signed(alice()), 2000, 1));

        let our_events = System::events();
        assert_eq!(
            our_events[our_events.len() - 1],
            record(MockEvent::TFTPriceModule(
                TftPriceEvent::<TestRuntime>::AveragePriceIsAboveMaxPrice(
                    2000,
                    TFTPriceModule::max_tft_price()
                )
            ))
        );

        assert_eq!(TFTPriceModule::tft_price(), 2000);
        assert_eq!(TFTPriceModule::average_tft_price(), 2000);
    })
}

#[test]
fn test_set_price_not_validator_fails() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        assert_noop!(
            TFTPriceModule::set_prices(Origin::signed(bob()), 500, 1),
            Error::<TestRuntime>::AccountUnauthorizedToSetPrice
        );
    })
}

#[test]
fn test_parse_lowest_price_from_valid_request_works() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        let request_str = json_stellar_price_request_valid();
        let price = TFTPriceModule::parse_lowest_price_from_request(request_str).unwrap();
        assert_eq!(price, 33);
    })
}

#[test]
fn test_parse_lowest_price_from_empty_request_fails() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        let request_str = json_stellar_price_request_empty();
        assert_eq!(
            TFTPriceModule::parse_lowest_price_from_request(request_str),
            None
        );
    })
}

#[test]
fn test_parse_lowest_price_from_incomplete_request_fails() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        let request_str = json_stellar_price_request_incomplete();
        assert_eq!(
            TFTPriceModule::parse_lowest_price_from_request(request_str),
            None
        );
    })
}

#[test]
fn test_set_min_tft_price_works() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        assert_ok!(TFTPriceModule::set_min_tft_price(
            RawOrigin::Root.into(),
            20
        ));

        assert_eq!(TFTPriceModule::min_tft_price(), 20);
    })
}

#[test]
fn test_set_min_tft_price_wrong_origin_fails() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        assert_noop!(
            TFTPriceModule::set_min_tft_price(Origin::signed(bob()), 20),
            BadOrigin,
        );
    })
}

#[test]
fn test_set_min_tft_price_too_high_fails() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        assert_noop!(
            TFTPriceModule::set_min_tft_price(RawOrigin::Root.into(), 2000),
            Error::<TestRuntime>::MinPriceAboveMaxPriceError,
        );
    })
}

#[test]
fn test_set_max_tft_price_works() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        assert_ok!(TFTPriceModule::set_max_tft_price(
            RawOrigin::Root.into(),
            2000
        ));

        assert_eq!(TFTPriceModule::max_tft_price(), 2000);
    })
}

#[test]
fn test_set_max_tft_price_wrong_origin_fails() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        assert_noop!(
            TFTPriceModule::set_max_tft_price(Origin::signed(bob()), 2000),
            BadOrigin,
        );
    })
}

#[test]
fn test_set_max_tft_price_too_low_fails() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        assert_noop!(
            TFTPriceModule::set_max_tft_price(RawOrigin::Root.into(), 5),
            Error::<TestRuntime>::MaxPriceBelowMinPriceError,
        );
    })
}

fn record(event: Event) -> EventRecord<Event, H256> {
    EventRecord {
        phase: Phase::Initialization,
        event,
        topics: vec![],
    }
}

fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
    }
}

fn json_stellar_price_request_empty() -> &'static str {
    r#"
    {
    }"#
}

fn json_stellar_price_request_incomplete() -> &'static str {
    r#"
    {
        "_embedded": {
          "records": [
          ]
        }
    }"#
}

fn json_stellar_price_request_valid() -> &'static str {
    r#"
    {
        "_embedded": {
          "records": [
            {
              "source_asset_type": "credit_alphanum4",
              "source_asset_code": "USDC",
              "source_asset_issuer": "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN",
              "source_amount": "0.0328655",
              "destination_asset_type": "credit_alphanum4",
              "destination_asset_code": "TFT",
              "destination_asset_issuer": "GBOVQKJYHXRR3DX6NOX2RRYFRCUMSADGDESTDNBDS6CDVLGVESRTAC47",
              "destination_amount": "1.0000000",
              "path": [
                {
                  "asset_type": "native"
                },
                {
                  "asset_type": "credit_alphanum4",
                  "asset_code": "yXLM",
                  "asset_issuer": "GARDNV3Q7YGT4AKSDF25LT32YSCCW4EV22Y2TV3I2PU2MMXJTEDL5T55"
                }
              ]
            },
            {
              "source_asset_type": "credit_alphanum4",
              "source_asset_code": "USDC",
              "source_asset_issuer": "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN",
              "source_amount": "0.0340713",
              "destination_asset_type": "credit_alphanum4",
              "destination_asset_code": "TFT",
              "destination_asset_issuer": "GBOVQKJYHXRR3DX6NOX2RRYFRCUMSADGDESTDNBDS6CDVLGVESRTAC47",
              "destination_amount": "1.0000000",
              "path": [
                {
                  "asset_type": "native"
                }
              ]
            },
            {
              "source_asset_type": "credit_alphanum4",
              "source_asset_code": "USDC",
              "source_asset_issuer": "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN",
              "source_amount": "0.0351812",
              "destination_asset_type": "credit_alphanum4",
              "destination_asset_code": "TFT",
              "destination_asset_issuer": "GBOVQKJYHXRR3DX6NOX2RRYFRCUMSADGDESTDNBDS6CDVLGVESRTAC47",
              "destination_amount": "1.0000000",
              "path": []
            }
          ]
        }
    }"#
}
