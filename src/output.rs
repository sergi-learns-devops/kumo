use crate::models::{committed_rates, spot_rates, Environment, Recommendation, ServiceType, UserRequest, WorkloadType, CATALOG_META};
use std::env;
use std::time::Instant;

struct Colors {
    green: &'static str,
    cyan: &'static str,
    yellow: &'static str,
    red: &'static str,
    bold: &'static str,
    dim: &'static str,
    reset: &'static str,
}

const COLORS_ON: Colors = Colors {
    green: "\x1b[32m",
    cyan: "\x1b[36m",
    yellow: "\x1b[33m",
    red: "\x1b[31m",
    bold: "\x1b[1m",
    dim: "\x1b[2m",
    reset: "\x1b[0m",
};

const COLORS_OFF: Colors = Colors {
    green: "",
    cyan: "",
    yellow: "",
    red: "",
    bold: "",
    dim: "",
    reset: "",
};

fn get_colors() -> &'static Colors {
    if env::var("NO_COLOR").is_ok() { &COLORS_OFF } else { &COLORS_ON }
}

fn use_emoji() -> bool {
    env::var("NO_COLOR").is_err()
}

fn fmt_vcpu(vcpus: f32) -> String {
    if (vcpus - vcpus.round()).abs() < 0.01 && vcpus >= 1.0 {
        format!("{}", vcpus as u32)
    } else {
        format!("{:.1}", vcpus)
    }
}

fn env_badge(env: &Environment, c: &Colors) -> String {
    if env::var("NO_COLOR").is_ok() {
        match env {
            Environment::Production => "Production [!]".to_string(),
            Environment::PreProduction => "Pre-Prod [~]".to_string(),
            Environment::Test => "Test [t]".to_string(),
            Environment::Dev => "Dev [d]".to_string(),
        }
    } else {
        match env {
            Environment::Production => format!("{}{}Production 🔴{}", c.bold, c.red, c.reset),
            Environment::PreProduction => format!("{}{}Pre-Prod 🟡{}", c.bold, c.yellow, c.reset),
            Environment::Test => format!("{}{}Test 🔵{}", c.bold, c.cyan, c.reset),
            Environment::Dev => format!("{}{}Dev 🟢{}", c.bold, c.green, c.reset),
        }
    }
}

pub fn display(recommendation: &Recommendation, request: &UserRequest, start: Instant) {
    let c = get_colors();
    let emoji = use_emoji();
    let cloud_icon = if emoji { "☁" } else { "" };

    // Header
    println!();
    let service_label = match request.service {
        ServiceType::Vm => format!("{}", request.cloud),
        ServiceType::Database => match request.cloud {
            crate::models::CloudProvider::Gcp => "Cloud SQL".to_string(),
            crate::models::CloudProvider::Aws => "RDS".to_string(),
            crate::models::CloudProvider::Azure => "Azure Database".to_string(),
        },
    };
    println!(
        "  {}{} kumo{} — {}{}{} {}{}{}    {}",
        c.bold, cloud_icon, c.reset,
        c.bold, c.cyan, service_label,
        c.dim, request.region, c.reset,
        env_badge(&request.env, c)
    );
    println!(
        "  {}╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌{}",
        c.dim, c.reset
    );

    // Request summary
    print!(
        "  {}Request:{} {}{}{} vCPUs │ {}{:.1}{} GB RAM",
        c.dim, c.reset, c.bold, fmt_vcpu(request.vcpus), c.reset, c.bold, request.ram_gb, c.reset
    );
    if request.service == ServiceType::Database {
        if let Some(engine) = &request.db_engine {
            print!(" │ {}DB: {}{}", c.cyan, engine, c.reset);
        }
    }
    if let Some(ref family) = request.family {
        print!(" │ Family: {}{}{}", c.cyan, family, c.reset);
    }
    if request.workload != WorkloadType::General && request.service != ServiceType::Database {
        print!(" │ {}{}{}", c.yellow, request.workload, c.reset);
    }
    if request.spot {
        print!(" │ {}Spot{}", c.yellow, c.reset);
    }
    if let Some(net) = request.min_network_gbps {
        print!(" │ {}≥{:.0} Gbps{}", c.cyan, net, c.reset);
    }
    println!();
    println!();

    // Side-by-side table
    if recommendation.closest_matches.is_empty() {
        println!("  {}No instances found matching your criteria.{}", c.yellow, c.reset);
    } else {
        let rates = committed_rates(request.cloud);
        let show_gpu = request.workload == WorkloadType::Ai;

        // Table header
        if show_gpu {
            println!(
                "  {}{:<26} {:>5} {:>6} {:>14} {:>10} {:>10}   {:>5}{}",
                c.dim, "Instance", "vCPUs", "RAM", "GPU", "On-Demand", "1yr CUD", "Score", c.reset
            );
        } else {
            println!(
                "  {}{:<26} {:>5} {:>6} {:>10} {:>10} {:>10}   {:>5}{}",
                c.dim, "Instance", "vCPUs", "RAM", "On-Demand", "1yr CUD", "3yr CUD", "Score", c.reset
            );
        }
        println!(
            "  {}{}{}",
            c.dim,
            "─".repeat(if show_gpu { 86 } else { 82 }),
            c.reset
        );

        for (i, scored) in recommendation.closest_matches.iter().enumerate() {
            let inst = scored.instance;
            let monthly = inst.cost_hour_usd * 730.0;
            let monthly_1yr = monthly * (1.0 - rates.one_year_discount);
            let monthly_3yr = monthly * (1.0 - rates.three_year_discount);
            let pct = (scored.score * 100.0).min(100.0).max(0.0);

            let is_exact = recommendation.exact_match.is_some()
                && recommendation.exact_match.unwrap().name == inst.name;

            let name_fmt = if i == 0 {
                format!("{}{}", c.bold, c.cyan)
            } else {
                c.cyan.to_string()
            };

            let exact_marker = if is_exact { " ◉" } else { "" };
            let score_color = if pct >= 80.0 { c.green } else if pct >= 50.0 { c.yellow } else { c.dim };

            if show_gpu {
                let gpu_str = match &inst.gpu {
                    Some(gpu) => format!("{}x {} {}GB", gpu.count, gpu.model, gpu.vram_gb),
                    None => "—".to_string(),
                };
                println!(
                    "  {}{:<26}{} {:>5} {:>4} GB {:>14} {:>10} {:>10}   {}{:>4.0}%{}{}",
                    name_fmt, inst.name, c.reset,
                    fmt_vcpu(inst.vcpus),
                    inst.ram_gb as u32,
                    gpu_str,
                    format!("${:.0}/mo", monthly),
                    format!("${:.0}/mo", monthly_1yr),
                    score_color, pct, c.reset,
                    exact_marker,
                );
            } else {
                println!(
                    "  {}{:<26}{} {:>5} {:>4} GB {:>10} {:>10} {:>10}   {}{:>4.0}%{}{}",
                    name_fmt, inst.name, c.reset,
                    fmt_vcpu(inst.vcpus),
                    inst.ram_gb as u32,
                    format!("${:.0}/mo", monthly),
                    format!("${:.0}/mo", monthly_1yr),
                    format!("${:.0}/mo", monthly_3yr),
                    score_color, pct, c.reset,
                    exact_marker,
                );
            }
        }

        println!(
            "  {}{}{}",
            c.dim,
            "─".repeat(if show_gpu { 86 } else { 82 }),
            c.reset
        );

        // Savings summary
        if let Some(first) = recommendation.closest_matches.first() {
            let monthly = first.instance.cost_hour_usd * 730.0;
            let save_1yr = monthly * rates.one_year_discount;
            let save_3yr = monthly * rates.three_year_discount;
            println!(
                "  {}💰 Savings vs on-demand (#1): 1yr {}−${:.0}/mo (−{:.0}%){} │ 3yr {}−${:.0}/mo (−{:.0}%){}{}",
                c.dim,
                c.green, save_1yr, rates.one_year_discount * 100.0, c.dim,
                c.green, save_3yr, rates.three_year_discount * 100.0, c.reset,
                c.reset,
            );

            // Spot pricing line (if --spot)
            if request.spot {
                let srates = spot_rates(request.cloud);
                let spot_monthly = monthly * (1.0 - srates.discount);
                let spot_hr = first.instance.cost_hour_usd * (1.0 - srates.discount);
                let spot_label = match request.cloud {
                    crate::models::CloudProvider::Gcp => "Spot/Preemptible",
                    _ => "Spot",
                };
                println!(
                    "  {}⚡ {} (#1): {}${:.0}/mo{} {}(${:.4}/hr, −{:.0}%){} {}— interruptible, best for batch/dev{}",
                    c.dim,
                    spot_label,
                    c.yellow, spot_monthly, c.reset,
                    c.dim, spot_hr, srates.discount * 100.0, c.reset,
                    c.dim, c.reset,
                );
            }
        }

        // Explain breakdown (if --explain)
        if request.explain {
            print_explain(recommendation, c);
        }
    }

    // Custom machine option
    if let Some(custom) = &recommendation.custom_option {
        let custom_icon = if emoji { "🔧" } else { "[c]" };
        let monthly = custom.estimated_cost_hour * 730.0;
        println!();
        println!(
            "  {} {}Custom:{} {} │ {} vCPU │ {:.0} GB │ ~${:.0}/mo",
            custom_icon, c.bold, c.reset, custom.name, fmt_vcpu(custom.vcpus), custom.ram_gb, monthly
        );
    }

    // Separator
    println!();
    println!(
        "  {}╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌{}",
        c.dim, c.reset
    );

    // Tips
    if !recommendation.notes.is_empty() {
        let tip_icon = if emoji { "💡" } else { "(i)" };
        for note in &recommendation.notes {
            println!("  {} {}{}{}", tip_icon, c.dim, note, c.reset);
        }
    }

    // Disclaimer + time
    let warn_icon = if emoji { "⚠" } else { "(!)" };
    println!(
        "  {}  {}Prices from {} · CUD rates are approximate averages{}",
        warn_icon, c.dim, CATALOG_META.last_updated, c.reset
    );
    println!(
        "  {}╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌{}",
        c.dim, c.reset
    );
    let time_icon = if emoji { "⚡" } else { "" };
    let elapsed = start.elapsed();
    println!(
        "  {}Done in {:.0}ms {}{}",
        c.dim, elapsed.as_secs_f64() * 1000.0, time_icon, c.reset
    );
    println!();
}

/// Print the scoring breakdown for each recommendation (--explain mode)
fn print_explain(recommendation: &Recommendation, c: &Colors) {
    println!();
    println!("  {}🔍 Scoring breakdown (score = 1.0 − weighted penalties):{}", c.bold, c.reset);
    for (i, scored) in recommendation.closest_matches.iter().enumerate() {
        let b = &scored.breakdown;
        let pct = (scored.score * 100.0).min(100.0).max(0.0);
        println!(
            "  {}#{} {}{}{} → {}{:.0}%{}",
            c.bold, i + 1, c.cyan, scored.instance.name, c.reset, c.green, pct, c.reset
        );
        println!(
            "      {}CPU  penalty: −{:.3} (weight {:.0}%){}",
            c.dim, b.cpu_component, b.cpu_weight * 100.0, c.reset
        );
        println!(
            "      {}RAM  penalty: −{:.3} (weight {:.0}%){}",
            c.dim, b.ram_component, b.ram_weight * 100.0, c.reset
        );
        println!(
            "      {}Cost penalty: −{:.3} (weight {:.0}%){}",
            c.dim, b.cost_component, b.cost_weight * 100.0, c.reset
        );
        if b.network_weight > 0.0 {
            println!(
                "      {}Net  penalty: −{:.3} (weight {:.0}%, {:.0} Gbps){}",
                c.dim, b.network_component, b.network_weight * 100.0,
                scored.instance.effective_network_gbps(), c.reset
            );
        }
    }
}

/// Display multi-region comparison (same provider, different regions)
pub fn display_multiregion(results: &[(String, Recommendation)], request: &UserRequest, start: Instant) {
    let c = get_colors();
    let emoji = use_emoji();
    let cloud_icon = if emoji { "☁" } else { "" };

    println!();
    println!(
        "  {}{} kumo{} — {}{}{} {}Multi-region compare{}    {}",
        c.bold, cloud_icon, c.reset,
        c.bold, c.cyan, request.cloud, c.reset,
        c.bold, env_badge(&request.env, c)
    );
    println!(
        "  {}╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌{}",
        c.dim, c.reset
    );
    print!(
        "  {}Request:{} {}{}{} vCPUs │ {}{:.1}{} GB RAM",
        c.dim, c.reset, c.bold, fmt_vcpu(request.vcpus), c.reset, c.bold, request.ram_gb, c.reset
    );
    if request.workload != WorkloadType::General {
        print!(" │ {}{}{}", c.yellow, request.workload, c.reset);
    }
    println!();
    println!();

    // Table header
    println!(
        "  {}{:<16} {:<24} {:>5} {:>6} {:>12} {:>5}{}",
        c.dim, "Region", "Instance", "vCPUs", "RAM", "On-Demand", "Score", c.reset
    );
    println!("  {}{}{}", c.dim, "─".repeat(74), c.reset);

    // Track cheapest for highlighting
    let mut cheapest_region = String::new();
    let mut cheapest_cost = f32::MAX;
    for (region, rec) in results {
        if let Some(best) = rec.closest_matches.first() {
            let m = best.instance.cost_hour_usd * 730.0;
            if m < cheapest_cost {
                cheapest_cost = m;
                cheapest_region = region.clone();
            }
        }
    }

    for (region, rec) in results {
        if let Some(best) = rec.closest_matches.first() {
            let inst = best.instance;
            let monthly = inst.cost_hour_usd * 730.0;
            let pct = (best.score * 100.0).min(100.0).max(0.0);
            let score_color = if pct >= 80.0 { c.green } else if pct >= 50.0 { c.yellow } else { c.dim };
            let cheapest_marker = if *region == cheapest_region {
                if emoji { " 💰" } else { " *" }
            } else { "" };

            println!(
                "  {}{:<16}{} {}{:<24}{} {:>5} {:>4} GB {:>12} {}{:>4.0}%{}{}",
                c.bold, region, c.reset,
                c.cyan, inst.name, c.reset,
                fmt_vcpu(inst.vcpus),
                inst.ram_gb as u32,
                format!("${:.0}/mo", monthly),
                score_color, pct, c.reset,
                cheapest_marker,
            );
        } else {
            println!("  {}{:<16}{} {}No match found{}", c.bold, region, c.reset, c.dim, c.reset);
        }
    }

    println!("  {}{}{}", c.dim, "─".repeat(74), c.reset);
    if !cheapest_region.is_empty() {
        println!(
            "  {}💰 Cheapest region: {}{} (${:.0}/mo){}",
            c.dim, c.green, cheapest_region, cheapest_cost, c.reset
        );
    }

    println!(
        "  {}╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌{}",
        c.dim, c.reset
    );
    let warn_icon = if emoji { "⚠" } else { "(!)" };
    println!(
        "  {}  {}Prices from {} · regional pricing may vary{}",
        warn_icon, c.dim, CATALOG_META.last_updated, c.reset
    );
    let time_icon = if emoji { "⚡" } else { "" };
    let elapsed = start.elapsed();
    println!(
        "  {}Done in {:.0}ms {}{}",
        c.dim, elapsed.as_secs_f64() * 1000.0, time_icon, c.reset
    );
    println!();
}

/// Display multi-cloud comparison
pub fn display_multicloud(results: &[(crate::models::CloudProvider, Recommendation)], request: &UserRequest, start: Instant) {
    let c = get_colors();
    let emoji = use_emoji();
    let cloud_icon = if emoji { "☁" } else { "" };

    println!();
    println!(
        "  {}{} kumo{} — {}{}Multi-cloud compare{}    {}",
        c.bold, cloud_icon, c.reset,
        c.bold, c.cyan, c.reset,
        env_badge(&request.env, c)
    );
    println!(
        "  {}╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌{}",
        c.dim, c.reset
    );
    print!(
        "  {}Request:{} {}{}{} vCPUs │ {}{:.1}{} GB RAM",
        c.dim, c.reset, c.bold, fmt_vcpu(request.vcpus), c.reset, c.bold, request.ram_gb, c.reset
    );
    if request.workload != WorkloadType::General {
        print!(" │ {}{}{}", c.yellow, request.workload, c.reset);
    }
    println!();
    println!();

    // Table header
    println!(
        "  {}{:<7} {:<26} {:>5} {:>6} {:>10} {:>10} {:>5}{}",
        c.dim, "Cloud", "Instance", "vCPUs", "RAM", "On-Demand", "1yr CUD", "Score", c.reset
    );
    println!("  {}{}{}", c.dim, "─".repeat(78), c.reset);

    for (cloud, rec) in results {
        let rates = committed_rates(*cloud);
        if let Some(best) = rec.closest_matches.first() {
            let inst = best.instance;
            let monthly = inst.cost_hour_usd * 730.0;
            let monthly_1yr = monthly * (1.0 - rates.one_year_discount);
            let pct = (best.score * 100.0).min(100.0).max(0.0);
            let score_color = if pct >= 80.0 { c.green } else if pct >= 50.0 { c.yellow } else { c.dim };

            println!(
                "  {}{:<7}{} {}{:<26}{} {:>5} {:>4} GB {:>10} {:>10} {}{:>4.0}%{}",
                c.bold, cloud, c.reset,
                c.cyan, inst.name, c.reset,
                fmt_vcpu(inst.vcpus),
                inst.ram_gb as u32,
                format!("${:.0}/mo", monthly),
                format!("${:.0}/mo", monthly_1yr),
                score_color, pct, c.reset,
            );
        } else {
            println!("  {}{:<7}{} {}No match found{}", c.bold, cloud, c.reset, c.dim, c.reset);
        }
    }

    println!("  {}{}{}", c.dim, "─".repeat(78), c.reset);

    // Separator + time
    println!(
        "  {}╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌{}",
        c.dim, c.reset
    );
    let warn_icon = if emoji { "⚠" } else { "(!)" };
    println!(
        "  {}  {}Prices from {} · CUD rates are approximate averages{}",
        warn_icon, c.dim, CATALOG_META.last_updated, c.reset
    );
    let time_icon = if emoji { "⚡" } else { "" };
    let elapsed = start.elapsed();
    println!(
        "  {}Done in {:.0}ms {}{}",
        c.dim, elapsed.as_secs_f64() * 1000.0, time_icon, c.reset
    );
    println!();
}
