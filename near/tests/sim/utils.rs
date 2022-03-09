/*
 * @Description:
 * @Author: kay
 * @Date: 2022-02-24 11:22:04
 * @LastEditTime: 2022-03-09 17:44:02
 * @LastEditors: kay
 */

use cross_chain::{Content, Message, Sqos};
use near_sdk::serde_json::json;
use near_sdk::{AccountId, PublicKey};
use near_sdk_sim::{init_simulator, to_yocto, UserAccount, DEFAULT_GAS};
use std::str::FromStr;

// Load in contract bytes at runtime
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    CC_WASM_BYTES => "../../dante-cross-chain/near/contract/cross_chain/res/cross_chain.wasm",
    VC_WASM_BYTES => "res/msg_verify.wasm",
    EC_WASM_BYTES => "res/node_evaluation.wasm",
}

const VC_ID: &str = "vc";
const EC_ID: &str = "ec";
const CC_ID: &str = "cc";

// pub fn create_message()
pub fn create_message() -> (Message, Message) {
    let message_1: Message = Message {
        from_chain: "OTHER_CHAIN".to_string(),
        to_chain: "NEAR_CHAIN".to_string(),
        sender: "OTHER_CHAIN_LOCKER".to_string(),
        signer: "OTHER_CHAIN_CALLER".to_string(),
        sqos: Sqos { reveal: false },
        content: Content {
            contract: "ft.shanks.testnet".to_string(),
            action: "ft_balance_of".to_string(),
            data: "{\"account_id\": \"shanks.testnet\"}".to_string(),
        }, // content: "alice".to_string(),
    };

    let message_2: Message = Message {
        from_chain: "OTHER_CHAIN".to_string(),
        to_chain: "NEAR_CHAIN".to_string(),
        sender: "OTHER_CHAIN_LOCKER".to_string(),
        signer: "OTHER_CHAIN_CALLER".to_string(),
        sqos: Sqos { reveal: false },
        content: Content {
            contract: "ft_shanks.testnet".to_string(),
            action: "ft_balance_of".to_string(),
            data: "{\"account_id\": \"other_account\"}".to_string(),
        }, // content: "alice".to_string(),
    };
    (message_1, message_2)
}

pub fn init_no_macros(
    credibility_weight_threshold: u32,
    initial_crediblity_value: u32,
) -> (UserAccount, UserAccount, UserAccount, UserAccount) {
    let root = init_simulator(None);
    let cc = root.deploy(&CC_WASM_BYTES, CC_ID.parse().unwrap(), to_yocto("2000"));
    cc.call(
        CC_ID.parse().unwrap(),
        "new",
        &json!({
          "owner": CC_ID.parse::<AccountId>().unwrap(),
          "verification_contract": VC_ID.parse::<AccountId>().unwrap(),
          "evaluation_contract": EC_ID.parse::<AccountId>().unwrap(),
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS / 2,
        0,
    )
    .assert_success();

    let vc = root.deploy(&VC_WASM_BYTES, VC_ID.parse().unwrap(), to_yocto("2000"));

    vc.call(
        VC_ID.parse().unwrap(),
        "init",
        &json!({
          "cross_contract_id": CC_ID.parse::<AccountId>().unwrap(),
          "node_eva_addr": EC_ID.parse::<AccountId>().unwrap(),
          "credibility_weight_threshold": credibility_weight_threshold,
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS / 2,
        0,
    )
    .assert_success();

    let ec = root.deploy(&EC_WASM_BYTES, EC_ID.parse().unwrap(), to_yocto("2000"));
    ec.call(
        EC_ID.parse().unwrap(),
        "inite",
        &json!({
          "cross_contract_id": CC_ID.parse::<AccountId>().unwrap(),
          "vc_contract_id": VC_ID.parse::<AccountId>().unwrap(),
          "initial_credibility_value": initial_crediblity_value,
          "max_trustworthy_ratio": 7000,
          "min_trustworthy_ratio": 2000,
          "min_seleted_threshold": 1000,
          "trustworthy_threshold": 3000,
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS / 2,
        0,
    )
    .assert_success();
    (root, cc, vc, ec)
}

pub fn register_validators(
    creater: &UserAccount,
    account_num: u32,
) -> (Vec<UserAccount>, Vec<PublicKey>) {
    let mut validators: Vec<UserAccount> = Vec::new();
    let mut validators_pk: Vec<PublicKey> = Vec::new();
    for num in 1..=account_num {
        let account_str = format!("validator{}", num);
        let validator = creater.create_user(AccountId::new_unchecked(account_str), to_yocto("10"));
        validator
            .call(
                EC_ID.parse::<AccountId>().unwrap(),
                "register_node",
                b"",
                near_sdk_sim::DEFAULT_GAS / 2,
                0,
            )
            .assert_success();
        let pk = format!("{}", validator.signer.public_key);
        validators.push(validator);
        validators_pk.push(PublicKey::from_str(&pk).unwrap());
    }
    (validators, validators_pk)
}

pub fn call_receive_message(messages: Vec<(&[UserAccount], &Message)>) {
    let id: u32 = 1;
    for (validators, message) in messages {
        for validator in validators {
            validator
                .call(
                    CC_ID.parse().unwrap(),
                    "receive_message",
                    &json!({"id": id, 
                            "from_chain": message.from_chain,
                            "to_chain": message.to_chain,
                            "sender": message.sender,
                            "signer": message.signer,
                            "sqos": message.sqos,
                            "content": message.content})
                    .to_string()
                    .into_bytes(),
                    DEFAULT_GAS,
                    0,
                )
                .assert_success();
        }
    }
}
