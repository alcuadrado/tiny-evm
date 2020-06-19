extern crate tiny_evm;

use tiny_evm::{run, BlockContext, Bytecode, CallContext};

use ethereum_types::{Address, U256};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

type TestFile = HashMap<String, VmTest>;

const SKIPPED_TEST_NAMES: &'static [&'static str] = &[
    // Expected OOG
    "return1",
    "mloadOutOfGasError2",
    "jump0_foreverOutOfGas",
    "DynamicJump0_foreverOutOfGas",
    "BlockNumberDynamicJump0_foreverOutOfGas",
    "sha3_3",
    "sha3MemExp",
    "sha3_4",
    // Use GAS instruction
    "gas1",
    "gas0",
    // Use logs
    "log_2logs",
    "log0_emptyMem",
    "log0_logMemsizeTooHigh",
    "log0_logMemsizeZero",
    "log0_logMemStartTooHigh",
    "log0_nonEmptyMem_logMemSize1_logMemStart31",
    "log0_nonEmptyMem_logMemSize1",
    "log0_nonEmptyMem",
    "log1_Caller",
    "log1_emptyMem",
    "log1_logMemsizeTooHigh",
    "log1_logMemsizeZero",
    "log1_logMemStartTooHigh",
    "log1_MaxTopic",
    "log1_nonEmptyMem_logMemSize1_logMemStart31",
    "log1_nonEmptyMem_logMemSize1",
    "log1_nonEmptyMem",
    "log2_Caller",
    "log2_emptyMem",
    "log2_logMemsizeTooHigh",
    "log2_logMemsizeZero",
    "log2_logMemStartTooHigh",
    "log2_MaxTopic",
    "log2_nonEmptyMem_logMemSize1_logMemStart31",
    "log2_nonEmptyMem_logMemSize1",
    "log2_nonEmptyMem",
    "log3_Caller",
    "log3_emptyMem",
    "log3_logMemsizeTooHigh",
    "log3_logMemsizeZero",
    "log3_logMemStartTooHigh",
    "log3_MaxTopic",
    "log3_nonEmptyMem_logMemSize1_logMemStart31",
    "log3_nonEmptyMem_logMemSize1",
    "log3_nonEmptyMem",
    "log3_PC",
    "log4_Caller",
    "log4_emptyMem",
    "log4_logMemsizeTooHigh",
    "log4_logMemsizeZero",
    "log4_logMemStartTooHigh",
    "log4_MaxTopic",
    "log4_nonEmptyMem_logMemSize1_logMemStart31",
    "log4_nonEmptyMem_logMemSize1",
    "log4_nonEmptyMem",
    "log4_PC",
];

#[derive(Debug, Serialize, Deserialize)]
struct VmTest {
    only: Option<bool>,

    skip: Option<bool>,

    env: VMTestEnv,

    exec: VmTestExec,

    #[serde(default)]
    #[serde(deserialize_with = "option_buffer_from_hex")]
    out: Option<Vec<u8>>,

    pre: HashMap<String, VmTestAccount>,

    post: Option<HashMap<String, VmTestAccount>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VMTestEnv {
    #[serde(rename = "currentCoinbase")]
    #[serde(deserialize_with = "address_from_hex")]
    current_coinbase: Address,

    #[serde(rename = "currentDifficulty")]
    #[serde(deserialize_with = "u32_from_hex")]
    current_difficulty: u32,

    #[serde(rename = "currentGasLimit")]
    #[serde(deserialize_with = "u256_from_hex")]
    current_gas_limit: U256,

    #[serde(rename = "currentNumber")]
    #[serde(deserialize_with = "u32_from_hex")]
    current_number: u32,

    #[serde(rename = "currentTimestamp")]
    #[serde(deserialize_with = "u32_from_hex")]
    current_timestamp: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct VmTestExec {
    #[serde(deserialize_with = "address_from_hex")]
    address: Address,

    #[serde(deserialize_with = "address_from_hex")]
    caller: Address,

    #[serde(deserialize_with = "buffer_from_hex")]
    code: Vec<u8>,

    #[serde(deserialize_with = "buffer_from_hex")]
    data: Vec<u8>,

    #[serde(deserialize_with = "u256_from_hex")]
    gas: U256,

    #[serde(rename = "gasPrice")]
    #[serde(deserialize_with = "u256_from_hex")]
    gas_price: U256,

    #[serde(deserialize_with = "address_from_hex")]
    origin: Address,

    #[serde(deserialize_with = "u256_from_hex")]
    value: U256,
}

#[derive(Debug, Serialize, Deserialize)]
struct VmTestAccount {
    #[serde(deserialize_with = "u256_from_hex")]
    balance: U256,

    #[serde(deserialize_with = "buffer_from_hex")]
    code: Vec<u8>,

    #[serde(deserialize_with = "u32_from_hex")]
    nonce: u32,

    storage: HashMap<String, String>,
}

fn buffer_from_hex<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    hex::decode(&s[2..]).map_err(|_| serde::de::Error::custom("Invalid hex value"))
}

fn option_buffer_from_hex<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let maybe_str: Option<&str> = Deserialize::deserialize(deserializer)?;
    if maybe_str.is_none() {
        return Ok(None);
    }

    let s = maybe_str.unwrap();

    hex::decode(&s[2..])
        .map(Some)
        .map_err(|_| serde::de::Error::custom("Invalid hex value"))
}

fn u256_from_hex<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let buffer = buffer_from_hex(deserializer)?;
    Ok(U256::from_big_endian(buffer.as_slice()))
}

fn u32_from_hex<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(u256_from_hex(deserializer)?.as_u32())
}

fn address_from_hex<'de, D>(deserializer: D) -> Result<Address, D::Error>
where
    D: Deserializer<'de>,
{
    let buffer = buffer_from_hex(deserializer)?;
    let mut address = Address::zero();
    address.assign_from_slice(&buffer[0..20]);
    Ok(address)
}

#[test]
fn run_official_tests() {
    let files = get_dir_files(&PathBuf::from("ethereum-tests/VMTests"));

    let mut passed = 0;
    let mut skipped = 0;

    for file in files {
        if file.starts_with("ethereum-tests/VMTests/vmPerformance") {
            skipped += 1;
            continue;
        }

        let test_file = read_test_file(&file);

        run_test_file(&test_file, &mut passed, &mut skipped);
    }

    println!("{:} tests passed", passed);
    println!("{:} tests skipped", skipped);
}

fn get_dir_files(path: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(&path).unwrap() {
        let entry_path = entry.unwrap().path();

        if entry_path.is_file() {
            files.push(entry_path);
        } else {
            let mut descendants = get_dir_files(&entry_path);
            files.append(&mut descendants);
        }
    }

    files
}

fn read_test_file(file_path: &PathBuf) -> TestFile {
    let file_content = fs::read_to_string(file_path).unwrap();
    serde_json::from_str(&file_content).unwrap()
}

fn run_test_file(test_file: &TestFile, passed: &mut u32, skipped: &mut u32) {
    for (test_name, test) in test_file {
        if SKIPPED_TEST_NAMES.contains(&&test_name[..]) {
            println!("  Skipping test {}", test_name);
            *skipped += 1;
            continue;
        }

        if test.pre.len() > 1 {
            println!(
                "  Skipping test {} because it includes multiple accounts in pre",
                test_name
            );
            *skipped += 1;
            continue;
        }

        let address_str = format!("0x{:x}", &test.exec.address);

        if test.pre.len() == 1 && !test.pre.contains_key(&address_str) {
            println!(
                "  Skipping test {} because it includes another account in pre",
                test_name
            );
            *skipped += 1;
            continue;
        }

        if test.pre.get(&address_str).unwrap().storage.len() > 0 {
            println!(
                "  Skipping test {} because it includes values in the account's pre",
                test_name
            );
            *skipped += 1;
            continue;
        }

        println!("Running test {}", test_name);
        let block_context = BlockContext {
            difficulty: test.env.current_difficulty,
            number: test.env.current_number,
            timestamp: test.env.current_timestamp,
            coinbase_address: test.env.current_coinbase,
            gas_limit: test.env.current_gas_limit,
            chain_id: 0,
        };

        let call_context = CallContext {
            value: test.exec.value,
            calldata: test.exec.data.as_slice(),
            contract_address: test.exec.address,
            caller_address: test.exec.caller,
            origin_address: test.exec.origin,
            gas_price: test.exec.gas_price,
        };

        let bytecode = Bytecode::new(test.exec.code.as_slice());

        let result = run(&bytecode, &call_context, &block_context);

        if test.out.is_some() {
            assert_eq!(result.error, None);
            assert_eq!(result.return_data, *test.out.as_ref().unwrap());
        } else {
            assert_ne!(result.error, None);
        }

        *passed += 1;
    }
}
