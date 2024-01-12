use my_service_bus::shared::sub_page::SubPageId;

pub struct ActiveSubPages {
    pages: Vec<SubPageId>,
}

impl ActiveSubPages {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    pub fn add_if_not_exists(&mut self, sub_page_id: SubPageId) {
        if self.has_sub_page(sub_page_id) {
            return;
        }

        self.pages.push(sub_page_id);
    }

    pub fn has_sub_page(&self, page_id: SubPageId) -> bool {
        self.pages.contains(&page_id)
    }
}
