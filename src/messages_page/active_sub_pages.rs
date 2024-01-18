use my_service_bus::shared::sub_page::SubPageId;

pub struct ActiveSubPages {
    pages: Vec<SubPageId>,
}

impl ActiveSubPages {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    pub fn add_if_not_exists(&mut self, sub_page_id: SubPageId) {
        let index = self.pages.binary_search_by(|itm| itm.cmp(&sub_page_id));

        match index {
            Ok(_) => {}
            Err(index) => {
                self.pages.insert(index, sub_page_id);
            }
        }
    }

    pub fn has_sub_page(&self, sub_page_id: SubPageId) -> bool {
        let index = self.pages.binary_search_by(|itm| itm.cmp(&sub_page_id));
        index.is_ok()
    }
}
