use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: u32,
    pub page_size: u32,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
            sort_by: None,
            sort_order: Some("asc".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub page_size: u32,
    pub total_items: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationParams {
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.page_size
    }

    pub fn validate(&self, max_page_size: u32) -> Result<(), String> {
        if self.page == 0 {
            return Err("Page must be greater than 0".to_string());
        }
        if self.page_size == 0 {
            return Err("Page size must be greater than 0".to_string());
        }
        if self.page_size > max_page_size {
            return Err(format!("Page size cannot exceed {}", max_page_size));
        }
        Ok(())
    }
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, params: &PaginationParams, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (params.page_size as f64)).ceil() as u32;
        let has_next = params.page < total_pages;
        let has_prev = params.page > 1;

        Self {
            data,
            pagination: PaginationInfo {
                page: params.page,
                page_size: params.page_size,
                total_items,
                total_pages,
                has_next,
                has_prev,
            },
        }
    }
}