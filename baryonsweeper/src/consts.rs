pub struct ChallengeSecret {
   pub version: u8,
   pub secret: [u8;8],
}

pub struct ChallengeKey {
    pub version: u8,
    pub key: [u8;16]
}

pub const SERIALNO: [u8; 4] = [0xFF; 4];

pub const SECRETS1: [ChallengeSecret;15] = [
    ChallengeSecret {
        version: 0x00,
        secret: [0xD2, 0x07, 0x22, 0x53, 0xA4, 0xF2, 0x74, 0x68]
    },
    ChallengeSecret {
        version: 0x01,
        secret: [0xF5, 0xD7, 0xD4, 0xB5, 0x75, 0xF0, 0x8E, 0x4E]
    },
        ChallengeSecret {
        version: 0x02,
        secret: [0xB3, 0x7A, 0x16, 0xEF, 0x55, 0x7B, 0xD0, 0x89]
    },       
    ChallengeSecret {
        version: 0x03,
        secret: [0xCC, 0x69, 0x95, 0x81, 0xFD, 0x89, 0x12, 0x6C]
    },
    ChallengeSecret {
        version: 0x04,
        secret: [0xA0, 0x4E, 0x32, 0xBB, 0xA7, 0x13, 0x9E, 0x46]
    },                                                                  
    ChallengeSecret {
        version: 0x05,
        secret: [0x49, 0x5E, 0x03, 0x47, 0x94, 0x93, 0x1D, 0x7B]
    },
    ChallengeSecret {
        version: 0x06,
        secret: [0xB0, 0xB8, 0x09, 0x83, 0x39, 0x89, 0xFA, 0xE2]
    },
    ChallengeSecret {
        version: 0x08,
        secret: [0xAD, 0x40, 0x43, 0xB2, 0x56, 0xEB, 0x45, 0x8B]
    },
    ChallengeSecret {
        version: 0x0A,
        secret: [0xC2, 0x37, 0x7E, 0x8A, 0x74, 0x09, 0x6C, 0x5F]
    },
    ChallengeSecret {
        version: 0x0D,
        secret: [0x58, 0x1C, 0x7F, 0x19, 0x44, 0xF9, 0x62, 0x62]
    },
    ChallengeSecret {
        version: 0x2F,
        secret: [0xF1, 0xBC, 0x56, 0x2B, 0xD5, 0x5B, 0xB0, 0x77]
    },
    ChallengeSecret {
        version: 0x97,
        secret: [0xAF, 0x60, 0x10, 0xA8, 0x46, 0xF7, 0x41, 0xF3]
    },
    ChallengeSecret {
        version: 0xB3,
        secret: [0xD2, 0x07, 0x22, 0x53, 0xA4, 0xF2, 0x74, 0x68]
    },
    ChallengeSecret {
        version: 0xD9,
        secret: [0x90, 0xE1, 0xF0, 0xC0, 0x01, 0x78, 0xE3, 0xFF]
    },
    ChallengeSecret {
        version: 0xEB,
        secret: [0x0B, 0xD9, 0x02, 0x7E, 0x85, 0x1F, 0xA1, 0x23]
    },
];

pub const SECRETS2: [ChallengeSecret; 15] = [
    ChallengeSecret {
        version: 0x00,
        secret: [0xF4, 0xE0, 0x43, 0x13, 0xAD, 0x2E, 0xB4, 0xDB]
    },
    ChallengeSecret {
        version: 0x01,
        secret: [0xFE, 0x7D, 0x78, 0x99, 0xBF, 0xEC, 0x47, 0xC5]
    },
        ChallengeSecret {
        version: 0x02,
        secret: [0x86, 0x5E, 0x3E, 0xEF, 0x9D, 0xFB, 0xB1, 0xFD]
    },       
    ChallengeSecret {
        version: 0x03,
        secret: [0x30, 0x6F, 0x3A, 0x03, 0xD8, 0x6C, 0xBE, 0xE4]
    },
    ChallengeSecret {
        version: 0x04,
        secret: [0xFF, 0x72, 0xBD, 0x2B, 0x83, 0xB8, 0x9D, 0x2F]
    },                                                                  
    ChallengeSecret {
        version: 0x05,
        secret: [0x84, 0x22, 0xDF, 0xEA, 0xE2, 0x1B, 0x63, 0xC2]
    },
    ChallengeSecret {
        version: 0x06,
        secret: [0x58, 0xB9, 0x5A, 0xAE, 0xF3, 0x99, 0xDB, 0xD0]
    },
    ChallengeSecret {
        version: 0x08,
        secret: [0x67, 0xC0, 0x72, 0x15, 0xD9, 0x6B, 0x39, 0xA1]
    },
    ChallengeSecret {
        version: 0x0A,
        secret: [0x09, 0x3E, 0xC5, 0x19, 0xAF, 0x0F, 0x50, 0x2D]
    },
    ChallengeSecret {
        version: 0x0D,
        secret: [0x31, 0x80, 0x53, 0x87, 0x5C, 0x20, 0x3E, 0x24]
    },
    ChallengeSecret {
        version: 0x2F,
        secret: [0x1B, 0xDF, 0x24, 0x33, 0xEB, 0x29, 0x15, 0x5B]
    },
    ChallengeSecret {
        version: 0x97,
        secret: [0x9D, 0xEE, 0xC0, 0x11, 0x44, 0xB6, 0x6F, 0x41]
    },
    ChallengeSecret {
        version: 0xB3,
        secret: [0xE3, 0x2B, 0x8F, 0x56, 0xB2, 0x64, 0x12, 0x98]
    },
    ChallengeSecret {
        version: 0xD9,
        secret: [0xC3, 0x4A, 0x6A, 0x7B, 0x20, 0x5F, 0xE8, 0xF9]
    },
    ChallengeSecret {
        version: 0xEB,
        secret: [0xF7, 0x91, 0xED, 0x0B, 0x3F, 0x49, 0xA4, 0x48]
    },
];

pub const KEYS: [ChallengeKey; 15] = [
    ChallengeKey {
        version: 0x00,
        key: [0x5C, 0x52, 0xD9, 0x1C, 0xF3, 0x82, 0xAC, 0xA4, 0x89, 0xD8, 0x81, 0x78, 0xEC, 0x16, 0x29, 0x7B]
    },
    ChallengeKey {
        version: 0x01,
        key: [0x9D, 0x4F, 0x50, 0xFC, 0xE1, 0xB6, 0x8E, 0x12, 0x09, 0x30, 0x7D, 0xDB, 0xA6, 0xA5, 0xB5, 0xAA]
    },
    ChallengeKey {
        version: 0x02,
        key: [0x09, 0x75, 0x98, 0x88, 0x64, 0xAC, 0xF7, 0x62, 0x1B, 0xC0, 0x90, 0x9D, 0xF0, 0xFC, 0xAB, 0xFF]
    },
    ChallengeKey {
        version: 0x03,
        key: [0xC9, 0x11, 0x5C, 0xE2, 0x06, 0x4A, 0x26, 0x86, 0xD8, 0xD6, 0xD9, 0xD0, 0x8C, 0xDE, 0x30, 0x59]
    },
    ChallengeKey {
        version: 0x04,
        key: [0x66, 0x75, 0x39, 0xD2, 0xFB, 0x42, 0x73, 0xB2, 0x90, 0x3F, 0xD7, 0xA3, 0x9E, 0xD2, 0xC6, 0x0C]
    },
    ChallengeKey {
        version: 0x05,
        key: [0xF4, 0xFA, 0xEF, 0x20, 0xF4, 0xDB, 0xAB, 0x31, 0xD1, 0x86, 0x74, 0xFD, 0x8F, 0x99, 0x05, 0x66]
    },
    ChallengeKey {
        version: 0x06,
        key: [0xEA, 0x0C, 0x81, 0x13, 0x63, 0xD7, 0xE9, 0x30, 0xF9, 0x61, 0x13, 0x5A, 0x4F, 0x35, 0x2D, 0xDC]
    },
    ChallengeKey {
        version: 0x08,
        key: [0x0A, 0x2E, 0x73, 0x30, 0x5C, 0x38, 0x2D, 0x4F, 0x31, 0x0D, 0x0A, 0xED, 0x84, 0xA4, 0x18, 0x00]
    },
    ChallengeKey {
        version: 0x0A,
        key: [0xAC, 0x00, 0xC0, 0xE3, 0xE8, 0x0A, 0xF0, 0x68, 0x3F, 0xDD, 0x17, 0x45, 0x19, 0x45, 0x43, 0xBD]
    },
    ChallengeKey {
        version: 0x0D,
        key: [0xDF, 0xF3, 0xFC, 0xD6, 0x08, 0xB0, 0x55, 0x97, 0xCF, 0x09, 0xA2, 0x3B, 0xD1, 0x7D, 0x3F, 0xD2]
    },
    ChallengeKey {
        version: 0x2F,
        key: [0x4A, 0xA7, 0xC7, 0xB0, 0x11, 0x34, 0x46, 0x6F, 0xAC, 0x82, 0x16, 0x3E, 0x4B, 0xB5, 0x1B, 0xF9]
    },
    ChallengeKey {
        version: 0x97,
        key: [0xCA, 0xC8, 0xB8, 0x7A, 0xCD, 0x9E, 0xC4, 0x96, 0x90, 0xAB, 0xE0, 0x81, 0x39, 0x20, 0xB1, 0x10]
    },
    ChallengeKey {
        version: 0xB3,
        key: [0x03, 0xBE, 0xB6, 0x54, 0x99, 0x14, 0x04, 0x83, 0xBA, 0x18, 0x7A, 0x64, 0xEF, 0x90, 0x26, 0x1D]
    },
    ChallengeKey {
        version: 0xD9,
        key: [0xC7, 0xAC, 0x13, 0x06, 0xDE, 0xFE, 0x39, 0xEC, 0x83, 0xA1, 0x48, 0x3B, 0x0E, 0xE2, 0xEC, 0x89]
    },
    ChallengeKey {
        version: 0xEB,
        key: [0x41, 0x84, 0x99, 0xBE, 0x9D, 0x35, 0xA3, 0xB9, 0xFC, 0x6A, 0xD0, 0xD6, 0xF0, 0x41, 0xBB, 0x26]
    },
];

pub const GO_KEY1: [u8; 16] = [0xC6, 0x6E, 0x9E, 0xD6, 0xEC, 0xBC, 0xB1, 0x21, 0xB7, 0x46, 0x5D, 0x25, 0x03, 0x7D, 0x66, 0x46];

pub const GO_KEY2: [u8; 16] = [0xDA, 0x24, 0xDA, 0xB4, 0x3A, 0x61, 0xCB, 0xDF, 0x61, 0xFD, 0x25, 0x5D, 0x0A, 0xEA, 0x79, 0x57];

pub const GO_SECRET: [u8; 16] = [0x88, 0x0E, 0x2A, 0x94, 0x11, 0x09, 0x26, 0xB2, 0x0E, 0x53, 0xE2, 0x2A, 0xE6, 0x48, 0xAE, 0x9D];
