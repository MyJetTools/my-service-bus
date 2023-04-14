use my_service_bus_shared::protobuf_models::MessageProtobufModel;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessagesProtobufModel {
    #[prost(message, repeated, tag = "1")]
    pub messages: Vec<MessageProtobufModel>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewMessagesProtobufContract {
    #[prost(string, tag = "1")]
    pub topic_id: ::prost::alloc::string::String,

    #[prost(repeated, message, tag = "2")]
    pub messages: Vec<MessageProtobufModel>,
}

impl NewMessagesProtobufContract {
    pub fn into_protobuf_vec(&self) -> Vec<u8> {
        let mut payload: Vec<u8> = Vec::new();
        prost::Message::encode(self, &mut payload).unwrap();
        payload
    }
}
