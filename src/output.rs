use crate::models::{Environment, Recommendation, UserRequest, CATALOG_META};
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
    if env::var("NO_COLOR").is_ok() {
        &COLORS_OFF
    } else {
        &COLORS_ON
    }
}

fn use_emoji() -> bool {
    env::var("NO_COLOR").is_err()
}

/// Format vCPU count: show as integer if whole number, otherwise as decimal
fn fmt_vcpu(vcpus: f32) -> String {
    if (vcpus - vcpus.round()).abs() < 0.01 && vcpus >= 1.0 {
        format!("{}", vcpus as u32)
    } else {
        format!("{:.1}", vcpus)
    }
}

/// Generate a score bar [████████░░░░░░░░░░░░] 
fn score_bar(score: f32, c: &Colors) -> String {
    let pct = (score * 100.0).min(100.0).max(0.0);
    let filled = ((pct / 100.0) * 20.0).round() as usize;
    let empty = 20 - filled;

    let bar_color = if pct >= 80.0 {
        c.green
    } else if pct >= 50.0 {
        c.yellow
    } else {
        c.dim
    };

    let score_color = bar_color;

    if env::var("NO_COLOR").is_ok() {
        format!(
            "[{}{}] {:.0}%",
            "#".repeat(filled),
            ".".repeat(empty),
            pct
        )
    } else {
        format!(
            "{}[{}{}{}{}]{} {}{:.0}%{}",
            c.dim,
            bar_color,
            "█".repeat(filled),
            c.dim,
            "░".repeat(empty),
            c.reset,
            score_color,
            pct,
            c.reset
        )
    }
}

/// Format delta with arrows and color
fn fmt_delta_vcpu(delta: f32, c: &Colors) -> String {
    if delta.abs() < 0.01 {
        format!("{}═{}", c.green, c.reset)
    } else if delta > 0.0 {
        format!("{}↑{}{}", c.green, fmt_vcpu(delta), c.reset)
    } else {
        format!("{}↓{}{}", c.red, fmt_vcpu(delta.abs()), c.reset)
    }
}

fn fmt_delta_ram(delta: f32, c: &Colors) -> String {
    if delta.abs() < 0.01 {
        format!("{}═{}", c.green, c.reset)
    } else if delta > 0.0 {
        format!("{}↑{:.0}{}", c.green, delta, c.reset)
    } else {
        format!("{}↓{:.0}{}", c.red, delta.abs(), c.reset)
    }
}

/// Environment badge with color
fn env_badge(env: &Environment, c: &Colors) -> String {
    if env::var("NO_COLOR").is_ok() {
        match env {
            Environment::Production => format!("{} [!]", env),
            Environment::PreProduction => format!("{} [~]", env),
            Environment::Test => format!("{} [t]", env),
            Environment::Dev => format!("{} [d]", env),
        }
    } else {
        match env {
            Environment::Production => format!("{}{}Production 🔴{}", c.bold, c.red, c.reset),
            Environment::PreProduction => {
                format!("{}{}Pre-Production 🟡{}", c.bold, c.yellow, c.reset)
            }
            Environment::Test => format!("{}{}Test 🔵{}", c.bold, c.cyan, c.reset),
            Environment::Dev => format!("{}{}Dev 🟢{}", c.bold, c.green, c.reset),
        }
    }
}

/// Rank badges
fn rank_badge(rank: usize) -> &'static str {
    if env::var("NO_COLOR").is_ok() {
        match rank {
            1 => "[1]",
            2 => "[2]",
            3 => "[3]",
            _ => "[-]",
        }
    } else {
        match rank {
            1 => "❶",
            2 => "❷",
            3 => "❸",
            _ => "•",
        }
    }
}

pub fn display(recommendation: &Recommendation, request: &UserRequest, start: Instant) {
    let c = get_colors();
    let emoji = use_emoji();

    let cloud_icon = if emoji { "☁" } else { "" };

    // Header
    println!();
    println!(
        "  {}{} kumo{} — {}{}{} {}{}{}    {}",
        c.bold,
        cloud_icon,
        c.reset,
        c.bold,
        c.cyan,
        request.cloud,
        c.dim,
        request.region,
        c.reset,
        env_badge(&request.env, c)
    );
    println!(
        "  {}╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌{}",
        c.dim, c.reset
    );

    // Request summary
    print!(
        "  {}Request:{} {}{}{} vCPUs │ {}{:.1}{} GB RAM",
        c.dim, c.reset, c.bold, fmt_vcpu(request.vcpus), c.reset, c.bold, request.ram_gb, c.reset
    );
    if let Some(ref family) = request.family {
        print!(" │ Family: {}{}{}", c.cyan, family, c.reset);
    }
    println!();

    // Recommendations
    println!();
    let header_icon = if emoji { "▸" } else { ">" };
    println!("  {}{} Recommendations{}", c.bold, header_icon, c.reset);
    println!();

    let best_cost = recommendation
        .closest_matches
        .first()
        .map(|s| s.instance.cost_hour_usd * 730.0);

    for (i, scored) in recommendation.closest_matches.iter().enumerate() {
        let rank = i + 1;
        let inst = scored.instance;
        let monthly = inst.cost_hour_usd * 730.0;

        // Rank badge + instance name
        let is_exact = recommendation.exact_match.is_some()
            && recommendation.exact_match.unwrap().name == inst.name;

        let name_color = if rank == 1 { c.bold } else { "" };
        let exact_label = if is_exact {
            if emoji {
                format!(" {}◉ exact{}", c.green, c.reset)
            } else {
                format!(" (* exact)")
            }
        } else {
            String::new()
        };

        println!(
            "  {} {}{}{:<28}{}{}",
            rank_badge(rank),
            name_color,
            c.cyan,
            inst.name,
            c.reset,
            exact_label
        );

        // Specs with deltas
        println!(
            "     {} vCPU {} │ {:.0} GB {} │ {}{}${:.0}/mo{} {}(${:.4}/hr){}",
            fmt_vcpu(inst.vcpus),
            fmt_delta_vcpu(scored.delta_vcpu, c),
            inst.ram_gb,
            fmt_delta_ram(scored.delta_ram, c),
            c.bold,
            if monthly < 100.0 { c.green } else if monthly < 500.0 { c.yellow } else { c.red },
            monthly,
            c.reset,
            c.dim,
            inst.cost_hour_usd,
            c.reset
        );

        // Savings indicator (for #2, #3 relative to #1)
        if rank > 1 {
            if let Some(best) = best_cost {
                let savings = best - monthly;
                if savings > 1.0 {
                    println!(
                        "     {}saves ${:.0}/mo vs #1{}",
                        c.dim, savings, c.reset
                    );
                }
            }
        }

        // Score bar
        println!("     {}", score_bar(scored.score, c));
        println!();
    }

    // Custom machine option
    if let Some(custom) = &recommendation.custom_option {
        let custom_icon = if emoji { "🔧" } else { "[c]" };
        let monthly = custom.estimated_cost_hour * 730.0;
        println!(
            "  {} {}Custom:{} {}",
            custom_icon, c.bold, c.reset, custom.name
        );
        println!(
            "     {} vCPU │ {:.0} GB │ {}~${:.0}/mo{} {}(${:.4}/hr){}",
            fmt_vcpu(custom.vcpus),
            custom.ram_gb,
            c.yellow,
            monthly,
            c.reset,
            c.dim,
            custom.estimated_cost_hour,
            c.reset
        );
        println!();
    }

    // Separator
    println!(
        "  {}╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌{}",
        c.dim, c.reset
    );

    // Tips
    if !recommendation.notes.is_empty() {
        let tip_icon = if emoji { "💡" } else { "(i)" };
        for note in &recommendation.notes {
            println!("  {} {}{}{}", tip_icon, c.dim, note, c.reset);
        }
    }

    // Disclaimer
    let warn_icon = if emoji { "⚠" } else { "(!)" };
    println!(
        "  {}  {}Prices from {} · may vary by region & discounts{}",
        warn_icon, c.dim, CATALOG_META.last_updated, c.reset
    );

    // Separator
    println!(
        "  {}╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌{}",
        c.dim, c.reset
    );

    // Execution time
    let elapsed = start.elapsed();
    let time_icon = if emoji { "⚡" } else { "" };
    println!(
        "  {}Done in {:.0}ms {}{}",
        c.dim,
        elapsed.as_secs_f64() * 1000.0,
        time_icon,
        c.reset
    );
    println!();
}
