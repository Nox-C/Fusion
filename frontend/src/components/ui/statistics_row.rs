use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StatisticsRowProps {
    pub total_profit: String,
    pub arbitrages_found: u32,
    pub largest_spread: String,
    pub gas_used: String,
}

#[function_component(StatisticsRow)]
pub fn statistics_row(props: &StatisticsRowProps) -> Html {
    use yew::prelude::*;
    use gloo_timers::callback::Interval;

    let profit = props.total_profit.replace(",", "").parse::<f64>().unwrap_or(0.0);
    let arbitrages = props.arbitrages_found as f64;
    let spread = props.largest_spread.replace("%", "").parse::<f64>().unwrap_or(0.0);
    let gas = props.gas_used.replace(",", "").parse::<f64>().unwrap_or(0.0);

    let animated_profit = use_state(|| 0.0);
    let animated_arbs = use_state(|| 0.0);
    let animated_spread = use_state(|| 0.0);
    let animated_gas = use_state(|| 0.0);

    // Animate counters (one-time, on mount)
    {
        let animated_profit = animated_profit.clone();
        let animated_arbs = animated_arbs.clone();
        let animated_spread = animated_spread.clone();
        let animated_gas = animated_gas.clone();
        use_effect(move || {
            let mut i = 0;
            let animated_profit = animated_profit.clone();
            let animated_arbs = animated_arbs.clone();
            let animated_spread = animated_spread.clone();
            let animated_gas = animated_gas.clone();
            let interval = Interval::new(12, move || {
                i += 1;
                if i <= 80 {
                    animated_profit.set(profit * i as f64 / 80.0);
                    animated_arbs.set(arbitrages * i as f64 / 80.0);
                    animated_spread.set(spread * i as f64 / 80.0);
                    animated_gas.set(gas * i as f64 / 80.0);
                }
            });
            move || drop(interval)
        });
    }

    html! {
        <div class="fusion-hud-stats grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <div class="hud-widget glassmorph-neon flex flex-col items-center justify-center relative">
                <svg width="90" height="90" viewBox="0 0 90 90" class="absolute top-0 left-0 animate-spin-slow" style="z-index:0"><circle cx="45" cy="45" r="40" stroke="#00fff7" stroke-width="3" fill="none" opacity="0.25"/><circle cx="45" cy="45" r="35" stroke="#ae00ff" stroke-width="2" fill="none" opacity="0.5"/></svg>
                <span class="neon-text text-lg font-bold mt-4 z-10">{"Total Profit"}</span>
                <span class="text-3xl font-extrabold neon-text z-10">{format!("${:.2}", *animated_profit)}</span>
            </div>
            <div class="hud-widget glassmorph-neon flex flex-col items-center justify-center relative">
                <svg width="90" height="90" viewBox="0 0 90 90" class="absolute top-0 left-0 animate-spin-rev" style="z-index:0"><circle cx="45" cy="45" r="40" stroke="#00fff7" stroke-width="3" fill="none" opacity="0.25"/><circle cx="45" cy="45" r="35" stroke="#ae00ff" stroke-width="2" fill="none" opacity="0.5"/></svg>
                <span class="neon-text text-lg font-bold mt-4 z-10">{"Arbitrages"}</span>
                <span class="text-3xl font-extrabold neon-text z-10">{format!("{:.0}", *animated_arbs)}</span>
            </div>
            <div class="hud-widget glassmorph-neon flex flex-col items-center justify-center relative">
                <svg width="90" height="90" viewBox="0 0 90 90" class="absolute top-0 left-0 animate-spin-slow" style="z-index:0"><circle cx="45" cy="45" r="40" stroke="#00fff7" stroke-width="3" fill="none" opacity="0.25"/><circle cx="45" cy="45" r="35" stroke="#ae00ff" stroke-width="2" fill="none" opacity="0.5"/></svg>
                <span class="neon-text text-lg font-bold mt-4 z-10">{"Largest Spread"}</span>
                <span class="text-3xl font-extrabold neon-text z-10">{format!("{:.2}%", *animated_spread)}</span>
            </div>
            <div class="hud-widget glassmorph-neon flex flex-col items-center justify-center relative">
                <svg width="90" height="90" viewBox="0 0 90 90" class="absolute top-0 left-0 animate-spin-rev" style="z-index:0"><circle cx="45" cy="45" r="40" stroke="#00fff7" stroke-width="3" fill="none" opacity="0.25"/><circle cx="45" cy="45" r="35" stroke="#ae00ff" stroke-width="2" fill="none" opacity="0.5"/></svg>
                <span class="neon-text text-lg font-bold mt-4 z-10">{"Gas Used"}</span>
                <span class="text-3xl font-extrabold neon-text z-10">{format!("{:.0}", *animated_gas)}</span>
            </div>
            <style>{r#"
                .glassmorph-neon { backdrop-filter: blur(12px) saturate(180%); background: rgba(24,28,32,0.85); border-radius: 1.5rem; border: 1.5px solid #00fff7; box-shadow: 0 0 24px 4px #00fff7, 0 0 64px 8px #ae00ff33; position:relative; min-height:140px; margin-bottom:0.5rem; }
                .hud-widget { min-width:180px; min-height:160px; transition: box-shadow 0.2s, transform 0.2s; will-change: box-shadow, transform; }
                .hud-widget:hover { box-shadow: 0 0 48px 8px #00fff7, 0 0 96px 16px #ae00ff55; transform: scale(1.035); }
                .animate-spin-slow { animation: spin 12s linear infinite; }
                .animate-spin-rev { animation: spin-rev 16s linear infinite; }
                @keyframes spin { 100% { transform: rotate(360deg); } }
                @keyframes spin-rev { 100% { transform: rotate(-360deg); } }
            "#}</style>
        </div>
    }
}
