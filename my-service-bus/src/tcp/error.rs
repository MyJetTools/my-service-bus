#![allow(warnings)]
use my_tcp_sockets::socket_reader::ReadingTcpContractFail;

use crate::operations::OperationFailResult;

#[derive(Debug)]
pub enum MySbSocketError {
    ReadingTcpContractFail(ReadingTcpContractFail),
    OperationFailResult(OperationFailResult),
}

impl From<ReadingTcpContractFail> for MySbSocketError {
    fn from(src: ReadingTcpContractFail) -> Self {
        Self::ReadingTcpContractFail(src)
    }
}

impl From<OperationFailResult> for MySbSocketError {
    fn from(src: OperationFailResult) -> Self {
        Self::OperationFailResult(src)
    }
}
