#![allow(warnings)]

use my_grpc_extensions::GrpcReadError;
use my_service_bus::shared::page_compressor::CompressedPageReaderError;
use my_service_bus::shared::zip::result::ZipError;

#[derive(Debug)]
pub enum PersistenceError {
    ZipOperationError(ZipError),
    TonicError(tonic::Status),
    InvalidProtobufPayload(String),
    CompressedPageReaderError(CompressedPageReaderError),
    Timeout(Option<tokio::time::error::Elapsed>),
    GrpcReadError(GrpcReadError),
}

impl From<GrpcReadError> for PersistenceError {
    fn from(src: GrpcReadError) -> Self {
        Self::GrpcReadError(src)
    }
}

impl From<tokio::time::error::Elapsed> for PersistenceError {
    fn from(src: tokio::time::error::Elapsed) -> Self {
        Self::Timeout(Some(src))
    }
}

impl From<CompressedPageReaderError> for PersistenceError {
    fn from(src: CompressedPageReaderError) -> Self {
        Self::CompressedPageReaderError(src)
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
