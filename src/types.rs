/// Some Ethereum types represented as an array of bytes.
pub enum EthereumTypes {
    /// U160 - unsigned 160 bit number
    Address([u8; 20]),
    /// U256 - unsigned 256 bit number
    U256([u8; 32]),
}

impl EthereumTypes {
    #[inline]
    pub fn name_as_str(&self) -> &str {
        match self {
            Self::Address(_) => "address",
            Self::U256(_) => "uint256",
        }
    }

    #[inline]
    pub fn value_as_u256(&self) -> [u8; 32] {
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
    pub fn address_from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() <= 20, "Byte array doesn't fit into 160 bits");
        let mut val = [0_u8; 20];
        let diff = 20 - bytes.len();
        for i in diff..20 {
            val[i] = bytes[i - diff];
        }
        Self::Address(val)
    }

    #[inline]
    pub fn u256_from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() <= 32, "Byte array doesn't fit into 160 bits");
        let mut val = [0_u8; 32];
        let diff = 32 - bytes.len();
        for i in diff..32 {
            val[i] = bytes[i - diff];
        }
        Self::U256(val)
    }
}
