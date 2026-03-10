use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Filled,
    Partial,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct VenueOrder {
    pub id: String,
    pub venue: String,
    pub status: OrderStatus,
}

impl VenueOrder {
    pub fn new(id: &str, venue: &str) -> Self {
        Self {
            id: id.to_string(),
            venue: venue.to_string(),
            status: OrderStatus::Pending,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderRequest {
    pub venue: String,
    pub instrument: String,
    pub size: f64,
    pub is_buy: bool,
}

impl OrderRequest {
    pub fn new(venue: &str, instrument: &str, size: f64, is_buy: bool) -> Self {
        Self {
            venue: venue.to_string(),
            instrument: instrument.to_string(),
            size,
            is_buy,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AtomicExecutionPlan {
    pub legs: Vec<OrderRequest>,
    pub cancel_on_disconnect: bool,
}

#[derive(Default)]
pub struct ExecutorState {
    orders: HashMap<String, VenueOrder>,
}

impl ExecutorState {
    pub fn track(&mut self, order: VenueOrder) {
        self.orders.insert(order.id.clone(), order);
    }

    pub fn update_status(&mut self, order_id: &str, status: OrderStatus) {
        if let Some(order) = self.orders.get_mut(order_id) {
            order.status = status;
        }
    }

    pub fn get(&self, order_id: &str) -> Option<&VenueOrder> {
        self.orders.get(order_id)
    }
}

pub fn calc_fee_aware_size(capital: f64, price: f64, fee_rate: f64) -> f64 {
    if price <= 0.0 {
        return 0.0;
    }
    capital / (price * (1.0 + fee_rate))
}

pub fn execute_atomic_pair(buy_leg: &OrderRequest, sell_leg: &OrderRequest) -> AtomicExecutionPlan {
    AtomicExecutionPlan {
        legs: vec![buy_leg.clone(), sell_leg.clone()],
        cancel_on_disconnect: true,
    }
}

pub fn next_retry_delay_ms(attempt: u32) -> u64 {
    let value = 500_u64.saturating_mul(2_u64.saturating_pow(attempt));
    value.min(30_000)
}
