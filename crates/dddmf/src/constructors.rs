struct String50(String);

fn create_string50(s: String) -> Option<String50> {
    (s.len() <= 50).then_some(String50(s))
}

struct OrderLineQty(u32);

fn create_order_line_qty(qty: u32) -> Option<OrderLineQty> {
    (1..100).contains(&qty).then_some(OrderLineQty(qty))
}
