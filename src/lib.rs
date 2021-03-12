#![allow(unused)]
use sha3::{Digest, Keccak256};

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub enum EthereumTypes {
    /// U160 - unsigned 160 bit number
    Address([u8; 20]),
    /// U256 - unsigned 256 bit number
    U256([u8; 32]),
}

impl EthereumTypes {
    #[inline]
    fn name_as_str(&self) -> &str {
        match self {
            Self::Address(_) => "address",
            Self::U256(_) => "uint256",
        }
    }

    #[inline]
    fn value_as_u256(&self) -> [u8; 32] {
        match self {
            Self::Address(val) => {
                let mut extended = [0_u8; 32];
                // extend the 20 byte address by writing it to a 32 byte zero array
                for i in 12..32 {
                    extended[i] = val[i - 12];
                }
                extended
            }
            Self::U256(val) => *val,
        }
    }

    #[inline]
    fn address_from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() <= 20, "Byte array doesn't fit into 160 bits");
        let mut val = [0_u8; 20];
        let diff = 20 - bytes.len();
        for i in diff..20 {
            val[i] = bytes[i - diff];
        }
        Self::Address(val)
    }

    #[inline]
    fn u256_from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() <= 32, "Byte array doesn't fit into 160 bits");
        let mut val = [0_u8; 32];
        let diff = 32 - bytes.len();
        for i in diff..32 {
            val[i] = bytes[i - diff];
        }
        Self::U256(val)
    }
}

fn transaction(
    path_to_abi: &Path,
    function_name: &str,
    arguments: Vec<EthereumTypes>,
) -> Result<Vec<u8>, String> {
    let file = File::open(path_to_abi).map_err(|e| format!("Couldn't open file: {}", e))?;
    let reader = BufReader::new(file);
    let functions: serde_json::Value =
        serde_json::from_reader(reader).map_err(|e| format!("Couldn't parse json: {}", e))?;

    let mut i: usize = 0;
    let mut function_found: bool = false;

    // find the function name in the parsed json file
    while functions[i] != serde_json::Value::Null {
        if functions[i]["name"] == function_name {
            function_found = true;
            break;
        }
        i += 1;
    }

    // if the given function name was not found, return an error
    if !function_found {
        Err(format!(
            "Function name {} not found in the ABI json file.",
            function_name
        ))
    } else {
        let name = &functions[i]["name"];
        let mut inputs = Vec::<&str>::new();
        // list all the inputs of the file while iterating over input parameter list (lenght and types should match)
        for (j, arg) in arguments.iter().enumerate() {
            // if the j^th input type is a string, append it to the inputs
            if let Some(s) = functions[i]["inputs"][j]["type"].as_str() {
                // check whether the input arguments match such that we avoid the following example:
                // expected arguments: vec![Address, Address, U256]
                // but the given inputs: vec!["address", "uint256", "address"]
                if s != arg.name_as_str() {
                    return Err(format!(
                        "Input arguments doesn't match. Expected {}, found {}.",
                        s,
                        arg.name_as_str()
                    ));
                }
                inputs.push(s);
            } else {
                return Err(format!(
                    "Input type of function {} was not a String. ABI is not properly formatted.",
                    name
                ));
            }
        }

        // construct the complete function signature
        let mut signature = name.as_str().unwrap().to_owned() + "(";
        for inp in inputs.iter() {
            signature.push_str(inp);
            signature.push(',');
        }
        signature.pop(); // pop the last ',' character as it is not needed
        signature.push(')'); // pus the closing parenthesis

        // perform the keccak hashing
        let mut keccak = Keccak256::new();
        keccak.update(signature);

        // take the first 4 bytes representing the function signature
        let mut first_4_bytes = (&keccak.finalize()[0..4]).to_vec();

        // append the provided input parameters
        for arg in arguments {
            first_4_bytes.extend_from_slice(&arg.value_as_u256());
        }

        Ok(first_4_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn balance_of_test() {
        let path = Path::new("src/rust_abi.json");
        let function_name = "balanceOf";
        let arguments = vec![EthereumTypes::Address([
            0x30, 0xE7, 0xd7, 0xFf, 0xF8, 0x5C, 0x8d, 0x0E, 0x77, 0x51, 0x40, 0xb1, 0xaD, 0x93,
            0xC2, 0x30, 0xD5, 0x59, 0x52, 0x07,
        ])];
        let t = transaction(&path, function_name, arguments).unwrap();
        assert_eq!(
            t,
            vec![
                0x70, 0xa0, 0x82, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x30, 0xe7, 0xd7, 0xff, 0xf8, 0x5c, 0x8d, 0x0e, 0x77, 0x51, 0x40, 0xb1,
                0xad, 0x93, 0xc2, 0x30, 0xd5, 0x59, 0x52, 0x07
            ]
        );
    }

    #[test]
    fn transfer_test() {
        let path = Path::new("src/rust_abi.json");
        let function_name = "transfer";
        let arguments = vec![
            EthereumTypes::Address([
                0x30, 0xE7, 0xd7, 0xFf, 0xF8, 0x5C, 0x8d, 0x0E, 0x77, 0x51, 0x40, 0xb1, 0xaD, 0x93,
                0xC2, 0x30, 0xD5, 0x59, 0x52, 0x07,
            ]),
            EthereumTypes::u256_from_bytes(&20000000000_u64.to_be_bytes()),
        ];
        assert_eq!(
            transaction(&path, function_name, arguments).unwrap(),
            vec![
                0xa9, 0x05, 0x9c, 0xbb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x30, 0xe7, 0xd7, 0xff, 0xf8, 0x5c, 0x8d, 0x0e, 0x77, 0x51, 0x40, 0xb1,
                0xad, 0x93, 0xc2, 0x30, 0xd5, 0x59, 0x52, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xa8, 0x17, 0xc8, 0x00
            ]
        );
    }

    #[test]
    fn allowance_test() {
        let path = Path::new("src/rust_abi.json");
        let function_name = "allowance";
        let arguments = vec![
            EthereumTypes::Address([
                0x30, 0xE7, 0xd7, 0xFf, 0xF8, 0x5C, 0x8d, 0x0E, 0x77, 0x51, 0x40, 0xb1, 0xaD, 0x93,
                0xC2, 0x30, 0xD5, 0x59, 0x52, 0x07,
            ]),
            EthereumTypes::Address([
                0x81, 0xFb, 0xae, 0x3C, 0x69, 0x36, 0x24, 0xFE, 0xc9, 0xeF, 0x1a, 0x86, 0x62, 0x62,
                0x28, 0x98, 0x0b, 0xEB, 0x1C, 0x71,
            ]),
        ];
        assert_eq!(
            transaction(&path, function_name, arguments).unwrap(),
            vec![
                0xdd, 0x62, 0xed, 0x3e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x30, 0xe7, 0xd7, 0xff, 0xf8, 0x5c, 0x8d, 0x0e, 0x77, 0x51, 0x40, 0xb1,
                0xad, 0x93, 0xc2, 0x30, 0xd5, 0x59, 0x52, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x81, 0xfb, 0xae, 0x3c, 0x69, 0x36, 0x24, 0xfe,
                0xc9, 0xef, 0x1a, 0x86, 0x62, 0x62, 0x28, 0x98, 0x0b, 0xeb, 0x1c, 0x71
            ]
        );
    }
}

// NOTE TEST CASES
// balanceOf
// [0x30E7d7FfF85C8d0E775140b1aD93C230D5595207]
// 0x70a0823100000000000000000000000030e7d7fff85c8d0e775140b1ad93c230d5595207
//
// transfer
// [0x30E7d7FfF85C8d0E775140b1aD93C230D5595207, 20000000000]
// 0xa9059cbb00000000000000000000000030e7d7fff85c8d0e775140b1ad93c230d559520700000000000000000000000000000000000000000000000000000004a817c800
//
// allowance
// [0x30E7d7FfF85C8d0E775140b1aD93C230D5595207, 0x81Fbae3C693624FEc9eF1a86626228980bEB1C71]
// 0xdd62ed3e00000000000000000000000030e7d7fff85c8d0e775140b1ad93c230d559520700000000000000000000000081fbae3c693624fec9ef1a86626228980beb1c71
