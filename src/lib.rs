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

    if !function_found {
        Err(format!(
            "Function name {} not found in the ABI json file.",
            function_name
        ))
    } else {
        let name = &functions[i]["name"];
        let mut j: usize = 0;
        let mut inputs = Vec::<&str>::new();
        // list all the inputs of the file
        while functions[i]["inputs"][j] != serde_json::Value::Null {
            if let Some(s) = functions[i]["inputs"][j]["type"].as_str() {
                inputs.push(s);
            } else {
                return Err(format!(
                    "Input type of function {} was not a String. ABI is not properly formatted.",
                    name
                ));
            }
            j += 1;
        }
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let path = Path::new("src/rust_abi.json");
        let function_name = "balanceOf";
        let arguments = vec![EthereumTypes::Address([0; 20])];
        let t = transaction(&path, function_name, arguments).unwrap();
        //let mut keccak = Keccak256::new();
        //keccak.update("balanceOf(address)");
        //println!("{:x?}", keccak.finalize());
        assert!(false);
    }
}

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
