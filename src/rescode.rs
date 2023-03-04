use ux::u4;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ResultCode {
    NOERROR = 0,
    FORMERR = 1,
    SERVFAIL = 2,
    NXDOMAIN = 3,
    NOTIMP = 4,
    REFUSED = 5,
}

impl ResultCode {
    pub fn from_num(num: u4) -> ResultCode {
        match num.into() {
            0_u8 => ResultCode::NOERROR,
            1_u8 => ResultCode::FORMERR,
            2_u8 => ResultCode::SERVFAIL,
            3_u8 => ResultCode::NXDOMAIN,
            4_u8 => ResultCode::NOTIMP,
            5_u8 => ResultCode::REFUSED,
            _ => panic!("Invalid ResultCode: {}", num),
        }
    }
}