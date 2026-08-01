use my_service_bus::shared::validators::DEFAULT_NAMESPACE;

use crate::topics::TopicsList;

/// A namespace owns its topics and shares nothing with the others: a topic name is
/// unique only inside one namespace, so `orders` in `default` and `orders` in
/// `alpha` are two different topics with independent messages, queues and cursors.
pub struct Namespace {
    pub name: String,
    pub topic_list: TopicsList,
}

impl Namespace {
    pub fn new(name: String) -> Self {
        Self {
            topic_list: TopicsList::new(name.clone()),
            name,
        }
    }

    pub fn is_default(&self) -> bool {
        self.name == DEFAULT_NAMESPACE
    }

    /// The namespace as the persistence contract wants it. The default one is sent
    /// as `None` — exactly what a node knowing nothing about namespaces sends — so
    /// an un-upgraded persistence keeps working and nothing has to be migrated.
    pub fn as_grpc_namespace(&self) -> Option<String> {
        if self.is_default() {
            None
        } else {
            Some(self.name.clone())
        }
    }
}
