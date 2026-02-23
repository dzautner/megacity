//! Info panel sections: Production Chains, Market Prices, Specializations,
//! City Advisors, Achievements, and Mini-map.

use bevy::prelude::*;
use bevy_egui::egui;

use simulation::achievements::Achievement;
use simulation::production::GoodsType;
use simulation::specialization::{CitySpecialization, SpecializationScore};

use rendering::overlay::{OverlayMode, OverlayState};

use super::minimap::{build_minimap_pixels, MINIMAP_SIZE};
use super::types::{InfoPanelExtras, MinimapCache};

/// Render the Economy: Production Chains collapsing section.
pub fn draw_production_chains(ui: &mut egui::Ui, extras: &InfoPanelExtras) {
    let city_goods = &extras.city_goods;

    ui.separator();
    ui.collapsing("Economy: Production Chains", |ui| {
        for &g in GoodsType::all() {
            let prod = city_goods.production_rate.get(&g).copied().unwrap_or(0.0);
            let cons = city_goods.consumption_rate.get(&g).copied().unwrap_or(0.0);
            let stock = city_goods.available.get(&g).copied().unwrap_or(0.0);
            let net = prod - cons;

            ui.horizontal(|ui| {
                ui.label(format!("{:>14}", g.name()));

                let net_color = if net > 0.1 {
                    egui::Color32::from_rgb(50, 200, 50)
                } else if net < -0.1 {
                    egui::Color32::from_rgb(220, 50, 50)
                } else {
                    egui::Color32::from_rgb(180, 180, 180)
                };

                let sign = if net >= 0.0 { "+" } else { "" };
                ui.colored_label(net_color, format!("{}{:.1}", sign, net));
                ui.label(format!("({:.0})", stock));
            });
        }

        // Trade balance
        ui.separator();
        let tb = city_goods.trade_balance;
        let tb_color = if tb > 0.0 {
            egui::Color32::from_rgb(50, 200, 50)
        } else if tb < -1.0 {
            egui::Color32::from_rgb(220, 50, 50)
        } else {
            egui::Color32::from_rgb(180, 180, 180)
        };
        ui.horizontal(|ui| {
            ui.label("Trade balance:");
            ui.colored_label(tb_color, format!("${:.1}/tick", tb));
        });
    });
}

/// Render the Market Prices collapsing section.
pub fn draw_market_prices(ui: &mut egui::Ui, extras: &InfoPanelExtras) {
    let market = &extras.market_prices;

    ui.separator();
    ui.collapsing("Market Prices", |ui| {
        // Active market events
        if !market.active_events.is_empty() {
            ui.label("Active Events:");
            for active in &market.active_events {
                let event_color = egui::Color32::from_rgb(255, 200, 50);
                ui.colored_label(
                    event_color,
                    format!(
                        "{} ({} ticks left)",
                        active.event.name(),
                        active.remaining_ticks
                    ),
                );
            }
            ui.add_space(4.0);
        }

        // Goods prices
        ui.label("Goods:");
        egui::Grid::new("market_goods_grid")
            .num_columns(3)
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Good");
                ui.strong("Price");
                ui.strong("Trend");
                ui.end_row();

                for &g in GoodsType::all() {
                    if let Some(entry) = market.goods_prices.get(&g) {
                        ui.label(g.name());
                        let mult = entry.multiplier();
                        let price_color = if mult > 1.15 {
                            egui::Color32::from_rgb(220, 50, 50)
                        } else if mult < 0.85 {
                            egui::Color32::from_rgb(50, 200, 50)
                        } else {
                            egui::Color32::from_rgb(180, 180, 180)
                        };
                        ui.colored_label(price_color, format!("${:.1}", entry.current_price));

                        let trend = entry.trend();
                        let (trend_str, trend_color) = if trend > 0.1 {
                            (
                                format!("+{:.1} ^", trend),
                                egui::Color32::from_rgb(220, 50, 50),
                            )
                        } else if trend < -0.1 {
                            (
                                format!("{:.1} v", trend),
                                egui::Color32::from_rgb(50, 200, 50),
                            )
                        } else {
                            ("~0.0 -".to_string(), egui::Color32::from_rgb(140, 140, 140))
                        };
                        ui.colored_label(trend_color, trend_str);
                        ui.end_row();
                    }
                }
            });

        ui.add_space(4.0);

        // Resource prices
        ui.label("Resources:");
        egui::Grid::new("market_resource_grid")
            .num_columns(3)
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Resource");
                ui.strong("Price");
                ui.strong("Trend");
                ui.end_row();

                for (&rt, entry) in &market.resource_prices {
                    ui.label(rt.name());
                    let mult = entry.multiplier();
                    let price_color = if mult > 1.15 {
                        egui::Color32::from_rgb(220, 50, 50)
                    } else if mult < 0.85 {
                        egui::Color32::from_rgb(50, 200, 50)
                    } else {
                        egui::Color32::from_rgb(180, 180, 180)
                    };
                    ui.colored_label(price_color, format!("${:.1}", entry.current_price));

                    let trend = entry.trend();
                    let (trend_str, trend_color) = if trend > 0.05 {
                        (
                            format!("+{:.1} ^", trend),
                            egui::Color32::from_rgb(220, 50, 50),
                        )
                    } else if trend < -0.05 {
                        (
                            format!("{:.1} v", trend),
                            egui::Color32::from_rgb(50, 200, 50),
                        )
                    } else {
                        ("~0.0 -".to_string(), egui::Color32::from_rgb(140, 140, 140))
                    };
                    ui.colored_label(trend_color, trend_str);
                    ui.end_row();
                }
            });
    });
}

/// Render the City Specializations collapsing section.
pub fn draw_specializations(ui: &mut egui::Ui, extras: &InfoPanelExtras) {
    let specializations = &extras.specializations;

    ui.separator();
    ui.collapsing("City Specializations", |ui| {
        for &spec in CitySpecialization::ALL {
            let s = specializations.get(spec);
            let level_name = SpecializationScore::level_name(s.level);
            let level_color = match s.level {
                0 => egui::Color32::from_rgb(140, 140, 140),
                1 => egui::Color32::from_rgb(220, 200, 50),
                2 => egui::Color32::from_rgb(50, 200, 50),
                3 => egui::Color32::from_rgb(255, 200, 50),
                _ => egui::Color32::from_rgb(140, 140, 140),
            };

            ui.horizontal(|ui| {
                ui.label(format!("{:>10}", spec.name()));
                ui.colored_label(level_color, format!("[{}]", level_name));
            });

            let (rect, _) = ui.allocate_exact_size(egui::vec2(160.0, 10.0), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 2.0, egui::Color32::from_gray(40));
            let fill_pct = (s.score / 100.0).clamp(0.0, 1.0);
            let fill_rect = egui::Rect::from_min_size(
                rect.min,
                egui::vec2(rect.width() * fill_pct, rect.height()),
            );
            painter.rect_filled(fill_rect, 2.0, level_color);

            ui.add_space(2.0);
        }
    });
}

/// Render the City Advisors collapsing section.
pub fn draw_advisors(ui: &mut egui::Ui, extras: &InfoPanelExtras) {
    ui.separator();
    ui.collapsing("City Advisors", |ui| {
        let messages = &extras.advisor_panel.messages;
        if messages.is_empty() {
            ui.small("No advisor messages at this time.");
        } else {
            for msg in messages {
                let priority_color = match msg.priority {
                    5 => egui::Color32::from_rgb(220, 50, 50),
                    4 => egui::Color32::from_rgb(230, 150, 30),
                    3 => egui::Color32::from_rgb(220, 200, 50),
                    2 => egui::Color32::from_rgb(50, 130, 220),
                    _ => egui::Color32::from_rgb(150, 150, 150),
                };

                ui.horizontal(|ui| {
                    let (dot_rect, _) =
                        ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                    let painter = ui.painter_at(dot_rect);
                    painter.circle_filled(dot_rect.center(), 4.0, priority_color);

                    ui.colored_label(priority_color, msg.advisor_type.name());
                });

                ui.label(&msg.message);
                ui.small(&msg.suggestion);
                ui.add_space(4.0);
            }
        }
    });
}

/// Render the Achievements collapsing section and notification popups.
pub fn draw_achievements(ui: &mut egui::Ui, extras: &mut InfoPanelExtras) {
    ui.separator();
    {
        let tracker = &extras.achievement_tracker;
        let unlocked = tracker.unlocked_count();
        let total = Achievement::total_count();

        ui.collapsing(format!("Achievements ({}/{})", unlocked, total), |ui| {
            for &achievement in Achievement::ALL {
                let is_unlocked = tracker.is_unlocked(achievement);
                ui.horizontal(|ui| {
                    if is_unlocked {
                        ui.colored_label(egui::Color32::from_rgb(50, 200, 50), "[v]");
                        ui.label(achievement.name());
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(100, 100, 100), "[ ]");
                        ui.colored_label(
                            egui::Color32::from_rgb(100, 100, 100),
                            achievement.name(),
                        );
                    }
                });
                if is_unlocked {
                    ui.small(format!(
                        "  {} ({})",
                        achievement.description(),
                        achievement.reward().description(),
                    ));
                } else {
                    ui.small(format!("  {}", achievement.description()));
                }
                ui.add_space(1.0);
            }
        });
    }

    // Achievement Notifications Popup
    {
        let recent = extras.achievement_notifications.take();
        if !recent.is_empty() {
            for achievement in &recent {
                ui.separator();
                ui.colored_label(
                    egui::Color32::from_rgb(255, 215, 0),
                    format!("Achievement Unlocked: {}", achievement.name()),
                );
                ui.small(format!("  Reward: {}", achievement.reward().description()));
            }
        }
    }
}

/// Render the mini-map with overlay info.
pub fn draw_minimap(
    ui: &mut egui::Ui,
    grid: &simulation::grid::WorldGrid,
    overlay: &OverlayState,
    minimap_cache: &mut MinimapCache,
    time: &Time,
) {
    let needs_update = minimap_cache.texture_handle.is_none() || minimap_cache.dirty_timer <= 0.0;

    ui.separator();
    ui.heading("Mini-map");

    let overlay_text = match overlay.mode {
        OverlayMode::None => "Tab to cycle overlays",
        OverlayMode::Power => "Power overlay [Tab]",
        OverlayMode::Water => "Water overlay [Tab]",
        OverlayMode::Traffic => "Traffic overlay [Tab]",
        OverlayMode::Pollution => "Pollution overlay [Tab]",
        OverlayMode::LandValue => "Land Value overlay [Tab]",
        OverlayMode::Education => "Education overlay [Tab]",
        OverlayMode::Garbage => "Garbage overlay [Tab]",
        OverlayMode::Noise => "Noise overlay [Tab]",
        OverlayMode::WaterPollution => "Water Pollution overlay [Tab]",
        OverlayMode::GroundwaterLevel => "GW Level overlay [Tab]",
        OverlayMode::GroundwaterQuality => "GW Quality overlay [Tab]",
        OverlayMode::Wind => "Wind overlay [Tab]",
    };
    ui.small(overlay_text);

    if needs_update {
        let pixels = build_minimap_pixels(grid, overlay);
        let color_image = egui::ColorImage {
            size: [MINIMAP_SIZE, MINIMAP_SIZE],
            pixels,
        };
        let texture = ui
            .ctx()
            .load_texture("minimap", color_image, egui::TextureOptions::NEAREST);
        minimap_cache.texture_handle = Some(texture);
        minimap_cache.dirty_timer = 2.0;
    }

    if let Some(ref tex) = minimap_cache.texture_handle {
        let size = egui::vec2(MINIMAP_SIZE as f32, MINIMAP_SIZE as f32);
        ui.image(egui::load::SizedTexture::new(tex.id(), size));
    }

    minimap_cache.dirty_timer -= time.delta_secs();
}
