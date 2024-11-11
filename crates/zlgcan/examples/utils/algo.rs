use iso14229_1::UdsError;

#[allow(unused_variables)]

pub(crate) fn uds_security_algo(level: u8, seed: Vec<u8>, salt: Vec<u8>) -> Result<Option<Vec<u8>>, UdsError> {
    todo!()
}

#[test]
fn algo_check() -> anyhow::Result<()> {
    todo!()
}