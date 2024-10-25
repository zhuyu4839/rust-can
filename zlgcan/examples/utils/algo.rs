use ecu_uds::error::Error;

#[allow(unused_variables)]

pub(crate) fn uds_security_algo(level: u8, seed: Vec<u8>, salt: Vec<u8>) -> Result<Vec<u8>, Error> {
    todo!()
}

#[test]
fn algo_check() -> anyhow::Result<()> {
    todo!()
}