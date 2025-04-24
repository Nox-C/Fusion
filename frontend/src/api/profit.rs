use gloo_net::http::Request;

pub async fn fetch_profit() -> Option<f64> {
    let resp = Request::get("/api/profit").send().await.ok()?;
    if resp.ok() {
        let profit: f64 = resp.json().await.ok()?;
        Some(profit)
    } else {
        None
    }
}
