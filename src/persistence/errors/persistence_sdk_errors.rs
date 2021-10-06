use my_service_bus_shared::page_compressor::CompressedPageReaderError;
use zip::result::ZipError;

use super::GrpcClientError;

#[derive(Debug)]
pub enum PersistenceError {
    ZipOperationError(ZipError),
    TonicError(tonic::Status),
    InvalidProtobufPayload(String),
    GrpcClientError(tonic::transport::Error),
    GrpcClientIsNotInitialized(String),
    CompressedPageReaderError(CompressedPageReaderError),
}

impl From<CompressedPageReaderError> for PersistenceError {
    fn from(src: CompressedPageReaderError) -> Self {
        Self::CompressedPageReaderError(src)
    }
}

impl From<GrpcClientError> for PersistenceError {
    fn from(src: GrpcClientError) -> Self {
        Self::GrpcClientError(src.err)
    }
}

impl From<tonic::Status> for PersistenceError {
    fn from(src: tonic::Status) -> Self {
        Self::TonicError(src)
    }
}

impl From<prost::DecodeError> for PersistenceError {
    fn from(src: prost::DecodeError) -> Self {
        Self::InvalidProtobufPayload(format!("{:?}", src))
    }
}

impl From<ZipError> for PersistenceError {
    fn from(src: ZipError) -> Self {
        Self::ZipOperationError(src)
    }
}
