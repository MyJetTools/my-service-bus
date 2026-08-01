use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use my_service_bus::shared::validators::{
    validate_namespace_name, InvalidNamespaceName, DEFAULT_NAMESPACE,
};

use super::Namespace;

/// A node carries single digits of namespaces, so a lookup is a linear scan over a
/// snapshot: readers never take a lock at all — they clone an `Arc` and walk two or
/// three entries. The mutex is held only while a new snapshot is being published,
/// which happens the first time a namespace is mentioned and never again.
pub struct NamespacesList {
    inner: ArcSwap<Vec<Arc<Namespace>>>,
    write_lock: Mutex<()>,
}

impl NamespacesList {
    pub fn new() -> Self {
        // The default namespace exists from the start: it is where every
        // pre-namespace client works, and resolving it must never fail.
        let default = Arc::new(Namespace::new(DEFAULT_NAMESPACE.to_string()));

        Self {
            inner: ArcSwap::from_pointee(vec![default]),
            write_lock: Mutex::new(()),
        }
    }

    pub fn get(&self, name: &str) -> Option<Arc<Namespace>> {
        self.inner
            .load()
            .iter()
            .find(|itm| itm.name == name)
            .cloned()
    }

    pub fn get_default(&self) -> Arc<Namespace> {
        self.get(DEFAULT_NAMESPACE)
            .expect("Default namespace is created together with the list")
    }

    pub fn get_all(&self) -> Arc<Vec<Arc<Namespace>>> {
        self.inner.load_full()
    }

    /// Returns the namespace, creating it if this is the first time it is
    /// mentioned. A namespace is a client-owned name, not something an admin has to
    /// register first — the same way a topic is.
    pub fn get_or_create(&self, name: &str) -> Result<Arc<Namespace>, InvalidNamespaceName> {
        if let Some(result) = self.get(name) {
            return Ok(result);
        }

        validate_namespace_name(name)?;

        let _guard = self.write_lock.lock().unwrap();

        // Somebody could have created it while we were waiting for the lock.
        if let Some(result) = self.get(name) {
            return Ok(result);
        }

        let namespace = Arc::new(Namespace::new(name.to_string()));

        let mut new_list = self.inner.load().as_ref().clone();
        new_list.push(namespace.clone());

        self.inner.store(Arc::new(new_list));

        Ok(namespace)
    }

    /// `None` — and an empty string — mean the default namespace: that is what every
    /// request which does not mention one carries.
    pub fn get_or_create_optional(
        &self,
        name: Option<&str>,
    ) -> Result<Arc<Namespace>, InvalidNamespaceName> {
        match name {
            Some(name) if !name.is_empty() => self.get_or_create(name),
            _ => Ok(self.get_default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_namespace_exists_from_the_start() {
        let namespaces = NamespacesList::new();

        assert_eq!(DEFAULT_NAMESPACE, namespaces.get_default().name.as_str());
        assert_eq!(1, namespaces.get_all().len());
    }

    #[test]
    fn test_namespace_is_created_on_first_mention_and_reused_afterwards() {
        let namespaces = NamespacesList::new();

        let first = namespaces.get_or_create("alpha").unwrap();
        let second = namespaces.get_or_create("alpha").unwrap();

        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(2, namespaces.get_all().len());
    }

    #[test]
    fn test_invalid_name_is_an_error_and_creates_nothing() {
        let namespaces = NamespacesList::new();

        assert_eq!(true, namespaces.get_or_create("Alpha").is_err());
        assert_eq!(1, namespaces.get_all().len());
    }

    #[test]
    fn test_no_name_resolves_to_the_default_namespace() {
        let namespaces = NamespacesList::new();

        assert_eq!(
            DEFAULT_NAMESPACE,
            namespaces.get_or_create_optional(None).unwrap().name
        );
        assert_eq!(
            DEFAULT_NAMESPACE,
            namespaces.get_or_create_optional(Some("")).unwrap().name
        );
        assert_eq!(1, namespaces.get_all().len());
    }
}
